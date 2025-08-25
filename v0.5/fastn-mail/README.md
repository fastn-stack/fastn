# fastn-mail

Complete email handling and storage system for FASTN accounts with full
SMTP/IMAP compatibility.

## Overview

The fastn-mail crate provides a **hybrid storage system** that combines the best
of database indexing and file-based storage to support real-world email clients.
This design ensures full RFC 5322 compliance while enabling fast IMAP
operations.

## Storage Architecture

### **Hybrid Storage Design**

- **Database (mail.sqlite)**: Headers and envelope information for fast IMAP
  operations (search, threading, flags)
- **Files (mails/folder/)**: Complete raw RFC 5322 message content for full
  compatibility
- **Best of both worlds**: Fast indexing + perfect SMTP/IMAP client support

### **Directory Structure**

```
account/{primary-id52}/
  mail.sqlite                    # Headers and indexing database
  mails/
    default/                     # Default mail identity
      INBOX/
        20250825_001234_msg1.eml # Raw RFC 5322 message files
        20250825_001235_msg2.eml
      Sent/
        20250825_001240_msg3.eml
      Drafts/
        draft_20250825_001245.eml
      Trash/
        deleted_20250825_001250.eml
      Custom_Folder/             # User-created folders
        ...
```

### **Database Schema**

```sql
CREATE TABLE fastn_emails
(
    email_id         TEXT PRIMARY KEY,        -- Unique ID for this email
    folder           TEXT    NOT NULL,        -- inbox, sent, drafts, trash
    file_path        TEXT    NOT NULL UNIQUE, -- Relative path to .eml file

    -- RFC 5322 Headers (extracted for IMAP indexing)
    message_id       TEXT UNIQUE,             -- Message-ID header
    from_addr        TEXT    NOT NULL,        -- From header (full email address)
    to_addr          TEXT    NOT NULL,        -- To header (comma-separated)
    cc_addr          TEXT,                    -- CC header (comma-separated)
    bcc_addr         TEXT,                    -- BCC header (comma-separated)
    subject          TEXT,                    -- Subject header
    
    -- P2P Routing Information (extracted from email addresses)
    our_alias_used   TEXT,                    -- Which of our aliases was used in this email
    our_username     TEXT,                    -- Our username (extracted from our email address)
    their_alias      TEXT,                    -- Other party's alias (sender if inbound, recipient if outbound)
    their_username   TEXT,                    -- Other party's username (extracted from email address)

    -- Threading Support (RFC 5322)
    in_reply_to      TEXT,                    -- In-Reply-To header
    references       TEXT,                    -- References header (space-separated)

    -- Timestamps
    date_sent        INTEGER,                 -- Date header (unix timestamp)
    date_received    INTEGER NOT NULL,        -- When we received it

    -- MIME Information
    content_type     TEXT,                    -- Content-Type header
    content_encoding TEXT,                    -- Content-Transfer-Encoding
    has_attachments  BOOLEAN DEFAULT 0,       -- Multipart/mixed detection

    -- File Metadata
    size_bytes       INTEGER NOT NULL,        -- Complete message size

    -- IMAP Flags
    is_seen          BOOLEAN DEFAULT 0,       -- \Seen flag
    is_flagged       BOOLEAN DEFAULT 0,       -- \Flagged flag
    is_draft         BOOLEAN DEFAULT 0,       -- \Draft flag
    is_answered      BOOLEAN DEFAULT 0,       -- \Answered flag
    is_deleted       BOOLEAN DEFAULT 0,       -- \Deleted flag
    custom_flags     TEXT                     -- JSON array of custom IMAP flags
);

-- Indexes for fast IMAP operations
CREATE INDEX idx_folder ON fastn_emails (folder);
CREATE INDEX idx_date_received ON fastn_emails (date_received DESC);
CREATE INDEX idx_date_sent ON fastn_emails (date_sent DESC);
CREATE INDEX idx_message_id ON fastn_emails (message_id);
CREATE INDEX idx_thread ON fastn_emails (in_reply_to, references);
CREATE INDEX idx_from ON fastn_emails (from_addr);
CREATE INDEX idx_subject ON fastn_emails (subject);

-- Indexes for P2P routing and delivery
CREATE INDEX idx_our_alias ON fastn_emails (our_alias_used);
CREATE INDEX idx_their_alias ON fastn_emails (their_alias);
CREATE INDEX idx_alias_pair ON fastn_emails (our_alias_used, their_alias);

CREATE TABLE fastn_email_peers
(
    peer_alias     TEXT PRIMARY KEY, -- Peer's alias ID52
    last_seen      INTEGER,          -- Last interaction timestamp
    our_alias_used TEXT NOT NULL     -- Which of our aliases they know
);
```

## Public API

### **Core Types**

#### `Mail` struct

Main object for mail operations following create/load pattern:

```rust
pub struct Mail {
    pub fn create(account_path: &Path) -> Result<Self, MailCreateError>;
    pub fn load(account_path: &Path) -> Result<Self, MailLoadError>;
    pub fn create_test() -> Self; // For testing
}
```

#### `DefaultMail` (Automerge Document)

Mail configuration stored in automerge:

```rust
pub struct DefaultMail {
    pub password_hash: String,    // SMTP/IMAP authentication
    pub is_active: bool,         // Whether mail service is enabled
    pub created_at: i64,         // Creation timestamp
}
```

### **Error Types**

- `MailCreateError` - Mail::create() failures
- `MailLoadError` - Mail::load() failures

## Public API Methods

### **A. P2P Mail Delivery**

```rust
impl Mail {
    // Periodic task - check what needs to be delivered
    pub async fn get_pending_deliveries(&self) -> Result<Vec<PendingDelivery>, MailError>;
    
    // Peer inbound - when peer contacts us for their emails
    pub async fn get_emails_for_peer(&self, peer_id52: &fastn_id52::PublicKey) -> Result<Vec<EmailForDelivery>, MailError>;
    
    // Mark email as successfully delivered to peer
    pub async fn mark_delivered_to_peer(&self, email_id: &str, peer_id52: &fastn_id52::PublicKey) -> Result<(), MailError>;
}
```

### **B. SMTP Operations**

```rust
impl Mail {
    // SMTP server receives an email and handles delivery (local storage or P2P queuing)
    pub async fn smtp_receive(&self, raw_message: Vec<u8>) -> Result<String, MailError>;
}
```

### **C. IMAP Operations**

```rust
impl Mail {
    // Folder management
    pub async fn imap_list_folders(&self) -> Result<Vec<String>, MailError>;
    pub async fn imap_select_folder(&self, folder: &str) -> Result<FolderInfo, MailError>;
    
    // Message operations
    pub async fn imap_fetch(&self, folder: &str, uid: u32) -> Result<Vec<u8>, MailError>;
    pub async fn imap_search(&self, folder: &str, criteria: &str) -> Result<Vec<u32>, MailError>;
    pub async fn imap_store_flags(&self, folder: &str, uid: u32, flags: &[String]) -> Result<(), MailError>;
    pub async fn imap_expunge(&self, folder: &str) -> Result<Vec<u32>, MailError>;
    
    // Threading
    pub async fn imap_thread(&self, folder: &str, algorithm: &str) -> Result<ThreadTree, MailError>;
}
```

### **Supporting Types**

```rust
pub struct PendingDelivery {
    pub peer_id52: fastn_id52::PublicKey,  // Which peer needs emails
    pub email_count: usize,                // How many emails pending
    pub oldest_email_date: i64,            // When oldest email was queued
}

pub struct EmailForDelivery {
    pub email_id: String,            // Internal email ID
    pub raw_message: Vec<u8>,        // Complete RFC 5322 message
    pub size_bytes: usize,           // Message size
    pub date_queued: i64,            // When queued for delivery
}

pub struct FolderInfo {
    pub name: String,                // Folder name
    pub exists: u32,                 // Number of messages
    pub recent: u32,                 // Number of recent messages
    pub unseen: u32,                 // Number of unseen messages
    pub uid_validity: u32,           // UID validity number
    pub uid_next: u32,               // Next UID to be assigned
}

pub struct ThreadTree {
    pub root_message_id: String,     // Root message of the thread
    pub children: Vec<ThreadNode>,   // Child threads
}

pub struct ThreadNode {
    pub message_id: String,          // This message's ID
    pub uid: u32,                    // IMAP UID
    pub children: Vec<ThreadNode>,   // Replies to this message
}
```

## P2P Message Format

For peer-to-peer email delivery between FASTN accounts:

```rust
// In fastn-account crate
pub enum AccountToAccountMessage {
    Email {
        /// Complete RFC 5322 message as bytes
        /// Contains all headers, body, attachments, MIME encoding
        raw_message: Vec<u8>,
    }
}
```

## Email Delivery Workflow

### **Outbound Email Flow**

1. **SMTP Submission**: User's email client sends email via SMTP to FASTN
2. **Address Parsing**: Extract ID52s from recipients (username@id52 format)
3. **Queue for Delivery**: Store in outbound queue with delivery status
4. **Periodic Delivery Task**: Every minute, check `get_pending_deliveries()`
5. **P2P Connection**: Connect to recipient's FASTN node using ID52
6. **Message Transfer**: Send `AccountToAccountMessage::Email` with raw RFC 5322
   bytes
7. **Delivery Confirmation**: Mark as delivered using `mark_delivered_to_peer()`

### **Inbound Email Flow**

1. **P2P Connection**: Peer FASTN node connects to deliver email
2. **Authentication**: Verify peer identity using ID52 cryptographic
   verification
3. **Email Request**: Peer requests emails for specific ID52 recipient
4. **Email Retrieval**: Use `get_emails_for_peer()` to get queued emails
5. **Transfer**: Send queued emails as `AccountToAccountMessage::Email`
6. **Local Delivery**: Peer stores in local INBOX using `smtp_deliver()`
7. **IMAP Access**: User's email client accesses via IMAP server

### **Address Format and Alias Mapping**

- **Format**: `username@id52` (e.g., `alice@abc123def456ghi789`)
- **ID52**: 64-character base32 public key identifier
- **Username**: Human-readable local part (can be any valid email local part)

#### **Alias Mapping Logic**
For each email, we extract and store alias relationships:

**Inbound Email** (received in INBOX):
- `our_alias_used` = our alias that received this email (from To/CC/BCC headers)
- `our_username` = our username that received this email (from To/CC/BCC headers)
- `their_alias` = sender's alias (extracted from From header)
- `their_username` = sender's username part (extracted from From header)

**Outbound Email** (stored in Sent):
- `our_alias_used` = our alias that sent this email (from From header)
- `our_username` = our username that sent this email (from From header)
- `their_alias` = recipient's alias (extracted from To header, primary recipient)
- `their_username` = recipient's username part (extracted from To header)

This allows us to:
- **Route P2P delivery**: Use `their_alias` to find the recipient's FASTN node
- **Track conversations**: Pair `(our_alias_used, their_alias)` represents a conversation
- **Reconstruct addresses**: Combine `our_username@our_alias_used` and `their_username@their_alias` for IMAP clients
- **Handle multi-alias accounts**: Know which persona was used in each conversation
- **Display names**: Show proper email addresses in email clients

### **Delivery Status Tracking**

```sql
-- Additional table for delivery tracking
CREATE TABLE fastn_email_delivery
(
    email_id        TEXT NOT NULL,     -- References fastn_emails.email_id
    recipient_id52  TEXT NOT NULL,     -- Target peer ID52
    delivery_status TEXT NOT NULL,     -- queued, delivered, failed
    attempts        INTEGER DEFAULT 0, -- Delivery attempt count
    last_attempt    INTEGER,           -- Last delivery attempt timestamp
    next_retry      INTEGER,           -- When to retry delivery
    error_message   TEXT,              -- Last delivery error (if any)

    PRIMARY KEY (email_id, recipient_id52),
    FOREIGN KEY (email_id) REFERENCES fastn_emails (email_id)
);
```

### **Periodic Tasks**

- **Every 1 minute**: Check `get_pending_deliveries()` and attempt P2P delivery
- **Every 5 minutes**: Retry failed deliveries with exponential backoff
- **Every hour**: Clean up old delivered messages and expired delivery attempts
- **Every day**: Compact database and optimize indexes

## Benefits

- ✅ **Full SMTP/IMAP Compatibility**: Raw RFC 5322 messages work with any email
  client (Thunderbird, Outlook, Apple Mail, etc.)
- ✅ **Fast IMAP Operations**: Database indexing enables efficient search,
  threading, flags, sorting
- ✅ **Simple P2P Protocol**: Just raw message bytes, no complex envelope parsing
  in transit
- ✅ **Storage Efficiency**: Headers indexed once, content stored as standard
  .eml files
- ✅ **Real-world Ready**: Handles any email that existing mail servers can
  handle
- ✅ **Delivery Reliability**: Retry logic, delivery tracking, failure handling
- ✅ **Threading Support**: Full RFC 5322 threading with In-Reply-To and
  References
- ✅ **Multi-client Support**: Multiple email clients can connect via IMAP
  simultaneously

This design ensures that FASTN can act as a drop-in replacement for traditional
mail servers (like Postfix + Dovecot) while providing decentralized P2P email
delivery.
