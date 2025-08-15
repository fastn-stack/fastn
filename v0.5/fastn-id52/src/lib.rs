//! # fastn-id52
//!
//! ID52 identity and cryptographic key management for the fastn ecosystem.
//!
//! This crate provides a secure implementation of ID52 encoding for Ed25519 public keys,
//! along with comprehensive cryptographic operations including key generation, signature
//! creation, and verification.
//!
//! ## What is ID52?
//!
//! ID52 is a 52-character encoding format using BASE32_DNSSEC that provides a compact,
//! URL-safe representation of Ed25519 public keys. Each ID52 string is exactly 52 characters
//! long and uses only lowercase letters and digits, making it ideal for use in URLs, DNS
//! records, and other contexts where special characters might cause issues.
//!
//! ## Quick Start
//!
//! ```
//! use fastn_id52::SecretKey;
//!
//! // Generate a new key pair
//! let secret_key = SecretKey::generate();
//! let public_key = secret_key.public_key();
//!
//! // Get the ID52 representation
//! let id52 = public_key.to_string();
//! assert_eq!(id52.len(), 52);
//!
//! // Sign and verify a message
//! let message = b"Hello, fastn!";
//! let signature = secret_key.sign(message);
//! assert!(public_key.verify(message, &signature).is_ok());
//! ```
//!
//! ## Key Types
//!
//! - [`SecretKey`]: Ed25519 secret key for signing operations
//! - [`PublicKey`]: Ed25519 public key with ID52 encoding
//! - [`Signature`]: Ed25519 signature (64 bytes)
//!
//! ## Error Types
//!
//! - [`ParseId52Error`]: Errors when parsing ID52 strings
//! - [`InvalidKeyBytesError`]: Invalid key byte format
//! - [`ParseSecretKeyError`]: Errors parsing secret key strings
//! - [`InvalidSignatureBytesError`]: Invalid signature byte format
//! - [`SignatureVerificationError`]: Signature verification failures
//!
//! ## Security
//!
//! This crate uses `ed25519-dalek` for all cryptographic operations, which provides
//! constant-time implementations to prevent timing attacks. Random key generation
//! uses the operating system's secure random number generator.

mod errors;
mod keys;

pub use errors::{
    InvalidKeyBytesError, InvalidSignatureBytesError, ParseId52Error, ParseSecretKeyError,
    SignatureVerificationError,
};
pub use keys::{PublicKey, SecretKey, Signature};
