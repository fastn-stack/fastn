//! # Get Emails for Peer

use crate::errors::*;
use crate::EmailForDelivery;

impl crate::Store {
    /// Called when peer contacts us requesting their emails
    pub async fn get_emails_for_peer(
        &self,
        peer_id52: &fastn_id52::PublicKey,
    ) -> Result<Vec<EmailForDelivery>, GetEmailsForPeerError> {
        let conn = self.connection().lock().await;
        let peer_id52_str = peer_id52.id52();

        // Get emails queued for this peer
        let mut stmt = conn
            .prepare(
                "SELECT e.email_id, e.file_path, e.size_bytes, d.last_attempt
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
                ))
            })
            .map_err(|e| GetEmailsForPeerError::DatabaseQueryFailed { source: e })?;

        let mut emails = Vec::new();
        for row in rows {
            let (email_id, file_path, size_bytes, last_attempt) =
                row.map_err(|e| GetEmailsForPeerError::DatabaseQueryFailed { source: e })?;

            // Read the email file
            let full_path = self.account_path().join(&file_path);
            let raw_message =
                std::fs::read(&full_path).map_err(|e| GetEmailsForPeerError::FileReadFailed {
                    path: full_path,
                    source: e,
                })?;

            emails.push(EmailForDelivery {
                email_id,
                raw_message,
                size_bytes,
                date_queued: last_attempt.unwrap_or(0),
            });
        }

        Ok(emails)
    }
}