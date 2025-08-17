impl crate::AccountManager {
    /// Creates a new AccountManager for the given fastn_home directory
    ///
    /// # Arguments
    ///
    /// * `fastn_home` - The fastn_home directory path (provided by fastn crate)
    pub fn create(fastn_home: std::path::PathBuf) -> Self {
        tracing::info!("Creating AccountManager at {fastn_home:?}");

        // Simple creation - just stores the path
        // The fastn crate handles directory creation and path resolution
        Self {
            path: fastn_home,
            online_status: std::collections::HashMap::new(),
            last: None,
        }
    }

    /// Get the last used account's ID52
    pub fn last(&self) -> Option<&str> {
        self.last.as_deref()
    }

    /// Set the last used account
    pub fn set_last(&mut self, id52: String) {
        self.last = Some(id52);
    }

    /// Get the online status for an account
    pub fn is_online(&self, id52: &str) -> bool {
        self.online_status.get(id52).copied().unwrap_or(false)
    }

    /// Set the online status for an account
    pub fn set_online(&mut self, id52: &str, online: bool) {
        self.online_status.insert(id52.to_string(), online);
        tracing::info!(
            "Set account {} to {}",
            id52,
            if online { "online" } else { "offline" }
        );
    }
}
