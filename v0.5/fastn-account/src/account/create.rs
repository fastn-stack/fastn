
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
    pub async fn create(parent_dir: &std::path::Path) -> eyre::Result<Self> {
        use eyre::WrapErr;

        // Generate primary alias
        let secret_key = fastn_id52::SecretKey::generate();
        let public_key = secret_key.public_key();
        let id52 = public_key.to_string();

        // Account folder is named by primary alias ID52
        let account_path = parent_dir.join(&id52);

        // Create account directory structure
        std::fs::create_dir_all(&account_path)
            .wrap_err_with(|| format!("Failed to create account directory at {account_path:?}"))?;

        // Create subdirectories
        std::fs::create_dir_all(account_path.join("aliases"))
            .wrap_err("Failed to create aliases directory")?;
        std::fs::create_dir_all(account_path.join("mails/default/inbox"))
            .wrap_err("Failed to create mail directories")?;
        std::fs::create_dir_all(account_path.join("mails/default/sent"))
            .wrap_err("Failed to create sent directory")?;
        std::fs::create_dir_all(account_path.join("mails/default/drafts"))
            .wrap_err("Failed to create drafts directory")?;
        std::fs::create_dir_all(account_path.join("mails/default/trash"))
            .wrap_err("Failed to create trash directory")?;

        // Store keys based on SKIP_KEYRING environment variable
        let key_path = account_path.join("aliases").join(&id52);
        if std::env::var("SKIP_KEYRING")
            .map(|v| v == "true")
            .unwrap_or(false)
        {
            // Save private key to file
            tracing::info!("SKIP_KEYRING set, saving private key to file");
            let private_key_file = key_path.with_extension("private-key");
            std::fs::write(&private_key_file, secret_key.to_string())
                .wrap_err("Failed to write private key file")?;
        } else {
            // Save public key to file and store private key in keyring
            let id52_file = key_path.with_extension("id52");
            std::fs::write(&id52_file, &id52).wrap_err("Failed to write ID52 file")?;

            // Store in keyring
            secret_key
                .store_in_keyring()
                .wrap_err("Failed to store private key in keyring")?;
        }

        // Create and initialize three databases
        let automerge_path = account_path.join("automerge.sqlite");
        let mail_path = account_path.join("mail.sqlite");
        let user_path = account_path.join("db.sqlite");

        // Initialize automerge database
        let automerge_db = fastn_automerge::Db::init(&automerge_path, &public_key)
            .wrap_err("Failed to initialize automerge database")?;

        let mail =
            rusqlite::Connection::open(&mail_path).wrap_err("Failed to create mail database")?;
        let user =
            rusqlite::Connection::open(&user_path).wrap_err("Failed to create user database")?;

        // Run database migrations for mail and user databases
        Self::migrate_mail_database(&mail).wrap_err("Failed to run mail database migrations")?;
        Self::migrate_user_database(&user).wrap_err("Failed to run user database migrations")?;

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
            mail: std::sync::Arc::new(tokio::sync::Mutex::new(mail)),
            user: std::sync::Arc::new(tokio::sync::Mutex::new(user)),
        })
    }

    /// Run migrations for mail database
    pub(crate) fn migrate_mail_database(conn: &rusqlite::Connection) -> eyre::Result<()> {
        use eyre::WrapErr;

        conn.execute_batch(
            r#"
            -- Email index
            CREATE TABLE IF NOT EXISTS fastn_emails (
                email_id          TEXT PRIMARY KEY,
                folder            TEXT NOT NULL,
                original_to       TEXT NOT NULL,
                from_address      TEXT NOT NULL,
                to_addresses      TEXT NOT NULL,
                cc_addresses      TEXT,
                bcc_addresses     TEXT,
                received_at_alias TEXT,
                sent_from_alias   TEXT,
                subject           TEXT,
                body_preview      TEXT,
                has_attachments   INTEGER DEFAULT 0,
                file_path         TEXT NOT NULL UNIQUE,
                size_bytes        INTEGER NOT NULL,
                message_id        TEXT,
                in_reply_to       TEXT,
                email_references  TEXT,
                date_sent         INTEGER,
                date_received     INTEGER,
                is_read           INTEGER DEFAULT 0,
                is_starred        INTEGER DEFAULT 0,
                flags             TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_folder ON fastn_emails(folder);
            CREATE INDEX IF NOT EXISTS idx_date ON fastn_emails(date_received DESC);
            CREATE INDEX IF NOT EXISTS idx_message_id ON fastn_emails(message_id);

            -- Email peers
            CREATE TABLE IF NOT EXISTS fastn_email_peers (
                peer_alias        TEXT PRIMARY KEY,
                last_seen         INTEGER,
                endpoint          BLOB,
                our_alias_used    TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_our_alias ON fastn_email_peers(our_alias_used);
            "#,
        )
        .wrap_err("Failed to initialize mail database schema")?;

        Ok(())
    }

    /// Create initial Automerge documents for a new account
    fn create_initial_documents(
        db: &fastn_automerge::Db,
        public_key: &fastn_id52::PublicKey,
        name: Option<String>,
        bio: Option<String>,
    ) -> eyre::Result<()> {
        let id52 = public_key.id52();
        
        // 1. Create /-/mails/default document with password and service flags
        let password = crate::auth::generate_password();
        let password_hash = crate::auth::hash_password(&password)?;

        // Print password to stdout (one-time display)
        println!("==================================================");
        println!("Account created successfully!");
        println!("ID52: {id52}");
        println!("Username: default@{id52}");
        println!("Password: {password}");
        println!("==================================================");
        println!("IMPORTANT: Save this password - it cannot be recovered!");
        println!("==================================================");

        // Create default mail document using type-safe API
        let default_mail = crate::automerge::DefaultMail {
            password_hash,
            is_active: true,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        };
        default_mail.save(db)?;

        // 2. Create /-/aliases/{id52}/readme document (public info)
        let alias_readme = crate::automerge::AliasReadme {
            alias: *public_key,
            name,
            bio,
        };
        alias_readme.save(db)?;

        // 3. Create /-/aliases/{id52}/notes document (private notes)
        // For our own account, we don't need notes initially
        let alias_notes = crate::automerge::AliasNotes {
            alias: *public_key,
            nickname: None,
            notes: None,
            relationship_started_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        };
        alias_notes.save(db)?;

        Ok(())
    }

    /// Run migrations for user database
    pub(crate) fn migrate_user_database(conn: &rusqlite::Connection) -> eyre::Result<()> {
        use eyre::WrapErr;

        // User database starts empty - user can create their own tables
        // We might add fastn_ prefixed system tables here in the future
        conn.execute_batch(
            r#"
            -- User database for application-specific tables
            -- Currently empty - user can create their own tables
            PRAGMA journal_mode = WAL;
            "#,
        )
        .wrap_err("Failed to initialize user database")?;

        Ok(())
    }
}
