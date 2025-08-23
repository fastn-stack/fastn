# fastn-automerge

A high-level Rust library for managing Automerge CRDT documents with SQLite storage. 
Provides three different APIs through derive macros for maximum flexibility and type safety.

## 🚀 Quick Start

```rust
use fastn_automerge::{Db, Document, Reconcile, Hydrate};
use fastn_id52::PublicKey;

#[derive(Debug, Clone, Document, Reconcile, Hydrate)]
#[document_path("/-/users/{id52}/profile")]
struct UserProfile {
    #[document_id52]
    id: PublicKey,
    name: String,
    bio: Option<String>,
}

// Simple usage
let db = Db::init("app.sqlite", &entity)?;
let user = UserProfile { /* ... */ };
user.save(&db)?;
let loaded = UserProfile::load(&db, &user.id)?;
let all_users = UserProfile::document_list(&db)?; // List all users!
```

## 📋 Three APIs for Different Use Cases

| API | When to Use | Example | Generated Functions |
|-----|-------------|---------|-------------------|
| **Template-based** | Entity-specific documents | `#[document_path("/-/users/{id52}/profile")]` | `save(db)`, `load(db, &id)`, `document_list(db)` |
| **Singleton** | Global/config documents | `#[document_path("/-/app/settings")]` | `save(db)`, `load(db)` |
| **Path-based** | Maximum flexibility | No `#[document_path]` | `save(db, &path)`, `load(db, &path)` |

## 🎯 Features

- **🦀 Type-safe**: Compile-time path validation and type checking
- **🔍 Smart listing**: `document_list()` with exact DNSSEC32 validation  
- **⚡ Performance**: SQL prefix filtering + Rust validation
- **🗄️ SQLite storage**: Efficient persistence with indexing
- **🔄 CRDT support**: Built on Automerge for conflict-free editing
- **🎭 Actor tracking**: Automatic device/entity management
- **📦 Feature-gated CLI**: Optional command-line tools
- **📚 Full history**: Complete edit history inspection

## 📖 Documentation

- **[Tutorial](TUTORIAL.md)** - Comprehensive guide with examples
- **[API Docs](https://docs.rs/fastn-automerge)** - Full API reference  
- **[CLI Guide](#cli-usage)** - Command-line tool documentation

## 🛠️ Installation

### Library Only (Recommended)
```toml
[dependencies]
fastn-automerge = "0.1"
```
*Lightweight build - no CLI dependencies*

### With CLI Tools
```toml
[dependencies]
fastn-automerge = { version = "0.1", features = ["cli"] }
```

### CLI Binary
```bash
cargo install fastn-automerge --features=cli
```

## 💡 Examples

### Template-based API (Most Common)

```rust
#[derive(Document, Reconcile, Hydrate)]
#[document_path("/-/notes/{id52}/content")]
struct Note {
    #[document_id52] id: PublicKey,
    title: String,
    content: String,
    tags: Vec<String>,
}

// Usage
note.save(&db)?;                     // Auto path: /-/notes/abc123.../content
let loaded = Note::load(&db, &id)?;  // Load by ID
let all_notes = Note::document_list(&db)?; // List all notes
```

### Singleton API (Global State)

```rust  
#[derive(Document, Reconcile, Hydrate)]
#[document_path("/-/app/config")]
struct Config {
    theme: String,
    debug: bool,
}

// Usage
config.save(&db)?;           // Fixed path: /-/app/config
let loaded = Config::load(&db)?; // No ID needed
```

### Path-based API (Maximum Flexibility)

```rust
#[derive(Document, Reconcile, Hydrate)]
struct FlexibleDoc {
    #[document_id52] id: PublicKey,
    data: String,
}

// Usage  
let path = DocumentPath::from_string("/-/custom/path")?;
doc.save(&db, &path)?;           // Explicit path required
let loaded = FlexibleDoc::load(&db, &path)?;
```

## 🎯 CLI Usage

```bash
# Initialize database
fastn-automerge init

# Document operations
fastn-automerge create /-/users/alice '{"name": "Alice", "age": 30}'
fastn-automerge get /-/users/alice --pretty
fastn-automerge update /-/users/alice '{"age": 31}'
fastn-automerge list --prefix /-/users/

# Document history
fastn-automerge history /-/users/alice
fastn-automerge info /-/users/alice

# Cleanup
fastn-automerge delete /-/users/alice
```

## 🔧 Advanced Features

### Document History Inspection

```rust
let history = db.history(&path, None)?;
for edit in &history.edits {
    println!("Edit by {}: {} operations", edit.actor_id, edit.operations.len());
}
```

### Bulk Operations with `document_list()`

```rust
// Process all user profiles
let user_paths = UserProfile::document_list(&db)?;
for path in user_paths {
    if let Some(id) = extract_id_from_path(&path) {
        let user = UserProfile::load(&db, &id)?;
        // Process user...
    }
}
```

### Error Handling

```rust
use fastn_automerge::db::{GetError, CreateError};

match UserProfile::load(&db, &user_id) {
    Ok(profile) => { /* use profile */ },
    Err(GetError::NotFound(_)) => { /* create default */ },
    Err(e) => return Err(e.into()),
}
```

## 🏗️ Architecture

- **Automerge**: CRDT for conflict-free collaborative editing
- **Autosurgeon**: Type-safe serialization with derive macros  
- **SQLite**: Persistent storage with efficient indexing
- **Actor ID system**: Device/entity tracking for privacy protection
- **Path validation**: Compile-time and runtime path checking

## 🔒 Privacy & Security

- **Actor ID rewriting**: Prevents account linkage across aliases
- **Device management**: Automatic device number assignment
- **History preservation**: Full audit trail of all changes
- **Path validation**: Prevents injection and malformed paths

## 📄 License

MIT

---

**Built for the FASTN P2P system** but designed as a standalone library for any application needing collaborative document storage.