# fastn-rig

Central coordination layer for the FASTN P2P network.

## Overview

The Rig is the fundamental node in the FASTN network that coordinates all entities (accounts, devices) and manages network endpoints. Each Rig has its own ID52 identity and maintains persistent state about which endpoints are online and which entity is currently active.

## Key Responsibilities

- **Network Identity**: Each Rig has its own ID52 for P2P communication
- **Entity Management**: Coordinates accounts and devices (future)
- **Endpoint Lifecycle**: Controls which endpoints are online/offline
- **Current Entity Tracking**: Maintains which entity is currently active
- **Database Persistence**: Stores state in `rig.sqlite`

## Database Schema

The Rig maintains a single SQLite database (`rig.sqlite`) with:

```sql
CREATE TABLE fastn_endpoints (
    id52       TEXT PRIMARY KEY,
    is_online  INTEGER NOT NULL DEFAULT 0,
    is_current INTEGER NOT NULL DEFAULT 0
);
```

- Only one endpoint can have `is_current = 1` at a time (enforced by unique index)
- The `is_online` flag tracks which endpoints are currently active

## Architecture

```
fastn_home/
  rig/
    rig.id52           # Public key (if not using keyring)
    rig.private-key    # Secret key (if SKIP_KEYRING=true)
    owner              # Optional owner public key
  rig.sqlite           # Rig database
```

## Usage

```rust
// First time initialization
let rig = fastn_rig::Rig::create(fastn_home, None)?;

// Loading existing Rig
let rig = fastn_rig::Rig::load(fastn_home)?;

// Manage endpoint status
rig.set_endpoint_online(&id52, true).await;
rig.set_current(&id52).await?;
```

## Endpoint Management

The `EndpointManager` handles the actual network connections for all endpoints. It:
- Creates Iroh P2P endpoints for each ID52
- Routes incoming messages through a single channel
- Manages endpoint lifecycle (bring online/take offline)

## Integration

The Rig integrates with:
- `fastn-account`: Manages user accounts with multiple aliases
- `fastn-device`: (Future) Will manage device entities
- `fastn-id52`: Provides cryptographic identity
- `iroh`: P2P networking protocol