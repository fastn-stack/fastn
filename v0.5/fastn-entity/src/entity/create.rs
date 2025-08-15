impl fastn_entity::Entity {
    /// Creates a new entity in the specified directory.
    ///
    /// This will:
    /// 1. Generate a new cryptographic identity
    /// 2. Create the entity directory named by its ID52
    /// 3. Save the public key to `entity.id52`
    /// 4. Store the private key in the system keyring (or file if SKIP_KEYRING=true)
    /// 5. Create an empty SQLite database
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Directory creation fails
    /// - Key storage fails (keyring access or file write)
    /// - Database creation fails
    pub async fn create(parent_dir: &std::path::Path) -> eyre::Result<Self> {
        use eyre::WrapErr;

        // Generate new identity
        let secret_key = fastn_id52::SecretKey::generate();
        let id52 = secret_key.id52();

        // Entity folder is always named by its ID52
        let entity_path = parent_dir.join(&id52);

        // Create entity directory
        std::fs::create_dir_all(&entity_path)
            .wrap_err_with(|| format!("Failed to create entity directory at {entity_path:?}"))?;

        // Save public key to file
        let id52_file = entity_path.join("entity.id52");
        std::fs::write(&id52_file, &id52).wrap_err("Failed to write entity.id52 file")?;

        // Store private key based on SKIP_KEYRING environment variable
        if std::env::var("SKIP_KEYRING").map(|v| v == "true").unwrap_or(false) {
            // If SKIP_KEYRING=true, save to file
            tracing::info!(
                "SKIP_KEYRING is set, saving private key to file for {}",
                id52
            );
            let private_key_file = entity_path.join("entity.private-key");
            std::fs::write(&private_key_file, secret_key.to_string())
                .wrap_err("Failed to write entity.private-key file")?;
        } else {
            // Default: store in keyring
            secret_key
                .store_in_keyring()
                .wrap_err("Failed to store private key in keyring")?;
            tracing::info!("Stored entity private key in system keyring for {}", id52);
        }

        // Create SQLite database and run migrations
        let db_path = entity_path.join("db.sqlite");
        let conn =
            rusqlite::Connection::open(&db_path).wrap_err("Failed to create SQLite database")?;

        fastn_entity::migration::migrate(&conn).wrap_err("Failed to run database migrations")?;
        
        let conn = std::sync::Arc::new(tokio::sync::Mutex::new(conn));

        Ok(fastn_entity::Entity {
            id52: id52.clone(),
            path: entity_path,
            secret_key,
            conn,
        })
    }
}
