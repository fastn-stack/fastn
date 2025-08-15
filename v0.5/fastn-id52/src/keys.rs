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

/// Ed25519 digital signature.
///
/// A `Signature` is a 64-byte Ed25519 signature created by signing a message with
/// a [`SecretKey`]. Signatures can be verified using the corresponding [`PublicKey`].
///
/// # Examples
///
/// ```
/// use fastn_id52::{SecretKey, Signature};
///
/// let secret_key = SecretKey::generate();
/// let message = b"Hello, world!";
/// let signature = secret_key.sign(message);
///
/// // Convert to bytes
/// let bytes: [u8; 64] = signature.to_bytes();
/// ```
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
    pub fn to_bytes(&self) -> [u8; 32] {
        *self.0.as_bytes()
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
    pub fn to_bytes(&self) -> [u8; 64] {
        self.0.to_bytes()
    }

    /// Converts the signature to a `Vec<u8>`.
    ///
    /// This is useful when you need an owned copy of the signature bytes.
    pub fn to_vec(&self) -> Vec<u8> {
        self.to_bytes().to_vec()
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
}
