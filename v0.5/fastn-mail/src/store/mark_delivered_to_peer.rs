//! # Mark Delivered to Peer

use crate::errors::*;

impl crate::Store {
    /// Mark email as delivered to peer
    pub async fn mark_delivered_to_peer(
        &self,
        email_id: &str,
        peer_id52: &fastn_id52::PublicKey,
    ) -> Result<(), MarkDeliveredError> {
        let conn = self.connection().lock().await;
        let peer_id52_str = peer_id52.id52();

        let updated = conn
            .execute(
                "UPDATE fastn_email_delivery 
             SET delivery_status = 'delivered', last_attempt = ?
             WHERE email_id = ? AND recipient_id52 = ?",
                rusqlite::params![chrono::Utc::now().timestamp(), email_id, &peer_id52_str],
            )
            .map_err(|e| MarkDeliveredError::DatabaseUpdateFailed { source: e })?;

        if updated == 0 {
            return Err(MarkDeliveredError::EmailNotFound {
                email_id: email_id.to_string(),
            });
        }

        Ok(())
    }
}