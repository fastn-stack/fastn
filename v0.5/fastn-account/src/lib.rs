//! # fastn-account
//!
//! Account management for the fastn P2P network with support for multiple aliases.
//!
//! This crate provides account management where each account can have multiple
//! aliases (ID52 identities). This is different from fastn-entity which assumes
//! one ID52 per entity.

/// Thread-safe handle to an account
#[derive(Debug, Clone)]
pub struct Account {
    inner: std::sync::Arc<tokio::sync::Mutex<AccountInner>>,
}

/// Internal account data
#[derive(Debug)]
struct AccountInner {
    /// Path to the account's storage directory
    path: std::path::PathBuf,

    /// All aliases belonging to this account  
    aliases: Vec<Alias>,

    /// Database connection for storing Automerge documents and emails
    #[expect(unused)]
    conn: rusqlite::Connection,
}

/// Represents a single alias (ID52 identity) within an account
#[derive(Debug, Clone)]
pub struct Alias {
    /// The public key
    public_key: fastn_id52::PublicKey,

    /// The secret key
    secret_key: fastn_id52::SecretKey,

    /// Optional friendly name for this alias
    name: Option<String>,

    /// Whether this is the primary alias (first one created)
    is_primary: bool,
}

impl Account {
    /// Create a new account (for testing - real creation will be in separate module)
    #[cfg(test)]
    async fn new(path: std::path::PathBuf, aliases: Vec<Alias>) -> Self {
        // Create in-memory database for testing
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        Self {
            inner: std::sync::Arc::new(tokio::sync::Mutex::new(AccountInner {
                path,
                aliases,
                conn,
            })),
        }
    }

    /// Get the primary alias ID52 (used for folder naming)
    pub async fn primary_id52(&self) -> Option<String> {
        let inner = self.inner.lock().await;
        inner
            .aliases
            .iter()
            .find(|a| a.is_primary)
            .map(|a| a.id52())
    }

    /// Get the primary alias
    pub async fn primary_alias(&self) -> Option<Alias> {
        let inner = self.inner.lock().await;
        inner.aliases.iter().find(|a| a.is_primary).cloned()
    }

    /// Check if an ID52 belongs to this account
    pub async fn has_alias(&self, id52: &str) -> bool {
        let inner = self.inner.lock().await;
        inner.aliases.iter().any(|a| a.id52() == id52)
    }

    /// Get the account's storage path
    pub async fn path(&self) -> std::path::PathBuf {
        let inner = self.inner.lock().await;
        inner.path.clone()
    }

    /// Get all aliases (returns a clone)
    pub async fn aliases(&self) -> Vec<Alias> {
        let inner = self.inner.lock().await;
        inner.aliases.clone()
    }
}

impl Alias {
    /// Get the ID52 string for this alias
    pub fn id52(&self) -> String {
        self.public_key.to_string()
    }

    /// Get the public key
    pub fn public_key(&self) -> &fastn_id52::PublicKey {
        &self.public_key
    }

    /// Get the secret key (use with caution)
    pub fn secret_key(&self) -> &fastn_id52::SecretKey {
        &self.secret_key
    }

    /// Get the friendly name if set
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Check if this is the primary alias
    pub fn is_primary(&self) -> bool {
        self.is_primary
    }

    /// Sign a message with this alias's private key
    pub fn sign(&self, message: &[u8]) -> fastn_id52::Signature {
        self.secret_key.sign(message)
    }
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
            name: Some("test".to_string()),
            is_primary: true,
        };

        assert!(alias.is_primary());
        assert_eq!(alias.name(), Some("test"));
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
            name: Some("primary".to_string()),
            is_primary: true,
        };

        let account = Account::new(std::path::PathBuf::from("/tmp/test"), vec![alias]).await;

        // Account should be cloneable and Send + Sync
        let account2 = account.clone();
        let primary_id52 = account2.primary_id52().await.unwrap();
        assert!(account.has_alias(&primary_id52).await);

        // Both handles point to same data
        assert_eq!(account.path().await, account2.path().await);
    }
}
