# fastn-account

Multi-alias account management for the FASTN P2P network.

## Overview

An Account in FASTN represents a user with potentially multiple identities (aliases). This design allows users to maintain separate personas for different contexts - work, personal, anonymous, etc. Each alias has its own ID52 identity and can send/receive emails independently.

## Key Features

- **Multiple Aliases**: Each account can have multiple ID52 identities
- **Three Databases**: Clean separation of concerns
  - `automerge.sqlite`: Automerge CRDT documents for configuration
  - `mail.sqlite`: Email storage and indexing
  - `db.sqlite`: User application data
- **Secure Key Management**: Private keys stored in system keyring or files
- **Email Support**: Each alias can send/receive emails independently

## Directory Structure

```
accounts/
  {primary-id52}/              # Account directory named by primary alias
    aliases/                   # Keys for all aliases
      {id52}.id52             # Public key file
      {id52}.private-key      # Secret key (if SKIP_KEYRING=true)
    mails/                    # Email storage
      default/
        inbox/
        sent/
        drafts/
        trash/
    automerge.sqlite          # Automerge documents
    mail.sqlite               # Email database
    db.sqlite                 # User data
```

## Database Schemas

### mail.sqlite
- `fastn_emails`: Email index with metadata
- `fastn_email_attachments`: Attachment tracking
- `fastn_email_threads`: Thread management

### automerge.sqlite
- `fastn_documents`: Automerge document storage
- Stores configuration, alias metadata, etc.

### db.sqlite
- User-defined tables for application data
- No predefined schema

## Usage

```rust
// Create a new account with default alias
let account = fastn_account::Account::create(&accounts_dir).await?;

// Load existing account
let account = fastn_account::Account::load(&account_path).await?;

// Get primary alias ID52
let primary_id52 = account.primary_id52().await;

// Access all aliases
let aliases = account.aliases().await;
```

## Alias Management

Each alias consists of:
- **Public Key**: The ID52 identity visible to others
- **Secret Key**: For signing and decryption
- **Name**: Public name visible to others (stored in `/-/aliases/{id52}/readme`)
- **Reason**: Private note about why this alias exists (stored in `/-/aliases/{id52}/notes`)
- **Primary Flag**: Indicates if this is the primary alias

## AccountManager

The `AccountManager` coordinates multiple accounts within a fastn_home:

```rust
// First time setup
let (manager, primary_id52) = AccountManager::create(fastn_home).await?;

// Load existing
let manager = AccountManager::load(fastn_home).await?;

// Get all endpoints from all accounts
let endpoints = manager.get_all_endpoints().await?;
```

## Security

- Private keys are stored in the system keyring by default
- Set `SKIP_KEYRING=true` to store keys in files (less secure)
- Each alias has independent cryptographic identity
- Keys are Ed25519 for signing and X25519 for encryption

## Integration

This crate integrates with:
- `fastn-rig`: The coordination layer that manages accounts
- `fastn-automerge`: CRDT document storage
- `fastn-id52`: Cryptographic identity management
- System keyring for secure key storage