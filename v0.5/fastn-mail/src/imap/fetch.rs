//! # IMAP Fetch

use fastn_mail::errors::*;

impl fastn_mail::Store {
    /// Fetch email message by UID
    pub async fn imap_fetch(&self, folder: &str, uid: u32) -> Result<Vec<u8>, ImapFetchError> {
        let conn = self.connection().lock().await;

        // Get file path for the UID
        let file_path: String = conn.query_row(
            "SELECT file_path FROM fastn_emails WHERE folder = ? AND rowid = ? AND is_deleted = 0",
            rusqlite::params![folder, uid],
            |row| row.get(0)
        ).map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => ImapFetchError::EmailNotFound { uid },
            _ => ImapFetchError::DatabaseQueryFailed { source: e },
        })?;

        // Read the email file
        let full_path = self.account_path().join(&file_path);
        let raw_message =
            std::fs::read(&full_path).map_err(|e| ImapFetchError::FileReadFailed {
                path: full_path,
                source: e,
            })?;

        Ok(raw_message)
    }
}
