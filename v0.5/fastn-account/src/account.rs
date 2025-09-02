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
                        allow_mail: true, // Default: accept mail from all peers
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

    /// Get existing mail configuration for this account
    /// Returns ConfigNotFound if no configuration exists
    pub async fn get_mail_config(&self) -> Result<fastn_mail::DefaultMail, crate::MailConfigError> {
        let automerge_db = self.automerge.lock().await;

        // Use modern Document API - DefaultMail is a singleton document
        match fastn_mail::DefaultMail::load(&automerge_db) {
            Ok(config) => Ok(config),
            Err(fastn_automerge::db::GetError::NotFound(_)) => {
                Err(crate::MailConfigError::ConfigNotFound)
            }
            Err(e) => Err(crate::MailConfigError::DatabaseAccessFailed {
                source: Box::new(e),
            }),
        }
    }

    /// Set SMTP password for this account
    /// Creates mail config if it doesn't exist
    pub async fn set_smtp_password(&self, password: &str) -> Result<(), crate::MailConfigError> {
        // Hash the password
        let password_hash = crate::auth::hash_password(password)
            .map_err(|e| crate::MailConfigError::PasswordGenerationFailed { source: e })?;

        // Try to get existing config, create if it doesn't exist
        let mut config = match self.get_mail_config().await {
            Ok(existing_config) => existing_config,
            Err(crate::MailConfigError::ConfigNotFound) => {
                // Create new config since it doesn't exist
                let now = chrono::Utc::now().timestamp();
                fastn_mail::DefaultMail {
                    password_hash: String::new(), // Will be updated below
                    is_active: false,
                    created_at: now,
                }
            }
            Err(e) => return Err(e),
        };

        // Update the config
        config.password_hash = password_hash;
        config.is_active = true; // Enable SMTP when password is set

        // Save the updated config using modern Document API
        let automerge_db = self.automerge.lock().await;
        config
            .save(&automerge_db)
            .map_err(|e| crate::MailConfigError::DocumentUpdateFailed {
                source: Box::new(e),
            })?;

        tracing::info!("SMTP password updated for account");
        Ok(())
    }

    /// Verify SMTP password for this account
    pub async fn verify_smtp_password(
        &self,
        password: &str,
    ) -> Result<bool, crate::MailConfigError> {
        let config = self.get_mail_config().await?;

        // Check if mail service is active
        if !config.is_active {
            return Ok(false);
        }

        // Verify password against stored hash
        match crate::auth::verify_password(password, &config.password_hash) {
            Ok(is_valid) => Ok(is_valid),
            Err(e) => {
                tracing::error!("Password verification failed: {}", e);
                Ok(false) // Don't expose verification errors to callers
            }
        }
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
