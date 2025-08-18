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
    let actor_id = "a3f2b1c8-9d4e-4a6b-8c3d-1e2f3a4b5c6d-1";
    let db = Db::open_with_actor(actor_id)?;
    
    // Create
    let config = Config { name: "app".into(), version: 1 };
    db.create("/-/config", &config)?;
    
    // Read
    let loaded: Config = db.get("/-/config")?;
    
    // Update (uses creation alias internally, marks for sync)
    db.modify::<Config, _>("/-/config", |c| c.version = 2)?;
    
    // Get patches for peer when they come online
    let patches = db.get_pending_patches_for_peer("alice123...", "bob456...")?;
    // Vec<(doc_path, patch)> - all docs bob needs
    
    // Check who needs sync
    let peers = db.get_peers_needing_sync()?;
    // Vec<(peer_alias, our_alias, doc_count)>
    
    // Apply received patch
    db.apply_patch_from_alias("/-/peer/config", &patch, "bob456...")?;
    
    // Group management
    db.create_group("engineers", "Engineering team")?;
    db.add_account_to_group("engineers", "alice123...")?;
    db.add_group_to_group("engineers", "backend-team")?;  // Nested
    db.remove_account_from_group("engineers", "alice123...")?;
    db.remove_group_from_group("engineers", "backend-team")?;
    
    // Grant permissions
    db.grant_account_access("/-/config", "bob456...", Permission::Read)?;
    db.grant_group_access("/-/config", "engineers", Permission::Write)?;
    db.revoke_account_access("/-/config", "bob456...")?;
    db.revoke_group_access("/-/config", "engineers")?;
    
    Ok(())
}
```

## CLI Usage

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
```

## Documentation

- [Tutorial](TUTORIAL.md) - Comprehensive guide with examples
- [API Docs](https://docs.rs/fastn-automerge) - Full API reference
- [CLI Reference](#cli-commands) - Command-line tool documentation

## CLI Commands

### Core Commands

- `init` - Initialize a new database
- `create <path> <json>` - Create a new document
- `get <path>` - Read a document as JSON
- `update <path> <json>` - Update an existing document
- `set <path> <json>` - Create or replace a document
- `delete <path>` - Delete a document
- `list` - List all documents
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