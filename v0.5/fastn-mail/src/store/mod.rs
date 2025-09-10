//! # Email Store Module
//!
//! Centralized storage operations for fastn email system.
//!
//! ## Organization
//! Each Store method is implemented in its own focused file:

mod create;
mod create_bounce_message;
mod get_emails_for_peer;
mod get_pending_deliveries;
mod mark_delivered_to_peer;
pub mod smtp_receive;

/// Email store for account-specific email operations
#[derive(Debug, Clone)]
pub struct Store {
    /// Path to account directory
    pub(crate) account_path: std::path::PathBuf,
    /// Mail database connection
    pub(crate) connection: std::sync::Arc<tokio::sync::Mutex<rusqlite::Connection>>,
}

impl Store {
    /// Get account path for file operations
    pub fn account_path(&self) -> &std::path::Path {
        &self.account_path
    }

    /// Get database connection for email operations
    pub fn connection(&self) -> &std::sync::Arc<tokio::sync::Mutex<rusqlite::Connection>> {
        &self.connection
    }

    /// Create a test store in memory (for testing only)
    pub fn create_test() -> Self {
        let connection = rusqlite::Connection::open_in_memory().unwrap();
        fastn_mail::database::create_schema(&connection).unwrap();

        // Use temp directory for test files
        let temp_dir = std::env::temp_dir().join(format!("fastn-mail-test-{}", std::process::id()));

        Self {
            account_path: temp_dir,
            connection: std::sync::Arc::new(tokio::sync::Mutex::new(connection)),
        }
    }
}
