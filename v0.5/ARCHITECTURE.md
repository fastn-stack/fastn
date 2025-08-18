# FASTN Architecture

## Table of Contents

1. [Overview](#overview)
2. [Core Entity Types](#core-entity-types)
3. [Automerge Documents](#automerge-documents)
4. [Connection Model](#connection-model)
5. [Email System](#email-system)
6. [File Serving & Web Capabilities](#file-serving--web-capabilities)
7. [Network Protocol](#network-protocol)
8. [Security Model](#security-model)
9. [Future Considerations](#future-considerations)

## Overview

FASTN is a decentralized peer-to-peer network built on Iroh. Every node runs a *
*Rig** that can
host multiple **Accounts** and **Devices**. Each entity has its own
cryptographic identity (ID52) and communicates over the Iroh protocol.

Key principles:

- **Automerge First**: Configuration and metadata stored as Automerge documents
- **No Central Servers**: Direct P2P communication between entities
- **Privacy by Design**: Multiple aliases, device ID protection
- **Offline First**: Full functionality offline with sync when reconnected

## Core Entity Types

### 1. Rig

- **Definition**: The fundamental node in the FASTN network
- **Identity**: Has its own ID52 (52-character public key)
- **Role**: Hosts and manages Accounts and Devices
- **Ownership**: The first Account created owns the Rig (stored in `/-/rig/{rig_id52}/config` Automerge document)
- **Cardinality**: One Rig per `fastn_home` directory
- **Storage**: `{fastn_home}/rig/` directory containing:
    - `rig.id52` - Public key
    - `rig.private-key` - Private key (or keyring reference)
    - `automerge.sqlite` - Automerge documents and configuration
    - `public/` - Public web content (folder-based routing)

### 2. Account

- **Definition**: A user or organization identity with multiple aliases
- **Types**:
    - **Personal Account**: Root account, not owned by any other account
    - **Group Account**: Owned by another account (organization, team, etc.)
- **Identity**:
    - Collection of aliases (each alias is a separate ID52 with own keypair)
    - All aliases are equal - no "primary" alias concept
    - Each alias can have different public profiles
    - Folder uses first alias ID52 (implementation detail only)
- **Storage**: `{fastn_home}/accounts/{first_alias_id52}/` containing:
    - `automerge.actor-id` - Account actor ID: `{first-alias}-1` (device 1 is always the account itself)
    - `mail.sqlite` - Email index and metadata
    - `automerge.sqlite` - Automerge documents and derived/cache tables
    - `db.sqlite` - User-defined tables (future use)
    - `aliases/` - All alias keys (including first one)
        - `{alias1_id52}.id52` - Public key
        - `{alias1_id52}.private-key` - Private key
        - `{alias2_id52}.id52` - Public key
        - `{alias2_id52}.private-key` - Private key
    - `mails/` - Email storage directory (organized by username)
        - `amitu/` - All emails to/from amitu@ any alias
            - `inbox/` - Received emails
            - `sent/` - Sent emails
            - `drafts/` - Draft emails
        - `bob/` - All emails to/from bob@ any alias
            - `inbox/` - Received emails
            - `sent/` - Sent emails
    - `public/` - Public web content (folder-based routing)
- **Relationships**:
    - Can own multiple Devices
    - Can own other Accounts (group accounts)
    - Can own the Rig (first Account created becomes owner)
    - Can have peer relationships with other Accounts
    - Each peer relationship uses a specific alias

### 3. Device

- **Definition**: A client entity owned by exactly one Account
- **Identity**: Has its own ID52 (kept private from non-owner accounts)
- **Owner**: Must have exactly one Account owner
- **Storage**: `{fastn_home}/devices/{device_id52}/` containing:
    - `device.id52` - Public key
    - `device.private-key` - Private key
    - `automerge.actor-id` - Device actor ID: `{owner-alias}-{device-num}` (received from account during acceptance)
    - `automerge.sqlite` - Automerge documents and derived/cache tables
    - `db.sqlite` - User-defined tables (future use)
    - `public/` - Public web content (folder-based routing)
- **Relationships**:
    - Can only connect directly to its owner Account using device ID52
    - Never connects directly to other Devices
    - Can browse non-owner Accounts using temporary browsing identities
- **Actor ID Assignment**:
    - When device is accepted by account, account assigns the actor ID
    - Format: `{account-alias}-{device-num}` where device-num is next available number
    - Device stores this in `automerge.actor-id` file for all future operations

## Email System

### Overview

FASTN implements a fully decentralized email system where:

- Accounts send emails directly to each other via P2P (no central servers)
- Each alias acts as an independent email domain
- Emails are organized by username across all aliases
- Standard email clients work via IMAP/SMTP bridges
- Devices do NOT store or handle emails (only accounts do)

### Email Addressing

#### Address Format

- **Pattern**: `username@alias_id52`
- **Username Rules**:
    - Alphanumeric, dots, dashes, underscores
    - Case-insensitive (alice@... same as Alice@...)
    - Max 64 characters
- **Alias**: The full 52-character ID52 of the account alias
- **Examples**:
    - `alice@abc123...def456` (alice using alias abc123...def456)
    - `admin.backup@ghi789...xyz123` (admin.backup using alias ghi789...xyz123)

#### Address Resolution

1. Extract username and alias from email address
2. Look up alias ID52 in peer database or via discovery
3. Connect to peer using Iroh with that ID52
4. Send email message via P2P protocol

### Storage Organization

#### Filesystem Layout

```
accounts/{account_id52}/
└── mails/
    ├── {username}/                # One folder per username
    │   ├── inbox/
    │   │   ├── {timestamp}-{id}.eml   # RFC 2822 format
    │   │   └── {timestamp}-{id}.json  # Metadata
    │   ├── sent/
    │   │   └── {timestamp}-{id}.eml
    │   ├── drafts/
    │   │   └── {timestamp}-{id}.eml
    │   └── trash/
    │       └── {timestamp}-{id}.eml
└── mail.db                   # SQLite index for fast queries
```

#### Key Design Decisions

- **Username-based folders**: All emails for `alice@` go in `mails/alice/`
  regardless of which alias
- **Timestamp prefixes**: Files named as `{unix_timestamp}-{id}.eml` for
  chronological ordering
- **Metadata sidecar**: JSON file alongside each .eml with FASTN-specific
  metadata
- **SQLite index**: For fast searching without scanning all files

### Database Schema

```sql
-- Email user accounts (local usernames)
CREATE TABLE email_users
(
    username     TEXT PRIMARY KEY,
    display_name TEXT,
    signature    TEXT,
    created_at   INTEGER NOT NULL,
    is_active    BOOLEAN DEFAULT TRUE
);

-- Email index for fast queries
CREATE TABLE emails
(
    email_id          TEXT PRIMARY KEY,        -- Generated ID
    username          TEXT    NOT NULL,        -- Local username
    folder            TEXT    NOT NULL,        -- 'inbox', 'sent', 'drafts', 'trash'

    -- Addressing
    from_address      TEXT    NOT NULL,        -- Full: username@id52
    to_addresses      TEXT    NOT NULL,        -- JSON array of addresses
    cc_addresses      TEXT,                    -- JSON array of addresses
    bcc_addresses     TEXT,                    -- JSON array of addresses

    -- Alias tracking
    received_at_alias TEXT,                    -- Which of our aliases received this
    sent_from_alias   TEXT,                    -- Which of our aliases sent this

    -- Content
    subject           TEXT,
    body_preview      TEXT,                    -- First 200 chars
    has_attachments   BOOLEAN DEFAULT FALSE,

    -- Metadata
    file_path         TEXT    NOT NULL UNIQUE, -- Path to .eml file
    size_bytes        INTEGER NOT NULL,
    message_id        TEXT,                    -- RFC 2822 Message-ID
    in_reply_to       TEXT,                    -- Threading
    references        TEXT,                    -- Threading (JSON array)

    -- Timestamps
    date_sent         INTEGER,                 -- From email header
    date_received     INTEGER,                 -- When we received it

    -- Status
    is_read           BOOLEAN DEFAULT FALSE,
    is_starred        BOOLEAN DEFAULT FALSE,
    flags             TEXT,                    -- JSON array: answered, forwarded, etc.

    -- Indexes
    FOREIGN KEY (username) REFERENCES email_users (username),
    INDEX             idx_username_folder(username, folder),
    INDEX             idx_date(date_received DESC),
    INDEX             idx_from(from_address),
    INDEX             idx_subject(subject)
);

-- Email attachments
CREATE TABLE email_attachments
(
    attachment_id TEXT PRIMARY KEY,
    email_id      TEXT NOT NULL,
    filename      TEXT NOT NULL,
    content_type  TEXT,
    size_bytes    INTEGER,
    file_path     TEXT, -- If saved separately
    FOREIGN KEY (email_id) REFERENCES emails (email_id)
);
```

### P2P Email Protocol

#### Sending Email Flow

1. **Compose**: User creates email via client (SMTP or API)
2. **Resolve**: Look up recipient's alias ID52
3. **Connect**: Establish Iroh connection to recipient
4. **Deliver**: Send EmailDelivery message
5. **Store**: Save copy in sender's 'sent' folder
6. **Confirm**: Wait for delivery acknowledgment

#### Receiving Email Flow

1. **Accept**: Receive EmailDelivery message via Iroh
2. **Validate**: Check recipient alias belongs to us
3. **Parse**: Extract email content and metadata
4. **Store**: Save to appropriate username folder
5. **Index**: Update SQLite database
6. **Acknowledge**: Send delivery confirmation

#### Message Format

```rust
pub struct EmailDelivery {
    // Envelope
    from: String,           // username@sender_alias_id52
    to: Vec<String>,        // username@recipient_alias_id52

    // Content (RFC 2822 format)
    raw_email: Vec<u8>,     // Complete email with headers

    // Metadata
    timestamp: u64,
    message_id: String,
}
```

### IMAP/SMTP Bridge

#### Server Configuration

```yaml
IMAP Server:
  Host: localhost
  Port: 143 (plain), 993 (TLS)
  Auth: Username + Password

SMTP Server:
  Host: localhost
  Port: 587 (submission), 465 (TLS)
  Auth: Username + Password
```

#### Authentication

- **Username format**: `username@alias_id52`
- **Password**: Account-specific or per-username
- Server extracts alias from username to determine which identity to use

#### IMAP Features

- **Folders**: INBOX, Sent, Drafts, Trash (mapped to filesystem)
- **Flags**: \Seen, \Answered, \Flagged, \Deleted, \Draft
- **Search**: SEARCH command uses SQLite index
- **Threading**: THREAD command using References headers

#### SMTP Features

- **Submission**: Accept emails from authenticated users
- **Relay**: Only for P2P delivery (no external SMTP)
- **Queue**: Retry failed P2P deliveries
- **DSN**: Delivery status notifications

### Email Security

#### Transport Security

- All P2P connections encrypted via Iroh
- No email content on devices (account-only)
- Each alias has independent email identity

#### Anti-Spam Considerations

- No open relay (only authenticated sending)
- Rate limiting per sender
- Allowlist/blocklist by sender ID52
- Future: Reputation system per alias

### Future Email Features

1. **Email Encryption**: End-to-end encryption using alias keys
2. **Mailing Lists**: Group email via special accounts
3. **Email Filters**: Server-side filtering rules
4. **Full-Text Search**: Advanced search capabilities
5. **Email Backup**: Automated backup to owned devices
6. **External Gateway**: Bridge to regular email (optional)

## File Serving & Web Capabilities

### Folder-Based Routing

Every entity uses a single `public/` directory with folder-based routing:

 ```
 public/
 ├── index.html              → /
 ├── about.html              → /about
 ├── about/                  
 │   └── team.html          → /about/team
 ├── blog/
 │   ├── index.html         → /blog/
 │   └── post1.html         → /blog/post1
 ├── app.wasm               → /app.wasm
 ├── styles.css             → /styles.css
 ├── dashboard.fhtml        → /dashboard (rendered)
 └── api/
     └── users.wasm         → /api/users (executed)
 ```

### File Types and Handling

1. **Static Files** (`.html`, `.css`, `.js`, images, etc.):
    - Served as-is with appropriate MIME types
    - Direct mapping from URL path to file path

2. **FHTML Templates** (`.fhtml`):
    - Server-side rendered with entity context
    - Access to entity data, aliases, relationships
    - Output as HTML

3. **WebAssembly Modules** (`.wasm`):
    - Can be served as static files for client-side execution
    - Can be executed server-side when accessed as endpoint
    - Execution context determines behavior

### URL Structure

 ```
 {id52}.localhost:8080/path
    ↓
 1. Check if ID52 is local entity
 2. Look for file in entity's public/ directory:
    - Exact match (e.g., /about → public/about.html)
    - Directory index (e.g., /blog/ → public/blog/index.html)
    - Template (e.g., /dashboard → public/dashboard.fhtml)
    - WASM handler (e.g., /api/users → public/api/users.wasm)
 3. If not found locally and ID52 is remote → proxy over Iroh
 4. Return 404 if not found
 ```

## Automerge Documents

### Overview

FASTN uses Automerge for collaborative, conflict-free document synchronization
across entities.

### Actor ID Design

FASTN uses a simplified actor ID system with creation-alias optimization:

#### Actor ID Format
- **Structure**: `{alias-id52}-{device-number}` (e.g., `1oem6e10...-1`)
- **Creation Alias**: Each document stores the alias used at creation
- **Consistency**: All edits use the creation alias as actor prefix
- **No GUID needed**: Direct use of alias IDs throughout

#### Optimization Strategy
1. **Common case (90%+)**: Same alias for creation and sharing = no history rewriting
2. **Rare case**: Different alias for sharing = history rewrite on export only
3. **Storage**: `created_alias` field in database tracks the creation alias

#### Security Properties
1. **Attribution**: Can track edits to specific devices
2. **Verification**: Can verify all edits come from claimed alias
3. **Consistency**: Document maintains same actor throughout its lifecycle
4. **Efficiency**: No rewriting overhead in common single-alias case

Example flow:
```
Alice creates with alice123...: actor = alice123...-1 (stored)
Alice edits: uses alice123...-1 (no rewrite)
Alice shares with Bob: no rewrite needed (same alias)
Bob receives: sees alice123...-1 as actor
Bob creates response: actor = bob456...-1 (stored)
Bob shares back: no rewrite needed
```

### Document Paths and Ownership

Document paths encode ownership:

**For Accounts:**
- **`mine/{doc-name}`** - Documents owned by this account (any of our aliases)
- **`{owner-alias-id52}/{doc-name}`** - Documents owned by others

**For Rigs:**
- **`/-/rig/{rig_id52}/config`** - Rig configuration (includes owner account ID52)

Examples:

- `mine/project-notes` - My project notes
- `mine/-/config` - My account configuration
- `abc123.../shared-doc` - Document owned by alias abc123...
- `abc123.../-/readme` - Public profile of alias abc123...
- `/-/rig/{rig_id52}/config` - Rig configuration (rig's automerge.sqlite)

Each entity stores their own copy in SQLite, so the same logical document may
have different paths:

- Alice sees her doc as: `mine/project`
- Bob sees Alice's doc as: `alice-id52/project`
- Carol sees Alice's doc as: `alice-id52/project`

### System Document Types

#### Rig Documents
- **`/-/rig/{rig_id52}/config`** - Rig configuration
  - `owner_id52`: ID52 of the owner account (first account created)
  - `created_at`: Timestamp when rig was created
  - `name`: Optional human-readable name for the rig

#### Account Documents  
- **`/-/mails/{username}`** - Email account configuration
  - `username`: Email username
  - `password_hash`: Argon2 hashed password
  - `smtp_enabled`: Whether SMTP is enabled
  - `imap_enabled`: Whether IMAP is enabled
  - `created_at`: Creation timestamp
  - `is_active`: Whether account is active
- **`/-/aliases/{id52}/readme`** - Public alias profile
  - `name`: Public display name
  - `display_name`: Alias display name
  - `created_at`: Creation timestamp
  - `is_primary`: Whether this is the primary alias
- **`/-/aliases/{id52}/notes`** - Private alias notes
  - `reason`: Why this alias exists (private)
  - `created_at`: Creation timestamp

### Document Storage

- Documents stored in SQLite as binary blobs with path as key
- Same document may have different paths in different accounts
- Document changes tracked as Automerge operations
- Sync state tracked per peer/device

### Document Naming Convention

**Special Documents**: Use `/-/` prefix within paths to prevent conflicts

- `mine/-/config` - My account configuration
- `mine/-/groups/{name}` - My permission groups
- `{path}/-/meta` - Metadata for any document

**User Documents**: Any path without `/-/` in the name

- Users can create documents with any name
- Each user document has an associated `/-/meta` document

### Special Documents (System-managed)

#### Alias Notes and Permissions

**`/-/{alias-id52}/notes`** (Private notes about an alias and their permissions)

```json
{
  "alias": "bob456...",
  "relationship": "coworker",
  "notes": "Met at conference 2024",
  "permissions": {
    "can_manage_groups": true,
    "can_grant_access": true,
    "is_admin": false
  },
  "trusted": true,
  "created_at": 1234567890,
  "last_interaction": 1234567890
}
```

Note: Only account owner and their devices can manage groups unless `can_manage_groups` is true.

#### Account Configuration

**`mine/-/config`** (Account-wide Settings)

```json
{
  "primary_alias": "abc123...",
  // First alias, used for folder naming
  "my_aliases": {
    "abc123...": {
      "name": "work",
      "created_at": 1234567890,
      "readme": {
        "display_name": "Alice Smith (Work)",
        "bio": "Software engineer at Example Corp"
      }
    },
    "def456...": {
      "name": "personal",
      "created_at": 1234567890,
      "readme": {
        "display_name": "Alice",
        "bio": "Indie developer"
      }
    }
  },
  "settings": {
    "email_enabled": true,
    "default_permissions": "read"
  }
}
```

- Uses special path `mine/-/config`
- Contains all my aliases and their public profiles
- Only synced with my owned devices

#### Groups (Permission Management)

**`mine/-/groups/{group-name}`** (My Permission Groups)

```json
{
  "name": "engineering-team",
  "description": "Engineering team members",
  "created_by": "abc123...",
  "created_at": 1234567890,
  "members": {
    "accounts": [
      "def456...",
      // Bob's alias
      "ghi789...",
      // Carol's alias
      "jkl012..."
      // Dave's alias
    ],
    "groups": [
      "senior-engineers",
      // Nested group
      "contractors"
      // Another nested group
    ]
  },
  "settings": {
    "allow_members_to_see_members": true,
    "allow_members_to_add_members": false
  }
}
```

- Groups simplify permission management
- Can contain account aliases and other groups (nested)
- Synced with anyone who needs to resolve group membership
- Documents can grant permissions to entire groups

#### Alias Documents (About Others)

**`{alias-id52}/-/readme`** (Their Public Profile)

```json
{
  "display_name": "Bob Johnson",
  "bio": "Designer and developer",
  "avatar_url": "...",
  "services": [
    "email",
    "chat"
  ],
  "created_at": 1234567890
}
```

- Public profile maintained by that alias owner
- Automatically synced when connected

**`{alias-id52}/-/notes`** (My Private Notes)

```json
{
  "nickname": "Bob from conference",
  "trust_level": 8,
  "tags": [
    "work",
    "design"
  ],
  "notes": "Great designer, met at P2P conf",
  "my_aliases_that_know_them": [
    "abc123..."
  ],
  "blocked": false
}
```

- My private notes about this specific alias
- Only synced between my account and my devices
- Never shared with the alias owner

#### Device Documents

**`mine/-/devices/{device-id52}/readme`** (Device Info)

```json
{
  "device_name": "Alice's Laptop",
  "device_type": "laptop",
  "os": "macOS 14.0",
  "last_seen": 1234567890,
  "capabilities": [
    "email",
    "automerge",
    "wasm"
  ],
  "browsing_id52": "xyz789..."
  // For anonymous browsing
}
```

- Device information and capabilities
- Synced between my account and all my devices

**`mine/-/devices/{device-id52}/config`** (Device Settings)

```json
{
  "sync_enabled": true,
  "sync_interval": 300,
  "storage_limit": 5368709120,
  "proxy_mode": "direct"
  // or "via-account"
}
```


### User Documents (User-created)

User documents can have any path (except containing `/-/`). Each has an
associated meta document for sharing control.

#### Document Content

**`mine/project-notes`** (My Document)

```json
{
  "title": "Project Notes",
  "content": "...",
  "created_by": "abc123...",
  "created_at": 1234567890
}
```

**`def456.../shared-doc`** (Their Document I Have Access To)

- Document owned by alias def456...
- I have access based on permissions in their meta

#### Document Metadata

**`mine/project-notes/-/meta`** (Sharing & Metadata)

```json
{
  "owner": "abc123...",
  "created_at": 1234567890,
  "updated_at": 1234567890,
  "permissions": {
    "def456...": {
      "level": "write",
      // admin, share, write, comment, read
      "granted_at": 1234567890,
      "granted_by": "abc123..."
    },
    "group:engineering-team": {
      "level": "read",
      "granted_at": 1234567890,
      "granted_by": "abc123..."
    }
  },
  "settings": {
    "public": false,
    "link_sharing": false
  }
}
```

- Permissions can be granted to aliases or groups (prefixed with "group:")
- **Meta document is shared with everyone who has SHARE permission**
- Groups are resolved recursively to find all members

### Permission Levels

1. **admin**: Full control, can delete document, change all permissions
2. **share**: Can grant/revoke read, comment, write permissions to others
3. **write**: Can edit document content
4. **comment**: Can add comments but not edit content
5. **read**: Can only view document

### Group Resolution

When checking if alias X has permission to document D:

1. Check direct permission for X in D's meta
2. For each group G in D's meta:
    - Load `mine/-/groups/G` or `{owner}/-/groups/G`
    - Check if X is in G's accounts
    - Recursively check nested groups
3. Cache resolution results for performance

### Database Architecture

FASTN uses three separate SQLite databases per account for isolation and
performance:

#### 1. automerge.sqlite - Configuration & Sync

- **Purpose**: Store all Automerge documents and sync state
- **Accessed by**: Sync logic, configuration management
- **Tables**: All prefixed with `fastn_`
    - `fastn_documents` - Automerge document blobs
    - `fastn_sync_state` - Sync state per peer
    - `fastn_relationship_cache` - Derived from relationship documents
    - `fastn_permission_cache` - Derived from meta documents
    - `fastn_group_cache` - Derived from group documents

#### 2. mail.sqlite - Email System

- **Purpose**: Email index and metadata
- **Accessed by**: Email delivery, IMAP/SMTP servers
- **Cross-DB access**: Read-only connection to automerge.sqlite for config
- **Tables**: All prefixed with `fastn_`
    - `fastn_emails` - Email index
    - `fastn_email_peers` - Known email peers
    - `fastn_auth_sessions` - IMAP/SMTP sessions

#### 3. db.sqlite - User Space

- **Purpose**: User-defined tables for applications
- **Accessed by**: User applications via WASM
- **Tables**: No `fastn_` prefix - user owns this namespace

Benefits of this separation:

- **Reduced contention**: Each subsystem uses its own database
- **Security**: User cannot accidentally corrupt system tables
- **Performance**: Parallel access to different databases
- **Backup**: Each database can be backed up independently
- **Migration**: Easier to upgrade schema per subsystem

### Database Schema (Automerge)

Since we've moved all configuration and relationship data to Automerge
documents, we only need tables for:

1. Storing Automerge document binaries
2. Tracking sync state
3. Caching for performance

```sql
-- In automerge.sqlite:

-- Core Automerge document storage
CREATE TABLE fastn_documents
(
    path             TEXT PRIMARY KEY, -- mine/doc or {alias}/doc
    automerge_binary BLOB    NOT NULL, -- Current Automerge state
    heads            TEXT    NOT NULL, -- JSON array of head hashes
    updated_at       INTEGER NOT NULL,

    INDEX            idx_updated(updated_at DESC)
);
-- Note: actor_id not stored - determined at runtime:
--   Internal: {account-guid}-{device-num}
--   External: {alias-id52}-{device-num}

-- Automerge sync state per document per peer
CREATE TABLE fastn_sync_state
(
    document_path TEXT    NOT NULL,
    peer_id52     TEXT    NOT NULL,

    -- Automerge sync protocol state
    sync_state    BLOB    NOT NULL, -- Binary sync state from Automerge
    their_heads   TEXT,             -- JSON array of their head hashes
    our_heads     TEXT,             -- JSON array of our head hashes

    -- Metadata
    last_sync_at  INTEGER NOT NULL,
    sync_errors   INTEGER DEFAULT 0,

    PRIMARY KEY (document_path, peer_id52),
    INDEX         idx_last_sync(last_sync_at)
);

-- Cache tables (derived from Automerge for performance)

-- Alias notes cache (extracted from /-/{alias-id52}/notes)
CREATE TABLE fastn_alias_cache
(
    alias_id52        TEXT PRIMARY KEY,
    relationship      TEXT,
    can_manage_groups INTEGER DEFAULT 0,  -- Boolean: can manage our groups
    can_grant_access  INTEGER DEFAULT 0,  -- Boolean: can grant access
    is_admin          INTEGER DEFAULT 0,  -- Boolean: admin privileges
    trusted           INTEGER DEFAULT 0,  -- Boolean: trusted peer
    last_interaction  INTEGER,
    extracted_at      INTEGER NOT NULL,   -- When we extracted from Automerge

    INDEX             idx_trusted(trusted)
);

-- Permission cache (extracted from */meta documents)
CREATE TABLE fastn_permission_cache
(
    document_path    TEXT    NOT NULL,
    grantee_alias    TEXT,
    grantee_group    TEXT,
    permission_level TEXT    NOT NULL, -- admin, share, write, comment, read
    extracted_at     INTEGER NOT NULL,

    INDEX            idx_path(document_path),
    INDEX            idx_grantee(grantee_alias)
);

-- Group membership cache (extracted from /-/groups/*)
CREATE TABLE fastn_group_cache
(
    group_name    TEXT    NOT NULL,
    member_alias  TEXT,              -- Direct account member (NULL if group)
    member_group  TEXT,              -- Nested group member (NULL if account)
    extracted_at  INTEGER NOT NULL,

    PRIMARY KEY (group_name, COALESCE(member_alias, member_group)),
    INDEX       idx_group(group_name),
    INDEX       idx_member(member_alias)
);
```

**Important Notes:**

- No more `fastn_account` or `account_aliases` tables - this data lives in
  Automerge documents
- Cache tables are rebuilt from Automerge documents and can be dropped/recreated
- `extracted_at` timestamps help identify stale cache entries
- All source of truth is in Automerge documents

### Automerge Sync Implementation

#### How Sync State Works

```rust
use automerge::{AutoCommit, sync::{self, SyncState, Message}};
use rusqlite::{Connection, params};

// Structure to hold sync state from database
struct StoredSyncState {
    sync_state: Vec<u8>,      // Binary blob
    their_heads: Vec<String>,  // Their document version hashes
    our_heads: Vec<String>,     // Our document version hashes
}

// Initialize sync for a new peer relationship
async fn init_sync_state(
    db: &Connection,
    document_path: &str,
    peer_id52: &str,
    doc: &AutoCommit,
) -> Result<SyncState> {
    let sync_state = SyncState::new();

    // Get current document heads (version hashes)
    let our_heads: Vec<String> = doc.get_heads()
        .iter()
        .map(|h| h.to_string())
        .collect();

    // Store initial sync state
    db.execute(
        "INSERT INTO sync_state (document_path, peer_id52, sync_state, our_heads, their_heads, last_sync_at)
         VALUES (?1, ?2, ?3, ?4, '[]', ?5)",
        params![
            document_path,
            peer_id52,
            sync_state.encode(),  // Serialize to binary
            serde_json::to_string(&our_heads)?,
            chrono::Utc::now().timestamp(),
        ],
    )?;

    Ok(sync_state)
}

// Load sync state for existing peer
async fn load_sync_state(
    db: &Connection,
    document_path: &str,
    peer_id52: &str,
) -> Result<Option<SyncState>> {
    let row = db.query_row(
        "SELECT sync_state FROM sync_state WHERE document_path = ?1 AND peer_id52 = ?2",
        params![document_path, peer_id52],
        |row| {
            let blob: Vec<u8> = row.get(0)?;
            Ok(blob)
        },
    ).optional()?;

    match row {
        Some(blob) => Ok(Some(SyncState::decode(&blob)?)),
        None => Ok(None),
    }
}

// Perform one sync round with a peer
async fn sync_document_with_peer(
    db: &Connection,
    document_path: &str,
    peer_id52: &str,
    doc: &mut AutoCommit,
    peer_connection: &mut PeerConnection,
) -> Result<()> {
    // 1. Load or create sync state
    let mut sync_state = match load_sync_state(db, document_path, peer_id52).await? {
        Some(state) => state,
        None => init_sync_state(db, document_path, peer_id52, doc).await?,
    };

    // 2. Generate sync message to send to peer
    // This contains only the changes the peer hasn't seen yet
    let message_to_send = doc.sync().generate_sync_message(&mut sync_state);

    if let Some(message) = message_to_send {
        // 3. Send our changes to peer
        peer_connection.send_sync_message(document_path, &message).await?;

        // The sync_state now tracks that we've sent these changes
    }

    // 4. Receive sync message from peer
    if let Some(peer_message) = peer_connection.receive_sync_message().await? {
        // 5. Apply peer's changes to our document
        doc.sync().receive_sync_message(&mut sync_state, peer_message)?;

        // The document now contains merged changes
        // The sync_state tracks what we've received
    }

    // 6. Update database with new sync state
    update_sync_state_in_db(db, document_path, peer_id52, &sync_state, doc).await?;

    Ok(())
}

// Update sync state after successful sync
async fn update_sync_state_in_db(
    db: &Connection,
    document_path: &str,
    peer_id52: &str,
    sync_state: &SyncState,
    doc: &AutoCommit,
) -> Result<()> {
    let our_heads: Vec<String> = doc.get_heads()
        .iter()
        .map(|h| h.to_string())
        .collect();

    // Note: Getting their_heads requires tracking from sync messages
    // In practice, you'd extract this from the peer's sync messages

    db.execute(
        "UPDATE sync_state 
         SET sync_state = ?1, 
             our_heads = ?2,
             last_sync_at = ?3,
             sync_errors = 0
         WHERE document_path = ?4 AND peer_id52 = ?5",
        params![
            sync_state.encode(),
            serde_json::to_string(&our_heads)?,
            chrono::Utc::now().timestamp(),
            document_path,
            peer_id52,
        ],
    )?;

    Ok(())
}

// Continuous sync loop for a document
async fn sync_loop(
    db: Arc<Mutex<Connection>>,
    document_path: String,
    peer_id52: String,
) {
    let mut interval = tokio::time::interval(Duration::from_secs(5));

    loop {
        interval.tick().await;

        // Load document from database
        let mut doc = load_document(&db, &document_path).await?;

        // Sync with peer
        if let Err(e) = sync_document_with_peer(
            &db,
            &document_path,
            &peer_id52,
            &mut doc,
            &peer_connection,
        ).await {
            // Increment error counter on failure
            db.execute(
                "UPDATE sync_state SET sync_errors = sync_errors + 1 
                 WHERE document_path = ?1 AND peer_id52 = ?2",
                params![document_path, peer_id52],
            )?;
        }

        // Save updated document
        save_document(&db, &document_path, &doc).await?;
    }
}
```

#### Key Concepts

1. **SyncState is Opaque**: Automerge's `SyncState` is an opaque type that
   tracks:
    - What changes we've sent to each peer
    - What changes we've received from each peer
    - Efficiently determines what needs to be sent next

2. **Incremental Sync**: The sync protocol only sends changes since last sync:
    - First sync: sends entire document history
    - Subsequent syncs: only new changes
    - Handles network failures gracefully (can resume)

3. **Convergence**: All peers converge to the same state:
    - CRDTs ensure conflict-free merging
    - Order of sync doesn't matter
    - Eventually consistent

4. **Peer-Specific State**: Each (document, peer) pair has its own sync state:
    - Can sync same document with multiple peers
    - Each peer relationship tracked independently
    - Allows different sync progress per peer

### Sync Rules

#### Document Path Translation

- When syncing `mine/project` to peer, it becomes `{my-alias}/project` in their
  system
- When receiving `{their-alias}/doc`, it stays as `{their-alias}/doc` in my
  system
- Devices see the same paths as their owner account

#### Sync Patterns

1. **My Documents** (`mine/*`):
    - Automatically sync to all my devices
    - Sync to peers based on `mine/{doc}/-/meta` permissions
    - Become `{my-alias}/*` in peer systems

2. **Others' Documents** (`{alias-id52}/*`):
    - Sync if I have permission in their meta
    - Path remains unchanged across syncs
    - My devices inherit my access

3. **Special Documents**:
    - `mine/-/config`: Only my devices
    - `mine/-/groups/*`: Shared with those needing resolution
    - `{alias}/-/readme`: Public, synced when connected
    - `{alias}/-/notes`: My private notes, only my devices

4. **Offline Support**: Changes accumulate locally and sync when connected

## Connection Model

### Device ↔ Account (Owner Relationship)

 ```
 Device D1 ←→ Account A (owner)
    ↑
    └─ D1 connects to A using device's real ID52
    └─ A can connect to D1 anytime
    └─ One-to-one ownership
    └─ Automerge documents sync automatically
    └─ Device ID52 is NOT private from owner
 ```

### Device → Foreign Account Browsing

Devices NEVER expose their real ID52 to non-owner accounts. Three browsing
modes:

#### 1. Direct Anonymous Browsing

 ```
 Device D1 → [Temporary ID52] → Foreign Account B
    ↑
    └─ Creates temporary browsing ID52 pair
    └─ Can reuse same browsing ID52 across sessions (reduces latency)
    └─ Account B never learns D1's real ID52
    └─ Device IP is visible during P2P connection setup
    └─ No authentication - appears as anonymous visitor
 ```

#### 2. Proxied Browsing (Maximum Privacy)

 ```
 Device D1 → Owner Account A → [A's connection] → Foreign Account B
    ↑
    └─ All traffic proxied through owner account
    └─ Foreign account only sees owner account's IP
    └─ Device IP completely hidden
    └─ Higher latency due to proxy hop
    └─ Still anonymous to foreign account
 ```

Anonymous or not depends on the mode.

#### 3. Delegated Browsing (Acting as Owner Account)

 ```
 Device D1 → [Browsing ID52 + Signed Token] → Foreign Account B
    ↑
    └─ Still uses browsing ID52 (not device ID52!)
    └─ Obtains signed delegation from owner account
    └─ Appears logged in as owner account to B
    └─ Can access documents shared with owner
    └─ Privacy not the goal (authenticating as owner)
 ```

**Delegation Flow:**

1. Device creates/reuses browsing ID52
2. Device requests delegation from owner account (via P2P)
3. Owner account signs: "browsing_id52 X can act as alias Y"
4. Device includes signed token in HTTP requests
5. Foreign account validates signature and treats as authenticated

**Privacy vs Performance Trade-offs:**

- **Direct Anonymous**: Fast, hides identity but not IP
- **Proxied Anonymous**: Slower, complete IP privacy
- **Delegated**: Fast, authenticated but not anonymous

### Account ↔ Account (Peer Relationship)

 ```
 Account A (using alias A2) ←→ Account B (using alias B1)
    ↑
    └─ A knows B as B1
    └─ B knows A as A2
    └─ Can share Automerge documents
    └─ Can exchange emails
 ```

### Account → Account (Ownership Relationship)

 ```
 Account A (owner) → Account B (owned group)
    ↑
    └─ A has full control over B
    └─ A can manage B's aliases
    └─ A can access B's resources
    └─ B can have its own peer relationships
 ```

### Device-to-Device Communication

**PROHIBITED**: Devices can NEVER communicate directly with each other, even if
owned by the same account. All device-to-device data flow must go through the
owner account.

## Network Protocol

### Message Types

 ```rust
 pub enum AccountMessage {
    // Email messages - must specify which alias received it
    EmailDelivery {
        from: String,        // username@sender_id52
        to: String,          // username@our_alias_id52
        email_content: Vec<u8>,
    },

    // Automerge sync
    AutomergeSync {
        document_id: String,
        sync_message: automerge::sync::Message,
    },

    // Connection must specify alias
    PeerRequest {
        from_alias: String,  // Which of their aliases
        to_alias: String,    // Which of our aliases
    },
}
 ```

## Security Model

### Alias Security

- Each alias has independent keypair
- No default alias prevents accidental exposure
- Explicit alias selection required for all operations
- Compromising one alias doesn't compromise others

### Device Privacy

- **Device ID52 Protection**: Real device ID52 is NEVER exposed to non-owner
  accounts
- **Browsing ID52 Isolation**: Temporary browsing ID52s prevent correlation
- **Delegation Security**: Signed tokens prove authorization without exposing
  device identity
- **Connection Reuse**: Browsing ID52 can be reused to reduce latency while
  maintaining privacy

### Email Security

- Emails stored by username across all aliases
- Each email tracks which alias sent/received it
- P2P delivery without intermediaries
- No email content on devices (only on accounts)

## Future Considerations

1. **Alias Rotation**: Periodic alias renewal for security
2. **Alias Reputation**: Building trust per alias
3. **Alias Migration**: Moving aliases between accounts
4. **Username Reservation**: Preventing username conflicts
5. **Email Filtering**: Per-alias or per-username filtering rules
6. **Alias Unlinking**: Breaking connection between aliases
