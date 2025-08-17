//! # fastn-automerge
//!
//! CRDT document storage layer for FASTN using Automerge.
//!
//! This crate provides persistent storage for Automerge CRDT documents in SQLite,
//! enabling conflict-free replicated data across the FASTN network. It's used by
//! both fastn-account and fastn-device for storing configuration and metadata.
//!
//! ## Key Features
//!
//! - **CRDT Storage**: Persistent storage of Automerge documents
//! - **Path-based Access**: Documents organized by filesystem-like paths
//! - **Version Tracking**: Maintains document version history
//! - **Shared Schema**: Common database schema for accounts and devices
//!
//! ## Document Paths
//!
//! Documents use filesystem-like paths:
//! - `/-/config`: Main configuration document
//! - `/-/aliases/{id52}/readme`: Public alias information
//! - `/-/aliases/{id52}/notes`: Private alias notes
//!
//! ## Database Schema
//!
//! ```sql
//! CREATE TABLE fastn_documents (
//!     path     TEXT PRIMARY KEY,
//!     data     BLOB NOT NULL,
//!     version  INTEGER NOT NULL,
//!     updated  TEXT NOT NULL
//! );
//! ```

extern crate self as fastn_automerge;

pub mod migration;

/// A stored Automerge document
#[derive(Debug, Clone)]
pub struct StoredDocument {
    /// Document path (e.g., "/-/config", "/-/aliases/{id52}/readme")
    pub path: String,
    /// The Automerge document
    pub doc: automerge::AutoCommit,
    /// Last update timestamp
    pub updated_at: i64,
}
