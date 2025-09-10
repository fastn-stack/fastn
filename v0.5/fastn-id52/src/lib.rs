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
//! ## Installation
//!
//! This crate can be used as a library or installed as a CLI tool:
//!
//! ```bash
//! # As a library dependency
//! cargo add fastn-id52
//!
//! # As a CLI tool
//! cargo install fastn-id52
//! ```
//!
//! ## CLI Usage
//!
//! The `fastn-id52` CLI tool generates entity identities:
//!
//! ```bash
//! # Default: Generate and store in system keyring
//! fastn-id52 generate
//! # Output: ID52 printed to stdout, secret key stored in keyring
//!
//! # Save to file (less secure, requires explicit flag)
//! fastn-id52 generate --file
//! fastn-id52 generate --file my-entity.key
//! # Output: Secret key saved to file, ID52 printed to stderr
//!
//! # Print to stdout
//! fastn-id52 generate --file -
//! fastn-id52 generate -f -
//! # Output: Secret key (hex) printed to stdout, ID52 printed to stderr
//!
//! # Short output (only ID52, no descriptive messages)
//! fastn-id52 generate --short
//! fastn-id52 generate -f - -s
//! # Output: Secret key stored in keyring, only ID52 printed (no messages)
//! ```
//!
//! By default, secret keys are stored securely in the system keyring and can be
//! viewed in your password manager. File storage requires explicit user consent.
//!
//! ## Quick Start (Library)
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
//! ## Key Loading
//!
//! The crate provides comprehensive key loading with automatic fallback:
//!
//! ```rust,no_run
//! use fastn_id52::SecretKey;
//! use std::path::Path;
//!
//! // Load from directory (checks for .id52 or .private-key files)
//! let (id52, key) = SecretKey::load_from_dir(Path::new("/path"), "entity")?;
//!
//! // Load with fallback: keyring → FASTN_SECRET_KEYS_FILE → FASTN_SECRET_KEYS
//! let key = SecretKey::load_for_id52("i66fo538...")?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ### Environment Variables
//!
//! - `FASTN_SECRET_KEYS`: Keys directly in env var (format: `prefix: hexkey`)
//! - `FASTN_SECRET_KEYS_FILE`: Path to file containing keys (more secure)
//!
//! Cannot have both set (strict mode). Files support comments (`#`) and empty lines.
//!
//! ## Error Types
//!
//! - [`ParseId52Error`]: Errors when parsing ID52 strings
//! - [`InvalidKeyBytesError`]: Invalid key byte format
//! - [`ParseSecretKeyError`]: Errors parsing secret key strings
//! - [`InvalidSignatureBytesError`]: Invalid signature byte format
//! - [`SignatureVerificationError`]: Signature verification failures
//! - [`KeyringError`]: Errors when accessing the system keyring
//!
//! ## Security
//!
//! This crate uses `ed25519-dalek` for all cryptographic operations, which provides
//! constant-time implementations to prevent timing attacks. Random key generation
//! uses the operating system's secure random number generator.

mod errors;
mod keyring;
mod keys;

pub use errors::{
    InvalidKeyBytesError, InvalidSignatureBytesError, ParseId52Error, ParseSecretKeyError,
    SignatureVerificationError,
};
pub use keyring::KeyringError;
pub use keys::{PublicKey, SecretKey, Signature};

#[cfg(feature = "dns")]
pub use errors::ResolveError;

#[cfg(feature = "dns")]
pub mod dns;

#[cfg(feature = "automerge")]
mod automerge;
