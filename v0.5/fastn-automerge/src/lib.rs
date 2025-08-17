//! # fastn-automerge
//!
//! Automerge document management for fastn.
//!
//! This crate provides a unified interface for storing and managing Automerge
//! documents in SQLite, used by both fastn-account and fastn-device.

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
