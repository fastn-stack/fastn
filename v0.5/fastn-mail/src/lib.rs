//! # fastn-mail
//!
//! Email handling and storage for FASTN accounts.
//!
//! This crate provides email-related functionality including:
//! - **Mail Database**: SQLite schema for email storage
//! - **Automerge Documents**: Mail configuration documents
//! - **Email Management**: Core email handling logic
//!
//! ## Architecture
//!
//! Each account has a dedicated mail.sqlite database with:
//! - `fastn_emails` table for email storage
//! - `fastn_email_peers` table for peer tracking
//! - Folder-based organization (inbox, sent, drafts, trash)
//!
//! ## Integration
//!
//! This crate integrates with:
//! - `fastn-automerge`: For mail configuration documents
//! - `fastn-id52`: For email peer identification
//! - `rusqlite`: For email storage

extern crate self as fastn_mail;

pub mod automerge;
pub mod database;
pub mod errors;

// Re-export main types
pub use errors::{
    MailCreateError, MailDatabaseConnectionError, MailLoadError, MigrateMailDatabaseError,
};

pub use automerge::DefaultMail;

/// Mail handling for a FASTN account
#[derive(Debug, Clone)]
pub struct Mail {
    /// Path to the mail database
    pub(crate) db_path: std::path::PathBuf,
    /// Mail database connection
    pub(crate) connection: std::sync::Arc<tokio::sync::Mutex<rusqlite::Connection>>,
}

impl Mail {
    /// Create new mail system for an account
    pub async fn create(account_path: &std::path::Path) -> Result<Self, MailCreateError> {
        let mail_path = account_path.join("mail.sqlite");

        // Create mail directory structure
        database::create_mail_directories(account_path).map_err(|e| {
            MailCreateError::DirectoryCreationFailed {
                path: account_path.join("mails"),
                source: e,
            }
        })?;

        // Create and connect to database
        let connection = database::create_connection(&mail_path)
            .map_err(|e| MailCreateError::ConnectionFailed { source: e })?;

        // Run migrations
        database::migrate_database(&connection)
            .map_err(|e| MailCreateError::MigrationFailed { source: e })?;

        Ok(Self {
            db_path: mail_path,
            connection: std::sync::Arc::new(tokio::sync::Mutex::new(connection)),
        })
    }

    /// Load existing mail system for an account
    pub async fn load(account_path: &std::path::Path) -> Result<Self, MailLoadError> {
        let mail_path = account_path.join("mail.sqlite");

        // Check if database exists
        if !database::database_exists(&mail_path) {
            return Err(MailLoadError::DatabaseNotFound { path: mail_path });
        }

        // Connect to existing database
        let connection = database::create_connection(&mail_path).map_err(|e| {
            MailLoadError::ConnectionFailed {
                path: mail_path.clone(),
                source: Box::new(e),
            }
        })?;

        // Run migrations
        database::migrate_database(&connection).map_err(|e| MailLoadError::ConnectionFailed {
            path: mail_path.clone(),
            source: Box::new(e),
        })?;

        Ok(Self {
            db_path: mail_path,
            connection: std::sync::Arc::new(tokio::sync::Mutex::new(connection)),
        })
    }

    /// Get the database path
    pub fn db_path(&self) -> &std::path::Path {
        &self.db_path
    }

    /// Get a reference to the database connection
    pub fn connection(&self) -> &std::sync::Arc<tokio::sync::Mutex<rusqlite::Connection>> {
        &self.connection
    }

    /// Create a test mail system in memory (for testing only)
    pub fn create_test() -> Self {
        let connection = rusqlite::Connection::open_in_memory().unwrap();
        Self {
            db_path: std::path::PathBuf::from(":memory:"),
            connection: std::sync::Arc::new(tokio::sync::Mutex::new(connection)),
        }
    }
}
