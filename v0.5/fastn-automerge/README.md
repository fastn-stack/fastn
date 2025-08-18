# fastn-automerge

A high-level Rust library and CLI for managing Automerge CRDT documents with SQLite storage. Built for the FASTN P2P system but usable as a standalone tool.

## Features

- 🗄️ SQLite-backed persistent storage for Automerge documents
- 🔍 Path-based document organization (filesystem-like paths)
- 🦀 Type-safe Rust API with derive macros
- 🛠️ CLI for document management operations
- 📦 Zero-copy document access where possible
- 🔄 Preserves full document history and metadata
- 🎭 Unique actor ID per document for clean history

## Quick Start

### Installation

```toml
[dependencies]
fastn-automerge = "0.1"
```

No need to add `autosurgeon` separately - we re-export everything you need.

### Basic Usage

```rust
use fastn_automerge::{Db, Reconcile, Hydrate, Result};

#[derive(Debug, Clone, Reconcile, Hydrate)]
struct Config {
    name: String,
    version: u32,
}

fn main() -> Result<()> {
    // Open with actor ID (required for all operations)
    let actor_id = "a3f2b1c8-9d4e-4a6b-8c3d-1e2f3a4b5c6d-1".to_string();
    let db_path = std::path::Path::new("fastn-automerge.sqlite");
    let db = Db::open_with_actor(db_path, actor_id)?;
    
    // Create
    let config = Config { name: "app".into(), version: 1 };
    db.create("/-/config", &config)?;
    
    // Read
    let loaded: Config = db.get("/-/config")?;
    
    // Update entire document
    let mut updated_config = loaded;
    updated_config.version = 2;
    db.update("/-/config", &updated_config)?;
    
    // Or modify in place
    db.modify::<Config, _>("/-/config", |c| c.version = 3)?;
    
    // Check if document exists
    if db.exists("/-/config")? {
        println!("Config exists!");
    }
    
    // List documents
    let all_docs = db.list(None)?;
    let config_docs = db.list(Some("/-/config"))?;
    
    // Get document history
    let history = db.history("/-/config", None)?;
    println!("Document has {} edits", history.edits.len());
    
    // Get raw AutoCommit document for advanced operations
    let raw_doc = db.get_document("/-/config")?;
    
    // Delete document
    db.delete("/-/config")?;
    
    Ok(())
}
```

## CLI Usage (Coming Soon)

```bash
# Initialize database
fastn-automerge init

# CRUD operations
fastn-automerge create /-/config '{"name": "test", "version": 1}'
fastn-automerge get /-/config
fastn-automerge update /-/config '{"version": 2}'
fastn-automerge delete /-/config

# List documents
fastn-automerge list --prefix /-/config

# Clean database
fastn-automerge clean [--force]

# Show document history
fastn-automerge history <doc-id> [<commit-hash>] [--short]
```

## Documentation

- [Tutorial](TUTORIAL.md) - Comprehensive guide with examples
- [API Docs](https://docs.rs/fastn-automerge) - Full API reference
- [CLI Reference](#cli-commands) - Command-line tool documentation

## CLI Commands (Coming Soon)

### Core Commands

- `init` - Initialize a new database
- `create <path> <json>` - Create a new document
- `get <path>` - Read a document as JSON
- `update <path> <json>` - Update an existing document
- `set <path> <json>` - Create or replace a document
- `delete <path>` - Delete a document
- `list` - List all documents
- `clean [--force]` - Clean up database (remove all documents)
- `history <doc-id> [<commit-hash>] [--short]` - Show document edit history
- `info <path>` - Show document metadata

### Options

All commands support:
- `--db <path>` - Use custom database (default: `fastn-automerge.sqlite`)
- `--pretty` - Pretty-print JSON output
- `--help` - Show command help

## Path Conventions

Documents use filesystem-like paths:
- `/-/config` - Global configuration
- `/-/accounts/{id}` - Account documents  
- `/-/rig/{id}/config` - Rig configuration

## Design: Actor IDs and Permissions

### Actor ID Scheme
- **Format**: `{alias-id52}-{device-number}` (e.g., `1oem6e10...-1`)
- **Creation**: Each document stores its creation alias in database
- **Consistency**: All edits use the creation alias (no history rewriting in common case)
- **Cross-alias sharing**: History rewritten only when sharing with different alias (rare)

### Permission System
- **Built-in enforcement**: Read-only documents cannot be edited locally
- **Metadata documents**: Stored at `{doc_path}/-/meta` for each document
- **Group documents**: Stored at `/-/groups/{group-name}`
- **Permission levels**: Read, Write, Admin
- **Permission propagation**: Meta documents shared alongside content
- **Nested groups**: Groups can contain other groups
- **Permission inheritance**: Members inherit permissions from their groups

#### Document Structure

**Group Document** (`/-/groups/{name}`):
```json
{
  "name": "engineers",
  "description": "Engineering team",
  "accounts": ["alice123...", "bob456..."],
  "groups": ["backend-team", "frontend-team"],  // Nested groups
  "created_by": "alice123...",
  "created_at": 1234567890
}
```

**Document Metadata** (`{doc_path}/-/meta`):
```json
{
  "owner": "alice123...",
  "permissions": {
    "accounts": {
      "bob456...": "read",
      "carol789...": "write"
    },
    "groups": {
      "engineers": "write",
      "managers": "admin"
    }
  },
  "created_at": 1234567890
}
```

### Optimization
- **Single alias optimization**: Most users have one alias, no rewriting needed
- **Creation alias stored**: In `fastn_documents.created_alias` column
- **History rewrite only on mismatch**: When sharing alias differs from creation alias

## Database Schema

```sql
CREATE TABLE fastn_documents (
    path              TEXT PRIMARY KEY,
    created_alias     TEXT NOT NULL,      -- Alias used at creation (for actor ID)
    automerge_binary  BLOB NOT NULL,
    heads             TEXT NOT NULL,      -- JSON array
    updated_at        INTEGER NOT NULL    -- Unix timestamp
);

-- Actor ID format: {created_alias}-{device-num}
-- No history rewriting needed when using same alias
```

## License

MIT