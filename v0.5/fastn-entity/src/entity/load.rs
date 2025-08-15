impl fastn_entity::Entity {
    /// Loads an existing entity from the specified directory.
    ///
    /// This will:
    /// 1. Load the entity keys using SecretKey::load_from_dir
    /// 2. Open the existing database and run migrations
    ///
    /// # Arguments
    ///
    /// * `entity_path` - Path to the entity's directory
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Keys cannot be loaded (see SecretKey::load_from_dir for details)
    /// - Database cannot be opened or migrations fail
    pub async fn load(entity_path: &std::path::Path) -> eyre::Result<Self> {
        use eyre::WrapErr;

        tracing::info!("Loading entity from {:?}", entity_path);

        // Load keys using the comprehensive loader
        let (id52, secret_key) = fastn_id52::SecretKey::load_from_dir(entity_path, "entity")
            .map_err(|e| {
                tracing::error!("Failed to load entity keys: {}", e);
                eyre::eyre!("Failed to load entity keys: {}", e)
            })?;

        tracing::info!("Successfully loaded keys for entity {}", id52);

        // Open existing database
        let db_path = entity_path.join("db.sqlite");
        if !db_path.exists() {
            let err = eyre::eyre!("Database file not found at {:?}", db_path);
            tracing::error!("{}", err);
            return Err(err);
        }

        let conn = rusqlite::Connection::open(&db_path)
            .wrap_err("Failed to open SQLite database")
            .map_err(|e| {
                tracing::error!("Failed to open SQLite database at {:?}: {}", db_path, e);
                e
            })?;

        // Run migrations to ensure schema is up to date
        fastn_entity::migration::migrate(&conn)
            .wrap_err("Failed to run database migrations")
            .map_err(|e| {
                tracing::error!("Failed to run database migrations: {}", e);
                e
            })?;

        let conn = std::sync::Arc::new(tokio::sync::Mutex::new(conn));

        tracing::info!("Successfully loaded entity {} from {:?}", id52, entity_path);

        Ok(fastn_entity::Entity {
            public_key: secret_key.public_key(),
            path: entity_path.to_path_buf(),
            secret_key,
            conn,
        })
    }
}
