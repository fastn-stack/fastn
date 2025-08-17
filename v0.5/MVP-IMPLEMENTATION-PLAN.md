# MVP Implementation Plan - P2P Email System

## Overview

This document outlines the implementation plan for the FASTN MVP - a P2P email system with IMAP/SMTP bridges. Based on current progress, we have Rig and Account entities with three-database architecture already implemented.

## Current State

### Already Implemented
- ✅ **fastn-rig**: Rig entity with endpoint management
- ✅ **fastn-account**: Account entity with multi-alias support
- ✅ **fastn-net**: P2P utilities, protocols, graceful shutdown
- ✅ **Three databases per account**: automerge.sqlite, mail.sqlite, db.sqlite
- ✅ **Database migrations**: All schemas in place
- ✅ **Endpoint management**: Protocol-based message routing
- ✅ **Folder structure**: mails/default/{inbox,sent,drafts,trash}

### Critical Missing Components
- ❌ **Authentication system**: No password storage or auth tables
- ❌ **Email protocol handlers**: AccountToAccount message processing
- ❌ **IMAP/SMTP servers**: Not started
- ❌ **Email delivery logic**: P2P email sending/receiving

## Implementation Phases

### Phase 1: Authentication System (Week 1 - PRIORITY)

#### 1.1 Design Decision: Where to Store Auth
**Recommendation**: Add auth tables to `mail.sqlite` since auth is primarily for email access.

```sql
-- Add to mail.sqlite migrations
CREATE TABLE IF NOT EXISTS fastn_auth (
    password_hash     TEXT NOT NULL,      -- Argon2 hash
    created_at        INTEGER NOT NULL,
    updated_at        INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS fastn_auth_sessions (
    session_id        TEXT PRIMARY KEY,
    username          TEXT NOT NULL,       -- 'default' for MVP
    alias_used        TEXT NOT NULL,       -- Which alias authenticated
    client_info       TEXT,                -- User agent, IP, etc.
    created_at        INTEGER NOT NULL,
    last_activity     INTEGER NOT NULL,
    expires_at        INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_sessions_expires ON fastn_auth_sessions(expires_at);
```

#### 1.2 Implementation Tasks
- [ ] Update mail.sqlite migrations with auth tables
- [ ] Add password generation on account creation
- [ ] Implement Argon2 password hashing
- [ ] Store password hash in fastn_auth table
- [ ] Print password to stdout (one-time display)
- [ ] Add session management functions

#### 1.3 Code Location
```rust
// In fastn-account/src/auth.rs (new file)
pub fn generate_password() -> String;
pub fn hash_password(password: &str) -> Result<String>;
pub fn verify_password(password: &str, hash: &str) -> Result<bool>;
pub fn create_session(username: &str, alias: &str) -> Result<String>;
pub fn verify_session(session_id: &str) -> Result<(String, String)>;
```

### Phase 2: Email Protocol Messages (Week 1)

#### 2.1 Message Types Definition
```rust
// In fastn-account/src/email/protocol.rs (new file)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmailMessage {
    Deliver {
        from: String,           // username@sender_alias
        to: Vec<String>,        // [username@recipient_alias, ...]
        raw_email: Vec<u8>,     // RFC 2822 format
        message_id: String,
        timestamp: u64,
    },
    
    Acknowledge {
        message_id: String,
        status: DeliveryStatus,
        timestamp: u64,
    },
    
    Bounce {
        message_id: String,
        reason: String,
        timestamp: u64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeliveryStatus {
    Accepted,
    Queued,
    Rejected(String),
}
```

#### 2.2 Protocol Handler Integration
- [ ] Update handle_connection in fastn-rig/src/endpoint.rs
- [ ] Process AccountToAccount protocol messages
- [ ] Route EmailMessage types to account handlers
- [ ] Send acknowledgments back to sender

### Phase 3: Email Storage & Delivery (Week 1-2)

#### 3.1 Email Receiver
```rust
// In fastn-account/src/email/receiver.rs (new file)
impl Account {
    pub async fn receive_email(&self, message: EmailMessage) -> Result<()> {
        // 1. Validate recipient belongs to this account
        // 2. Parse raw email (RFC 2822)
        // 3. Save to filesystem (mails/default/inbox/)
        // 4. Index in mail.sqlite
        // 5. Send acknowledgment
    }
}
```

#### 3.2 Email Sender
```rust
// In fastn-account/src/email/sender.rs (new file)
impl Account {
    pub async fn send_email(
        &self,
        from_alias: &str,
        to: Vec<String>,
        raw_email: Vec<u8>,
    ) -> Result<()> {
        // 1. Parse recipients
        // 2. Group by destination alias
        // 3. For each destination:
        //    a. Resolve peer endpoint
        //    b. Connect using AccountToAccount protocol
        //    c. Send EmailMessage::Deliver
        //    d. Wait for acknowledgment
        // 4. Queue failed deliveries
    }
}
```

#### 3.3 Implementation Tasks
- [ ] Implement email receiver in Account
- [ ] Implement email sender in Account
- [ ] Add .eml file storage with timestamps
- [ ] Update mail.sqlite with email records
- [ ] Add peer resolution and caching
- [ ] Implement offline queuing

### Phase 4: IMAP Server (Week 2)

#### 4.1 Server Structure
```rust
// In fastn-account/src/imap/mod.rs (new file)
pub struct ImapServer {
    account: Account,
    port: u16,
}

impl ImapServer {
    pub async fn start(&self) -> Result<()>;
    async fn handle_client(account: Account, stream: TcpStream);
}
```

#### 4.2 Core Commands
- [ ] **CAPABILITY**: Return server capabilities
- [ ] **LOGIN**: Authenticate with default@alias and password
- [ ] **LIST**: List folders (INBOX, Sent, Drafts, Trash)
- [ ] **SELECT**: Select a folder
- [ ] **FETCH**: Retrieve messages
- [ ] **STORE**: Update flags (read, starred)
- [ ] **SEARCH**: Search messages
- [ ] **LOGOUT**: End session

#### 4.3 Implementation Tasks
- [ ] Create IMAP server skeleton
- [ ] Implement authentication with session management
- [ ] Add folder operations
- [ ] Implement message retrieval from mail.sqlite
- [ ] Add flag updates
- [ ] Test with email clients

### Phase 5: SMTP Server (Week 2-3)

#### 5.1 Server Structure
```rust
// In fastn-account/src/smtp/mod.rs (new file)
pub struct SmtpServer {
    account: Account,
    port: u16,
}

impl SmtpServer {
    pub async fn start(&self) -> Result<()>;
    async fn handle_client(account: Account, stream: TcpStream);
}
```

#### 5.2 Core Commands
- [ ] **EHLO/HELO**: Greeting
- [ ] **AUTH LOGIN**: Authenticate
- [ ] **MAIL FROM**: Set sender
- [ ] **RCPT TO**: Add recipients
- [ ] **DATA**: Receive email content
- [ ] **QUIT**: Close connection

#### 5.3 Implementation Tasks
- [ ] Create SMTP server skeleton
- [ ] Implement authentication
- [ ] Parse email headers and body
- [ ] Queue for P2P delivery
- [ ] Integrate with email sender
- [ ] Test with email clients

### Phase 6: Integration & Testing (Week 3)

#### 6.1 CLI Commands
- [ ] `fastn account create --password-display`
- [ ] `fastn account online <id52>`
- [ ] `fastn email send --from --to --subject --body`
- [ ] `fastn email list`
- [ ] `fastn imap start`
- [ ] `fastn smtp start`

#### 6.2 End-to-End Testing
- [ ] Account creation with password
- [ ] SMTP authentication and send
- [ ] P2P delivery between accounts
- [ ] IMAP retrieval
- [ ] Offline queuing and retry
- [ ] Multiple alias handling

#### 6.3 Email Client Testing
- [ ] Thunderbird configuration
- [ ] Apple Mail configuration
- [ ] Outlook configuration
- [ ] Mobile client (K-9 Mail)

## File Structure Updates

```
fastn-account/
├── src/
│   ├── lib.rs              # Existing
│   ├── account.rs          # Existing
│   ├── alias.rs            # Existing
│   ├── auth.rs             # NEW: Authentication system
│   ├── email/              # NEW: Email subsystem
│   │   ├── mod.rs
│   │   ├── protocol.rs     # Message types
│   │   ├── receiver.rs     # Receive emails
│   │   ├── sender.rs       # Send emails
│   │   ├── storage.rs      # File storage
│   │   └── queue.rs        # Offline queue
│   ├── imap/               # NEW: IMAP server
│   │   ├── mod.rs
│   │   ├── commands.rs
│   │   ├── session.rs
│   │   └── mailbox.rs
│   └── smtp/               # NEW: SMTP server
│       ├── mod.rs
│       ├── commands.rs
│       ├── parser.rs
│       └── queue.rs

fastn-rig/
├── src/
│   ├── endpoint.rs         # UPDATE: Add email handlers
│   └── email_handler.rs    # NEW: Route emails to accounts
```

## Dependencies to Add

```toml
# In fastn-account/Cargo.toml
[dependencies]
# Authentication
argon2 = "0.5"
rand = "0.8"

# Email parsing
mail-parser = "0.9"  # or mailparse = "0.14"

# IMAP/SMTP protocols
async-trait = "0.1"
bytes = "1"

# Existing dependencies
rusqlite = "0.32"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
eyre = "0.6"
tracing = "0.1"
```

## Implementation Priority

1. **Critical Path** (Must have for MVP):
   - Authentication system
   - Email protocol messages
   - Basic email send/receive
   - Simple IMAP (just enough for reading)
   - Simple SMTP (just enough for sending)

2. **Important** (Should have):
   - Offline queuing
   - Retry mechanism
   - Multiple alias support
   - Session management

3. **Nice to Have** (Can defer):
   - Advanced IMAP features (IDLE, THREAD)
   - Email search
   - Attachment handling
   - Rate limiting

## Success Metrics

### Week 1 Completion
- [ ] Password authentication works
- [ ] Can send P2P email between two accounts
- [ ] Email stored in mail.sqlite and filesystem

### Week 2 Completion
- [ ] IMAP server accepts connections
- [ ] Can read emails via Thunderbird
- [ ] SMTP server accepts emails
- [ ] Emails queued for P2P delivery

### Week 3 Completion
- [ ] End-to-end flow works
- [ ] Multiple email clients tested
- [ ] Offline/online transitions handled
- [ ] Documentation complete

## Risk Mitigation

### Technical Risks
- **IMAP Complexity**: Start with READ-ONLY IMAP, add write operations later
- **Email Parsing**: Use well-tested library (mail-parser)
- **P2P Reliability**: Simple retry with exponential backoff
- **Authentication**: Keep it simple - single password per account

### Schedule Risks
- **Feature Creep**: Strictly follow MVP scope
- **Testing Time**: Automate tests early
- **Client Compatibility**: Test with one client first (Thunderbird)

## Next Immediate Actions

1. **Add auth tables to mail.sqlite migration**
2. **Implement password generation and hashing**
3. **Update account creation to store password hash**
4. **Define EmailMessage types**
5. **Update endpoint handler for AccountToAccount protocol**

This plan reflects the current state of implementation and provides a clear path forward for completing the P2P email MVP.