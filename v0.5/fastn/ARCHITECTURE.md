+  # FASTN Architecture & Core Concepts

## Overview

FASTN is a peer-to-peer (P2P) network where every node is a **Rig** that can
host multiple **Accounts** and **Devices**. Each entity has its own
cryptographic + identity (ID52) and can communicate with other entities over the
Iroh network protocol.

## Core Entity Types

### 1. Rig

- **Definition**: The fundamental node in the FASTN network
- **Identity**: Has its own ID52 (52-character public key)
- **Role**: Hosts and manages Accounts and Devices
- **Cardinality**: One Rig per `fastn_home` directory
- **Storage**: `{fastn_home}/rig/` directory containing:
    - `rig.id52` - Public key
    - `rig.private-key` - Private key (or keyring reference)
    - `rig.db` - SQLite database
    - `rig.json` - Configuration
    - `public/` - Public web content (folder-based routing)

### 2. Account

- **Definition**: A user or group identity that can have multiple aliases
- **Types**:
    - **Personal Account**: Owned by an individual
    - **Group Account**: Owned by another account (organization, team, etc.)
- **Identity**: Collection of aliases, each alias is a separate ID52 with its
  own keypair
- **No Primary**: All aliases are equal, no special "primary" alias
- **Folder Name**: Uses first created alias ID52 as folder name (implementation
  detail only)
- **Storage**: `{fastn_home}/accounts/{first_alias_id52}/` containing:
    - `db.sqlite` - Account database (includes Automerge documents)
    - `profile.json` - Account metadata
    - `public/` - Public web content (folder-based routing)
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
- **Relationships**:
    - Can own multiple Devices
    - Can own other Accounts (group accounts)
    - Can have peer relationships with other Accounts
    - Each peer relationship uses a specific alias

### 3. Device

- **Definition**: A client entity owned by exactly one Account
- **Identity**: Has its own ID52 (kept private from non-owner accounts)
- **Owner**: Must have exactly one Account owner
- **Storage**: `{fastn_home}/devices/{device_id52}/` containing:
    - `device.id52` - Public key
    - `device.private-key` - Private key
    - `db.sqlite` - Device database (includes synced Automerge documents)
    - `device_info.json` - Device metadata
    - `public/` - Public web content (folder-based routing)
- **Relationships**:
    - Can only connect directly to its owner Account using device ID52
    - Never connects directly to other Devices
    - Can browse non-owner Accounts using temporary browsing identities

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
    └── .mail-index.db              # SQLite index for fast queries
```

#### Key Design Decisions
- **Username-based folders**: All emails for `alice@` go in `mails/alice/` regardless of which alias
- **Timestamp prefixes**: Files named as `{unix_timestamp}-{uuid}.eml` for chronological ordering
- **Metadata sidecar**: JSON file alongside each .eml with FASTN-specific metadata
- **SQLite index**: For fast searching without scanning all files

### Database Schema

```sql
-- Email user accounts (local usernames)
CREATE TABLE email_users (
    username     TEXT PRIMARY KEY,
    display_name TEXT,
    signature    TEXT,
    created_at   INTEGER NOT NULL,
    is_active    BOOLEAN DEFAULT TRUE
);

-- Email index for fast queries
CREATE TABLE emails (
    email_id          TEXT PRIMARY KEY,     -- UUID
    username          TEXT NOT NULL,        -- Local username
    folder            TEXT NOT NULL,        -- 'inbox', 'sent', 'drafts', 'trash'
    
    -- Addressing
    from_address      TEXT NOT NULL,        -- Full: username@id52
    to_addresses      TEXT NOT NULL,        -- JSON array of addresses
    cc_addresses      TEXT,                 -- JSON array of addresses
    bcc_addresses     TEXT,                 -- JSON array of addresses
    
    -- Alias tracking
    received_at_alias TEXT,                 -- Which of our aliases received this
    sent_from_alias   TEXT,                 -- Which of our aliases sent this
    
    -- Content
    subject           TEXT,
    body_preview      TEXT,                 -- First 200 chars
    has_attachments   BOOLEAN DEFAULT FALSE,
    
    -- Metadata
    file_path         TEXT NOT NULL UNIQUE, -- Path to .eml file
    size_bytes        INTEGER NOT NULL,
    message_id        TEXT,                 -- RFC 2822 Message-ID
    in_reply_to       TEXT,                 -- Threading
    references        TEXT,                 -- Threading (JSON array)
    
    -- Timestamps
    date_sent         INTEGER,              -- From email header
    date_received     INTEGER,              -- When we received it
    
    -- Status
    is_read           BOOLEAN DEFAULT FALSE,
    is_starred        BOOLEAN DEFAULT FALSE,
    flags             TEXT,                 -- JSON array: answered, forwarded, etc.
    
    -- Indexes
    FOREIGN KEY (username) REFERENCES email_users(username),
    INDEX idx_username_folder (username, folder),
    INDEX idx_date (date_received DESC),
    INDEX idx_from (from_address),
    INDEX idx_subject (subject)
);

-- Email attachments
CREATE TABLE email_attachments (
    attachment_id TEXT PRIMARY KEY,
    email_id      TEXT NOT NULL,
    filename      TEXT NOT NULL,
    content_type  TEXT,
    size_bytes    INTEGER,
    file_path     TEXT,                    -- If saved separately
    FOREIGN KEY (email_id) REFERENCES emails(email_id)
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

## Identity & Aliases

### Account Aliases

An Account is not just a single identity but a collection of aliases. This
provides:

1. **Privacy**: Different aliases for different social contexts
2. **Security**: Compartmentalization of relationships
3. **Flexibility**: Can retire aliases without losing the account
4. **Equality**: No "primary" alias - all aliases are equal peers

 ```
 Account (folder named after first alias for technical reasons)
 ├── Alias 1: abc123... (work contacts)
 ├── Alias 2: def456... (family) 
 ├── Alias 3: ghi789... (gaming friends)
 └── All equal - no primary
 ```

### Why No Primary Alias?

Having a "primary" alias creates privacy risks:

- Developers might default to primary when they should choose carefully
- Users might accidentally expose their "main" identity
- It creates a false hierarchy where all aliases should be equal
- The folder name using first alias is purely an implementation detail

### Alias Usage in Connections

When Account A connects to Account B:

- A MUST choose which alias to use (no default)
- B only knows about that specific alias of A
- A can have different aliases for different peers

Example:

 ```
 Alice's Account:
 - Alias 1: alice_work (used with work colleagues)
 - Alias 2: alice_family (used with family)
 - Alias 3: alice_gaming (used with gaming friends)
 
 When connecting to Bob (work colleague):
 - Alice explicitly chooses alice_work alias
 - Bob only knows alice_work, not her other aliases
 - No risk of accidentally using alice_family
 ```

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

### Document Storage

- Documents stored in SQLite as binary blobs
- Each document has a unique ID and owner
- Document changes tracked as Automerge operations
- Sync state tracked per peer/device

### Document Ownership & Sharing

 ```sql
 CREATE TABLE automerge_documents
 (
     document_id    TEXT PRIMARY KEY,
     owner_id52     TEXT    NOT NULL, -- Account that owns this document
     document_type  TEXT    NOT NULL, -- 'note', 'spreadsheet', 'canvas', etc.
     title          TEXT,
     automerge_data BLOB    NOT NULL, -- Automerge binary format
     created_at     INTEGER NOT NULL,
     updated_at     INTEGER NOT NULL,
     is_public      BOOLEAN DEFAULT FALSE
 );

CREATE TABLE document_shares
(
    document_id      TEXT    NOT NULL,
    shared_with_id52 TEXT    NOT NULL, -- ID52 of account/device with access
    permission       TEXT    NOT NULL, -- 'read', 'write', 'admin'
    shared_at        INTEGER NOT NULL,
    shared_by_id52   TEXT    NOT NULL, -- Who shared it
    PRIMARY KEY (document_id, shared_with_id52),
    FOREIGN KEY (document_id) REFERENCES automerge_documents (document_id)
);

CREATE TABLE document_sync_state
(
    document_id TEXT    NOT NULL,
    peer_id52   TEXT    NOT NULL, -- Peer we're syncing with
    last_sync   INTEGER NOT NULL, -- Timestamp of last sync
    sync_vector BLOB,             -- Automerge sync state vector
    PRIMARY KEY (document_id, peer_id52)
);
 ```

### Sync Rules

1. **Owner → Devices**: Documents automatically sync to all devices owned by the
   account
2. **Peer Sharing**: Documents sync with peers based on sharing permissions
3. **Conflict Resolution**: Automerge handles conflicts automatically
4. **Offline Support**: Changes accumulate locally and sync when connected

## Ownership Hierarchy

### Account Ownership Types

1. **Personal Account**: Not owned by any other account (root ownership)
2. **Group Account**: Owned by another account

 ```
 amitu (Personal Account)
 ├── owns → fifthtry (Group Account)
 │          ├── owns → fifthtry_marketing (Sub-group Account)
 │          └── owns → fifthtry_engineering (Sub-group Account)
 └── owns → amitu_phone (Device)
 ```

### Ownership Rules

- An Account can own multiple other Accounts (groups/organizations)
- An Account can be owned by at most one other Account
- Ownership creates a tree hierarchy (no cycles allowed)
- Owner has full control over owned accounts

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

Devices NEVER expose their real ID52 to non-owner accounts. Three browsing modes:

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

#### 2. Proxied Anonymous Browsing (Maximum Privacy)
 ```
 Device D1 → Owner Account A → [A's connection] → Foreign Account B
    ↑
    └─ All traffic proxied through owner account
    └─ Foreign account only sees owner account's IP
    └─ Device IP completely hidden
    └─ Higher latency due to proxy hop
    └─ Still anonymous to foreign account
 ```

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

**PROHIBITED**: Devices can NEVER communicate directly with each other, even if owned by the same account. All device-to-device data flow must go through the owner account.

## Database Schema

### Core Tables

 ```sql
 -- Peer relationships
CREATE TABLE fastn_account
(
    peer_id52            TEXT PRIMARY KEY,
    peer_name            TEXT,
    relationship_type    TEXT    NOT NULL, -- 'peer', 'owner', 'owned_by'
    our_alias_id52       TEXT    NOT NULL, -- Which of our aliases they know
    first_seen           INTEGER NOT NULL,
    last_seen            INTEGER,
    connection_direction TEXT,
    notes                TEXT,
    trust_level          INTEGER DEFAULT 0,
    FOREIGN KEY (our_alias_id52) REFERENCES account_aliases (id52)
);

-- Account aliases (ALL aliases including first)
CREATE TABLE account_aliases
(
    id52                 TEXT PRIMARY KEY,
    alias_name           TEXT,
    created_at           INTEGER NOT NULL,
    private_key_location TEXT -- 'keyring' or 'file'
);

-- No is_primary field! All aliases are equal
 ```

## Directory Structure Example

 ```
 {fastn_home}/
 ├── rig/
 │   ├── rig.id52
 │   ├── rig.private-key
 │   ├── rig.db
 │   ├── rig.json
 │   └── public/                    # Folder-based routing
 │       ├── index.html
 │       └── api/
 │           └── status.wasm
 ├── accounts/
 │   └── {first_alias_id52}/        # Folder named after first alias (implementation detail)
 │       ├── db.sqlite
 │       ├── profile.json
 │       ├── aliases/               # ALL aliases stored here
 │       │   ├── abc123.id52
 │       │   ├── abc123.private-key
 │       │   ├── def456.id52
 │       │   └── def456.private-key
 │       ├── public/
 │       │   ├── index.html
 │       │   └── blog/
 │       │       └── post1.html
 │       └── mails/                # Organized by username
 │           ├── alice/            # All alice@ emails
 │           │   ├── inbox/
 │           │   └── sent/
 │           └── bob/              # All bob@ emails
 │               ├── inbox/
 │               └── sent/
 └── devices/
     └── {device_id52}/
         ├── device.id52
         ├── device.private-key
         ├── db.sqlite
         └── public/
             └── index.html
 ```

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

- **Device ID52 Protection**: Real device ID52 is NEVER exposed to non-owner accounts
- **Browsing ID52 Isolation**: Temporary browsing ID52s prevent correlation
- **Delegation Security**: Signed tokens prove authorization without exposing device identity
- **Connection Reuse**: Browsing ID52 can be reused to reduce latency while maintaining privacy

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
