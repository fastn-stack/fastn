# fastn-entity

Entity management for the fastn P2P network.

## Overview

This crate provides entity management for fastn's peer-to-peer network. Each fastn instance is called an "entity" and is uniquely identified by an ID52 - a 52-character encoded Ed25519 public key.

## Features

- **Entity Creation**: Generate new entities with cryptographic identities
- **Entity Loading**: Load existing entities from disk
- **Entity Manager**: Manage multiple entities with persistent configuration
- **Secure Key Storage**: Keys stored in system keyring or files (with explicit user consent)
- **SQLite Database**: Each entity has its own database for local storage
- **Online/Offline Tracking**: Track which entities are currently online

## Entity Structure

Each entity consists of:
- **ID52 Identity**: A unique 52-character public key identifier
- **Secret Key**: Ed25519 private key for signing operations
- **SQLite Database**: Local storage at `{entity_folder}/db.sqlite`
- **Entity Folder**: Named by the entity's ID52

### Entity Folder Layout

```
{entities_dir}/
├── config.json                    # EntityManager configuration
├── {id52}/                       # Entity folder (named by ID52)
│   ├── entity.id52               # Public key file
│   ├── entity.private-key        # Private key (if not in keyring)
│   └── db.sqlite                 # Entity's database
└── {another_id52}/              # Another entity
    └── ...
```

## EntityManager

The `EntityManager` handles multiple entities with persistent configuration:

```rust
use fastn_entity::EntityManager;

// Initialize manager (creates default entity if needed)
let mut manager = EntityManager::new(None).await?;

// Get the last used entity
let last_id52 = manager.last();

// Load an entity
let entity = manager.load_entity(last_id52).await?;

// Create a new entity
let new_entity = manager.create_entity().await?;

// Manage online status
manager.set_online(&new_entity.id52(), true)?;
let is_online = manager.is_online(&new_entity.id52());

// Update last used entity
manager.set_last(&new_entity.id52())?;
```

### Configuration (config.json)

The EntityManager persists its state to `config.json`:

```json
{
  "path": "/path/to/entities",
  "last": "i66fo538lfl5ombdf6tcdbrabp4hmp9asv7nrffuc2im13ct4q60",
  "online_status": {
    "i66fo538lfl5ombdf6tcdbrabp4hmp9asv7nrffuc2im13ct4q60": true,
    "j77gp649mgm6pnce7udecsbqc5inp0btw8osgguv3jn24dt5r70": false
  }
}
```

## Entity Operations

### Creating an Entity

```rust
use fastn_entity::Entity;

// Create new entity (folder named by generated ID52)
let entity = Entity::create(&parent_dir).await?;
println!("Created entity: {}", entity.id52());
```

### Loading an Entity

```rust
use fastn_entity::Entity;

// Load existing entity from its folder
let entity = Entity::load(&entity_path).await?;

// Access entity components
let public_key = entity.public_key();
let conn = entity.conn().await;  // Database connection
```

### Signing and Verification

```rust
// Sign a message
let message = b"Hello, fastn!";
let signature = entity.sign(message);

// Verify with public key
let public_key = entity.public_key();
assert!(public_key.verify(message, &signature).is_ok());
```

## Key Storage

### Default: System Keyring (Secure)

By default, private keys are stored in the system keyring:
- **macOS**: Keychain
- **Linux**: Secret Service API
- **Windows**: Windows Credential Store

### File Storage (Explicit Opt-in)

Set `SKIP_KEYRING=true` to store keys in files:

```bash
SKIP_KEYRING=true cargo run
```

This creates `entity.private-key` files instead of using the keyring.

## Environment Variables

- `SKIP_KEYRING=true`: Store keys in files instead of keyring
- `FASTN_SECRET_KEYS`: Provide keys directly (format: `prefix: hexkey`)
- `FASTN_SECRET_KEYS_FILE`: Path to file containing keys

## Strict Mode

The crate enforces strict mode for safety:
- Cannot have both `.id52` and `.private-key` files (must choose one)
- Non-empty directories must have valid `config.json`
- Cannot set both `FASTN_SECRET_KEYS` and `FASTN_SECRET_KEYS_FILE`

## Platform Directories

When no path is specified, EntityManager uses platform-specific data directories:
- **Linux**: `~/.local/share/fastn/`
- **macOS**: `~/Library/Application Support/com.fastn.fastn/`
- **Windows**: `%APPDATA%\fastn\fastn\data\`