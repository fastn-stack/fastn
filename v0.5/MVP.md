# FASTN MVP - P2P Email System

## Overview

This MVP implements a minimal P2P email system using FASTN with Rig and Account entities. It focuses on email functionality over Iroh P2P networking, with IMAP/SMTP bridges for standard email clients.

## Current Implementation Status

### Completed
- ✅ Rig entity with SQLite database and endpoint management
- ✅ Account entity with multi-alias support and THREE databases (automerge.sqlite, mail.sqlite, db.sqlite)
- ✅ P2P endpoint management with fastn-net utilities
- ✅ Protocol types for entity communication (DeviceToAccount, AccountToAccount, etc.)
- ✅ Graceful shutdown pattern
- ✅ Connection pooling infrastructure
- ✅ Database migrations for all three account databases

### In Progress
- 🔄 Email message handlers for Account endpoints
- 🔄 Email storage system (mails/{username} folders structure created)

### Not Started
- ⏳ Email delivery over P2P
- ⏳ Authentication system (no auth tables exist yet)
- ⏳ IMAP/SMTP bridges
- ⏳ Offline queuing and retry

## Scope

### What's Included
- Rig entity (single instance per fastn_home)
- Account entities with multiple aliases
- Three separate databases per account (automerge, mail, user)
- P2P email delivery over Iroh
- IMAP/SMTP bridges for standard email clients
- Basic peer discovery and connection

### What's Excluded (for MVP)
- Device entities (postponed)
- HTTP server/web interface
- Automerge document sync (tables exist but no sync logic)
- File serving
- Groups (cache tables exist but not implemented)
- External email gateway

## Architecture

### Entity Hierarchy

```
Rig (fastn_home)
└── Accounts (multiple)
    └── Aliases (multiple per account)
```

### Storage Structure

```
{fastn_home}/
├── rig/
│   ├── rig.db                  # Rig configuration and endpoint state
│   ├── rig.id52                 # Rig public key
│   └── rig.private-key          # Rig private key
└── accounts/
    └── {primary_alias_id52}/   # Account folder named by first alias
        ├── automerge.sqlite     # Automerge documents & configuration
        ├── mail.sqlite          # Email index and metadata
        ├── db.sqlite            # User-space database (empty)
        ├── aliases/             # Keypairs for all aliases
        │   ├── {alias1}.id52
        │   ├── {alias1}.private-key
        │   └── {alias2}.id52
        └── mails/               # Email files
            └── default/         # Default username (MVP only)
                ├── inbox/
                ├── sent/
                ├── drafts/
                └── trash/
```

### Email Addressing

- **Format**: `username@alias_id52`
- **Example**: `default@1oem6e10tckm3edrf8mdcnutle8ie7tnf40h7oukvbeatpk0k6d0`
- Each alias acts as an independent email domain
- **MVP Simplification**: Only "default" username, all emails go to default folder

## Database Schema (Actual Implementation)

### 1. Rig Database (rig.db)

```sql
-- From fastn-rig/src/migration.rs
CREATE TABLE fastn_endpoints (
    id52              TEXT PRIMARY KEY,
    is_online         INTEGER NOT NULL DEFAULT 0,
    is_current        INTEGER NOT NULL DEFAULT 0
);

-- Indexes
CREATE INDEX idx_endpoints_online ON fastn_endpoints(is_online);
CREATE UNIQUE INDEX idx_endpoints_current_unique 
    ON fastn_endpoints(is_current) 
    WHERE is_current = 1;
```

### 2. Account Databases (Three Separate SQLite Files)

#### 2.1 automerge.sqlite - Configuration & Documents

```sql
-- From fastn-automerge/src/migration.rs
-- Core document storage
CREATE TABLE fastn_documents (
    path              TEXT PRIMARY KEY,
    automerge_binary  BLOB NOT NULL,
    heads             TEXT NOT NULL,
    actor_id          TEXT NOT NULL,
    updated_at        INTEGER NOT NULL
);

-- Sync state (for future use)
CREATE TABLE fastn_sync_state (
    document_path     TEXT NOT NULL,
    peer_id52         TEXT NOT NULL,
    sync_state        BLOB NOT NULL,
    their_heads       TEXT,
    our_heads         TEXT,
    last_sync_at      INTEGER NOT NULL,
    sync_errors       INTEGER DEFAULT 0,
    PRIMARY KEY (document_path, peer_id52)
);

-- Cache tables
CREATE TABLE fastn_relationship_cache (
    their_alias       TEXT PRIMARY KEY,
    my_alias_used     TEXT NOT NULL,
    relationship_type TEXT,
    last_seen         INTEGER,
    extracted_at      INTEGER NOT NULL
);

CREATE TABLE fastn_permission_cache (
    document_path     TEXT NOT NULL,
    grantee_alias     TEXT,
    grantee_group     TEXT,
    permission_level  TEXT NOT NULL,
    extracted_at      INTEGER NOT NULL
);

CREATE TABLE fastn_group_cache (
    group_name        TEXT NOT NULL,
    member_alias      TEXT,
    member_group      TEXT,
    extracted_at      INTEGER NOT NULL
);
```

#### 2.2 mail.sqlite - Email System

```sql
-- From fastn-account/src/account/create.rs
CREATE TABLE fastn_emails (
    email_id          TEXT PRIMARY KEY,
    folder            TEXT NOT NULL,
    original_to       TEXT NOT NULL,
    from_address      TEXT NOT NULL,
    to_addresses      TEXT NOT NULL,
    cc_addresses      TEXT,
    bcc_addresses     TEXT,
    received_at_alias TEXT,
    sent_from_alias   TEXT,
    subject           TEXT,
    body_preview      TEXT,
    has_attachments   INTEGER DEFAULT 0,
    file_path         TEXT NOT NULL UNIQUE,
    size_bytes        INTEGER NOT NULL,
    message_id        TEXT,
    in_reply_to       TEXT,
    references        TEXT,
    date_sent         INTEGER,
    date_received     INTEGER,
    is_read           INTEGER DEFAULT 0,
    is_starred        INTEGER DEFAULT 0,
    flags             TEXT
);

CREATE TABLE fastn_email_peers (
    peer_alias        TEXT PRIMARY KEY,
    last_seen         INTEGER,
    endpoint          BLOB,
    our_alias_used    TEXT NOT NULL
);
```

#### 2.3 db.sqlite - User Space

```sql
-- Empty database for user-defined tables
-- User can create any tables without fastn_ prefix
PRAGMA journal_mode = WAL;
```

### Important: No Authentication Tables Yet!

The current implementation does NOT have:
- No password_hash storage
- No auth_sessions table
- No authentication mechanism

This needs to be added for the MVP to support IMAP/SMTP authentication.

## P2P Protocol Implementation

### Protocol Types (Already Implemented)

```rust
// In fastn-net/src/protocol.rs
pub enum Protocol {
    DeviceToAccount,    // Future use
    AccountToAccount,   // Email between accounts
    AccountToDevice,    // Future use
    RigControl,         // Rig management
    // ... other protocols
}
```

### Email Message Flow

1. **Sender Account** composes email via SMTP bridge
2. **Sender** resolves recipient alias via P2P discovery
3. **Sender** connects to recipient using `AccountToAccount` protocol
4. **Sender** sends `EmailDelivery` message over Iroh
5. **Recipient** receives and stores email in mail.sqlite
6. **Recipient** saves .eml file to mails/default/{folder}/
7. **Recipient** sends acknowledgment
8. **User** retrieves email via IMAP bridge

## Implementation Tasks

### Phase 1: Authentication System (CRITICAL - Missing!)
- [ ] Add password generation and storage
- [ ] Create auth tables in automerge.sqlite or separate auth.sqlite
- [ ] Implement IMAP/SMTP authentication
- [ ] Add session management

### Phase 2: Email Message Handling
- [ ] Create EmailMessage types
- [ ] Implement handlers in fastn-rig for AccountToAccount protocol
- [ ] Add email parsing and validation

### Phase 3: P2P Email Delivery
- [ ] Implement email sender using AccountToAccount protocol
- [ ] Create email receiver with acknowledgments
- [ ] Add offline queuing in mail.sqlite
- [ ] Implement retry with exponential backoff

### Phase 4: IMAP Bridge
- [ ] Basic IMAP4rev1 server
- [ ] Authentication with default@alias
- [ ] Folder operations (LIST, SELECT)
- [ ] Message operations (FETCH, STORE, SEARCH)

### Phase 5: SMTP Bridge
- [ ] SMTP submission server (port 587)
- [ ] Authentication system
- [ ] Email parsing and validation
- [ ] Queue for P2P delivery

### Phase 6: Integration
- [ ] End-to-end email flow testing
- [ ] Email client compatibility testing
- [ ] Performance optimization
- [ ] Documentation

## Key Design Decisions

### Three-Database Architecture (As Implemented)

Benefits of the current implementation:
1. **automerge.sqlite**: All configuration and Automerge documents
   - Isolated for sync operations
   - Contains relationship and permission caches
   
2. **mail.sqlite**: Dedicated email storage
   - Optimized for email queries
   - Separate from config for performance
   
3. **db.sqlite**: User space
   - Clean slate for user applications
   - No system tables to interfere

### Missing Components for MVP

1. **Authentication**: No auth system exists yet
   - Need to decide: separate auth.sqlite or use automerge.sqlite?
   - Need password storage (Argon2 hashed)
   - Need session management

2. **Email Delivery Protocol**: Not implemented
   - Message types need definition
   - Protocol handlers need implementation

3. **IMAP/SMTP**: Not started
   - Depends on authentication system
   - Need to implement protocol handlers

## CLI Commands (Planned)

```bash
# Start fastn with email services
fastn run
# Output:
# 🚀 Starting fastn at /Users/alice/.fastn
# 📨 P2P: active on 2 endpoints
# 📬 SMTP: listening on port 587
# 📥 IMAP: listening on port 143

# Account management
fastn account create --name alice
# Output:
# Account created: 1oem6e10tckm3edrf8mdcnutle8ie7tnf40h7oukvbeatpk0k6d0
# Password: Xy3mN9pQ2wLk8Rfv
# SAVE THIS PASSWORD - it cannot be recovered!

# Bring account online
fastn account online 1oem6e10tckm3edrf8mdcnutle8ie7tnf40h7oukvbeatpk0k6d0
```

## Success Criteria

- [ ] Can create accounts with auto-generated passwords (needs auth implementation)
- [ ] Passwords work for IMAP/SMTP authentication (needs auth implementation)
- [ ] Can send emails between P2P accounts (needs protocol handlers)
- [ ] Emails persist in mail.sqlite and filesystem
- [ ] Standard email clients can connect (needs IMAP/SMTP)
- [ ] Offline queuing and retry works
- [ ] Multiple aliases per account supported (✅ already implemented)
- [ ] Graceful shutdown preserves all data (✅ already implemented)

## Next Immediate Steps

1. **Decide on Authentication Storage**:
   - Option A: Add auth tables to automerge.sqlite
   - Option B: Create separate auth.sqlite (fourth database)
   - Option C: Use mail.sqlite for auth (not recommended)

2. **Implement Authentication**:
   - Password generation and hashing
   - Storage in chosen database
   - Session management

3. **Define Email Protocol Messages**:
   - EmailDelivery message structure
   - Acknowledgment protocol
   - Error handling

4. **Implement Email Handlers**:
   - Process AccountToAccount messages
   - Store emails in database and filesystem
   - Send acknowledgments