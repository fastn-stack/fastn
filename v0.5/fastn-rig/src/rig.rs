use automerge::ReadDoc;
use automerge::transaction::Transactable;

impl fastn_rig::Rig {
    /// Create a new Rig and initialize the fastn_home with the first account
    /// Returns (Rig, AccountManager, primary_account_id52)
    pub async fn create(
        fastn_home: std::path::PathBuf,
    ) -> eyre::Result<(Self, fastn_account::AccountManager, String)> {
        use eyre::WrapErr;
        use std::str::FromStr;

        // Create fastn_home directory if it doesn't exist
        std::fs::create_dir_all(&fastn_home)
            .wrap_err_with(|| format!("Failed to create fastn_home at {fastn_home:?}"))?;

        // Generate rig's secret key
        let secret_key = fastn_id52::SecretKey::generate();
        let id52 = secret_key.id52();

        // Create and store rig's key
        let rig_key_path = fastn_home.join("rig");
        std::fs::create_dir_all(&rig_key_path).wrap_err("Failed to create rig directory")?;

        // Store key based on SKIP_KEYRING
        if std::env::var("SKIP_KEYRING")
            .map(|v| v == "true")
            .unwrap_or(false)
        {
            let private_key_file = rig_key_path.join("rig.private-key");
            std::fs::write(&private_key_file, secret_key.to_string())
                .wrap_err("Failed to write rig private key")?;
        } else {
            let id52_file = rig_key_path.join("rig.id52");
            std::fs::write(&id52_file, &id52).wrap_err("Failed to write rig ID52")?;
            secret_key
                .store_in_keyring()
                .wrap_err("Failed to store rig key in keyring")?;
        }

        tracing::info!("Creating new Rig with ID52: {}", id52);

        // Create automerge database
        let automerge_path = fastn_home.join("automerge.sqlite");
        let automerge_conn = rusqlite::Connection::open(&automerge_path)
            .wrap_err("Failed to create automerge database")?;

        // Initialize automerge schema
        fastn_automerge::migration::initialize_database(&automerge_conn)
            .wrap_err("Failed to initialize automerge database")?;

        // Create AccountManager and first account
        let (account_manager, primary_id52) =
            fastn_account::AccountManager::create(fastn_home.clone())
                .await
                .wrap_err("Failed to create AccountManager and first account")?;

        // Parse owner key
        let owner = fastn_id52::PublicKey::from_str(&primary_id52)
            .wrap_err("Failed to parse owner public key")?;

        // Create rig config document with owner
        let mut config_doc = automerge::AutoCommit::new();
        config_doc.put(
            automerge::ROOT,
            "created_at",
            chrono::Utc::now().timestamp(),
        )?;
        config_doc.put(automerge::ROOT, "owner_id52", primary_id52.clone())?;

        let _config_path = format!("/-/rig/{id52}/config");
        // TODO: Integrate with new fastn-automerge Db API
        // fastn_automerge::create_and_save_document(&automerge_conn, &config_path, config_doc)
        //     .wrap_err("Failed to create rig config document")?;

        tracing::info!(
            "Created new Rig with ID52: {} (owner: {})",
            id52,
            primary_id52
        );

        let rig = Self {
            path: fastn_home,
            secret_key,
            owner,
            automerge: std::sync::Arc::new(tokio::sync::Mutex::new(automerge_conn)),
        };

        Ok((rig, account_manager, primary_id52))
    }

    /// Load an existing Rig from fastn_home
    pub fn load(fastn_home: std::path::PathBuf) -> eyre::Result<Self> {
        use eyre::WrapErr;
        use std::str::FromStr;

        // Load rig's secret key
        let rig_key_path = fastn_home.join("rig");
        let (rig_id52, secret_key) = fastn_id52::SecretKey::load_from_dir(&rig_key_path, "rig")
            .wrap_err("Failed to load rig secret key")?;

        // Open automerge database
        let automerge_path = fastn_home.join("automerge.sqlite");
        let automerge_conn = if automerge_path.exists() {
            let conn = rusqlite::Connection::open(&automerge_path)
                .wrap_err("Failed to open automerge database")?;
            // Run migrations in case schema has changed
            fastn_automerge::migration::initialize_database(&conn)
                .wrap_err("Failed to migrate automerge database")?;
            conn
        } else {
            // Create automerge database for older rigs
            let conn = rusqlite::Connection::open(&automerge_path)
                .wrap_err("Failed to create automerge database")?;
            fastn_automerge::migration::initialize_database(&conn)
                .wrap_err("Failed to initialize automerge database")?;

            return Err(eyre::eyre!(
                "Rig automerge database missing - corrupt installation"
            ));
        };

        // Load owner from Automerge document
        let _config_path = format!("/-/rig/{rig_id52}/config");
        // TODO: Integrate with new fastn-automerge Db API
        // let config_doc = fastn_automerge::load_document(&automerge_conn, &config_path)?
        //     .ok_or_else(|| eyre::eyre!("Rig config document not found at {}", config_path))?;
        struct TempWrapper {
            doc: automerge::AutoCommit,
        }
        let config_doc = TempWrapper {
            doc: automerge::AutoCommit::new(),
        }; // Temporary placeholder

        let owner_id52 = config_doc
            .doc
            .get(automerge::ROOT, "owner_id52")
            .wrap_err("Failed to get owner_id52 from config")?
            .ok_or_else(|| eyre::eyre!("owner_id52 not found in rig config"))?;

        let owner_id52_str = match &owner_id52 {
            (automerge::Value::Scalar(s), _) => match s.as_ref() {
                automerge::ScalarValue::Str(str_val) => str_val.to_string(),
                _ => return Err(eyre::eyre!("owner_id52 is not a string")),
            },
            _ => return Err(eyre::eyre!("owner_id52 is not a scalar value")),
        };

        let owner = fastn_id52::PublicKey::from_str(&owner_id52_str)
            .wrap_err("Failed to parse owner public key")?;

        tracing::info!(
            "Loaded Rig with ID52: {} (owner: {})",
            rig_id52,
            owner_id52_str
        );

        Ok(Self {
            path: fastn_home,
            secret_key,
            owner,
            automerge: std::sync::Arc::new(tokio::sync::Mutex::new(automerge_conn)),
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

    /// Check if an endpoint is online
    pub async fn is_endpoint_online(&self, id52: &str) -> bool {
        let _automerge = self.automerge.lock().await;
        let _endpoint_path = format!("/-/endpoints/{id52}/status");

        // TODO: Integrate with new fastn-automerge Db API
        // match fastn_automerge::load_document(&automerge, &endpoint_path) {
        match Ok(None::<automerge::AutoCommit>)
            as Result<Option<automerge::AutoCommit>, rusqlite::Error>
        {
            Ok(Some(doc)) => doc
                .get(automerge::ROOT, "is_online")
                .ok()
                .and_then(|v| v)
                .and_then(|(v, _)| match v {
                    automerge::Value::Scalar(s) => match s.as_ref() {
                        automerge::ScalarValue::Boolean(b) => Some(*b),
                        _ => None,
                    },
                    _ => None,
                })
                .unwrap_or(false),
            _ => false,
        }
    }

    /// Set endpoint online status
    pub async fn set_endpoint_online(&self, id52: &str, online: bool) {
        let _automerge = self.automerge.lock().await;
        let _endpoint_path = format!("/-/endpoints/{id52}/status");

        // TODO: Integrate with new fastn-automerge Db API
        // let mut doc = match fastn_automerge::load_document(&automerge, &endpoint_path) {
        let mut doc = match Ok(None::<automerge::AutoCommit>)
            as Result<Option<automerge::AutoCommit>, rusqlite::Error>
        {
            Ok(Some(stored)) => stored,
            _ => {
                let mut doc = automerge::AutoCommit::new();
                doc.put(automerge::ROOT, "id52", id52).ok();
                doc.put(
                    automerge::ROOT,
                    "created_at",
                    chrono::Utc::now().timestamp(),
                )
                .ok();
                doc
            }
        };

        doc.put(automerge::ROOT, "is_online", online).ok();
        doc.put(
            automerge::ROOT,
            "updated_at",
            chrono::Utc::now().timestamp(),
        )
        .ok();

        if online {
            // TODO: Integrate with new fastn-automerge Db API
            // fastn_automerge::update_document(&automerge, &endpoint_path, &mut doc).ok();
        } else {
            // TODO: Integrate with new fastn-automerge Db API
            // fastn_automerge::create_and_save_document(&automerge, &endpoint_path, doc).ok();
        }
    }

    /// Get the current entity's ID52
    pub async fn get_current(&self) -> Option<String> {
        let _automerge = self.automerge.lock().await;
        let rig_id52 = self.id52();
        let _config_path = format!("/-/rig/{rig_id52}/config");

        // TODO: Integrate with new fastn-automerge Db API
        // match fastn_automerge::load_document(&automerge, &config_path) {
        match Ok(None::<automerge::AutoCommit>)
            as Result<Option<automerge::AutoCommit>, rusqlite::Error>
        {
            Ok(Some(doc)) => doc
                .get(automerge::ROOT, "current_entity")
                .ok()
                .and_then(|v| v)
                .and_then(|(v, _)| match v {
                    automerge::Value::Scalar(s) => match s.as_ref() {
                        automerge::ScalarValue::Str(str_val) => Some(str_val.to_string()),
                        _ => None,
                    },
                    _ => None,
                }),
            _ => None,
        }
    }

    /// Set the current entity
    pub async fn set_current(&self, id52: &str) -> eyre::Result<()> {
        let _automerge = self.automerge.lock().await;
        let rig_id52 = self.id52();
        let _config_path = format!("/-/rig/{rig_id52}/config");

        // TODO: Integrate with new fastn-automerge Db API
        // let mut doc = fastn_automerge::load_document(&automerge, &config_path)?
        //     .ok_or_else(|| eyre::eyre!("Rig config not found"))?
        //     .doc;
        let mut doc = automerge::AutoCommit::new(); // Temporary placeholder

        doc.put(automerge::ROOT, "current_entity", id52)?;
        doc.put(
            automerge::ROOT,
            "current_set_at",
            chrono::Utc::now().timestamp(),
        )?;

        // TODO: Integrate with new fastn-automerge Db API
        // fastn_automerge::update_document(&automerge, &config_path, &mut doc)
        //     .wrap_err("Failed to update current entity")?;

        tracing::info!("Set current entity to {}", id52);
        Ok(())
    }
}
