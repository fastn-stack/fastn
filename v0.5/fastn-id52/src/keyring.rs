//! Keyring integration for secure key storage.
//!
//! This module provides functionality for storing and retrieving secret keys
//! from the system keyring service. Keys are stored as hex-encoded strings
//! for better user experience with password managers.

/// Error returned when keyring operations fail.
///
/// This error can occur when accessing the system keyring, storing keys,
/// or retrieving keys from the keyring.
#[derive(Debug, Clone)]
pub enum KeyringError {
    /// The keyring service is not accessible
    Access(String),
    /// The requested key was not found in the keyring
    NotFound(String),
    /// The key data in the keyring is invalid or corrupted
    InvalidKey(String),
}

impl std::fmt::Display for KeyringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeyringError::Access(msg) => write!(f, "Keyring access error: {msg}"),
            KeyringError::NotFound(msg) => write!(f, "Key not found in keyring: {msg}"),
            KeyringError::InvalidKey(msg) => write!(f, "Invalid key in keyring: {msg}"),
        }
    }
}

impl std::error::Error for KeyringError {}

impl crate::SecretKey {
    /// Stores the secret key in the system keyring.
    ///
    /// The key is stored as a hex-encoded string under the service "fastn" with
    /// the ID52 as the account name. This allows users to view and copy the key
    /// from their password manager.
    ///
    /// # Errors
    ///
    /// Returns an error if the keyring is not accessible or the key cannot be stored.
    pub fn store_in_keyring(&self) -> Result<(), keyring::Error> {
        let id52 = self.id52();
        let entry = keyring::Entry::new("fastn", &id52)?;
        // Store as hex string for better UX (viewable in password managers)
        entry.set_password(&self.to_string())
    }

    /// Loads a secret key from the system keyring.
    ///
    /// Attempts to load the key for the given ID52 from the keyring. Supports both
    /// the new format (hex string via get_password) and legacy format (raw bytes
    /// via get_secret) for compatibility.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The keyring is not accessible
    /// - No key exists for the given ID52
    /// - The stored key is invalid
    /// - The loaded key doesn't match the expected ID52
    pub fn from_keyring(id52: &str) -> Result<Self, KeyringError> {
        let entry =
            keyring::Entry::new("fastn", id52).map_err(|e| KeyringError::Access(e.to_string()))?;

        // Try new format first (hex string)
        let secret_key = match entry.get_password() {
            Ok(hex_string) => std::str::FromStr::from_str(&hex_string).map_err(
                |e: crate::errors::ParseSecretKeyError| KeyringError::InvalidKey(e.to_string()),
            )?,
            Err(_) => {
                // Fall back to legacy format (raw bytes)
                let secret_bytes = entry
                    .get_secret()
                    .map_err(|e| KeyringError::NotFound(e.to_string()))?;

                if secret_bytes.len() != 32 {
                    return Err(KeyringError::InvalidKey(format!(
                        "expected 32 bytes, got {}",
                        secret_bytes.len()
                    )));
                }

                let bytes: [u8; 32] = secret_bytes[..32]
                    .try_into()
                    .map_err(|_| KeyringError::InvalidKey("conversion failed".to_string()))?;
                Self::from_bytes(&bytes)
            }
        };

        // Verify the key matches the expected ID52
        if secret_key.id52() != id52 {
            return Err(KeyringError::InvalidKey(format!(
                "key mismatch: expected {}, got {}",
                id52,
                secret_key.id52()
            )));
        }

        Ok(secret_key)
    }

    /// Deletes the secret key from the system keyring.
    ///
    /// # Errors
    ///
    /// Returns an error if the keyring is not accessible or the key cannot be deleted.
    pub fn delete_from_keyring(&self) -> Result<(), keyring::Error> {
        let id52 = self.id52();
        let entry = keyring::Entry::new("fastn", &id52)?;
        entry.delete_credential()
    }
}
