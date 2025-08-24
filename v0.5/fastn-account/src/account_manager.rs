impl fastn_account::AccountManager {
    /// Creates a new AccountManager and initializes with a default account
    /// This should only be called when fastn_home is being initialized for the first time
    ///
    /// # Arguments
    ///
    /// * `fastn_home` - The fastn_home directory path (provided by fastn crate)
    ///
    /// Returns the AccountManager and the primary ID52 of the created account
    pub async fn create(fastn_home: std::path::PathBuf) -> Result<(Self, String), crate::AccountManagerCreateError> {
        tracing::info!("Creating AccountManager at {fastn_home:?}");

        let manager = Self { path: fastn_home };

        // Create accounts directory
        let accounts_dir = manager.path.join("accounts");
        std::fs::create_dir_all(&accounts_dir).map_err(|e| {
            crate::AccountManagerCreateError::AccountCreationFailed {
                source: crate::AccountCreateError::DirectoryCreationFailed {
                    path: accounts_dir.clone(),
                    source: e,
                }
            }
        })?;

        // Create default account
        println!("📝 Creating default account...");
        let account = fastn_account::Account::create(&accounts_dir).await.map_err(|e| {
            crate::AccountManagerCreateError::AccountCreationFailed { source: e }
        })?;

        let primary_id52 = account
            .primary_id52()
            .await
            .ok_or_else(|| crate::AccountManagerCreateError::PrimaryAccountIdNotFound)?;

        println!("✅ Created new account: {primary_id52}");

        Ok((manager, primary_id52))
    }

    /// Loads an existing AccountManager from the given fastn_home directory
    /// This should only be called when fastn_home already exists (lock file present)
    ///
    /// # Arguments
    ///
    /// * `fastn_home` - The fastn_home directory path (provided by fastn crate)
    pub async fn load(fastn_home: std::path::PathBuf) -> Result<Self, crate::AccountManagerLoadError> {
        tracing::info!("Loading AccountManager from {fastn_home:?}");

        let accounts_dir = fastn_home.join("accounts");
        if !accounts_dir.exists() {
            return Err(crate::AccountManagerLoadError::AccountsDirectoryNotFound {
                path: accounts_dir,
            });
        }

        // Test that we can read the directory
        std::fs::read_dir(&accounts_dir).map_err(|e| {
            crate::AccountManagerLoadError::AccountsScanFailed {
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
    ) -> Result<Vec<(String, Vec<u8>, std::path::PathBuf)>, crate::GetAllEndpointsError> {
        let accounts_dir = self.path.join("accounts");
        let mut all_endpoints = Vec::new();

        let entries = std::fs::read_dir(&accounts_dir).map_err(|e| {
            crate::GetAllEndpointsError::AccountsScanFailed {
                path: accounts_dir.clone(),
                source: e,
            }
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                crate::GetAllEndpointsError::AccountsScanFailed {
                    path: accounts_dir.clone(),
                    source: e,
                }
            })?;
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
                        return Err(crate::GetAllEndpointsError::AccountLoadFailed {
                            path,
                            source: e,
                        });
                    }
                }
            }
        }

        Ok(all_endpoints)
    }
}
