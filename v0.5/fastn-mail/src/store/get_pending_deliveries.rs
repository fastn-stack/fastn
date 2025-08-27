//! # Get Pending Deliveries

use crate::errors::*;
use crate::PendingDelivery;

impl crate::Store {
    /// Called by periodic task to check outbound queue
    pub async fn get_pending_deliveries(
        &self,
    ) -> Result<Vec<PendingDelivery>, GetPendingDeliveriesError> {
        let conn = self.connection().lock().await;

        // Query delivery table for queued emails grouped by recipient
        let mut stmt = conn
            .prepare(
                "SELECT recipient_id52, COUNT(*) as email_count, 
                        COALESCE(MIN(last_attempt), 0) as oldest_date
             FROM fastn_email_delivery 
             WHERE delivery_status = 'queued' OR (delivery_status = 'failed' AND next_retry <= ?)
             GROUP BY recipient_id52",
            )
            .map_err(|e| GetPendingDeliveriesError::DatabaseQueryFailed { source: e })?;

        let now = chrono::Utc::now().timestamp();
        let rows = stmt
            .query_map([now], |row| {
                let peer_id52_str: String = row.get(0)?;
                let peer_id52 = std::str::FromStr::from_str(&peer_id52_str).map_err(|_| {
                    rusqlite::Error::InvalidColumnType(
                        0,
                        "peer_id52".to_string(),
                        rusqlite::types::Type::Text,
                    )
                })?;

                Ok(PendingDelivery {
                    peer_id52,
                    email_count: row.get(1)?,
                    oldest_email_date: row.get(2)?,
                })
            })
            .map_err(|e| GetPendingDeliveriesError::DatabaseQueryFailed { source: e })?;

        let mut deliveries = Vec::new();
        for row in rows {
            deliveries.push(
                row.map_err(|e| GetPendingDeliveriesError::DatabaseQueryFailed { source: e })?,
            );
        }

        Ok(deliveries)
    }
}