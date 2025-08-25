impl fastn_account::Account {
    /// Loads an existing account from the specified directory.
    ///
    /// This will:
    /// 1. Open the SQLite database
    /// 2. Load all aliases from the aliases directory
    /// 3. Load keys from keyring or files
    ///
    /// Note: Does NOT verify that the folder name matches an alias ID52.
    /// The fastn crate is responsible for providing the correct path.
    ///
    /// # Arguments
    ///
    /// * `account_dir` - Path to the account directory (from fastn crate)
    ///
    /// # Returns
    ///
    /// The loaded Account
    ///
    /// # Errors
    ///
    /// Returns error if the directory doesn't exist or database can't be opened
    pub async fn load(
        account_dir: &std::path::Path,
    ) -> Result<Self, fastn_account::AccountLoadError> {
        if !account_dir.exists() {
            return Err(fastn_account::AccountLoadError::AccountDirectoryNotFound {
                path: account_dir.to_path_buf(),
            });
        }

        if !account_dir.is_dir() {
            return Err(fastn_account::AccountLoadError::AccountDirectoryInvalid {
                path: account_dir.to_path_buf(),
            });
        }

        // Open all three databases
        let automerge_path = account_dir.join("automerge.sqlite");
        let mail_path = account_dir.join("mail.sqlite");
        let user_path = account_dir.join("db.sqlite");

        if !automerge_path.exists() {
            return Err(fastn_account::AccountLoadError::AutomergeDatabaseNotFound {
                path: automerge_path,
            });
        }
        if !mail_path.exists() {
            return Err(fastn_account::AccountLoadError::MailDatabaseNotFound { path: mail_path });
        }
        if !user_path.exists() {
            return Err(fastn_account::AccountLoadError::UserDatabaseNotFound { path: user_path });
        }

        // Get account ID from directory name (which is the primary alias ID52)
        // let account_id52 = account_dir.file_name()
        //     .and_then(|name| name.to_str())
        //     .ok_or_else(|| eyre::eyre!("Invalid account directory name"))?;

        let automerge_db = fastn_automerge::Db::open(&automerge_path).map_err(|e| {
            fastn_account::AccountLoadError::DatabaseOpenFailed {
                path: automerge_path.clone(),
                source: Box::new(e),
            }
        })?;
        // Load mail system
        let mail = fastn_mail::Store::load(account_dir).await.map_err(|e| {
            fastn_account::AccountLoadError::DatabaseOpenFailed {
                path: mail_path.clone(),
                source: Box::new(e),
            }
        })?;
        let user = rusqlite::Connection::open(&user_path).map_err(|e| {
            fastn_account::AccountLoadError::DatabaseOpenFailed {
                path: user_path.clone(),
                source: Box::new(e),
            }
        })?;

        // Run user database migrations
        fastn_account::Account::migrate_user_database(&user).map_err(|e| {
            fastn_account::AccountLoadError::DatabaseOpenFailed {
                path: user_path.clone(),
                source: Box::new(e),
            }
        })?;

        // Load aliases from the aliases directory
        let aliases_dir = account_dir.join("aliases");
        let mut aliases = Vec::new();

        if aliases_dir.exists() {
            // Get unique prefixes (ID52s) from the directory
            let mut seen_prefixes = std::collections::HashSet::new();

            let entries = std::fs::read_dir(&aliases_dir).map_err(|e| {
                fastn_account::AccountLoadError::AliasLoadingFailed {
                    id52: "aliases_directory".to_string(),
                    source: Box::new(e),
                }
            })?;

            for entry in entries {
                let entry =
                    entry.map_err(|e| fastn_account::AccountLoadError::AliasLoadingFailed {
                        id52: "directory_entry".to_string(),
                        source: Box::new(e),
                    })?;
                let path = entry.path();

                // Skip if not a file
                if !path.is_file() {
                    continue;
                }

                // Extract the prefix (ID52) from filename
                if let Some(stem) = path.file_stem() {
                    let prefix = stem.to_string_lossy().to_string();

                    // Skip if we've already processed this prefix
                    if !seen_prefixes.insert(prefix.clone()) {
                        continue;
                    }

                    // Use load_from_dir which handles both keyring and file loading
                    match fastn_id52::SecretKey::load_from_dir(&aliases_dir, &prefix) {
                        Ok((id52, secret_key)) => {
                            let public_key = secret_key.public_key();

                            // TODO: Load name from /-/aliases/{id52}/readme Automerge document
                            // TODO: Load reason from /-/aliases/{id52}/notes Automerge document
                            // For now, use placeholder values
                            aliases.push(fastn_account::Alias {
                                public_key,
                                secret_key,
                                name: format!("Alias {}", aliases.len() + 1), // Placeholder
                                reason: "Loaded alias".to_string(),           // Placeholder
                                is_primary: aliases.is_empty(), // First one is primary
                            });

                            tracing::debug!("Loaded alias: {}", id52);
                        }
                        Err(e) => {
                            tracing::warn!("Failed to load alias with prefix '{}': {}", prefix, e);
                            // Continue loading other aliases
                        }
                    }
                }
            }
        }

        if aliases.is_empty() {
            return Err(fastn_account::AccountLoadError::NoAliasesFound);
        }

        tracing::info!(
            "Loaded account with {} aliases from {account_dir:?}",
            aliases.len()
        );

        Ok(Self {
            path: std::sync::Arc::new(account_dir.to_path_buf()),
            aliases: std::sync::Arc::new(tokio::sync::RwLock::new(aliases)),
            automerge: std::sync::Arc::new(tokio::sync::Mutex::new(automerge_db)),
            mail,
            user: std::sync::Arc::new(tokio::sync::Mutex::new(user)),
        })
    }
}
