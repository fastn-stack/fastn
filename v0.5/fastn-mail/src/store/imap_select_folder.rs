//! # IMAP Select Folder

use fastn_mail::errors::*;
use fastn_mail::{Flag, FolderInfo};

impl fastn_mail::Store {
    /// Select folder and return folder information
    pub async fn imap_select_folder(
        &self,
        folder: &str,
    ) -> Result<FolderInfo, ImapSelectFolderError> {
        let conn = self.connection().lock().await;

        // Check if folder exists
        let folder_path = self.account_path().join("mails/default").join(folder);
        if !folder_path.exists() {
            return Err(ImapSelectFolderError::FolderNotFound {
                folder: folder.to_string(),
            });
        }

        // Query folder statistics
        let exists: u32 = conn
            .query_row(
                "SELECT COUNT(*) FROM fastn_emails WHERE folder = ? AND is_deleted = 0",
                [folder],
                |row| row.get(0),
            )
            .map_err(|e| ImapSelectFolderError::DatabaseQueryFailed { source: e })?;

        let recent: u32 = conn.query_row(
            "SELECT COUNT(*) FROM fastn_emails WHERE folder = ? AND is_deleted = 0 AND date_received > ?",
            rusqlite::params![folder, chrono::Utc::now().timestamp() - 86400], // Last 24 hours
            |row| row.get(0)
        ).unwrap_or(0);

        let unseen = conn.query_row(
            "SELECT COUNT(*) FROM fastn_emails WHERE folder = ? AND is_deleted = 0 AND is_seen = 0",
            [folder], |row| row.get(0)
        ).ok();

        Ok(FolderInfo {
            flags: vec![
                Flag::Seen,
                Flag::Answered,
                Flag::Flagged,
                Flag::Deleted,
                Flag::Draft,
            ],
            exists,
            recent,
            unseen,
            permanent_flags: vec![
                Flag::Seen,
                Flag::Answered,
                Flag::Flagged,
                Flag::Deleted,
                Flag::Draft,
            ],
            uid_next: Some(exists + 1),
            uid_validity: Some(1), // TODO: Implement proper UID validity
        })
    }
}
