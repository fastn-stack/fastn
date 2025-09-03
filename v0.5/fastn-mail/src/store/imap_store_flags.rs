//! # IMAP Store Flags

use fastn_mail::Flag;
use fastn_mail::errors::*;

impl fastn_mail::Store {
    /// Store flags for email messages
    pub async fn imap_store_flags(
        &self,
        folder: &str,
        uid: u32,
        flags: &[Flag],
        replace: bool,
    ) -> Result<(), ImapStoreFlagsError> {
        let conn = self.connection().lock().await;

        // Convert flags to database columns
        let mut seen = false;
        let mut flagged = false;
        let mut draft = false;
        let mut answered = false;
        let mut deleted = false;

        for flag in flags {
            match flag {
                Flag::Seen => seen = true,
                Flag::Flagged => flagged = true,
                Flag::Draft => draft = true,
                Flag::Answered => answered = true,
                Flag::Deleted => deleted = true,
                _ => {} // Custom flags ignored for now
            }
        }

        let sql = if replace {
            "UPDATE fastn_emails SET is_seen = ?, is_flagged = ?, is_draft = ?, is_answered = ?, is_deleted = ? 
             WHERE folder = ? AND rowid = ?"
        } else {
            // TODO: Implement flag addition (not replacement)
            "UPDATE fastn_emails SET is_seen = ?, is_flagged = ?, is_draft = ?, is_answered = ?, is_deleted = ? 
             WHERE folder = ? AND rowid = ?"
        };

        let updated = conn
            .execute(
                sql,
                rusqlite::params![seen, flagged, draft, answered, deleted, folder, uid],
            )
            .map_err(|e| ImapStoreFlagsError::DatabaseUpdateFailed { source: e })?;

        if updated == 0 {
            return Err(ImapStoreFlagsError::EmailNotFound { uid });
        }

        Ok(())
    }
}
