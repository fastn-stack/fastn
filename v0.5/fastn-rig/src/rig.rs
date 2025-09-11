use std::str::FromStr;

impl fastn_rig::Rig {
    /// Check if a fastn_home directory is already initialized
    pub fn is_initialized(fastn_home: &std::path::Path) -> bool {
        let lock_path = fastn_home.join(".fastn.lock");
        lock_path.exists()
    }

    /// Create a new Rig and initialize the fastn_home with the first account
    /// Returns (Rig, AccountManager, primary_account_id52)
    pub async fn create(
        fastn_home: std::path::PathBuf,
    ) -> Result<(Self, fastn_account::AccountManager, String), fastn_rig::RigCreateError> {
        use std::str::FromStr;

        // Create fastn_home directory if it doesn't exist
        std::fs::create_dir_all(&fastn_home).map_err(|e| {
            fastn_rig::RigCreateError::FastnHomeCreation {
                path: fastn_home.clone(),
                source: e,
            }
        })?;

        // Create the lock file to mark fastn_home as initialized
        let lock_path = fastn_home.join(".fastn.lock");
        std::fs::write(&lock_path, "").map_err(|e| {
            fastn_rig::RigCreateError::KeyFileWrite {
                path: lock_path,
                source: e,
            }
        })?;

        // Generate rig's secret key
        let secret_key = fastn_id52::SecretKey::generate();
        let id52 = secret_key.id52();

        // Create and store rig's key
        let rig_key_path = fastn_home.join("rig");
        std::fs::create_dir_all(&rig_key_path).map_err(|e| {
            fastn_rig::RigCreateError::FastnHomeCreation {
                path: rig_key_path.clone(),
                source: e,
            }
        })?;

        // Store key based on SKIP_KEYRING
        if std::env::var("SKIP_KEYRING")
            .map(|v| v == "true")
            .unwrap_or(false)
        {
            let private_key_file = rig_key_path.join("rig.private-key");
            std::fs::write(&private_key_file, secret_key.to_string()).map_err(|e| {
                fastn_rig::RigCreateError::KeyFileWrite {
                    path: private_key_file,
                    source: e,
                }
            })?;
        } else {
            let id52_file = rig_key_path.join("rig.id52");
            std::fs::write(&id52_file, &id52).map_err(|e| {
                fastn_rig::RigCreateError::KeyFileWrite {
                    path: id52_file,
                    source: e,
                }
            })?;
            secret_key
                .store_in_keyring()
                .map_err(|_| fastn_rig::RigCreateError::KeyringStorage)?;
        }

        tracing::info!("Creating new Rig with ID52: {}", id52);

        // Initialize automerge database with rig's entity
        let automerge_path = rig_key_path.join("automerge.sqlite");

        eprintln!(
            "🔍 Debug: Initializing automerge DB at {}",
            automerge_path.display()
        );
        eprintln!("🔍 Debug: Rig entity = {}", secret_key.public_key());

        let automerge_db = fastn_automerge::Db::init(&automerge_path, &secret_key.public_key())
            .map_err(|e| fastn_rig::RigCreateError::AutomergeInit {
                source: Box::new(e),
            })?;

        eprintln!("🔍 Debug: Automerge DB initialized successfully");

        // Create AccountManager and first account
        eprintln!("🔍 Debug: Creating AccountManager...");
        let (account_manager, primary_id52) =
            fastn_account::AccountManager::create(fastn_home.clone())
                .await
                .map_err(|e| fastn_rig::RigCreateError::AccountManagerCreate { source: e })?;

        eprintln!("🔍 Debug: AccountManager created, primary_id52 = {primary_id52}");

        // Parse owner key
        let owner = fastn_id52::PublicKey::from_str(&primary_id52)
            .map_err(|_| fastn_rig::RigCreateError::OwnerKeyParsing)?;

        // Create rig config struct with all configuration data
        let rig_config = fastn_rig::automerge::RigConfig {
            rig: secret_key.public_key(), // Rig's own identity
            owner,                        // Account that owns this rig
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            current_entity: owner, // Owner is the initial current entity
            email_certificate: fastn_rig::automerge::EmailCertificate::SelfSigned,
        };

        // Store the complete config struct in the database
        rig_config.save(&automerge_db).map_err(|e| {
            fastn_rig::RigCreateError::RigConfigCreation {
                source: Box::new(e),
            }
        })?;

        tracing::info!(
            "Created new Rig with ID52: {} (owner: {})",
            id52,
            primary_id52
        );

        let rig = Self {
            path: fastn_home,
            secret_key,
            owner,
            automerge: std::sync::Arc::new(tokio::sync::Mutex::new(automerge_db)),
        };

        // Set the newly created account as current and online
        rig.set_entity_online(&primary_id52, true)
            .await
            .map_err(|e| fastn_rig::RigCreateError::RigConfigCreation {
                source: Box::new(e),
            })?;
        rig.set_current(&primary_id52).await.map_err(|e| {
            fastn_rig::RigCreateError::RigConfigCreation {
                source: Box::new(e),
            }
        })?;

        // Set the rig itself online by default
        let rig_id52 = rig.id52();
        rig.set_entity_online(&rig_id52, true).await.map_err(|e| {
            fastn_rig::RigCreateError::RigConfigCreation {
                source: Box::new(e),
            }
        })?;

        Ok((rig, account_manager, primary_id52))
    }

    /// Load an existing Rig from fastn_home
    pub fn load(fastn_home: std::path::PathBuf) -> Result<Self, fastn_rig::RigLoadError> {
        // Load rig's secret key
        let rig_key_path = fastn_home.join("rig");
        let (rig_id52, secret_key) = fastn_id52::SecretKey::load_from_dir(&rig_key_path, "rig")
            .map_err(|e| fastn_rig::RigLoadError::KeyLoading {
                path: rig_key_path.clone(),
                source: Box::new(e),
            })?;

        // Open existing automerge database
        let automerge_path = rig_key_path.join("automerge.sqlite");
        let automerge_db = fastn_automerge::Db::open(&automerge_path).map_err(|e| {
            fastn_rig::RigLoadError::AutomergeDatabaseOpen {
                path: automerge_path,
                source: Box::new(e),
            }
        })?;

        // Load owner from Automerge document using typed API
        let config = fastn_rig::automerge::RigConfig::load(&automerge_db, &secret_key.public_key())
            .map_err(|e| fastn_rig::RigLoadError::RigConfigLoad {
                source: Box::new(e),
            })?;

        let owner = config.owner;

        tracing::info!(
            "Loaded Rig with ID52: {} (owner: {})",
            rig_id52,
            owner.id52()
        );

        Ok(Self {
            path: fastn_home,
            secret_key,
            owner,
            automerge: std::sync::Arc::new(tokio::sync::Mutex::new(automerge_db)),
        })
    }

    /// Get the Rig's ID52
    pub fn id52(&self) -> String {
        self.secret_key.id52()
    }

    /// Get the Rig's public key
    pub fn public_key(&self) -> fastn_id52::PublicKey {
        self.secret_key.public_key()
    }

    /// Get the Rig's secret key (use with caution)
    pub fn secret_key(&self) -> &fastn_id52::SecretKey {
        &self.secret_key
    }

    /// Get the Rig's owner public key
    pub fn owner(&self) -> &fastn_id52::PublicKey {
        &self.owner
    }

    /// Check if an entity is online
    pub async fn is_entity_online(&self, id52: &str) -> Result<bool, fastn_rig::EntityStatusError> {
        let automerge_db = self.automerge.lock().await;

        // Parse entity ID52 to PublicKey for type safety
        let entity_key = fastn_id52::PublicKey::from_str(id52).map_err(|_| {
            fastn_rig::EntityStatusError::InvalidId52 {
                id52: id52.to_string(),
            }
        })?;

        let is_online = fastn_rig::automerge::EntityStatus::is_online(&automerge_db, &entity_key)
            .map_err(|e| fastn_rig::EntityStatusError::DatabaseAccessFailed {
            source: Box::new(e),
        })?;
        Ok(is_online)
    }

    /// Set entity online status
    pub async fn set_entity_online(
        &self,
        id52: &str,
        online: bool,
    ) -> Result<(), fastn_rig::EntityStatusError> {
        let automerge_db = self.automerge.lock().await;

        // Parse entity ID52 to PublicKey for type safety
        let entity_key = fastn_id52::PublicKey::from_str(id52).map_err(|_| {
            fastn_rig::EntityStatusError::InvalidId52 {
                id52: id52.to_string(),
            }
        })?;

        fastn_rig::automerge::EntityStatus::set_online(&automerge_db, &entity_key, online)
            .map_err(|e| fastn_rig::EntityStatusError::DatabaseAccessFailed {
                source: Box::new(e),
            })?;

        Ok(())
    }

    /// Get the current entity's ID52
    pub async fn get_current(&self) -> Result<String, fastn_rig::CurrentEntityError> {
        let automerge_db = self.automerge.lock().await;

        let current_entity = fastn_rig::automerge::RigConfig::get_current_entity(
            &automerge_db,
            &self.secret_key.public_key(),
        )
        .map_err(|e| fastn_rig::CurrentEntityError::DatabaseAccessFailed {
            source: Box::new(e),
        })?;
        Ok(current_entity.id52())
    }

    /// Set the current entity
    pub async fn set_current(&self, id52: &str) -> Result<(), fastn_rig::CurrentEntityError> {
        let automerge_db = self.automerge.lock().await;

        // Parse entity ID52 to PublicKey for type safety
        let entity_key = fastn_id52::PublicKey::from_str(id52).map_err(|_| {
            fastn_rig::CurrentEntityError::InvalidId52 {
                id52: id52.to_string(),
            }
        })?;

        fastn_rig::automerge::RigConfig::update_current_entity(
            &automerge_db,
            &self.secret_key.public_key(),
            &entity_key,
        )
        .map_err(|e| fastn_rig::CurrentEntityError::DatabaseAccessFailed {
            source: Box::new(e),
        })?;

        tracing::info!("Set current entity to {}", id52);
        Ok(())
    }
}

