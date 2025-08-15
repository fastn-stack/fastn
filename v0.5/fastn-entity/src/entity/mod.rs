mod create;
mod load;

impl fastn_entity::Entity {
    /// Returns the path to the entity's directory.
    pub fn path(&self) -> &std::path::Path {
        &self.path
    }

    /// Returns the path to the entity's SQLite database.
    pub fn database_path(&self) -> std::path::PathBuf {
        self.path.join("db.sqlite")
    }

    /// Returns the entity's public key.
    pub fn public_key(&self) -> fastn_id52::PublicKey {
        self.public_key
    }

    /// Returns the entity's secret key.
    ///
    /// Use with caution - the secret key should not be exposed unnecessarily.
    pub fn secret_key(&self) -> &fastn_id52::SecretKey {
        &self.secret_key
    }

    /// Signs a message with the entity's private key.
    pub fn sign(&self, message: &[u8]) -> fastn_id52::Signature {
        self.secret_key.sign(message)
    }

    /// Gets a reference to the database connection.
    pub async fn conn(&self) -> tokio::sync::MutexGuard<'_, rusqlite::Connection> {
        self.conn.lock().await
    }
}
