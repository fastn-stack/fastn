//! # Get Emails for Peer

use crate::EmailForDelivery;
use crate::errors::*;

impl crate::Store {
    /// Called when peer contacts us requesting their emails
    pub async fn get_emails_for_peer(
        &self,
        peer_id52: &fastn_id52::PublicKey,
    ) -> Result<Vec<EmailForDelivery>, GetEmailsForPeerError> {
        let conn = self.connection().lock().await;
        let peer_id52_str = peer_id52.id52();

        // Get emails queued for this peer with envelope data
        let mut stmt = conn
            .prepare(
                "SELECT e.email_id, e.file_path, e.size_bytes, d.last_attempt, e.from_addr, d.recipient_id52
             FROM fastn_emails e
             JOIN fastn_email_delivery d ON e.email_id = d.email_id
             WHERE d.recipient_id52 = ? AND d.delivery_status IN ('queued', 'failed')
             ORDER BY e.date_received ASC",
            )
            .map_err(|e| GetEmailsForPeerError::DatabaseQueryFailed { source: e })?;

        let rows = stmt
            .query_map([&peer_id52_str], |row| {
                Ok((
                    row.get::<_, String>(0)?,      // email_id
                    row.get::<_, String>(1)?,      // file_path
                    row.get::<_, usize>(2)?,       // size_bytes
                    row.get::<_, Option<i64>>(3)?, // last_attempt
                    row.get::<_, String>(4)?,      // from_addr
                    row.get::<_, String>(5)?,      // recipient_id52
                ))
            })
            .map_err(|e| GetEmailsForPeerError::DatabaseQueryFailed { source: e })?;

        let mut emails = Vec::new();
        for row in rows {
            let (email_id, file_path, size_bytes, last_attempt, from_addr, recipient_id52) =
                row.map_err(|e| GetEmailsForPeerError::DatabaseQueryFailed { source: e })?;

            // Read the email file
            let full_path = self.account_path().join(&file_path);
            println!("📂 DEBUG: P2P delivery reading email from: {}", full_path.display());
            println!("📂 DEBUG: Account path: {}", self.account_path().display());
            println!("📂 DEBUG: Relative file path: {}", file_path);
            let raw_message =
                std::fs::read(&full_path).map_err(|e| GetEmailsForPeerError::FileReadFailed {
                    path: full_path.clone(),
                    source: e,
                })?;

            // Construct envelope_to from recipient_id52 (we need to find the actual email address)
            // For P2P, envelope_to should be something like "username@{recipient_id52}.domain"
            // But we don't store the full original recipient address, only the ID52
            // For now, use a placeholder format - this could be improved
            let envelope_to = format!("user@{}.local", recipient_id52);

            emails.push(EmailForDelivery {
                email_id,
                raw_message,
                size_bytes,
                date_queued: last_attempt.unwrap_or(0),
                envelope_from: from_addr,
                envelope_to,
            });
        }

        Ok(emails)
    }
}
