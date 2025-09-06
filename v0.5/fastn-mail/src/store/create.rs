//! # Store Creation and Loading

use fastn_mail::errors::*;

impl fastn_mail::Store {
    /// Create new email storage for an account
    pub async fn create(account_path: &std::path::Path) -> Result<Self, StoreCreateError> {
        let mail_db_path = account_path.join("mail.sqlite");

        // Create mail directory structure
        fastn_mail::database::create_directories(account_path).map_err(|e| {
            StoreCreateError::DirectoryCreationFailed {
                path: account_path.join("mails"),
                source: e,
            }
        })?;

        // Create and connect to database
        let connection = rusqlite::Connection::open(&mail_db_path).map_err(|e| {
            StoreCreateError::DatabaseCreationFailed {
                path: mail_db_path,
                source: e,
            }
        })?;

        // Create schema
        fastn_mail::database::create_schema(&connection)
            .map_err(|e| StoreCreateError::MigrationFailed { source: e })?;

        Ok(Self {
            account_path: account_path.to_path_buf(),
            connection: std::sync::Arc::new(tokio::sync::Mutex::new(connection)),
        })
    }

    /// Load existing email storage for an account
    pub async fn load(account_path: &std::path::Path) -> Result<Self, StoreLoadError> {
        let mail_db_path = account_path.join("mail.sqlite");

        // Check if database exists
        if !mail_db_path.exists() {
            return Err(StoreLoadError::DatabaseNotFound { path: mail_db_path });
        }

        // Connect to existing database
        let connection = rusqlite::Connection::open(&mail_db_path).map_err(|e| {
            StoreLoadError::DatabaseOpenFailed {
                path: mail_db_path,
                source: e,
            }
        })?;

        // Run migrations (idempotent)
        fastn_mail::database::create_schema(&connection)
            .map_err(|e| StoreLoadError::MigrationFailed { source: e })?;

        Ok(Self {
            account_path: account_path.to_path_buf(),
            connection: std::sync::Arc::new(tokio::sync::Mutex::new(connection)),
        })
    }
}
