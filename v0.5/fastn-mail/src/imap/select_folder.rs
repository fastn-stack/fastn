//! # IMAP Select Folder

use fastn_mail::errors::*;
use fastn_mail::{Flag, FolderInfo};

/// Count .eml files recursively in a directory (handles date subdirectories)
fn count_eml_files_recursive(dir: &std::path::Path) -> u32 {
    fn count_recursive(dir: &std::path::Path) -> u32 {
        if !dir.exists() {
            return 0;
        }

        let mut count = 0;
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    // Recursively count in subdirectories
                    count += count_recursive(&path);
                } else if path.extension().and_then(|s| s.to_str()) == Some("eml") {
                    count += 1;
                }
            }
        }
        count
    }

    count_recursive(dir)
}

impl fastn_mail::Store {
    /// Select folder and return folder information (always fresh read)
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

        // Query folder statistics (force fresh read)
        let exists: u32 = conn
            .query_row(
                "SELECT COUNT(*) FROM fastn_emails WHERE folder = ? AND is_deleted = 0",
                [folder],
                |row| row.get(0),
            )
            .map_err(|e| ImapSelectFolderError::DatabaseQueryFailed { source: e })?;

        // Debug: Show what database returns vs filesystem reality (recursive count)
        let filesystem_count = if folder_path.exists() {
            count_eml_files_recursive(&folder_path)
        } else {
            0
        };

        println!(
            "📊 Debug: Database count: {}, Filesystem count: {} for folder: {}",
            exists, filesystem_count, folder
        );

        // Use filesystem count if different from database (database might be stale)
        let exists = if filesystem_count != exists {
            println!(
                "⚠️ Database/filesystem mismatch - using filesystem count: {}",
                filesystem_count
            );
            filesystem_count
        } else {
            exists
        };

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
