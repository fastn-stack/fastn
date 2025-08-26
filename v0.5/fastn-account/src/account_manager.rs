impl fastn_account::AccountManager {
    /// Creates a new AccountManager and initializes with a default account
    /// This should only be called when fastn_home is being initialized for the first time
    ///
    /// # Arguments
    ///
    /// * `fastn_home` - The fastn_home directory path (provided by fastn crate)
    ///
    /// Returns the AccountManager and the primary ID52 of the created account
    pub async fn create(
        fastn_home: std::path::PathBuf,
    ) -> Result<(Self, String), fastn_account::AccountManagerCreateError> {
        tracing::info!("Creating AccountManager at {fastn_home:?}");

        let manager = Self { path: fastn_home };

        // Create accounts directory
        let accounts_dir = manager.path.join("accounts");
        std::fs::create_dir_all(&accounts_dir).map_err(|e| {
            fastn_account::AccountManagerCreateError::AccountCreationFailed {
                source: fastn_account::AccountCreateError::DirectoryCreationFailed {
                    path: accounts_dir.clone(),
                    source: e,
                },
            }
        })?;

        // Create default account
        println!("📝 Creating default account...");
        let account = fastn_account::Account::create(&accounts_dir)
            .await
            .map_err(
                |e| fastn_account::AccountManagerCreateError::AccountCreationFailed { source: e },
            )?;

        let primary_id52 = account
            .primary_id52()
            .await
            .ok_or_else(|| fastn_account::AccountManagerCreateError::PrimaryAccountIdNotFound)?;

        println!("✅ Created new account: {primary_id52}");

        Ok((manager, primary_id52))
    }

    /// Loads an existing AccountManager from the given fastn_home directory
    /// This should only be called when fastn_home already exists (lock file present)
    ///
    /// # Arguments
    ///
    /// * `fastn_home` - The fastn_home directory path (provided by fastn crate)
    pub async fn load(
        fastn_home: std::path::PathBuf,
    ) -> Result<Self, fastn_account::AccountManagerLoadError> {
        tracing::info!("Loading AccountManager from {fastn_home:?}");

        let accounts_dir = fastn_home.join("accounts");
        if !accounts_dir.exists() {
            return Err(
                fastn_account::AccountManagerLoadError::AccountsDirectoryNotFound {
                    path: accounts_dir,
                },
            );
        }

        // Test that we can read the directory
        std::fs::read_dir(&accounts_dir).map_err(|e| {
            fastn_account::AccountManagerLoadError::AccountsScanFailed {
                path: accounts_dir.clone(),
                source: e,
            }
        })?;

        Ok(Self { path: fastn_home })
    }

    /// Get all endpoints from all accounts
    /// Returns a tuple of (endpoint_id52, secret_key_bytes, account_path)
    pub async fn get_all_endpoints(
        &self,
    ) -> Result<Vec<(String, Vec<u8>, std::path::PathBuf)>, fastn_account::GetAllEndpointsError>
    {
        let accounts_dir = self.path.join("accounts");
        let mut all_endpoints = Vec::new();

        let entries = std::fs::read_dir(&accounts_dir).map_err(|e| {
            fastn_account::GetAllEndpointsError::AccountsScanFailed {
                path: accounts_dir.clone(),
                source: e,
            }
        })?;

        for entry in entries {
            let entry =
                entry.map_err(
                    |e| fastn_account::GetAllEndpointsError::AccountsScanFailed {
                        path: accounts_dir.clone(),
                        source: e,
                    },
                )?;
            let path = entry.path();

            if path.is_dir() {
                match fastn_account::Account::load(&path).await {
                    Ok(account) => {
                        if let Some(primary_id52) = account.primary_id52().await {
                            println!("📂 Loaded account: {primary_id52}");

                            // Get all aliases (endpoints) for this account
                            for alias in account.aliases().await {
                                all_endpoints.push((
                                    alias.id52(),
                                    alias.secret_key().to_bytes().to_vec(),
                                    path.clone(),
                                ));
                            }
                        }
                    }
                    Err(e) => {
                        return Err(fastn_account::GetAllEndpointsError::AccountLoadFailed {
                            path,
                            source: e,
                        });
                    }
                }
            }
        }

        Ok(all_endpoints)
    }

    /// Find account that owns the given alias PublicKey
    pub async fn find_account_by_alias(
        &self,
        alias_key: &fastn_id52::PublicKey,
    ) -> Result<crate::Account, crate::FindAccountByAliasError> {
        let accounts_dir = self.path.join("accounts");

        let entries = std::fs::read_dir(&accounts_dir).map_err(|e| {
            crate::FindAccountByAliasError::AccountsScanFailed {
                path: accounts_dir.clone(),
                source: e,
            }
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| crate::FindAccountByAliasError::AccountsScanFailed {
                path: accounts_dir.clone(),
                source: e,
            })?;

            let account_path = entry.path();
            if account_path.is_dir() {
                match crate::Account::load(&account_path).await {
                    Ok(account) => {
                        if account.has_alias(&alias_key.id52()).await {
                            return Ok(account);
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to load account at {:?}: {}", account_path, e);
                        // Continue checking other accounts
                        continue;
                    }
                }
            }
        }

        Err(crate::FindAccountByAliasError::AccountNotFound {
            alias_id52: alias_key.id52(),
        })
    }

    /// Handle an incoming P2P message from another account
    pub async fn handle_account_message(
        &self,
        peer_id52: &fastn_id52::PublicKey,
        our_endpoint_id52: &fastn_id52::PublicKey,
        message: crate::AccountToAccountMessage,
    ) -> Result<(), crate::HandleAccountMessageError> {
        println!(
            "📨 Handling account message from {} to {} (message size: {} bytes)",
            peer_id52.id52(),
            our_endpoint_id52.id52(),
            message.size()
        );

        // 1. Find which account owns our_endpoint_id52
        let account = self
            .find_account_by_alias(our_endpoint_id52)
            .await
            .map_err(|e| crate::HandleAccountMessageError::AccountLookupFailed { source: e })?;

        println!("✅ Found account that owns endpoint {our_endpoint_id52}");

        // 2. Process the message based on type
        match message {
            crate::AccountToAccountMessage::Email { raw_message } => {
                println!("📧 Processing email message: {} bytes", raw_message.len());

                // 3. Store in appropriate mailbox (INBOX for incoming P2P email)
                // Note: smtp_receive should handle peer tracking internally
                let email_id = account.mail.smtp_receive(raw_message).await.map_err(|e| {
                    crate::HandleAccountMessageError::EmailStorageFailed {
                        source: Box::new(e),
                    }
                })?;

                println!("✅ Email stored with ID: {}", email_id);

                // 4. Ensure peer tracking (connection should have already called authorize_connection)
                // The peer notes document should already exist from connection establishment

                Ok(())
            }
        }
    }
}
