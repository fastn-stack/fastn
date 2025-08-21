pub mod cli;
pub mod db;
pub mod error;
pub mod migration;
pub mod tests;
pub mod utils;

// Re-export autosurgeon traits and functions
pub use autosurgeon::{Hydrate, Reconcile, hydrate, reconcile};

// Re-export automerge types we use
pub use automerge::AutoCommit;

pub type Result<T> = std::result::Result<T, Box<Error>>;

#[derive(Debug, Clone, PartialEq)]
pub enum DocumentIdError {
    Empty,
    TooManyPrefixes { count: usize },
}

impl std::fmt::Display for DocumentIdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DocumentIdError::Empty => write!(f, "Document ID cannot be empty"),
            DocumentIdError::TooManyPrefixes { count } => {
                write!(f, "Document ID can contain at most one '/-/' prefix, found {count}")
            }
        }
    }
}

impl std::error::Error for DocumentIdError {}

/// Generic typed document ID that can only be created by ID constructors
#[derive(Debug, Clone, PartialEq)]
pub struct DocumentId(String);

impl DocumentId {
    /// Create document ID from string with validation - the only way to create document IDs
    pub fn from_string(id: &str) -> std::result::Result<Self, DocumentIdError> {
        // Add basic validation for all document IDs
        if id.is_empty() {
            return Err(DocumentIdError::Empty);
        }
        
        // Check that at most one /-/ prefix exists
        let slash_dash_count = id.matches("/-/").count();
        if slash_dash_count > 1 {
            return Err(DocumentIdError::TooManyPrefixes { count: slash_dash_count });
        }
        
        Ok(Self(id.to_string()))
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl rusqlite::ToSql for DocumentId {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        self.0.to_sql()
    }
}

impl std::fmt::Display for DocumentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
pub enum Error {
    NotFound(String),
    Database(rusqlite::Error),
    Automerge(automerge::AutomergeError),
    Autosurgeon(autosurgeon::HydrateError),
    ReconcileError(autosurgeon::ReconcileError),
}

pub struct Db {
    pub(crate) conn: rusqlite::Connection,
    pub(crate) actor_id: String,
}

/// Represents a single operation within an edit
#[derive(Debug, Clone)]
pub enum Operation {
    /// Set a key to a value in a map
    Set {
        path: Vec<String>, // e.g., ["user", "name"]
        key: String,
        value: String, // String representation of the value
    },
    /// Delete a key from a map
    Delete { path: Vec<String>, key: String },
    /// Insert an item into a list
    Insert {
        path: Vec<String>,
        index: usize,
        value: String,
    },
    /// Delete an item from a list
    Remove { path: Vec<String>, index: usize },
    /// Increment a counter
    Increment {
        path: Vec<String>,
        key: String,
        delta: i64,
    },
}

/// Represents a single edit/change in an Automerge document's history
#[derive(Debug, Clone)]
pub struct Edit {
    /// Sequential index of this edit in the document's history
    pub index: usize,
    /// Unique hash identifying this change
    pub hash: String,
    /// The actor (user/device) who made this change
    pub actor_id: String,
    /// Unix timestamp when this change was made
    pub timestamp: i64,
    /// Optional commit message describing the change
    pub message: Option<String>,
    /// The actual operations/changes made in this edit
    pub operations: Vec<Operation>,
}

/// Complete history of a document including metadata and all edits
#[derive(Debug)]
pub struct DocumentHistory {
    /// Path/ID of the document
    pub path: String,
    /// The alias/actor who originally created this document
    pub created_alias: String,
    /// Unix timestamp of last update
    pub updated_at: i64,
    /// Current heads of the document
    ///
    /// Heads in Automerge are like Git commits - they represent the latest changes
    /// in different branches of the document's history. When you have multiple heads,
    /// it means there are concurrent edits that haven't been merged yet.
    ///
    /// - Single head: Document has a linear history (no concurrent edits)
    /// - Multiple heads: Document has concurrent edits from different actors
    ///   that Automerge will automatically merge using CRDTs
    ///
    /// Each head is a hash of a change, similar to a Git commit hash.
    pub heads: Vec<String>,
    /// List of all edits/changes made to this document
    pub edits: Vec<Edit>,
}
