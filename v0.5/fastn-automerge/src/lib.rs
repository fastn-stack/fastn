extern crate self as fastn_automerge;

/// Sentinel value for uninitialized entity ID
pub const UNINITIALIZED_ENTITY: &str = "uninitialized";

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

// Use eyre::Result for now - provides good error handling with context
pub type Result<T> = eyre::Result<T>;

#[derive(Debug, Clone, PartialEq)]
pub enum DocumentIdError {
    Empty,
    TooManyPrefixes { count: usize },
}


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
            return Err(DocumentIdError::TooManyPrefixes {
                count: slash_dash_count,
            });
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

#[derive(Debug)]
pub enum LoadError {
    NotFound(std::path::PathBuf),
    NotInitialized(std::path::PathBuf),
    MissingActorCounter,
    DatabaseError(rusqlite::Error),
}


pub struct Db {
    pub(crate) conn: rusqlite::Connection,
    /// Entity this database belongs to
    pub(crate) entity_id52: String,
    /// Device number for this database
    pub(crate) device_number: u32,
    /// Mutex for serializing all database operations
    pub(crate) mutex: std::sync::Mutex<()>,
}

impl Db {
    /// Check if the actor ID has been properly initialized
    pub fn is_actor_id_initialized(&self) -> bool {
        self.entity_id52 != UNINITIALIZED_ENTITY
    }
    
    /// Get the full actor ID string (panics if not properly initialized)
    pub fn actor_id(&self) -> String {
        assert_ne!(self.entity_id52, UNINITIALIZED_ENTITY, 
            "Database actor ID not properly initialized - call set_actor_id() first");
        format!("{}-{}", self.entity_id52, self.device_number)
    }
    
    /// Set the actor ID (can only be called once during initialization)
    pub fn set_actor_id(&mut self, entity_id52: &str, device_number: u32) -> eyre::Result<()> {
        if self.is_actor_id_initialized() {
            return Err(eyre::eyre!("Actor ID already initialized - cannot change"));
        }
        self.entity_id52 = entity_id52.to_string();
        self.device_number = device_number;
        Ok(())
    }
    
    /// Ensure actor ID is initialized before database operations
    fn require_initialized(&self) -> std::result::Result<(), ActorIdNotSet> {
        if !self.is_actor_id_initialized() {
            return Err(ActorIdNotSet);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActorIdNotSet;



#[derive(Debug, Clone, PartialEq)]
pub struct ActorIdAlreadySet;



#[derive(Debug)]
pub enum CreateError {
    ActorNotSet(ActorIdNotSet),
    DocumentExists(DocumentId),
    Database(rusqlite::Error),
    Automerge(automerge::AutomergeError),
    Reconcile(autosurgeon::ReconcileError),
}



#[derive(Debug)]
pub enum GetError {
    ActorNotSet(ActorIdNotSet),
    NotFound(DocumentId),
    Database(rusqlite::Error),
    Automerge(automerge::AutomergeError),
    Hydrate(autosurgeon::HydrateError),
}



#[derive(Debug)]
pub enum UpdateError {
    ActorNotSet(ActorIdNotSet),
    NotFound(DocumentId),
    Database(rusqlite::Error),
    Automerge(automerge::AutomergeError),
    Reconcile(autosurgeon::ReconcileError),
}



impl std::fmt::Debug for Db {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Db")
            .field("entity_id52", &self.entity_id52)
            .field("device_number", &self.device_number)
            .field("conn", &"<rusqlite::Connection>")
            .finish()
    }
}

/// # Actor ID Management in fastn-automerge
///
/// fastn-automerge is **actor ID aware** and handles actor ID logic internally.
/// This enables proper CRDT functionality and **privacy protection**.
///
/// ## Actor ID Format
///
/// **Supported Format**: `{entity_id52}-{device_number}`
/// - `entity_id52`: The ID52 of the account/rig that owns this database
/// - `device_number`: Zero-based device counter (0 = primary device, 1+ = additional devices)
///
/// **Examples**:
/// - `alice123...-0` - Alice's primary device (account itself)
/// - `alice123...-1` - Alice's second device (phone, laptop, etc.)
/// - `rig456...-0` - Rig's primary device (rig itself)
///
/// ## Actor ID Rewriting for Privacy Protection
///
/// **Critical Privacy Feature**: fastn-automerge performs actor ID rewriting to prevent
/// account linkage attacks:
///
/// **The Problem**: If Alice shares documents from different aliases but uses the same
/// actor ID, recipients can discover that multiple aliases belong to the same person:
/// - Document A shows actor: `alice123...-0` (shared from alias1)  
/// - Document B shows actor: `alice123...-0` (shared from alias2)
/// - Recipient can link alias1 and alias2 as same account ❌
///
/// **The Solution**: Actor ID rewriting ensures each shared alias appears independent:
/// - When sharing from alias1: All edits appear as `alias1_id52-0` 
/// - When sharing from alias2: All edits appear as `alias2_id52-0`
/// - Recipients cannot link different aliases ✅
///
/// **Implementation**: Only supports `id52-count` format for privacy-preserving rewriting.
///
/// ## Database API
///
/// - `Db::init()` - Creates database with temporary actor ID
/// - `Db::open()` - Opens database with temporary actor ID  
/// - `db.set_actor_id(entity_id52, device_num)` - Sets proper actor ID after creation/opening
/// - `db.next_actor_id(entity_id52)` - Gets next device number atomically (thread-safe)

/// Actor counter document stored at /-/system/actor_counter
/// 
/// **Important**: Only account databases generate and assign device IDs to new devices.
/// Individual device databases do not need to track next_device.
#[derive(Debug, Clone, PartialEq, Reconcile, Hydrate)]
pub struct ActorCounter {
    /// Entity this database belongs to
    pub entity_id52: String,
    /// Next device number to assign (only used by account databases)
    pub next_device: u32,
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
