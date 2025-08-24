mod create;
mod load;

impl fastn_account::Account {
    /// Get the primary alias ID52 (used for folder naming)
    pub async fn primary_id52(&self) -> Option<String> {
        let aliases = self.aliases.read().await;
        aliases.iter().find(|a| a.is_primary).map(|a| a.id52())
    }

    /// Get the primary alias
    pub async fn primary_alias(&self) -> Option<fastn_account::Alias> {
        let aliases = self.aliases.read().await;
        aliases.iter().find(|a| a.is_primary).cloned()
    }

    /// Check if an ID52 belongs to this account
    pub async fn has_alias(&self, id52: &str) -> bool {
        let aliases = self.aliases.read().await;
        aliases.iter().any(|a| a.id52() == id52)
    }

    /// Get the account's storage path
    pub async fn path(&self) -> std::path::PathBuf {
        (*self.path).clone()
    }

    /// Get all aliases (returns a clone)
    pub async fn aliases(&self) -> Vec<fastn_account::Alias> {
        let aliases = self.aliases.read().await;
        aliases.clone()
    }

    /// Create a test account in memory (for testing only)
    #[cfg(test)]
    pub(crate) async fn new_for_test(
        path: std::path::PathBuf,
        aliases: Vec<fastn_account::Alias>,
    ) -> Self {
        // Create test databases - use fastn-automerge test utility
        let (automerge, _temp_dir) = fastn_automerge::create_test_db().unwrap();
        let mail = fastn_mail::Mail::new_for_test();
        let user = rusqlite::Connection::open_in_memory().unwrap();

        Self {
            path: std::sync::Arc::new(path),
            aliases: std::sync::Arc::new(tokio::sync::RwLock::new(aliases)),
            automerge: std::sync::Arc::new(tokio::sync::Mutex::new(automerge)),
            mail,
            user: std::sync::Arc::new(tokio::sync::Mutex::new(user)),
        }
    }
}
