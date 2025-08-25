//! Database operations for email storage with updated schema

/// Create the complete mail database schema with alias mapping
pub fn create_schema(conn: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        r#"
        -- Main email storage table with hybrid design
        CREATE TABLE IF NOT EXISTS fastn_emails (
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
            our_alias_used   TEXT,                    -- Which of our aliases was used
            our_username     TEXT,                    -- Our username part
            their_alias      TEXT,                    -- Other party's alias
            their_username   TEXT,                    -- Other party's username
            
            -- Threading Support (RFC 5322)
            in_reply_to      TEXT,                    -- In-Reply-To header
            email_references TEXT,                    -- References header (space-separated)
            
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
        CREATE INDEX IF NOT EXISTS idx_folder ON fastn_emails(folder);
        CREATE INDEX IF NOT EXISTS idx_date_received ON fastn_emails(date_received DESC);
        CREATE INDEX IF NOT EXISTS idx_date_sent ON fastn_emails(date_sent DESC);
        CREATE INDEX IF NOT EXISTS idx_message_id ON fastn_emails(message_id);
        CREATE INDEX IF NOT EXISTS idx_thread ON fastn_emails(in_reply_to, email_references);
        CREATE INDEX IF NOT EXISTS idx_from ON fastn_emails(from_addr);
        CREATE INDEX IF NOT EXISTS idx_subject ON fastn_emails(subject);

        -- Indexes for P2P routing and delivery
        CREATE INDEX IF NOT EXISTS idx_our_alias ON fastn_emails(our_alias_used);
        CREATE INDEX IF NOT EXISTS idx_their_alias ON fastn_emails(their_alias);
        CREATE INDEX IF NOT EXISTS idx_alias_pair ON fastn_emails(our_alias_used, their_alias);

        -- Email peer tracking
        CREATE TABLE IF NOT EXISTS fastn_email_peers (
            peer_alias       TEXT PRIMARY KEY,       -- Peer's alias ID52
            last_seen        INTEGER,                 -- Last interaction timestamp
            our_alias_used   TEXT NOT NULL            -- Which of our aliases they know
        );

        CREATE INDEX IF NOT EXISTS idx_peer_our_alias ON fastn_email_peers(our_alias_used);

        -- Delivery status tracking for P2P
        CREATE TABLE IF NOT EXISTS fastn_email_delivery (
            email_id        TEXT NOT NULL,            -- References fastn_emails.email_id
            recipient_id52  TEXT NOT NULL,            -- Target peer ID52
            delivery_status TEXT NOT NULL,            -- queued, delivered, failed
            attempts        INTEGER DEFAULT 0,        -- Delivery attempt count
            last_attempt    INTEGER,                  -- Last delivery attempt timestamp
            next_retry      INTEGER,                  -- When to retry delivery
            error_message   TEXT,                     -- Last delivery error (if any)
            
            PRIMARY KEY (email_id, recipient_id52),
            FOREIGN KEY (email_id) REFERENCES fastn_emails(email_id)
        );

        CREATE INDEX IF NOT EXISTS idx_delivery_status ON fastn_email_delivery(delivery_status);
        CREATE INDEX IF NOT EXISTS idx_next_retry ON fastn_email_delivery(next_retry);
        "#,
    )?;

    Ok(())
}

/// Create mail directory structure for an account
pub fn create_directories(account_path: &std::path::Path) -> Result<(), std::io::Error> {
    // Create standard IMAP folders
    std::fs::create_dir_all(account_path.join("mails/default/INBOX"))?;
    std::fs::create_dir_all(account_path.join("mails/default/Sent"))?;
    std::fs::create_dir_all(account_path.join("mails/default/Drafts"))?;
    std::fs::create_dir_all(account_path.join("mails/default/Trash"))?;
    Ok(())
}