//! # fastn-id52
//!
//! Entity identity and cryptographic key management for the fastn P2P network.
//!
//! This crate provides entity identity for fastn's peer-to-peer network. Each fastn
//! instance is called an "entity" and is uniquely identified by an ID52 - a 52-character
//! encoded Ed25519 public key.
//!
//! ## What is ID52?
//!
//! ID52 is the identity of an entity on the fastn peer-to-peer network. It's a
//! 52-character string using BASE32_DNSSEC encoding that uniquely identifies each
//! entity. The format is:
//! - Exactly 52 characters long
//! - Uses only lowercase letters and digits
//! - DNS-compatible (can be used in subdomains)
//! - URL-safe without special encoding
//!
//! ## Quick Start
//!
//! ```
//! use fastn_id52::SecretKey;
//!
//! // Generate a new entity identity
//! let secret_key = SecretKey::generate();
//! let public_key = secret_key.public_key();
//!
//! // Get the entity's ID52 identifier
//! let entity_id52 = public_key.to_string();
//! assert_eq!(entity_id52.len(), 52);
//!
//! // Sign and verify a message
//! let message = b"Hello, fastn!";
//! let signature = secret_key.sign(message);
//! assert!(public_key.verify(message, &signature).is_ok());
//! ```
//!
//! ## Key Types
//!
//! - [`SecretKey`]: Entity's private key for signing operations
//! - [`PublicKey`]: Entity's public key with ID52 encoding
//! - [`Signature`]: Ed25519 signature for entity authentication
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
