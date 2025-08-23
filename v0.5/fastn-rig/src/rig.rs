use std::str::FromStr;

impl fastn_rig::Rig {
    /// Create a new Rig and initialize the fastn_home with the first account
    /// Returns (Rig, AccountManager, primary_account_id52)
    pub async fn create(
        fastn_home: std::path::PathBuf,
    ) -> eyre::Result<(Self, fastn_account::AccountManager, String)> {
        use eyre::WrapErr;
        use std::str::FromStr;

        // Create fastn_home directory if it doesn't exist
        std::fs::create_dir_all(&fastn_home)
            .wrap_err_with(|| format!("Failed to create fastn_home at {fastn_home:?}"))?;

        // Generate rig's secret key
        let secret_key = fastn_id52::SecretKey::generate();
        let id52 = secret_key.id52();

        // Create and store rig's key
        let rig_key_path = fastn_home.join("rig");
        std::fs::create_dir_all(&rig_key_path).wrap_err("Failed to create rig directory")?;

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

        tracing::info!("Creating new Rig with ID52: {}", id52);

        // Initialize automerge database with rig's entity
        let automerge_path = fastn_home.join("automerge.sqlite");

        eprintln!(
            "🔍 Debug: Initializing automerge DB at {}",
            automerge_path.display()
        );
        eprintln!("🔍 Debug: Rig entity = {}", secret_key.public_key());

        let automerge_db = fastn_automerge::Db::init(&automerge_path, &secret_key.public_key())
            .wrap_err("Failed to initialize automerge database")?;

        eprintln!("🔍 Debug: Automerge DB initialized successfully");

        // Create AccountManager and first account
        eprintln!("🔍 Debug: Creating AccountManager...");
        let (account_manager, primary_id52) =
            fastn_account::AccountManager::create(fastn_home.clone())
                .await
                .wrap_err("Failed to create AccountManager and first account")?;

        eprintln!("🔍 Debug: AccountManager created, primary_id52 = {primary_id52}");

        // Parse owner key
        let owner = fastn_id52::PublicKey::from_str(&primary_id52)
            .wrap_err("Failed to parse owner public key")?;

        // Create rig config struct with all configuration data
        let rig_config = crate::automerge::RigConfig {
            rig: secret_key.public_key(), // Rig's own identity
            owner,                        // Account that owns this rig
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            current_entity: owner, // Owner is the initial current entity
        };

        // Store the complete config struct in the database
        rig_config
            .save(&automerge_db)
            .wrap_err("Failed to create rig config document")?;

        tracing::info!(
            "Created new Rig with ID52: {} (owner: {})",
            id52,
            primary_id52
        );

        let rig = Self {
            path: fastn_home,
            secret_key,
            owner,
            automerge: std::sync::Arc::new(tokio::sync::Mutex::new(automerge_db)),
        };

        Ok((rig, account_manager, primary_id52))
    }

    /// Load an existing Rig from fastn_home
    pub fn load(fastn_home: std::path::PathBuf) -> eyre::Result<Self> {
        use eyre::WrapErr;

        // Load rig's secret key
        let rig_key_path = fastn_home.join("rig");
        let (rig_id52, secret_key) = fastn_id52::SecretKey::load_from_dir(&rig_key_path, "rig")
            .wrap_err("Failed to load rig secret key")?;

        // Open existing automerge database
        let automerge_path = fastn_home.join("automerge.sqlite");
        let automerge_db = fastn_automerge::Db::open(&automerge_path)
            .wrap_err("Failed to open automerge database")?;

        // Load owner from Automerge document using typed API
        let config = crate::automerge::RigConfig::load(&automerge_db, &secret_key.public_key())
            .wrap_err("Failed to load rig config document")?;

        let owner = config.owner;

        tracing::info!(
            "Loaded Rig with ID52: {} (owner: {})",
            rig_id52,
            owner.id52()
        );

        Ok(Self {
            path: fastn_home,
            secret_key,
            owner,
            automerge: std::sync::Arc::new(tokio::sync::Mutex::new(automerge_db)),
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

    /// Get the Rig's owner public key
    pub fn owner(&self) -> &fastn_id52::PublicKey {
        &self.owner
    }

    /// Check if an entity is online
    pub async fn is_entity_online(&self, id52: &str) -> eyre::Result<bool> {
        let automerge_db = self.automerge.lock().await;

        // Parse entity ID52 to PublicKey for type safety
        let entity_key = fastn_id52::PublicKey::from_str(id52)?;

        let is_online = crate::automerge::EntityStatus::is_online(&automerge_db, &entity_key)?;
        Ok(is_online)
    }

    /// Set entity online status
    pub async fn set_entity_online(&self, id52: &str, online: bool) -> eyre::Result<()> {
        let automerge_db = self.automerge.lock().await;

        // Parse entity ID52 to PublicKey for type safety
        let entity_key = fastn_id52::PublicKey::from_str(id52)?;

        crate::automerge::EntityStatus::set_online(&automerge_db, &entity_key, online)?;

        Ok(())
    }

    /// Get the current entity's ID52
    pub async fn get_current(&self) -> eyre::Result<String> {
        let automerge_db = self.automerge.lock().await;

        let current_entity = crate::automerge::RigConfig::get_current_entity(
            &automerge_db,
            &self.secret_key.public_key(),
        )?;
        Ok(current_entity.id52())
    }

    /// Set the current entity
    pub async fn set_current(&self, id52: &str) -> eyre::Result<()> {
        let automerge_db = self.automerge.lock().await;

        // Parse entity ID52 to PublicKey for type safety
        let entity_key = fastn_id52::PublicKey::from_str(id52)?;

        crate::automerge::RigConfig::update_current_entity(
            &automerge_db,
            &self.secret_key.public_key(),
            &entity_key,
        )?;

        tracing::info!("Set current entity to {}", id52);
        Ok(())
    }
}
