//! # fastn-account
//!
//! Multi-alias account management for the FASTN P2P network.
//!
//! An Account in FASTN represents a user with potentially multiple identities (aliases).
//! Each alias has its own ID52 identity, allowing users to maintain separate personas
//! for different contexts (work, personal, etc.).
//!
//! ## Key Features
//!
//! - **Multiple Aliases**: Each account can have multiple ID52 identities
//! - **Three Databases**: Separation of concerns with automerge.sqlite, mail.sqlite, db.sqlite
//! - **Key Management**: Secure storage of private keys via keyring or files
//! - **Email Integration**: Built-in support for email handling per alias
//!
//! ## Architecture
//!
//! Each account is stored in a directory named by its primary alias ID52:
//! ```text
//! accounts/
//!   {primary-id52}/
//!     aliases/          # Keys for all aliases
//!     automerge.sqlite  # Automerge documents (config, etc.)
//!     mail.sqlite       # Email storage
//!     db.sqlite         # User data
//! ```
//!
//! ## Usage
//!
//! ```ignore
//! // Create a new account
//! let account = fastn_account::Account::create(&accounts_dir).await?;
//!
//! // Load existing account
//! let account = fastn_account::Account::load(&account_path).await?;
//! ```

extern crate self as fastn_account;

mod account;
mod account_manager;
mod alias;
pub mod auth;
pub mod automerge;
pub mod errors;

// Re-export specific error types
pub use errors::{
    HashPasswordError,
    VerifyPasswordError,
    AccountCreateError,
    AccountLoadError,
    AccountManagerCreateError,
    AccountManagerLoadError,
    GetAllEndpointsError,
    MigrateMailDatabaseError,
    MigrateUserDatabaseError,
    CreateInitialDocumentsError,
};

/// Thread-safe handle to an account
#[derive(Debug, Clone)]
pub struct Account {
    /// Path to the account's storage directory
    pub(crate) path: std::sync::Arc<std::path::PathBuf>,

    /// All aliases belonging to this account  
    pub(crate) aliases: std::sync::Arc<tokio::sync::RwLock<Vec<Alias>>>,

    /// Database connection for Automerge documents and configuration
    #[expect(unused)]
    pub(crate) automerge: std::sync::Arc<tokio::sync::Mutex<fastn_automerge::Db>>,

    /// Database connection for email storage
    #[expect(unused)]
    pub(crate) mail: std::sync::Arc<tokio::sync::Mutex<rusqlite::Connection>>,

    /// Database connection for user space data
    #[expect(unused)]
    pub(crate) user: std::sync::Arc<tokio::sync::Mutex<rusqlite::Connection>>,
}

/// Represents a single alias (ID52 identity) within an account
#[derive(Debug, Clone)]
pub struct Alias {
    /// The public key
    pub(crate) public_key: fastn_id52::PublicKey,

    /// The secret key
    pub(crate) secret_key: fastn_id52::SecretKey,

    /// Public name visible to others (from /-/aliases/{id52}/readme)
    /// This is what others see when they interact with this alias
    pub(crate) name: String,

    /// Private reason/note about why this alias exists (from /-/aliases/{id52}/notes)
    /// Only visible to the account owner - more important for personal reference
    /// e.g., "Company work alias", "Open source contributions", "Dating app"
    pub(crate) reason: String,

    /// Whether this is the primary alias (first one created)
    pub(crate) is_primary: bool,
}

/// Manages multiple accounts in a fastn_home directory
#[derive(Debug, Clone)]
pub struct AccountManager {
    /// Path to the fastn_home directory
    pub(crate) path: std::path::PathBuf,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alias_basics() {
        let secret_key = fastn_id52::SecretKey::generate();
        let public_key = secret_key.public_key();

        let alias = Alias {
            public_key,
            secret_key,
            name: "Work Profile".to_string(),
            reason: "Company work account".to_string(),
            is_primary: true,
        };

        assert!(alias.is_primary());
        assert_eq!(alias.name(), "Work Profile");
        assert_eq!(alias.reason(), "Company work account");
        assert!(!alias.id52().is_empty());

        // Test signing
        let message = b"test message";
        let signature = alias.sign(message);
        alias.public_key().verify(message, &signature).unwrap();
    }

    #[tokio::test]
    async fn test_account_thread_safety() {
        let secret_key = fastn_id52::SecretKey::generate();
        let alias = Alias {
            public_key: secret_key.public_key(),
            secret_key,
            name: "Primary".to_string(),
            reason: "Main account".to_string(),
            is_primary: true,
        };

        let account =
            Account::new_for_test(std::path::PathBuf::from("/tmp/test"), vec![alias]).await;

        // Account should be cloneable and Send + Sync
        let account2 = account.clone();
        let primary_id52 = account2.primary_id52().await.unwrap();
        assert!(account.has_alias(&primary_id52).await);

        // Both handles point to same data
        assert_eq!(account.path().await, account2.path().await);
    }
}
