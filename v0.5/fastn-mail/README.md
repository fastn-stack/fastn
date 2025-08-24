# fastn-mail

Email handling and storage for FASTN accounts.

## Features

- **Mail Database**: SQLite schema for email storage and indexing
- **Automerge Documents**: Mail configuration documents (DefaultMail)
- **Directory Management**: Creates proper mail folder structure
- **Database Migrations**: Handles mail database schema setup

## Usage

```rust
use fastn_mail;

// Create mail directory structure
fastn_mail::create_mail_directories(&account_path)?;

// Create mail database connection
let mail_conn = fastn_mail::create_connection(&mail_db_path)?;

// Run database migrations
fastn_mail::migrate_database(&mail_conn)?;

// Check if database exists
if fastn_mail::database_exists(&mail_db_path) {
    // Database already exists
}
```

## Database Schema

### fastn_emails Table
- Email storage with full metadata (subject, body preview, attachments, etc.)
- Folder organization (inbox, sent, drafts, trash)
- Full-text search indexes on key fields

### fastn_email_peers Table  
- P2P email peer tracking
- Endpoint information for email routing
- Alias-based peer relationships

## Directory Structure

```
account/
  mails/
    default/
      inbox/     # Incoming emails
      sent/      # Sent emails  
      drafts/    # Draft emails
      trash/     # Deleted emails
```

## Integration

- `fastn-automerge`: Mail configuration documents
- `fastn-id52`: Email peer identification
- `rusqlite`: Email storage and querying