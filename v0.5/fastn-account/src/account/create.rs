impl fastn_account::Account {
    /// Creates a new account in the specified parent directory.
    ///
    /// This will:
    /// 1. Generate a new primary alias (ID52 identity)
    /// 2. Create account directory named by the primary alias ID52
    /// 3. Initialize three SQLite databases (automerge, mail, user)
    /// 4. Create Automerge documents for config and alias
    /// 5. Store keys (in keyring or files based on SKIP_KEYRING)
    ///
    /// # Arguments
    ///
    /// * `parent_dir` - Parent directory where account folder will be created (from fastn crate)
    ///
    /// # Returns
    ///
    /// The newly created Account
    ///
    /// # Errors
    ///
    /// Returns error if directory creation or database initialization fails
    pub async fn create(
        parent_dir: &std::path::Path,
    ) -> Result<Self, fastn_account::AccountCreateError> {
        // Generate primary alias
        let secret_key = fastn_id52::SecretKey::generate();
        let public_key = secret_key.public_key();
        let id52 = public_key.to_string();

        // Account folder is named by primary alias ID52
        let account_path = parent_dir.join(&id52);

        // Check if account already exists
        if account_path.exists() {
            return Err(fastn_account::AccountCreateError::AccountAlreadyExists {
                path: account_path,
            });
        }

        // Create account directory structure
        std::fs::create_dir_all(&account_path).map_err(|e| {
            fastn_account::AccountCreateError::DirectoryCreationFailed {
                path: account_path.clone(),
                source: e,
            }
        })?;

        // Create subdirectories
        let aliases_path = account_path.join("aliases");
        std::fs::create_dir_all(&aliases_path).map_err(|e| {
            fastn_account::AccountCreateError::DirectoryCreationFailed {
                path: aliases_path,
                source: e,
            }
        })?;

        // Store keys based on SKIP_KEYRING environment variable
        let key_path = account_path.join("aliases").join(&id52);

        let skip_keyring = match std::env::var("SKIP_KEYRING") {
            Ok(value) => match value.to_lowercase().as_str() {
                "true" | "yes" | "1" | "on" => {
                    tracing::info!("SKIP_KEYRING={} interpreted as true", value);
                    true
                }
                "false" | "no" | "0" | "off" => {
                    tracing::info!("SKIP_KEYRING={} interpreted as false", value);
                    false
                }
                _ => {
                    tracing::warn!(
                        "SKIP_KEYRING={} is not a recognized value. Expected: true/false/yes/no/1/0/on/off. Defaulting to true.",
                        value
                    );
                    true
                }
            },
            Err(_) => {
                tracing::info!("SKIP_KEYRING not set, defaulting to true (keyring not working)");
                true // Default to true since keyring is not working
            }
        };

        if skip_keyring {
            // Save private key to file
            tracing::info!("SKIP_KEYRING set, saving private key to file");
            let private_key_file = key_path.with_extension("private-key");
            std::fs::write(&private_key_file, secret_key.to_string()).map_err(|e| {
                fastn_account::AccountCreateError::FileWriteFailed {
                    path: private_key_file,
                    source: e,
                }
            })?;
        } else {
            // Save public key to file and store private key in keyring
            let id52_file = key_path.with_extension("id52");
            std::fs::write(&id52_file, &id52).map_err(|e| {
                fastn_account::AccountCreateError::FileWriteFailed {
                    path: id52_file,
                    source: e,
                }
            })?;

            // Store in keyring
            secret_key.store_in_keyring().map_err(|_| {
                fastn_account::AccountCreateError::KeyringStorageFailed { id52: id52.clone() }
            })?;
        }

        // Create and initialize databases
        let automerge_path = account_path.join("automerge.sqlite");
        let user_path = account_path.join("db.sqlite");

        // Initialize automerge database
        let automerge_db =
            fastn_automerge::Db::init(&automerge_path, &public_key).map_err(|e| {
                fastn_account::AccountCreateError::AutomergeInitFailed {
                    source: Box::new(e),
                }
            })?;

        // Create mail system
        let mail = fastn_mail::Store::create(&account_path)
            .await
            .map_err(|e| fastn_account::AccountCreateError::MailCreationFailed { source: e })?;

        let user = rusqlite::Connection::open(&user_path).map_err(|_| {
            fastn_account::AccountCreateError::DatabaseConnectionFailed {
                path: user_path.clone(),
            }
        })?;

        // Run user database migrations
        Self::migrate_user_database(&user)
            .map_err(|e| fastn_account::AccountCreateError::UserMigrationFailed { source: e })?;

        // Create primary alias
        let primary_alias = fastn_account::Alias {
            public_key,
            secret_key,
            name: "Primary".to_string(), // TODO: Allow customization
            reason: "Primary account".to_string(), // TODO: Allow customization
            is_primary: true,
        };

        // Create Automerge documents for the account using type-safe API
        Self::create_initial_documents(&automerge_db, &public_key, None, None)?;

        tracing::info!("Created new account with primary alias: {}", id52);

        // Create account instance
        Ok(Self {
            path: std::sync::Arc::new(account_path),
            aliases: std::sync::Arc::new(tokio::sync::RwLock::new(vec![primary_alias])),
            automerge: std::sync::Arc::new(tokio::sync::Mutex::new(automerge_db)),
            mail,
            user: std::sync::Arc::new(tokio::sync::Mutex::new(user)),
        })
    }

    /// Create initial Automerge documents for a new account
    fn create_initial_documents(
        db: &fastn_automerge::Db,
        public_key: &fastn_id52::PublicKey,
        name: Option<String>,
        bio: Option<String>,
    ) -> Result<(), fastn_account::CreateInitialDocumentsError> {
        let id52 = public_key.id52();

        // 1. Create /-/mails/default document with password and service flags
        let password = fastn_account::auth::generate_password();
        let password_hash = fastn_account::auth::hash_password(&password).map_err(|e| {
            fastn_account::CreateInitialDocumentsError::AccountConfigCreationFailed {
                source: Box::new(e),
            }
        })?;

        // Print password to stdout (one-time display)
        println!("==================================================");
        println!("Account created successfully!");
        println!("ID52: {id52}");
        println!("Username: default@{id52}.com");
        println!("Password: {password}");
        println!("==================================================");
        println!("IMPORTANT: Save this password - it cannot be recovered!");
        println!("==================================================");

        // Create default mail document using type-safe API
        let default_mail = fastn_account::automerge::DefaultMail {
            password_hash,
            is_active: true,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        };
        default_mail.save(db).map_err(|e| {
            fastn_account::CreateInitialDocumentsError::AccountConfigCreationFailed {
                source: Box::new(e),
            }
        })?;

        // 2. Create /-/aliases/{id52}/readme document (public info)
        let alias_readme = fastn_account::automerge::AliasReadme {
            alias: *public_key,
            name,
            bio,
        };
        alias_readme.save(db).map_err(|e| {
            fastn_account::CreateInitialDocumentsError::AliasDocumentCreationFailed {
                source: Box::new(e),
            }
        })?;

        // 3. Create /-/aliases/{id52}/notes document (private notes)
        // For our own account, we don't need notes initially
        let alias_notes = fastn_account::automerge::AliasNotes {
            alias: *public_key,
            nickname: None,
            notes: None,
            relationship_started_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            first_connected_to: *public_key, // Self-referential for our own account
            allow_mail: true,                // Default: allow mail
        };
        alias_notes.save(db).map_err(|e| {
            fastn_account::CreateInitialDocumentsError::AliasDocumentCreationFailed {
                source: Box::new(e),
            }
        })?;

        Ok(())
    }

    /// Run migrations for user database
    pub(crate) fn migrate_user_database(
        conn: &rusqlite::Connection,
    ) -> Result<(), fastn_account::MigrateUserDatabaseError> {
        // User database starts empty - user can create their own tables
        // We might add fastn_ prefixed system tables here in the future
        conn.execute_batch(
            r#"
            -- User database for application-specific tables
            -- Currently empty - user can create their own tables
            PRAGMA journal_mode = WAL;
            "#,
        )
        .map_err(|e| {
            fastn_account::MigrateUserDatabaseError::SchemaInitializationFailed { source: e }
        })?;

        Ok(())
    }
}
