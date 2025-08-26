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

    /// Handle incoming P2P connection authorization and peer tracking
    /// Returns true if connection should be accepted, false if rejected
    pub async fn authorize_connection(
        &self,
        peer_id52: &fastn_id52::PublicKey,
        our_alias: &fastn_id52::PublicKey,
    ) -> Result<bool, crate::AuthorizeConnectionError> {
        // TODO: Check security policies (block list, allowlist, lockdown mode)
        // For now, accept all connections

        // Ensure peer notes document exists for alias association tracking
        let automerge_db = self.automerge.lock().await;
        let now = chrono::Utc::now().timestamp();

        automerge_db
            .load_or_create_with(
                &crate::automerge::AliasNotes::document_path(peer_id52),
                || {
                    tracing::debug!("Creating new peer notes document for {}", peer_id52);
                    crate::automerge::AliasNotes {
                        alias: *peer_id52,
                        nickname: None,
                        notes: None,
                        relationship_started_at: now,
                        first_connected_to: *our_alias,
                    }
                },
            )
            .map_err(|e| crate::AuthorizeConnectionError::DatabaseAccessFailed {
                source: Box::new(e),
            })?;

        tracing::info!(
            "✅ Connection authorized for peer {} to alias {}",
            peer_id52.id52(),
            our_alias.id52()
        );
        Ok(true) // Accept connection
    }

    /// Create a test account in memory (for testing only)
    #[cfg(test)]
    pub(crate) async fn new_for_test(
        path: std::path::PathBuf,
        aliases: Vec<fastn_account::Alias>,
    ) -> Self {
        // Create test databases - use fastn-automerge test utility
        let (automerge, _temp_dir) = fastn_automerge::create_test_db().unwrap();
        let mail = fastn_mail::Store::create_test();
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
