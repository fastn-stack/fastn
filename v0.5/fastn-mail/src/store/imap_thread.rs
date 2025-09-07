//! # IMAP Thread

use fastn_mail::ThreadTree;
use fastn_mail::errors::*;

impl fastn_mail::Store {
    /// Get thread tree for folder (IMAP THREAD extension)
    pub async fn imap_thread(
        &self,
        folder: &str,
        algorithm: &str,
    ) -> Result<Vec<ThreadTree>, ImapThreadError> {
        if algorithm != "REFERENCES" {
            return Err(ImapThreadError::DatabaseQueryFailed {
                source: rusqlite::Error::InvalidColumnName(format!(
                    "Unsupported algorithm: {algorithm}"
                )),
            });
        }

        let conn = self.connection().lock().await;

        // Basic threading by References header
        // TODO: Implement proper RFC 5256 threading algorithm
        let mut stmt = conn
            .prepare(
                "SELECT email_id, message_id, email_references 
             FROM fastn_emails 
             WHERE folder = ? AND is_deleted = 0 
             ORDER BY date_received",
            )
            .map_err(|e| ImapThreadError::DatabaseQueryFailed { source: e })?;

        let _rows = stmt
            .query_map([folder], |row| {
                Ok((
                    row.get::<_, String>(0)?,         // email_id
                    row.get::<_, String>(1)?,         // message_id
                    row.get::<_, Option<String>>(2)?, // email_references
                ))
            })
            .map_err(|e| ImapThreadError::DatabaseQueryFailed { source: e })?;

        // For now, return empty thread tree
        // TODO: Implement proper threading logic
        Ok(vec![])
    }
}
