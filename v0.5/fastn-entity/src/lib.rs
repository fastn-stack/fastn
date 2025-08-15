//! # fastn-entity
//!
//! Entity management for the fastn P2P network.
//!
//! This crate provides entity management for fastn's peer-to-peer network. Each fastn
//! instance is an "entity" identified by an ID52 - a 52-character encoded Ed25519
//! public key. Entities have their own SQLite database and cryptographic identity.
//!
//! ## Overview
//!
//! - **Entity**: A single fastn instance with ID52 identity and SQLite database
//! - **Storage**: Each entity stored in folder named by its ID52
//! - **Security**: Private keys stored in system keyring by default
//!
//! ## Entity Storage Structure
//!
//! ```txt
//! .fastn/
//! ├── entities/
//! │   ├── {id52}/
//! │   │   ├── entity.id52      # Public key (ID52 format)
//! │   │   └── db.sqlite        # Entity's SQLite database
//! │   └── {another-id52}/
//! │       ├── entity.id52
//! │       └── db.sqlite
//! ```
//!
//! ## Key Management
//!
//! By default, private keys are stored in the system keyring for security.
//! The crate will:
//!
//! 1. Check for `entity.id52` file (public key)
//! 2. Retrieve private key from keyring using the ID52
//! 3. Error if both `.id52` and `.private-key` files exist (strict mode)
//! 4. Read `.private-key` file if it exists (less secure, explicit user choice)

extern crate self as fastn_entity;

mod entity;
mod migration;

/// Represents a single entity in the fastn P2P network.
///
/// Each entity has:
/// - A unique ID52 identifier (52-character public key)
/// - A SQLite database for local storage
/// - A cryptographic identity for P2P communication
#[derive(Debug, Clone)]
pub struct Entity {
    /// The entity's ID52 identifier
    pub(crate) id52: String,
    /// Path to the entity's folder
    pub(crate) path: std::path::PathBuf,
    /// The entity's secret key (loaded from keyring or file)
    pub(crate) secret_key: fastn_id52::SecretKey,
    /// The entity's database connection
    pub(crate) conn: std::sync::Arc<tokio::sync::Mutex<rusqlite::Connection>>,
}
