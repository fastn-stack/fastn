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
//! - `/-/mails/{username}`: Email authentication and settings
//!
//! ## Database Schema
//!
//! ```sql
//! CREATE TABLE fastn_documents (
//!     path              TEXT PRIMARY KEY,
//!     automerge_binary  BLOB NOT NULL,
//!     heads             TEXT NOT NULL,
//!     actor_id          TEXT NOT NULL,
//!     updated_at        INTEGER NOT NULL
//! );
//! ```

extern crate self as fastn_automerge;

pub mod migration;

use automerge::AutoCommit;
use rusqlite::{Connection, OptionalExtension, params};

/// A stored Automerge document
#[derive(Debug, Clone)]
pub struct StoredDocument {
    /// Document path (e.g., "/-/config", "/-/aliases/{id52}/readme")
    pub path: String,
    /// The Automerge document
    pub doc: AutoCommit,
    /// Last update timestamp
    pub updated_at: i64,
}

/// Create and save an Automerge document at the specified path
pub fn create_and_save_document(
    conn: &Connection,
    path: &str,
    mut doc: AutoCommit,
) -> eyre::Result<()> {
    use eyre::WrapErr;

    let binary = doc.save();
    let heads = doc
        .get_heads()
        .iter()
        .map(|h| format!("{h:?}"))
        .collect::<Vec<_>>()
        .join(",");

    let actor_id = format!("{:?}", doc.get_actor());
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    conn.execute(
        "INSERT OR REPLACE INTO fastn_documents (path, automerge_binary, heads, actor_id, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![path, binary, heads, actor_id, now],
    )
    .wrap_err_with(|| format!("Failed to save document at path: {path}"))?;

    Ok(())
}

/// Load an Automerge document from the database
pub fn load_document(conn: &Connection, path: &str) -> eyre::Result<Option<StoredDocument>> {
    use eyre::WrapErr;

    let result = conn
        .query_row(
            "SELECT automerge_binary, updated_at FROM fastn_documents WHERE path = ?1",
            params![path],
            |row| {
                let binary: Vec<u8> = row.get(0)?;
                let updated_at: i64 = row.get(1)?;
                Ok((binary, updated_at))
            },
        )
        .optional()
        .wrap_err_with(|| format!("Failed to load document at path: {path}"))?;

    match result {
        Some((binary, updated_at)) => {
            let doc = AutoCommit::load(&binary)
                .wrap_err_with(|| format!("Failed to deserialize document at path: {path}"))?;
            Ok(Some(StoredDocument {
                path: path.to_string(),
                doc,
                updated_at,
            }))
        }
        None => Ok(None),
    }
}

/// Update an existing document in the database
pub fn update_document(conn: &Connection, path: &str, doc: &mut AutoCommit) -> eyre::Result<()> {
    use eyre::WrapErr;

    let binary = doc.save();
    let heads = doc
        .get_heads()
        .iter()
        .map(|h| format!("{h:?}"))
        .collect::<Vec<_>>()
        .join(",");

    let actor_id = format!("{:?}", doc.get_actor());
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    conn.execute(
        "UPDATE fastn_documents SET automerge_binary = ?1, heads = ?2, actor_id = ?3, updated_at = ?4
         WHERE path = ?5",
        params![binary, heads, actor_id, now, path],
    )
    .wrap_err_with(|| format!("Failed to update document at path: {path}"))?;

    Ok(())
}

/// Delete a document from the database
pub fn delete_document(conn: &Connection, path: &str) -> eyre::Result<()> {
    use eyre::WrapErr;

    conn.execute("DELETE FROM fastn_documents WHERE path = ?1", params![path])
        .wrap_err_with(|| format!("Failed to delete document at path: {path}"))?;

    Ok(())
}

/// List all document paths in the database
pub fn list_document_paths(conn: &Connection) -> eyre::Result<Vec<String>> {
    use eyre::WrapErr;

    let mut stmt = conn
        .prepare("SELECT path FROM fastn_documents ORDER BY path")
        .wrap_err("Failed to prepare document list query")?;

    let paths = stmt
        .query_map([], |row| row.get(0))?
        .collect::<Result<Vec<String>, _>>()
        .wrap_err("Failed to list document paths")?;

    Ok(paths)
}

/// List documents under a specific path prefix
pub fn list_documents_with_prefix(conn: &Connection, prefix: &str) -> eyre::Result<Vec<String>> {
    use eyre::WrapErr;

    let mut stmt = conn
        .prepare("SELECT path FROM fastn_documents WHERE path LIKE ?1 ORDER BY path")
        .wrap_err("Failed to prepare prefix query")?;

    let pattern = format!("{prefix}%");
    let paths = stmt
        .query_map([pattern], |row| row.get(0))?
        .collect::<Result<Vec<String>, _>>()
        .wrap_err("Failed to list documents with prefix")?;

    Ok(paths)
}

#[cfg(test)]
mod tests {
    use super::*;
    use automerge::ReadDoc;
    use automerge::transaction::Transactable;

    #[test]
    fn test_document_crud() {
        // Create in-memory database for testing
        let conn = Connection::open_in_memory().unwrap();
        migration::initialize_database(&conn).unwrap();

        // Create a document
        let mut doc = AutoCommit::new();
        doc.put(automerge::ROOT, "test_key", "test_value").unwrap();

        // Save it
        create_and_save_document(&conn, "/test/path", doc.clone()).unwrap();

        // Load it back
        let loaded = load_document(&conn, "/test/path").unwrap().unwrap();
        let value = loaded
            .doc
            .get(automerge::ROOT, "test_key")
            .unwrap()
            .map(|(v, _)| v.to_str().unwrap().to_string());
        assert_eq!(value, Some("test_value".to_string()));

        // Update it
        doc.put(automerge::ROOT, "test_key", "updated_value")
            .unwrap();
        update_document(&conn, "/test/path", &mut doc).unwrap();

        // Load updated version
        let updated = load_document(&conn, "/test/path").unwrap().unwrap();
        let value = updated
            .doc
            .get(automerge::ROOT, "test_key")
            .unwrap()
            .map(|(v, _)| v.to_str().unwrap().to_string());
        assert_eq!(value, Some("updated_value".to_string()));

        // List documents
        let paths = list_document_paths(&conn).unwrap();
        assert_eq!(paths, vec!["/test/path"]);

        // Delete it
        delete_document(&conn, "/test/path").unwrap();
        assert!(load_document(&conn, "/test/path").unwrap().is_none());
    }

    #[test]
    fn test_list_with_prefix() {
        let conn = Connection::open_in_memory().unwrap();
        migration::initialize_database(&conn).unwrap();

        // Create documents with different paths
        let doc = AutoCommit::new();
        create_and_save_document(&conn, "/-/aliases/abc123/readme", doc.clone()).unwrap();
        create_and_save_document(&conn, "/-/aliases/abc123/notes", doc.clone()).unwrap();
        create_and_save_document(&conn, "/-/aliases/def456/readme", doc.clone()).unwrap();
        create_and_save_document(&conn, "/-/mails/default", doc).unwrap();

        // List all under /-/aliases/abc123/
        let paths = list_documents_with_prefix(&conn, "/-/aliases/abc123/").unwrap();
        assert_eq!(paths.len(), 2);
        assert!(paths.contains(&"/-/aliases/abc123/readme".to_string()));
        assert!(paths.contains(&"/-/aliases/abc123/notes".to_string()));

        // List all under /-/aliases/
        let paths = list_documents_with_prefix(&conn, "/-/aliases/").unwrap();
        assert_eq!(paths.len(), 3);
    }
}
