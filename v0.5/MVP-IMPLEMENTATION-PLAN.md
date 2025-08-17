# MVP Implementation Plan - P2P Email System

## Overview

This document outlines the implementation plan for the FASTN MVP - a P2P email system with IMAP/SMTP bridges. The MVP focuses solely on email functionality without HTTP servers, devices, rigs, or Automerge documents.

## Architecture Decisions

### Crate Structure
- **fastn-entity**: Keep as reference for patterns and code reuse
- **fastn-account**: New crate containing Account, Email, IMAP, SMTP functionality (all-in-one for now)
- **fastn**: Main CLI handling folder paths and orchestration
- **Future**: Split fastn-account when it grows (fastn-automerge will be needed by both account and device)

### Folder Path Management
- **fastn CLI**: Determines and passes full folder paths to crates
- **fastn-account**: Receives complete paths, no default folder logic
- **Benefit**: Centralized path management in one place

## Implementation Phases

### Phase 1: Core Account Infrastructure (Week 1)

#### 1.1 Refactor fastn-entity to fastn-account
- [ ] Rename crate to `fastn-account` to reflect MVP focus
- [ ] Modify entity structure to support multiple aliases
- [ ] Add account-specific database schema
- [ ] Implement auto-generated password system

**New Account Structure**:
```rust
pub struct Account {
    pub primary_alias: String,           // First alias ID52 (folder name)
    pub aliases: Vec<Alias>,             // Multiple ID52 identities
    pub password_hash: String,           // Argon2 hash
    pub storage_path: PathBuf,           // Base directory
    pub db: Arc<Mutex<Connection>>,      // SQLite connection
}

pub struct Alias {
    pub id52: String,
    pub name: Option<String>,            // Friendly name
    pub public_key: PublicKey,
    pub secret_key: SecretKey,
    pub is_primary: bool,                // For folder naming only
}
```

#### 1.2 Database Schema Implementation
- [ ] Create migration system for database initialization
- [ ] Implement tables: `account`, `account_aliases`, `emails`, `email_peers`, `auth_sessions`
- [ ] Add email indexing functionality
- [ ] Create test suite for database operations

#### 1.3 CLI Foundation
- [ ] Create basic CLI structure using clap
- [ ] Implement `fastn account create` command
- [ ] Add password generation and display to stdout
- [ ] Store account configuration in SQLite (not config.json)

### Phase 2: P2P Networking Layer (Week 1-2)

#### 2.1 Iroh Integration
- [ ] Set up Iroh endpoint management
- [ ] Implement peer discovery mechanism
- [ ] Create connection pool for active peers
- [ ] Add retry logic for failed connections

#### 2.2 Email Protocol Messages
- [ ] Define message types (Deliver, Acknowledge, Announce)
- [ ] Implement serialization/deserialization
- [ ] Create message routing based on alias
- [ ] Add delivery confirmation system

#### 2.3 Email Delivery Engine
- [ ] Implement P2P email sender
- [ ] Create email receiver with folder routing
- [ ] Add queue for offline delivery
- [ ] Implement retry mechanism with backoff

### Phase 3: Email Storage System (Week 2)

#### 3.1 Filesystem Storage
- [ ] Create folder structure (`mails/default/inbox|sent|drafts|trash`)
- [ ] Implement .eml file storage with timestamp prefixes
- [ ] Add email indexing to SQLite
- [ ] Create cleanup/archival system

#### 3.2 Email Processing
- [ ] Parse RFC 2822 email format
- [ ] Extract headers and metadata
- [ ] Handle attachments
- [ ] Create search functionality using SQLite FTS

### Phase 4: IMAP Bridge (Week 2-3)

#### 4.1 IMAP Server Core
- [ ] Implement basic IMAP4rev1 protocol
- [ ] Add authentication with `default@alias_id52` username
- [ ] Create folder listing (INBOX, Sent, Drafts, Trash)
- [ ] Implement message retrieval

#### 4.2 IMAP Operations
- [ ] LIST - folder enumeration
- [ ] SELECT/EXAMINE - folder selection
- [ ] FETCH - message retrieval
- [ ] STORE - flag updates
- [ ] SEARCH - message searching
- [ ] IDLE - push notifications

### Phase 5: SMTP Bridge (Week 3)

#### 5.1 SMTP Server Core
- [ ] Implement SMTP submission (port 587)
- [ ] Add STARTTLS support
- [ ] Create authentication system
- [ ] Implement basic SMTP commands

#### 5.2 SMTP Operations
- [ ] MAIL FROM validation
- [ ] RCPT TO processing with alias resolution
- [ ] DATA handling and email parsing
- [ ] Queue management for P2P delivery

### Phase 6: Integration & Testing (Week 4)

#### 6.1 End-to-End Testing
- [ ] Test email flow: compose → SMTP → P2P → IMAP → read
- [ ] Multi-account testing
- [ ] Offline/online transitions
- [ ] Performance testing with large mailboxes

#### 6.2 Email Client Compatibility
- [ ] Test with Thunderbird
- [ ] Test with Apple Mail
- [ ] Test with Outlook
- [ ] Test with mobile clients (K-9 Mail)

#### 6.3 Documentation
- [ ] User guide for account setup
- [ ] Email client configuration guide
- [ ] Troubleshooting guide
- [ ] API documentation

## File Structure

```
fastn-entity/               # Keep as reference
├── src/
│   ├── lib.rs              # Single entity management
│   ├── entity.rs           # Entity implementation
│   ├── entity_manager.rs   # Manager for multiple entities
│   └── migration.rs        # Database migrations

fastn-account/              # New all-in-one crate for MVP
├── src/
│   ├── lib.rs              # Account management with multiple aliases
│   ├── account.rs          # Account struct and core logic
│   ├── alias.rs            # Alias (ID52) management
│   ├── auth.rs             # Password & authentication
│   ├── database.rs         # SQLite schema & queries
│   ├── migration.rs        # Database migrations
│   ├── email/
│   │   ├── mod.rs          # Email system core
│   │   ├── storage.rs      # Email file storage
│   │   ├── protocol.rs     # P2P email protocol
│   │   ├── delivery.rs     # Email delivery engine
│   │   └── indexing.rs     # Email search indexing
│   ├── imap/
│   │   ├── mod.rs          # IMAP server
│   │   ├── commands.rs     # IMAP command handlers
│   │   ├── session.rs      # Client session management
│   │   └── mailbox.rs      # Mailbox operations
│   └── smtp/
│       ├── mod.rs          # SMTP server
│       ├── commands.rs     # SMTP command handlers
│       ├── queue.rs        # Delivery queue
│       └── relay.rs        # P2P relay logic

fastn/                      # Main CLI
├── src/
│   ├── main.rs             # CLI entry point
│   ├── commands/
│   │   ├── account.rs      # Account management commands
│   │   ├── email.rs        # Email operations
│   │   └── server.rs       # Start IMAP/SMTP servers
│   ├── paths.rs            # Centralized path management
│   └── runner.rs           # Async task management
```

## Key Implementation Decisions

### 1. Storage Strategy
- **SQLite for all metadata**: Indexes, configuration, sessions
- **Filesystem for email content**: .eml files in organized folders
- **No Automerge in MVP**: Simplified storage model

### 2. Authentication
- **Single username**: "default" catches all emails
- **Auto-generated password**: Printed once at account creation
- **Argon2 hashing**: Industry standard password hashing

### 3. P2P Communication
- **Direct Iroh connections**: No intermediate servers
- **Alias-based routing**: Each alias is independent
- **Retry with exponential backoff**: Handle offline peers

### 4. Email Compatibility
- **Full RFC 2822 support**: Standard email format
- **IMAP4rev1 compliance**: Works with all email clients
- **SMTP submission only**: No open relay

## Testing Strategy

### Unit Tests
- Database operations
- Email parsing and storage
- Protocol message handling
- Authentication flows

### Integration Tests
- Account creation and management
- P2P email delivery
- IMAP/SMTP bridge operations
- Multi-account scenarios

### End-to-End Tests
- Complete email flow
- Client compatibility
- Performance benchmarks
- Failure recovery

## Success Metrics

1. **Core Functionality**
   - [ ] Can create account with auto-generated password
   - [ ] Password printed to stdout works for IMAP/SMTP
   - [ ] Can send email between P2P accounts
   - [ ] Can receive email to default folder
   - [ ] IMAP client can read emails
   - [ ] SMTP client can send emails

2. **Reliability**
   - [ ] Offline queuing works
   - [ ] Retry mechanism delivers delayed emails
   - [ ] No email loss during restarts
   - [ ] Database transactions are atomic

3. **Compatibility**
   - [ ] Works with major email clients
   - [ ] Handles various email formats
   - [ ] Supports attachments
   - [ ] Preserves email headers

## Timeline

- **Week 1**: Core infrastructure + Start networking
- **Week 2**: Complete networking + Email storage + Start IMAP
- **Week 3**: Complete IMAP + SMTP implementation
- **Week 4**: Integration, testing, and documentation

## Next Steps

1. **Immediate Action**: Refactor `fastn-entity` to `fastn-account` with multiple alias support
2. **Priority**: Get basic account creation with password generation working
3. **Quick Win**: Implement P2P email delivery between two accounts
4. **User Value**: Add IMAP server for email client access

## Risk Mitigation

### Technical Risks
- **IMAP Complexity**: Start with minimal command set, expand gradually
- **P2P Reliability**: Implement robust retry and queuing early
- **Email Parsing**: Use well-tested email parsing libraries

### Schedule Risks
- **Scope Creep**: Strictly follow MVP scope, no extra features
- **Testing Time**: Allocate full week for testing and fixes
- **Client Compatibility**: Test with one client first, then expand

## Dependencies

### External Crates
- `iroh`: P2P networking
- `rusqlite`: Database
- `argon2`: Password hashing
- `mail-parser` or `email`: Email parsing
- `async-imap` (reference): IMAP protocol
- `lettre` (reference): SMTP protocol
- `tokio`: Async runtime
- `clap`: CLI framework

### System Requirements
- SQLite3
- Rust 1.75+
- Network connectivity for P2P
- Email client for testing