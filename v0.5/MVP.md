# FASTN MVP - P2P Email Only

## Overview

This MVP implements a minimal P2P email system using FASTN. It focuses solely on email functionality over Iroh, without HTTP servers, devices, or rigs. Uses Automerge for account configuration (without sync).

## Scope

### What's Included
- Account entities with aliases
- P2P email delivery over Iroh
- IMAP/SMTP bridges for standard email clients
- SQLite storage for emails and configuration
- Basic peer discovery and connection

### What's Excluded
- HTTP server (no web interface)
- Device and Rig entities
- Automerge sync (documents used locally only)
- File serving
- Groups and permissions
- Browsing modes

## Architecture

### Account Entity

Each account:
- Has one or more aliases (ID52 keypairs)
- Stores all data in SQLite (emails, auth, config)
- Runs Iroh endpoint for P2P communication
- Provides IMAP/SMTP servers

Storage structure:
```
{fastn_home}/
└── accounts/
    └── {first_alias_id52}/
        ├── db.sqlite           # All account data (emails, auth, config)
        ├── aliases/            # Keypairs for all aliases
        │   ├── {alias1}.id52
        │   ├── {alias1}.private-key
        │   └── {alias2}.id52
        └── mails/              # Email files
            └── {username}/     # Per-username folders
                ├── inbox/
                │   └── {timestamp}-{id}.eml
                └── sent/
                    └── {timestamp}-{id}.eml
```

### Email Addressing

- Format: `username@alias_id52`
- Example: `alice@1oem6e10tckm3edrf8mdcnutle8ie7tnf40h7oukvbeatpk0k6d0`
- Each alias acts as an email domain
- **Single Username**: All emails go to "default" username (catch-all)

#### Simplified Username Resolution

- ALL emails to any `*@alias` address go to the `default` folder
- No custom usernames in MVP
- Examples:
  - `alice@alias` → delivered to default/inbox/
  - `bob@alias` → delivered to default/inbox/
  - `anything@alias` → delivered to default/inbox/

## Authentication

### IMAP/SMTP Authentication Design

For the MVP, we'll use an auto-generated password system:

1. **Account Password**: Auto-generated on account creation, printed to stdout
2. **Storage**: Argon2 hashed password in database
3. **Username Format**: `default@alias_id52` (only valid username)
4. **Password**: The auto-generated password

### Database Architecture

MVP uses two SQLite databases:

1. **automerge.sqlite** - Configuration and documents
   - Stores account config, aliases, settings
   - All tables prefixed with `fastn_`
   
2. **mail.sqlite** - Email system
   - Email index, peers, sessions
   - All tables prefixed with `fastn_`
   - Has read-only access to automerge.sqlite

### Database Schema

```sql
-- In automerge.sqlite:
CREATE TABLE fastn_documents (
    path              TEXT PRIMARY KEY,     -- e.g., '/-/config', '/-/aliases/{id52}/readme'
    automerge_binary  BLOB NOT NULL,        -- Automerge document as binary
    heads             TEXT NOT NULL,        -- JSON array of head hashes
    actor_id          TEXT NOT NULL,        -- Our actor ID for this doc
    updated_at        INTEGER NOT NULL
);

-- In mail.sqlite:
CREATE TABLE fastn_emails (
    email_id          TEXT PRIMARY KEY,
    folder            TEXT NOT NULL,        -- 'inbox', 'sent', 'drafts', 'trash'
    
    -- Original addressing (what was in the email)
    original_to       TEXT NOT NULL,        -- Original recipient address
    from_address      TEXT NOT NULL,        -- Full: username@id52
    to_addresses      TEXT NOT NULL,        -- JSON array
    cc_addresses      TEXT,
    bcc_addresses     TEXT,
    
    -- Which alias sent/received
    received_at_alias TEXT,
    sent_from_alias   TEXT,
    
    -- Content
    subject           TEXT,
    body_preview      TEXT,
    has_attachments   BOOLEAN DEFAULT FALSE,
    
    -- File reference
    file_path         TEXT NOT NULL UNIQUE,
    size_bytes        INTEGER NOT NULL,
    message_id        TEXT,
    in_reply_to       TEXT,
    references        TEXT,
    
    -- Timestamps
    date_sent         INTEGER,
    date_received     INTEGER,
    
    -- Status
    is_read           BOOLEAN DEFAULT FALSE,
    is_starred        BOOLEAN DEFAULT FALSE,
    flags             TEXT,            -- JSON array
    
    INDEX idx_folder (folder),
    INDEX idx_date (date_received DESC),
    INDEX idx_message_id (message_id)
);

-- Peer connections for email delivery
CREATE TABLE fastn_email_peers (
    peer_alias        TEXT PRIMARY KEY,  -- Their alias ID52
    last_seen         INTEGER,
    endpoint          BLOB,              -- Iroh endpoint info
    our_alias_used    TEXT NOT NULL,     -- Which of our aliases knows them
    
    INDEX idx_our_alias (our_alias_used)
);

-- Authentication sessions (for connection tracking)
CREATE TABLE fastn_auth_sessions (
    session_id        TEXT PRIMARY KEY,
    username          TEXT NOT NULL,
    alias_used        TEXT NOT NULL,
    client_info       TEXT,              -- User agent, IP, etc.
    created_at        INTEGER NOT NULL,
    last_activity     INTEGER NOT NULL,
    expires_at        INTEGER NOT NULL,
    
    INDEX idx_activity (last_activity),
    INDEX idx_expires (expires_at)
);
```

### Automerge Document Structure

For MVP, we'll have these Automerge documents:

**`/-/config`** - Main account configuration:
```json
{
  "primary_alias": "abc123...",  // First alias ID52 (folder name)
  "password_hash": "argon2...",  // Argon2 hash of auto-generated password
  "settings": {
    "smtp_enabled": true,
    "imap_enabled": true,
    "smtp_port": 587,
    "imap_port": 143
  },
  "created_at": 1234567890,
  "updated_at": 1234567890
}
```

**`/-/aliases/{alias-id52}/readme`** - Public info for each alias:
```json
{
  "name": "primary",           // Friendly name
  "created_at": 1234567890,
  "is_primary": true,          // Is this the primary alias?
  "private_key_in_keyring": true  // or false if in file
}
```

**`/-/aliases/{alias-id52}/notes`** - Private notes (for future use):
```json
{
  // Empty for MVP, will be used for private notes about other aliases
}
```

### Account Creation

```rust
use rand::Rng;

pub struct AccountManager {
    db: Arc<Mutex<Connection>>,
}

impl AccountManager {
    // Generate a secure random password
    fn generate_password() -> String {
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                                 abcdefghijklmnopqrstuvwxyz\
                                 0123456789";
        const PASSWORD_LEN: usize = 16;
        
        let mut rng = rand::thread_rng();
        
        (0..PASSWORD_LEN)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }
    
    // Create new account with auto-generated password
    pub fn create_account(&self, alias_name: Option<&str>) -> Result<(String, String)> {
        let password = Self::generate_password();
        
        // Hash the password
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)?
            .to_string();
        
        // Generate first alias keypair
        let (alias_id52, _private_key) = generate_keypair()?;
        let now = Utc::now().timestamp();
        
        let db = self.db.lock().unwrap();
        
        // Create account (using first alias as primary key)
        db.execute(
            "INSERT INTO account (
                primary_alias, password_hash, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?3)",
            params![alias_id52, password_hash, now],
        )?;
        
        // Store alias
        db.execute(
            "INSERT INTO account_aliases (alias_id52, alias_name, created_at, is_primary)
             VALUES (?1, ?2, ?3, 1)",
            params![alias_id52, alias_name, now],
        )?;
        
        // Create default email folder
        let mail_path = format!("{}/mails/default", alias_id52);
        std::fs::create_dir_all(&mail_path)?;
        std::fs::create_dir_all(format!("{}/inbox", mail_path))?;
        std::fs::create_dir_all(format!("{}/sent", mail_path))?;
        std::fs::create_dir_all(format!("{}/drafts", mail_path))?;
        std::fs::create_dir_all(format!("{}/trash", mail_path))?;
        
        Ok((alias_id52.clone(), password))
    }
}
```

### Email Reception Logic

```rust
pub struct EmailReceiver {
    db: Arc<Mutex<Connection>>,
}

impl EmailReceiver {
    // All emails go to 'default' folder
    
    pub async fn receive_email(&self, message: EmailMessage) -> Result<()> {
        let EmailMessage::Deliver { from, to, raw_email, message_id } = message else {
            return Err(EmailError::InvalidMessage);
        };
        
        for recipient in &to {
            let (original_username, alias) = parse_email_address(recipient)?;
            
            // Check if alias belongs to us
            if !self.has_alias(&alias)? {
                continue; // Not for us
            }
            
            // Resolve to actual storage username
            let storage_username = self.resolve_username(recipient)?;
            
            // Save email
            let email_path = self.save_email_file(&storage_username, &raw_email)?;
            
            // Index in database
            let db = self.db.lock().unwrap();
            db.execute(
                "INSERT INTO emails (
                    email_id, username, folder, original_to,
                    from_address, to_addresses, received_at_alias,
                    subject, file_path, size_bytes, message_id,
                    date_received, is_read
                ) VALUES (?1, ?2, 'inbox', ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 0)",
                params![
                    generate_email_id(),
                    storage_username,        // Where it's actually stored
                    recipient,               // Original recipient address
                    from,
                    serde_json::to_string(&to)?,
                    alias,
                    extract_subject(&raw_email),
                    email_path,
                    raw_email.len(),
                    message_id,
                    Utc::now().timestamp(),
                ],
            )?;
        }
        
        Ok(())
    }
}
```

### Authentication Flow

```rust
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use rusqlite::{Connection, params};

pub struct EmailAuth {
    db: Arc<Mutex<Connection>>,
}

impl EmailAuth {
    // Not needed in simplified MVP - account creation handles password init
    
    // Verify IMAP/SMTP login (only real usernames can authenticate)
    pub fn verify_login(
        &self,
        username_with_alias: &str,  // e.g., "alice@1oem6e..."
        password: &str,
    ) -> Result<(String, String)> {  // Returns (username, alias)
        // Parse username@alias
        let (username, alias) = parse_email_address(username_with_alias)?;
        
        let db = self.db.lock().unwrap();
        
        // Check if username exists and can authenticate
        let can_auth = db.query_row(
            "SELECT can_authenticate FROM allowed_usernames 
             WHERE username = ?1 AND is_active = 1",
            params![username],
            |row| row.get::<_, bool>(0),
        ).map_err(|_| AuthError::InvalidUsername)?;
        
        if !can_auth {
            return Err(AuthError::CannotAuthenticate);
        }
        
        // Check if alias exists
        let alias_exists = db.query_row(
            "SELECT 1 FROM account_aliases WHERE alias_id52 = ?1 AND is_active = 1",
            params![alias],
            |_| Ok(()),
        ).is_ok();
        
        if !alias_exists {
            return Err(AuthError::UnknownAlias);
        }
        
        // Get password hash from account table
        let password_hash: String = db.query_row(
            "SELECT password_hash FROM account LIMIT 1",
            [],
            |row| row.get(0),
        )?;
        
        // Verify password
        let parsed_hash = PasswordHash::new(&password_hash)?;
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| AuthError::InvalidPassword)?;
        
        // Create session
        let session_id = generate_session_id();
        let now = Utc::now().timestamp();
        let expires = now + 3600 * 24; // 24 hour sessions
        
        db.execute(
            "INSERT INTO auth_sessions 
             (session_id, username, alias_used, created_at, last_activity, expires_at)
             VALUES (?1, ?2, ?3, ?4, ?4, ?5)",
            params![session_id, username, alias, now, expires],
        )?;
        
        Ok((username, alias))
    }
}
```

### CLI Commands for Auth Management

```bash
# Create account with password and default username
fastn account create alice --password
Enter password: ****
Confirm password: ****
Set alice as default username? (y/n): y
Account created with alias: 1oem6e10tckm3edrf8mdcnutle8ie7tnf40h7oukvbeatpk0k6d0

# Add another username
fastn account add-username alice bob
# bob@{alias} now works for receiving email

# Set default username (catch-all)
fastn account set-default alice alice
# All unknown usernames will go to alice's mailbox

# Remove default (reject unknown usernames)
fastn account unset-default alice

# List usernames
fastn account list-usernames alice
# Output: 
#   alice (default, can authenticate)
#   bob (can authenticate)
```

## P2P Email Protocol

### Message Types

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmailMessage {
    // Email delivery
    Deliver {
        from: String,           // username@sender_alias
        to: Vec<String>,        // [username@recipient_alias, ...]
        raw_email: Vec<u8>,     // Complete RFC 2822 email
        message_id: String,
    },
    
    // Delivery confirmation
    Acknowledge {
        message_id: String,
        status: DeliveryStatus,
    },
    
    // Peer discovery
    Announce {
        alias: String,
        endpoint: Vec<u8>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeliveryStatus {
    Accepted,
    Rejected(String),
    Queued,
}
```

## Usage Example

```bash
# Create account with default username
fastn account create alice --password --default alice
Enter password: ****
Account created with alias: 1oem6e10tckm3edrf8mdcnutle8ie7tnf40h7oukvbeatpk0k6d0

# Add specific usernames
fastn account add-username alice support
fastn account add-username alice admin

# Start the account
fastn account start alice

# Now these all work:
# alice@1oem6e... → alice folder
# support@1oem6e... → support folder  
# admin@1oem6e... → admin folder
# randomuser@1oem6e... → alice folder (default)
# anything@1oem6e... → alice folder (default)

# But for IMAP/SMTP login, only real usernames work:
# ✓ alice@1oem6e... with password
# ✓ support@1oem6e... with password
# ✓ admin@1oem6e... with password
# ✗ randomuser@1oem6e... (cannot authenticate)
```

## Key Differences from Full Architecture

1. **No Automerge**: All data in SQLite only
2. **No HTTP**: No web interface or REST APIs
3. **No Devices/Rigs**: Only Account entities
4. **Simplified Auth**: Password-based, no device certificates
5. **No Groups**: Direct email addresses only
6. **No Document Sync**: Email-only protocol over Iroh

## Success Criteria

- [ ] Can create accounts with password protection
- [ ] IMAP/SMTP auth works with standard clients
- [ ] Can send/receive emails between P2P accounts
- [ ] Default username catches unmatched recipients
- [ ] Emails persist across restarts
- [ ] Multiple aliases per account work correctly
- [ ] Multiple usernames per account work correctly
- [ ] Offline queuing and retry for failed deliveries

## Future Extensions (Post-MVP)

After this MVP works, we can add:
1. Device certificates for password-less auth
2. Wildcard patterns for username matching
3. Per-username passwords
4. Email forwarding rules
5. Encrypted password sync via Automerge
6. Webmail interface with HTTP auth
7. External email gateway with separate credentials