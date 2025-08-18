# fastn-automerge Tutorial

This tutorial covers the fastn-automerge library and CLI in detail, with practical examples and best practices.

## Table of Contents

1. [Introduction](#introduction)
2. [Library Usage](#library-usage)
3. [CLI Usage](#cli-usage)
4. [Advanced Topics](#advanced-topics)
5. [Best Practices](#best-practices)

## Introduction

fastn-automerge provides a high-level interface for working with Automerge CRDT documents stored in SQLite. It combines:

- **Automerge**: A CRDT that enables automatic merging of concurrent changes
- **Autosurgeon**: Type-safe serialization with derive macros
- **SQLite**: Persistent storage with efficient indexing

### When to Use fastn-automerge

- Building distributed/P2P applications that need eventual consistency
- Managing configuration that can be edited from multiple sources
- Storing documents that need full version history
- Applications requiring offline-first capabilities

## Library Usage

### Setup and Initialization

```rust
use fastn_automerge::{Db, Reconcile, Hydrate, Result};

// Open default database (fastn-automerge.sqlite in current directory)
let db = Db::open()?;

// Open specific database
let db = Db::open_at("/path/to/database.sqlite")?;

// Initialize new database (fails if exists)
let db = Db::init()?;
let db = Db::init_at("/path/to/new.sqlite")?;
```

### Defining Your Data Structures

Use the `Reconcile` and `Hydrate` derive macros (re-exported from autosurgeon):

```rust
use fastn_automerge::{Reconcile, Hydrate};
use std::collections::HashMap;

#[derive(Debug, Clone, Reconcile, Hydrate, PartialEq)]
struct User {
    id: String,
    name: String,
    email: String,
    age: u32,
    active: bool,
}

// Nested structures work automatically
#[derive(Debug, Clone, Reconcile, Hydrate)]
struct Team {
    name: String,
    members: Vec<User>,
    settings: TeamSettings,
    metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Reconcile, Hydrate)]
struct TeamSettings {
    public: bool,
    max_members: usize,
    permissions: Vec<Permission>,
}

// Enums are supported
#[derive(Debug, Clone, Reconcile, Hydrate)]
enum Permission {
    Read,
    Write,
    Admin,
    Custom(String),
}
```

### CRUD Operations

#### Create

```rust
fn create_example(db: &Db) -> Result<()> {
    let user = User {
        id: "user123".to_string(),
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
        age: 30,
        active: true,
    };
    
    // Create new document (fails if exists)
    db.create("/-/users/user123", &user)?;
    
    // Create or replace (upsert)
    db.set("/-/users/user123", &user)?;
    
    Ok(())
}
```

#### Read

```rust
fn read_example(db: &Db) -> Result<()> {
    // Type-safe read
    let user: User = db.get("/-/users/user123")?;
    println!("User: {} ({})", user.name, user.email);
    
    // Handle not found
    match db.get::<User>("/-/users/unknown") {
        Ok(user) => println!("Found: {}", user.name),
        Err(fastn_automerge::Error::NotFound(path)) => {
            println!("User not found at: {}", path);
        }
        Err(e) => return Err(e),
    }
    
    Ok(())
}
```

#### Update

```rust
fn update_example(db: &Db) -> Result<()> {
    // Method 1: Modify with closure (recommended)
    db.modify::<User, _>("/-/users/user123", |user| {
        user.age = 31;
        user.email = "alice.new@example.com".to_string();
    })?;
    
    // Method 2: Load, modify, update
    let mut user: User = db.get("/-/users/user123")?;
    user.active = false;
    db.update("/-/users/user123", &user)?;
    
    // Method 3: Partial update with raw document
    let mut doc = db.get_document("/-/users/user123")?;
    doc.put(automerge::ROOT, "last_login", chrono::Utc::now().timestamp())?;
    db.save_document("/-/users/user123", &doc)?;
    
    Ok(())
}
```

#### Delete

```rust
fn delete_example(db: &Db) -> Result<()> {
    // Delete document
    db.delete("/-/users/user123")?;
    
    // Check if exists before deleting
    if db.exists("/-/users/user123")? {
        db.delete("/-/users/user123")?;
    }
    
    Ok(())
}
```

### Listing and Querying

```rust
fn list_example(db: &Db) -> Result<()> {
    // List all documents
    let all_paths = db.list(None)?;
    for path in &all_paths {
        println!("Document: {}", path);
    }
    
    // List with prefix
    let users = db.list(Some("/-/users/"))?;
    println!("Found {} users", users.len());
    
    // Load multiple documents
    for path in db.list(Some("/-/users/"))? {
        let user: User = db.get(&path)?;
        println!("- {} ({})", user.name, user.email);
    }
    
    Ok(())
}
```

### Working with Collections

```rust
use fastn_automerge::{Reconcile, Hydrate, Text, Counter};

#[derive(Debug, Clone, Reconcile, Hydrate)]
struct Document {
    title: String,
    content: Text,  // CRDT text for collaborative editing
    views: Counter, // CRDT counter
    tags: Vec<String>,
    contributors: Vec<String>,
}

fn collections_example(db: &Db) -> Result<()> {
    // Create document with CRDT types
    let mut doc = Document {
        title: "My Document".to_string(),
        content: Text::new(),
        views: Counter::new(),
        tags: vec!["important".to_string()],
        contributors: vec!["alice".to_string()],
    };
    
    // Modify Text (CRDT-aware)
    doc.content.push_str("Hello world!");
    
    // Increment Counter (CRDT-aware)
    doc.views.increment(1);
    
    db.create("/-/docs/doc1", &doc)?;
    
    // Later updates merge automatically
    db.modify::<Document, _>("/-/docs/doc1", |doc| {
        doc.tags.push("reviewed".to_string());
        doc.contributors.push("bob".to_string());
        doc.views.increment(1);
    })?;
    
    Ok(())
}
```

### Advanced Document Access

```rust
use automerge::AutoCommit;

fn advanced_example(db: &Db) -> Result<()> {
    // Get both typed value and raw document
    let (user, mut doc): (User, AutoCommit) = db.get_with_document("/-/users/user123")?;
    
    // Access document metadata
    println!("Actor: {}", doc.get_actor());
    println!("Heads: {:?}", doc.get_heads());
    
    // Add custom fields not in the struct
    doc.put(automerge::ROOT, "last_modified_by", "system")?;
    doc.put(automerge::ROOT, "schema_version", 2)?;
    
    // Save the modified document
    db.save_document("/-/users/user123", &doc)?;
    
    // Get just the raw document
    let doc = db.get_document("/-/users/user123")?;
    
    Ok(())
}
```

## CLI Usage

### Installation

```bash
# Install globally
cargo install fastn-automerge

# Or build from source
git clone https://github.com/fastn/fastn-automerge
cd fastn-automerge
cargo build --release
```

### Basic Commands

```bash
# Initialize a new database
fastn-automerge init
fastn-automerge init --db /path/to/custom.sqlite

# Create documents from JSON
fastn-automerge create /-/config '{"app_name": "MyApp", "version": 1}'

# Create from file
echo '{"name": "Alice", "age": 30}' > user.json
fastn-automerge create /-/users/alice --file user.json

# Read documents
fastn-automerge get /-/config
fastn-automerge get /-/config --pretty
fastn-automerge get /-/config --output config.json

# Update (merges with existing)
fastn-automerge update /-/config '{"version": 2}'

# Set (replaces entirely)
fastn-automerge set /-/config '{"app_name": "NewApp", "version": 3}'

# Delete
fastn-automerge delete /-/config
fastn-automerge delete /-/config --confirm

# List documents
fastn-automerge list
fastn-automerge list --prefix /-/users/
fastn-automerge list --details

# Show metadata
fastn-automerge info /-/config
```

### Batch Operations

```bash
# Create multiple documents
for i in {1..10}; do
    fastn-automerge create "/-/items/$i" "{\"id\": $i, \"name\": \"Item $i\"}"
done

# Export all documents
for path in $(fastn-automerge list); do
    fastn-automerge get "$path" > "$(basename $path).json"
done

# Delete with prefix
for path in $(fastn-automerge list --prefix /-/temp/); do
    fastn-automerge delete "$path"
done
```

### Using with Scripts

```bash
#!/bin/bash

# Get configuration value
APP_NAME=$(fastn-automerge get /-/config | jq -r '.app_name')

# Update counter
CURRENT=$(fastn-automerge get /-/stats | jq '.visits // 0')
NEW=$((CURRENT + 1))
fastn-automerge update /-/stats "{\"visits\": $NEW}"

# Conditional update
if fastn-automerge get /-/users/alice 2>/dev/null; then
    echo "User exists, updating..."
    fastn-automerge update /-/users/alice '{"last_seen": "2024-01-15"}'
else
    echo "Creating new user..."
    fastn-automerge create /-/users/alice '{"name": "Alice", "created": "2024-01-15"}'
fi
```

## Advanced Topics

### Actor ID Design

fastn-automerge uses a simplified actor ID system optimized for single-alias usage.

#### Actor ID Format

- **Structure**: `{alias-id52}-{device-number}` (e.g., `alice123...-1`)
- **No GUID needed**: Direct use of alias IDs, no internal/external distinction
- **Device numbering**: 
  - Device 1: The account itself
  - Device 2+: Additional devices owned by the account

#### Creation Alias Optimization

1. **Document creation**: Stores the alias used at creation in `created_alias` field
2. **Consistent editing**: All edits use the creation alias as actor prefix
3. **No rewriting in common case**: When sharing with same alias (90%+ of cases)
4. **Rewrite only on mismatch**: When sharing with different alias (rare cross-alias shares)

#### How It Works

```rust
// Account uses first alias for actor ID
let actor_id = "alice123...-1";  // Device 1 is the account

// Create document (stores creation alias)
db.create_with_alias("mine/doc", &content, "alice123...")?;

// Edit document (uses stored creation alias)
db.update("mine/doc", &updates)?;  // Uses alice123...-1 automatically
// This marks document as needing sync with all peers who have access

// When peer comes online, get ALL patches they need
let pending = db.get_pending_patches_for_peer("alice123...", "bob456...")?;
// Returns Vec<(doc_path, patch)> for all docs bob456 needs
// Handles history rewriting if our_alias != created_alias
// Updates sync state to mark as synced

// Periodic sync check - who needs updates?
let outdated_peers = db.get_peers_needing_sync()?;
// Returns Vec<(peer_alias, our_alias, doc_count)>
// Example: [("bob456...", "alice123...", 3), ("carol789...", "alice-work...", 1)]
```

#### Implementation Details

```rust
pub struct Db {
    conn: Connection,
    actor_id: String,  // Passed during creation (e.g., "guid-1")
}

impl Db {
    /// Open database with actor ID
    pub fn open_with_actor(actor_id: impl Into<String>) -> Result<Self> {
        let conn = Connection::open("fastn-automerge.sqlite")?;
        Ok(Self {
            conn,
            actor_id: actor_id.into(),
        })
    }
    
    /// Initialize database with actor ID from file
    pub fn open_from_path(base_path: &Path) -> Result<Self> {
        let actor_id = std::fs::read_to_string(base_path.join("automerge.actor-id"))?;
        let conn = Connection::open(base_path.join("automerge.sqlite"))?;
        Ok(Self { conn, actor_id })
    }
    
    /// Update document with current actor (internal use)
    pub fn update<T: Reconcile>(&self, path: &str, value: &T) -> Result<()> {
        let mut doc = self.load_document(path)?;
        doc.set_actor(ActorId::from(self.actor_id.as_bytes()));
        value.reconcile(&mut doc, automerge::ROOT)?;
        self.save_document(path, &doc)?;
        Ok(())
    }
    
    /// Get pending patches for a peer
    pub fn get_pending_patches_for_peer(
        &self, 
        our_alias: &str,
        peer_alias: &str
    ) -> Result<Vec<(String, Vec<u8>)>> {
        // Find all docs needing sync with this peer
        // Check created_alias vs our_alias for rewrite decision
        // Return (doc_path, patch) pairs
        Ok(vec![])
    }
    
    /// Get list of peers needing sync
    pub fn get_peers_needing_sync(&self) -> Result<Vec<(String, String, usize)>> {
        // Return (peer_alias, our_alias, doc_count)
        Ok(vec![])
    }
    
    /// Group management
    pub fn create_group(&self, name: &str, description: &str) -> Result<()> {
        let path = format!("/-/groups/{}", name);
        // Create group document
        Ok(())
    }
    
    pub fn add_account_to_group(&self, group: &str, account: &str) -> Result<()> {
        let path = format!("/-/groups/{}", group);
        // Update group document
        Ok(())
    }
    
    /// Permission management
    pub fn grant_account_access(
        &self, 
        doc_path: &str, 
        account: &str, 
        perm: Permission
    ) -> Result<()> {
        let meta_path = format!("{}/-/meta", doc_path);
        // Update metadata document
        Ok(())
    }
}

/// Rewrite actor IDs for cross-alias sharing (rare case)
fn rewrite_actor_history(
    doc: AutoCommit, 
    from_alias: &str,    // e.g., "alice123..."
    to_alias: &str       // e.g., "alice-work..."
) -> Result<AutoCommit> {
    // Extract device numbers and rewrite
    // from: alice123...-1, alice123...-2
    // to: alice-work...-1, alice-work...-2
    
    // (Implementation would use Automerge's internal APIs)
    Ok(rewritten_doc)
}

/// Verify all edits come from expected alias
fn verify_actor_history(doc: &AutoCommit, expected_alias: &str) -> Result<()> {
    // Check that all actor IDs match pattern: {expected_alias}-{number}
    for actor in doc.get_actors() {
        if !actor.starts_with(expected_alias) {
            return Err(Error::InvalidActor(actor));
        }
    }
    Ok(())
}
```

#### Security Properties

1. **Attribution**: Can track edits to specific devices
2. **Non-repudiation**: Can't claim someone else's edits as yours
3. **Verifiability**: Recipients can verify edit authenticity
4. **Efficiency**: No rewriting in common single-alias case

#### Example Flow

```rust
// Alice's account - device 1
let alice_actor = "alice123...-1";

// Alice creates document (stores creation alias)
db.create("/-/docs/shared", &doc)?;  // Uses alice123...-1
// DB stores created_alias = "alice123..."

// Alice shares with Bob (same alias - no rewrite!)
let shared_binary = db.get_patch("/-/docs/shared")?;
// Still shows alice123...-1 as actor

// Bob receives and verifies
let doc = bob_db.apply_patch_from_alias(&shared_binary, "alice123...")?;
// ✓ Verified: all edits from alice123...-{device}

// Bob creates his response
bob_db.create("/-/response", &response)?;  // Uses bob456...-1
// DB stores created_alias = "bob456..."

// Bob shares back (no rewrite needed)
let return_binary = bob_db.get_patch("/-/response")?;

// Alice receives and verifies Bob's edits
alice_db.apply_patch_from_alias(&return_binary, "bob456...")?;
// ✓ All edits show bob456...-1

// Rare case: Alice shares with different alias
let work_patch = db.get_patch_for_alias("/-/docs/shared", "alice-work...")?;
// NOW history is rewritten: alice123...-1 → alice-work...-1
```

### Understanding Automerge

Automerge is a CRDT (Conflict-free Replicated Data Type) that allows:

1. **Automatic Merging**: Changes from different sources merge without conflicts
2. **Full History**: Every change is preserved and can be inspected
3. **Deterministic**: Same changes always produce the same result

```rust
// Example: Two users editing simultaneously
fn concurrent_edits(db: &Db) -> Result<()> {
    // User A's changes (uses document's actor_id)
    db.modify::<Document, _>("/-/doc", |doc| {
        doc.title = "Updated by A".to_string();
        doc.tags.push("a-tag".to_string());
    })?;
    
    // User B's changes (same document actor_id)
    db.modify::<Document, _>("/-/doc", |doc| {
        doc.content.push_str(" Added by B");
        doc.tags.push("b-tag".to_string());
    })?;
    
    // Both changes are preserved with the same actor!
    let doc: Document = db.get("/-/doc")?;
    assert!(doc.title.contains("A"));
    assert!(doc.tags.contains(&"a-tag".to_string()));
    assert!(doc.tags.contains(&"b-tag".to_string()));
    
    Ok(())
}
```

### Performance Optimization

```rust
// Batch operations in a transaction
fn batch_operations(db: &Db) -> Result<()> {
    // Future: Transaction support
    // db.transaction(|tx| {
    //     for i in 0..1000 {
    //         tx.create(&format!("/-/items/{}", i), &item)?;
    //     }
    //     Ok(())
    // })?;
    
    // For now, use bulk operations when possible
    let mut doc = AutoCommit::new();
    let items = (0..1000).map(|i| Item { id: i }).collect::<Vec<_>>();
    items.reconcile(&mut doc, automerge::ROOT)?;
    db.save_document("/-/items", &doc)?;
    
    Ok(())
}
```

### Error Handling Patterns

```rust
use fastn_automerge::{Result, Error};

fn robust_operations(db: &Db) -> Result<()> {
    // Pattern 1: Default on not found
    let config = match db.get::<Config>("/-/config") {
        Ok(c) => c,
        Err(Error::NotFound(_)) => Config::default(),
        Err(e) => return Err(e),
    };
    
    // Pattern 2: Create if not exists
    if !db.exists("/-/config")? {
        db.create("/-/config", &Config::default())?;
    }
    
    // Pattern 3: Retry on database lock
    let mut retries = 3;
    loop {
        match db.update("/-/config", &config) {
            Ok(_) => break,
            Err(Error::Database(e)) if retries > 0 => {
                eprintln!("Database busy, retrying... {}", e);
                std::thread::sleep(std::time::Duration::from_millis(100));
                retries -= 1;
            }
            Err(e) => return Err(e),
        }
    }
    
    Ok(())
}
```

## Best Practices

### 1. Path Organization

Use a consistent naming scheme:

```rust
// Good: Hierarchical and predictable
"/-/config"                    // Global config
"/-/users/{id}"                // User documents
"/-/teams/{team_id}/members"  // Team members
"/-/logs/2024/01/15"          // Date-organized logs

// Avoid: Inconsistent or flat structure
"/config"
"/user_123"
"/stuff/misc/data"
```

### 2. Document Size

Keep documents reasonably sized:

```rust
// Good: Separate documents for different concerns
db.create("/-/users/alice/profile", &profile)?;
db.create("/-/users/alice/settings", &settings)?;
db.create("/-/users/alice/activity", &activity)?;

// Avoid: One huge document
db.create("/-/users/alice", &giant_user_object)?;
```

### 3. Schema Evolution

Plan for schema changes:

```rust
#[derive(Reconcile, Hydrate)]
struct ConfigV2 {
    version: u32,  // Always include version
    name: String,
    // New fields should be Option<T> for compatibility
    description: Option<String>,
    #[serde(default)]
    features: Vec<String>,
}

fn migrate(db: &Db) -> Result<()> {
    let doc = db.get_document("/-/config")?;
    // Check version and migrate if needed
    Ok(())
}
```

### 4. Concurrent Access

Design for concurrent modifications:

```rust
// Good: Modify specific fields
db.modify::<User, _>("/-/users/alice", |u| {
    u.last_seen = Utc::now();
})?;

// Avoid: Replace entire document for small changes
let mut user = db.get::<User>("/-/users/alice")?;
user.last_seen = Utc::now();
db.set("/-/users/alice", &user)?;  // Overwrites concurrent changes!
```

### 5. Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_operations() -> Result<()> {
        let tmp = TempDir::new()?;
        let db_path = tmp.path().join("test.sqlite");
        let db = Db::init_at(&db_path)?;
        
        // Test your operations
        let user = User { /* ... */ };
        db.create("/-/test", &user)?;
        
        let loaded: User = db.get("/-/test")?;
        assert_eq!(user.id, loaded.id);
        
        Ok(())
    }
}
```

## Troubleshooting

### Common Issues

1. **Database Locked**: SQLite only allows one writer at a time
   - Solution: Use connection pooling or retry logic

2. **Document Not Found**: Trying to update non-existent document
   - Solution: Use `db.set()` instead of `db.update()` or check with `db.exists()`

3. **Serialization Errors**: Type mismatch between stored and expected
   - Solution: Include version field and handle migrations

4. **Performance Issues**: Large documents or too many documents
   - Solution: Split large documents, use indexes, batch operations

## Further Reading

- [Automerge Documentation](https://automerge.org/)
- [Autosurgeon Documentation](https://docs.rs/autosurgeon)
- [SQLite Best Practices](https://www.sqlite.org/bestpractice.html)
- [CRDT Introduction](https://crdt.tech/)