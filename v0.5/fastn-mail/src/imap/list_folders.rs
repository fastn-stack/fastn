//! # IMAP List Folders

use fastn_mail::errors::*;

impl fastn_mail::Store {
    /// List available folders
    pub async fn imap_list_folders(&self) -> Result<Vec<String>, ImapListFoldersError> {
        let mails_path = self.account_path().join("mails/default");

        let mut folders = Vec::new();
        let entries = std::fs::read_dir(&mails_path)
            .map_err(|e| ImapListFoldersError::DirectoryScanFailed { source: e })?;

        for entry in entries {
            let entry =
                entry.map_err(|e| ImapListFoldersError::DirectoryScanFailed { source: e })?;

            if entry.path().is_dir()
                && let Some(folder_name) = entry.file_name().to_str()
            {
                folders.push(folder_name.to_string());
            }
        }

        Ok(folders)
    }
}
