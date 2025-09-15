use crate::errors::{
    InvalidKeyBytesError, InvalidSignatureBytesError, ParseId52Error, ParseSecretKeyError,
    SignatureVerificationError,
};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

// Internal type aliases - we only use ed25519-dalek, no iroh dependency
type InnerPublicKey = ed25519_dalek::VerifyingKey;
type InnerSecretKey = ed25519_dalek::SigningKey;
type InnerSignature = ed25519_dalek::Signature;

/// Ed25519 public key with ID52 encoding.
///
/// A `PublicKey` represents the public half of an Ed25519 key pair. It can be used
/// to verify signatures created with the corresponding [`SecretKey`]. The key is
/// displayed using ID52 encoding - a 52-character BASE32_DNSSEC format that is
/// URL-safe and DNS-compatible.
///
/// # Examples
///
/// ```
/// use fastn_id52::PublicKey;
/// use std::str::FromStr;
///
/// let id52 = "i66fo538lfl5ombdf6tcdbrabp4hmp9asv7nrffuc2im13ct4q60";
/// let public_key = PublicKey::from_str(id52).unwrap();
///
/// // Convert back to ID52
/// assert_eq!(public_key.to_string(), id52);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PublicKey(InnerPublicKey);

/// Ed25519 secret key for signing operations.
///
/// A `SecretKey` represents the private half of an Ed25519 key pair. It can be used
/// to sign messages and derive the corresponding [`PublicKey`]. The key is displayed
/// using hexadecimal encoding for compatibility and readability.
///
/// # Security
///
/// Secret keys should be kept confidential and never exposed in logs or transmitted
/// over insecure channels. Use [`SecretKey::generate`] to create cryptographically
/// secure random keys.
///
/// # Examples
///
/// ```
/// use fastn_id52::SecretKey;
///
/// // Generate a new random key
/// let secret_key = SecretKey::generate();
///
/// // Get the public key and ID52
/// let public_key = secret_key.public_key();
/// let id52 = secret_key.id52();
/// ```
pub struct SecretKey(InnerSecretKey);

// Manual Clone implementation for SecretKey
impl Clone for SecretKey {
    fn clone(&self) -> Self {
        // Clone by reconstructing from bytes
        SecretKey::from_bytes(&self.to_bytes())
    }
}

// Manual Debug implementation to avoid exposing the secret key material.
// Only shows the public ID52, omitting the actual 32-byte secret key value
// that would be exposed by a derived Debug implementation.
impl fmt::Debug for SecretKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SecretKey")
            .field("id52", &self.id52())
            .finish()
    }
}

/// Ed25519 digital signature.
///
/// A `Signature` is a 64-byte Ed25519 signature created by signing a message with
/// a [`SecretKey`]. Signatures can be verified using the corresponding [`PublicKey`].
/// The signature is displayed using hexadecimal encoding (128 characters).
///
/// # Examples
///
/// ```
/// use fastn_id52::{SecretKey, Signature};
/// use std::str::FromStr;
///
/// let secret_key = SecretKey::generate();
/// let message = b"Hello, world!";
/// let signature = secret_key.sign(message);
///
/// // Convert to hex string (128 characters)
/// let hex = signature.to_string();
/// assert_eq!(hex.len(), 128);
///
/// // Parse from hex string
/// let parsed = Signature::from_str(&hex).unwrap();
///
/// // Convert to bytes
/// let bytes: [u8; 64] = signature.to_bytes();
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Signature(InnerSignature);

// ============== PublicKey Implementation ==============

impl PublicKey {
    /// Creates a public key from its raw 32-byte representation.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidKeyBytesError`] if the bytes do not represent a valid Ed25519 public key.
    pub fn from_bytes(bytes: &[u8; 32]) -> Result<Self, InvalidKeyBytesError> {
        use ed25519_dalek::VerifyingKey;
        VerifyingKey::from_bytes(bytes)
            .map(PublicKey)
            .map_err(|_| InvalidKeyBytesError {
                expected: 32,
                got: 32, // The bytes were 32 but invalid for a public key
            })
    }

    /// Returns the raw 32-byte representation of the public key.
    pub fn to_bytes(self) -> [u8; 32] {
        *self.0.as_bytes()
    }

    /// Returns the ID52 string representation of this public key.
    pub fn id52(&self) -> String {
        self.to_string()
    }

    /// Verifies an Ed25519 signature for the given message.
    ///
    /// # Errors
    ///
    /// Returns [`SignatureVerificationError`] if the signature is invalid for this
    /// public key and message combination.
    pub fn verify(
        &self,
        message: &[u8],
        signature: &Signature,
    ) -> Result<(), SignatureVerificationError> {
        use ed25519_dalek::Verifier;
        self.0
            .verify(message, &signature.0)
            .map_err(|_| SignatureVerificationError)
    }

    /// Resolves a public key from DNS TXT records.
    ///
    /// Looks for TXT records on the given domain in the format "{scope}={public_key_id52}".
    /// For example, if the domain "fifthtry.com" has a TXT record "malai=abc123def456...",
    /// calling `PublicKey::resolve("fifthtry.com", "malai").await` will return the public key.
    ///
    /// # Arguments
    ///
    /// * `domain` - The domain to query for TXT records
    /// * `scope` - The scope/prefix to look for in TXT records
    ///
    /// # Returns
    ///
    /// Returns the resolved `PublicKey` on success, or a `ResolveError` on failure.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use fastn_id52::PublicKey;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let public_key = PublicKey::resolve("fifthtry.com", "malai").await?;
    /// println!("Resolved public key: {}", public_key.id52());
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "dns")]
    pub async fn resolve(domain: &str, scope: &str) -> Result<Self, crate::ResolveError> {
        crate::dns::resolve(domain, scope).await
    }
}

// Display implementation - uses ID52 (BASE32_DNSSEC) encoding
impl fmt::Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            data_encoding::BASE32_DNSSEC.encode(self.0.as_bytes())
        )
    }
}

// FromStr implementation - accepts ID52 format (BASE32_DNSSEC)
impl FromStr for PublicKey {
    type Err = ParseId52Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = data_encoding::BASE32_DNSSEC
            .decode(s.as_bytes())
            .map_err(|e| ParseId52Error {
                input: s.to_string(),
                reason: format!("invalid BASE32_DNSSEC encoding: {e}"),
            })?;

        if bytes.len() != 32 {
            return Err(ParseId52Error {
                input: s.to_string(),
                reason: format!("expected 32 bytes after decoding, got {}", bytes.len()),
            });
        }

        let bytes: [u8; 32] = bytes.try_into().unwrap();
        Self::from_bytes(&bytes).map_err(|_| ParseId52Error {
            input: s.to_string(),
            reason: "invalid public key bytes".to_string(),
        })
    }
}

// Serialize as ID52 string
impl Serialize for PublicKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

// Deserialize from ID52 string
impl<'de> Deserialize<'de> for PublicKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        PublicKey::from_str(&s).map_err(serde::de::Error::custom)
    }
}

// ============== SecretKey Implementation ==============

impl SecretKey {
    /// Generates a new cryptographically secure random secret key.
    ///
    /// Uses the operating system's secure random number generator.
    pub fn generate() -> Self {
        let mut rng = rand::rngs::OsRng;
        SecretKey(ed25519_dalek::SigningKey::generate(&mut rng))
    }

    /// Creates a secret key from its raw 32-byte representation.
    ///
    /// # Security
    ///
    /// The provided bytes should be generated securely and kept confidential.
    pub fn from_bytes(bytes: &[u8; 32]) -> Self {
        SecretKey(ed25519_dalek::SigningKey::from_bytes(bytes))
    }

    /// Returns the raw 32-byte representation of the secret key.
    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.to_bytes()
    }

    /// Derives the public key from this secret key.
    ///
    /// The public key is deterministically derived and will always be the same
    /// for a given secret key.
    pub fn public_key(&self) -> PublicKey {
        PublicKey(self.0.verifying_key())
    }

    /// Returns the ID52 string representation of this key's public key.
    ///
    /// This is a convenience method equivalent to `self.public_key().to_string()`.
    pub fn id52(&self) -> String {
        self.public_key().to_string()
    }

    /// Creates a digital signature for the given message.
    ///
    /// The signature can be verified by anyone with the corresponding public key.
    pub fn sign(&self, message: &[u8]) -> Signature {
        use ed25519_dalek::Signer;
        Signature(self.0.sign(message))
    }

    /// Loads a secret key from a directory with comprehensive fallback logic.
    ///
    /// Checks for keys in the following locations:
    /// 1. `{prefix}.id52` file → load ID52, then check keyring → env fallback
    /// 2. `{prefix}.private-key` file → load key directly
    ///
    /// Returns error if both files exist (strict mode).
    ///
    /// # Arguments
    ///
    /// * `dir` - Directory to look for key files
    /// * `prefix` - File prefix (e.g., "entity" for "entity.id52")
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Both `.id52` and `.private-key` files exist
    /// - Neither file exists
    /// - Key cannot be loaded or parsed
    pub fn load_from_dir(
        dir: &std::path::Path,
        prefix: &str,
    ) -> Result<(String, Self), crate::KeyringError> {
        let id52_file = dir.join(format!("{prefix}.id52"));
        let private_key_file = dir.join(format!("{prefix}.private-key"));

        // Check for conflicting files (strict mode)
        if id52_file.exists() && private_key_file.exists() {
            return Err(crate::KeyringError::InvalidKey(format!(
                "Both {prefix}.id52 and {prefix}.private-key files exist in {}. This is not allowed in strict mode.",
                dir.display()
            )));
        }

        if id52_file.exists() {
            // Load ID52 and get private key using fallback logic
            let id52 = std::fs::read_to_string(&id52_file)
                .map_err(|e| {
                    crate::KeyringError::Access(format!("Failed to read {prefix}.id52 file: {e}"))
                })?
                .trim()
                .to_string();

            // Try keyring first, then env fallback
            let secret_key = Self::load_for_id52(&id52)?;

            Ok((id52, secret_key))
        } else if private_key_file.exists() {
            // Load from private key file
            let key_str = std::fs::read_to_string(&private_key_file).map_err(|e| {
                crate::KeyringError::Access(format!(
                    "Failed to read {prefix}.private-key file: {e}"
                ))
            })?;

            let secret_key = Self::from_str(key_str.trim()).map_err(|e| {
                crate::KeyringError::InvalidKey(format!("Failed to parse private key: {e}"))
            })?;

            let id52 = secret_key.id52();

            Ok((id52, secret_key))
        } else {
            Err(crate::KeyringError::NotFound(format!(
                "Neither {prefix}.id52 nor {prefix}.private-key file found in {}",
                dir.display()
            )))
        }
    }

    /// Saves a secret key to a directory using the format expected by load_from_dir.
    ///
    /// Creates a `{prefix}.private-key` file containing the secret key in hex format.
    /// This format is compatible with `load_from_dir` and follows the strict mode rules.
    ///
    /// # Arguments
    ///
    /// * `dir` - Directory to save the key file
    /// * `prefix` - File prefix (e.g., "ssh" for "ssh.private-key")
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Directory cannot be created
    /// - File cannot be written
    /// - Key already exists (to prevent accidental overwrites)
    pub fn save_to_dir(
        &self,
        dir: &std::path::Path,
        prefix: &str,
    ) -> Result<(), crate::KeyringError> {
        let private_key_file = dir.join(format!("{prefix}.private-key"));
        let id52_file = dir.join(format!("{prefix}.id52"));

        // Check if any key file already exists (prevent overwrites)
        if private_key_file.exists() || id52_file.exists() {
            return Err(crate::KeyringError::InvalidKey(format!(
                "Key files already exist in {}. Use a different prefix or remove existing files.",
                dir.display()
            )));
        }

        // Create directory if it doesn't exist
        std::fs::create_dir_all(dir).map_err(|e| {
            crate::KeyringError::Access(format!(
                "Failed to create directory {}: {e}",
                dir.display()
            ))
        })?;

        // Write secret key to file
        let key_string = self.to_string();
        std::fs::write(&private_key_file, &key_string).map_err(|e| {
            crate::KeyringError::Access(format!("Failed to write {prefix}.private-key file: {e}"))
        })?;

        Ok(())
    }

    /// Loads a secret key for the given ID52 with fallback logic.
    ///
    /// Attempts to load the key in the following order:
    /// 1. From system keyring
    /// 2. From FASTN_SECRET_KEYS_FILE (path to file with keys) or
    ///    FASTN_SECRET_KEYS (keys directly in env var)
    ///
    /// Cannot have both FASTN_SECRET_KEYS_FILE and FASTN_SECRET_KEYS set (strict mode).
    ///
    /// The keys format is:
    /// ```text
    /// prefix1: hexkey1
    /// prefix2: hexkey2
    /// # Comments are allowed in files
    /// ```
    /// Where prefix can be any unique prefix of the ID52.
    /// Uses starts_with matching for flexibility (e.g., "i66f" or "i66fo538").
    /// Spaces around the colon are optional and trimmed.
    ///
    /// # Errors
    ///
    /// Returns an error if the key cannot be loaded from any source.
    pub fn load_for_id52(id52: &str) -> Result<Self, crate::KeyringError> {
        // Try keyring first
        match Self::from_keyring(id52) {
            Ok(key) => Ok(key),
            Err(keyring_err) => {
                // Try environment variable fallback
                Self::load_from_env(id52).ok_or_else(|| {
                    crate::KeyringError::NotFound(format!(
                        "Key not found in keyring ({keyring_err}) or FASTN_SECRET_KEYS env"
                    ))
                })
            }
        }
    }

    /// Loads a secret key from FASTN_SECRET_KEYS environment variable or file.
    ///
    /// Checks in strict order:
    /// 1. FASTN_SECRET_KEYS_FILE env var pointing to a file with keys
    /// 2. FASTN_SECRET_KEYS env var with keys directly
    ///
    /// Cannot have both set (strict mode).
    ///
    /// Format: "prefix1: hexkey1\nprefix2: hexkey2\n..."
    /// Where prefix can be any unique prefix of the ID52 (e.g., first 4-8 chars).
    /// Uses starts_with matching, so you can use as many or few characters as needed
    /// to uniquely identify the key. Spaces around the colon are allowed.
    fn load_from_env(id52: &str) -> Option<Self> {
        let has_file = std::env::var("FASTN_SECRET_KEYS_FILE").is_ok();
        let has_direct = std::env::var("FASTN_SECRET_KEYS").is_ok();

        // Strict mode: cannot have both
        if has_file && has_direct {
            eprintln!(
                "ERROR: Both FASTN_SECRET_KEYS_FILE and FASTN_SECRET_KEYS are set. \
                      This is not allowed in strict mode. Please use only one."
            );
            return None;
        }

        // Try file first if FASTN_SECRET_KEYS_FILE is set
        let keys_content = if has_file {
            let file_path = std::env::var("FASTN_SECRET_KEYS_FILE").ok()?;
            std::fs::read_to_string(&file_path).ok()?
        } else if has_direct {
            std::env::var("FASTN_SECRET_KEYS").ok()?
        } else {
            return None;
        };

        // Parse the keys content
        for line in keys_content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                // Allow comments in file
                continue;
            }

            if let Some((key_prefix, hex_key)) = line.split_once(':') {
                let key_prefix = key_prefix.trim();
                let hex_key = hex_key.trim();

                if id52.starts_with(key_prefix) {
                    return std::str::FromStr::from_str(hex_key).ok();
                }
            }
        }

        None
    }
}

// Display implementation - always uses hex encoding
impl fmt::Display for SecretKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", data_encoding::HEXLOWER.encode(&self.to_bytes()))
    }
}

// FromStr implementation - accepts both hex and base32
impl FromStr for SecretKey {
    type Err = ParseSecretKeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = if s.len() == 64 {
            // Hex encoding (our Display format)
            let mut result = [0u8; 32];
            data_encoding::HEXLOWER
                .decode_mut(s.as_bytes(), &mut result)
                .map_err(|e| ParseSecretKeyError {
                    reason: format!("invalid hex encoding: {e:?}"),
                })?;
            result
        } else {
            // For backward compatibility, also try BASE32_NOPAD
            let input = s.to_ascii_uppercase();
            let mut result = [0u8; 32];
            data_encoding::BASE32_NOPAD
                .decode_mut(input.as_bytes(), &mut result)
                .map_err(|e| ParseSecretKeyError {
                    reason: format!("invalid base32 encoding: {e:?}"),
                })?;
            result
        };

        Ok(SecretKey::from_bytes(&bytes))
    }
}

// Serialize as hex string
impl Serialize for SecretKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("{self}"))
    }
}

// Deserialize from hex or base32 string
impl<'de> Deserialize<'de> for SecretKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        SecretKey::from_str(&s).map_err(serde::de::Error::custom)
    }
}

// ============== Signature Implementation ==============

impl Signature {
    /// Creates a signature from its raw 64-byte representation.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidSignatureBytesError`] if the byte array is invalid.
    pub fn from_bytes(bytes: &[u8; 64]) -> Result<Self, InvalidSignatureBytesError> {
        Ok(Signature(InnerSignature::from_bytes(bytes)))
    }

    /// Returns the raw 64-byte representation of the signature.
    pub fn to_bytes(self) -> [u8; 64] {
        self.0.to_bytes()
    }

    /// Converts the signature to a `Vec<u8>`.
    ///
    /// This is useful when you need an owned copy of the signature bytes.
    pub fn to_vec(self) -> Vec<u8> {
        self.to_bytes().to_vec()
    }
}

// Display implementation - uses hex encoding (128 characters)
impl fmt::Display for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", data_encoding::HEXLOWER.encode(&self.to_bytes()))
    }
}

// FromStr implementation - accepts hex format
impl FromStr for Signature {
    type Err = InvalidSignatureBytesError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Expect 128 hex characters for 64 bytes
        if s.len() != 128 {
            return Err(InvalidSignatureBytesError {
                expected: 64,
                got: s.len() / 2,
            });
        }

        let mut bytes = [0u8; 64];
        data_encoding::HEXLOWER
            .decode_mut(s.as_bytes(), &mut bytes)
            .map_err(|_| InvalidSignatureBytesError {
                expected: 64,
                got: 0, // Invalid hex encoding
            })?;

        Self::from_bytes(&bytes)
    }
}

// Serialize as hex string
impl Serialize for Signature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

// Deserialize from hex string
impl<'de> Deserialize<'de> for Signature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Signature::from_str(&s).map_err(serde::de::Error::custom)
    }
}

// Implement From for Vec<u8> conversion
impl From<Signature> for Vec<u8> {
    fn from(sig: Signature) -> Vec<u8> {
        sig.to_bytes().to_vec()
    }
}

// Implement From for [u8; 64] conversion
impl From<Signature> for [u8; 64] {
    fn from(sig: Signature) -> [u8; 64] {
        sig.to_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_key_roundtrip() {
        let secret_key = SecretKey::generate();
        let public_key = secret_key.public_key();
        let id52 = public_key.to_string();

        let parsed = PublicKey::from_str(&id52).unwrap();
        assert_eq!(parsed, public_key);
    }

    #[test]
    fn test_secret_key_hex_roundtrip() {
        let secret_key = SecretKey::generate();
        let hex = secret_key.to_string();

        let parsed = SecretKey::from_str(&hex).unwrap();
        assert_eq!(parsed.to_bytes(), secret_key.to_bytes());
    }

    #[test]
    fn test_signature_verification() {
        let secret_key = SecretKey::generate();
        let public_key = secret_key.public_key();
        let message = b"test message";

        let signature = secret_key.sign(message);
        assert!(public_key.verify(message, &signature).is_ok());

        let wrong_message = b"wrong message";
        assert!(public_key.verify(wrong_message, &signature).is_err());
    }

    #[test]
    fn test_signature_hex_roundtrip() {
        let secret_key = SecretKey::generate();
        let message = b"test message";
        let signature = secret_key.sign(message);

        // Test hex encoding/decoding
        let hex = signature.to_string();
        assert_eq!(hex.len(), 128); // 64 bytes * 2 hex chars per byte

        let parsed = Signature::from_str(&hex).unwrap();
        assert_eq!(parsed.to_bytes(), signature.to_bytes());

        // Verify the parsed signature still works
        let public_key = secret_key.public_key();
        assert!(public_key.verify(message, &parsed).is_ok());
    }
}
