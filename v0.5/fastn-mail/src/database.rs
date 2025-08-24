//! Mail database schema and operations

use crate::errors::*;

/// Create and connect to a mail database
pub fn create_connection(
    mail_path: &std::path::Path,
) -> Result<rusqlite::Connection, MailDatabaseConnectionError> {
    rusqlite::Connection::open(mail_path).map_err(|e| {
        MailDatabaseConnectionError::ConnectionFailed {
            path: mail_path.to_path_buf(),
            source: e,
        }
    })
}

/// Check if mail database exists
pub fn database_exists(mail_path: &std::path::Path) -> bool {
    mail_path.exists()
}

/// Run migrations for mail database
pub fn migrate_database(conn: &rusqlite::Connection) -> Result<(), MigrateMailDatabaseError> {
    conn.execute_batch(
        r#"
        -- Email index
        CREATE TABLE IF NOT EXISTS fastn_emails (
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
            email_references  TEXT,
            date_sent         INTEGER,
            date_received     INTEGER,
            is_read           INTEGER DEFAULT 0,
            is_starred        INTEGER DEFAULT 0,
            flags             TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_folder ON fastn_emails(folder);
        CREATE INDEX IF NOT EXISTS idx_date ON fastn_emails(date_received DESC);
        CREATE INDEX IF NOT EXISTS idx_message_id ON fastn_emails(message_id);

        -- Email peers
        CREATE TABLE IF NOT EXISTS fastn_email_peers (
            peer_alias        TEXT PRIMARY KEY,
            last_seen         INTEGER,
            endpoint          BLOB,
            our_alias_used    TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_our_alias ON fastn_email_peers(our_alias_used);
        "#,
    )
    .map_err(|e| MigrateMailDatabaseError::SchemaInitializationFailed { source: e })?;

    Ok(())
}

/// Create mail directory structure for an account
pub fn create_mail_directories(account_path: &std::path::Path) -> Result<(), std::io::Error> {
    std::fs::create_dir_all(account_path.join("mails/default/inbox"))?;
    std::fs::create_dir_all(account_path.join("mails/default/sent"))?;
    std::fs::create_dir_all(account_path.join("mails/default/drafts"))?;
    std::fs::create_dir_all(account_path.join("mails/default/trash"))?;
    Ok(())
}
