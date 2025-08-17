impl fastn_rig::Rig {
    /// Create a new Rig and initialize the fastn_home
    pub fn create(
        fastn_home: std::path::PathBuf,
        owner: Option<fastn_id52::PublicKey>,
    ) -> eyre::Result<Self> {
        use eyre::WrapErr;

        // Generate rig's secret key
        let secret_key = fastn_id52::SecretKey::generate();
        let id52 = secret_key.id52();

        // Store rig's key
        let rig_key_path = fastn_home.join("rig");
        std::fs::create_dir_all(&rig_key_path)?;

        // Store key based on SKIP_KEYRING
        if std::env::var("SKIP_KEYRING")
            .map(|v| v == "true")
            .unwrap_or(false)
        {
            let private_key_file = rig_key_path.join("rig.private-key");
            std::fs::write(&private_key_file, secret_key.to_string())
                .wrap_err("Failed to write rig private key")?;
        } else {
            let id52_file = rig_key_path.join("rig.id52");
            std::fs::write(&id52_file, &id52).wrap_err("Failed to write rig ID52")?;
            secret_key
                .store_in_keyring()
                .wrap_err("Failed to store rig key in keyring")?;
        }

        // Store owner if provided
        if let Some(ref owner_key) = owner {
            let owner_file = rig_key_path.join("owner");
            std::fs::write(&owner_file, owner_key.to_string())
                .wrap_err("Failed to write rig owner")?;
            tracing::info!("Created new Rig with ID52: {} (owner: {})", id52, owner_key);
        } else {
            tracing::info!("Created new Rig with ID52: {} (no owner)", id52);
        }

        // Create rig database
        let rig_db_path = fastn_home.join("rig.sqlite");
        let conn =
            rusqlite::Connection::open(&rig_db_path).wrap_err("Failed to create rig database")?;

        // Run database migrations
        fastn_rig::migration::migrate_database(&conn)?;

        Ok(Self {
            path: fastn_home,
            secret_key,
            owner,
            db: std::sync::Arc::new(tokio::sync::Mutex::new(conn)),
        })
    }

    /// Load an existing Rig from fastn_home
    pub fn load(fastn_home: std::path::PathBuf) -> eyre::Result<Self> {
        use eyre::WrapErr;
        use std::str::FromStr;

        // Load rig's secret key
        let rig_key_path = fastn_home.join("rig");
        let (id52, secret_key) = fastn_id52::SecretKey::load_from_dir(&rig_key_path, "rig")
            .wrap_err("Failed to load rig secret key")?;

        // Load owner if exists
        let owner_file = rig_key_path.join("owner");
        let owner = if owner_file.exists() {
            let owner_id52 = std::fs::read_to_string(&owner_file)
                .wrap_err("Failed to read rig owner")?
                .trim()
                .to_string();
            Some(
                fastn_id52::PublicKey::from_str(&owner_id52)
                    .wrap_err("Failed to parse owner public key")?,
            )
        } else {
            None
        };

        if let Some(ref owner_key) = owner {
            tracing::info!("Loaded Rig with ID52: {} (owner: {})", id52, owner_key);
        } else {
            tracing::info!("Loaded Rig with ID52: {} (no owner)", id52);
        }

        // Open existing rig database
        let rig_db_path = fastn_home.join("rig.sqlite");
        if !rig_db_path.exists() {
            return Err(eyre::eyre!("Rig database not found at {:?}", rig_db_path));
        }

        let conn =
            rusqlite::Connection::open(&rig_db_path).wrap_err("Failed to open rig database")?;

        // Run migrations in case schema has changed since last load
        fastn_rig::migration::migrate_database(&conn)?;

        Ok(Self {
            path: fastn_home,
            secret_key,
            owner,
            db: std::sync::Arc::new(tokio::sync::Mutex::new(conn)),
        })
    }

    /// Get the Rig's ID52
    pub fn id52(&self) -> String {
        self.secret_key.id52()
    }

    /// Get the Rig's public key
    pub fn public_key(&self) -> fastn_id52::PublicKey {
        self.secret_key.public_key()
    }

    /// Get the Rig's secret key (use with caution)
    pub fn secret_key(&self) -> &fastn_id52::SecretKey {
        &self.secret_key
    }

    /// Get the Rig's owner public key (if any)
    pub fn owner(&self) -> Option<&fastn_id52::PublicKey> {
        self.owner.as_ref()
    }

    /// Check if an endpoint is online
    pub async fn is_endpoint_online(&self, id52: &str) -> bool {
        let db = self.db.lock().await;

        db.query_row(
            "SELECT is_online FROM fastn_endpoints WHERE id52 = ?1",
            rusqlite::params![id52],
            |row| row.get::<_, bool>(0),
        )
        .unwrap_or(false)
    }

    /// Set endpoint online status
    pub async fn set_endpoint_online(&self, id52: &str, online: bool) {
        let db = self.db.lock().await;

        // Use INSERT OR IGNORE to preserve is_current if the row exists
        if let Err(e) = db.execute(
            "INSERT OR IGNORE INTO fastn_endpoints (id52, is_online, is_current) VALUES (?1, ?2, 0)",
            rusqlite::params![id52, online as i32],
        ) {
            tracing::error!("Failed to insert endpoint {}: {}", id52, e);
        }

        // Then update the online status
        if let Err(e) = db.execute(
            "UPDATE fastn_endpoints SET is_online = ?1 WHERE id52 = ?2",
            rusqlite::params![online as i32, id52],
        ) {
            tracing::error!("Failed to update online status for {}: {}", id52, e);
        }
    }

    /// Get the current entity's ID52
    pub async fn get_current(&self) -> Option<String> {
        let db = self.db.lock().await;

        db.query_row(
            "SELECT id52 FROM fastn_endpoints WHERE is_current = 1",
            [],
            |row| row.get::<_, String>(0),
        )
        .ok()
    }

    /// Set the current entity
    pub async fn set_current(&self, id52: &str) -> eyre::Result<()> {
        let db = self.db.lock().await;

        // Transaction to ensure atomicity
        let tx = db.unchecked_transaction()?;

        // Clear current flag from all endpoints
        tx.execute("UPDATE fastn_endpoints SET is_current = 0", [])?;

        // Set current flag for the specified endpoint
        tx.execute(
            "UPDATE fastn_endpoints SET is_current = 1 WHERE id52 = ?1",
            rusqlite::params![id52],
        )?;

        tx.commit()?;

        tracing::info!("Set current entity to {}", id52);
        Ok(())
    }
}
