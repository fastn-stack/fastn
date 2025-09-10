//! # IMAP Search

use fastn_mail::errors::*;

impl fastn_mail::Store {
    /// Search for emails matching criteria
    pub async fn imap_search(
        &self,
        folder: &str,
        criteria: &str,
    ) -> Result<Vec<u32>, ImapSearchError> {
        let conn = self.connection().lock().await;

        // Basic search implementation - TODO: Parse IMAP search syntax properly
        let sql = match criteria {
            "ALL" => "SELECT rowid FROM fastn_emails WHERE folder = ? AND is_deleted = 0",
            "UNSEEN" => {
                "SELECT rowid FROM fastn_emails WHERE folder = ? AND is_deleted = 0 AND is_seen = 0"
            }
            _ => {
                return Err(ImapSearchError::DatabaseQueryFailed {
                    source: rusqlite::Error::InvalidColumnName(format!(
                        "Unsupported search criteria: {criteria}"
                    )),
                });
            }
        };

        let mut stmt = conn
            .prepare(sql)
            .map_err(|e| ImapSearchError::DatabaseQueryFailed { source: e })?;

        let rows = stmt
            .query_map([folder], |row| row.get::<_, u32>(0))
            .map_err(|e| ImapSearchError::DatabaseQueryFailed { source: e })?;

        let mut uids = Vec::new();
        for row in rows {
            uids.push(row.map_err(|e| ImapSearchError::DatabaseQueryFailed { source: e })?);
        }

        Ok(uids)
    }
}
