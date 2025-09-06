//! Email delivery using fastn-p2p for type-safe, clean P2P communication

use crate::protocols::RigProtocol;

/// Deliver emails to a peer using fastn-p2p
/// 
/// This is the new implementation using the locked-down fastn-p2p API
/// instead of the low-level fastn-net get_stream approach.
pub async fn deliver_emails_to_peer_v2(
    our_secret_key: fastn_id52::SecretKey,
    peer_id52: &fastn_id52::PublicKey,
    emails: Vec<fastn_rig::PendingEmail>,
) -> Result<Vec<fastn_account::EmailDeliveryResponse>, fastn_rig::EmailDeliveryError> {
    println!(
        "🔗 DEBUG: About to deliver {} emails via fastn-p2p to {}",
        emails.len(),
        peer_id52.id52()
    );

    let mut responses = Vec::new();

    // Send each email using the clean fastn-p2p API
    for email in emails {
        println!(
            "📧 DEBUG: Processing email {} for P2P delivery",
            email.email_id
        );

        // Create the request message (same structure as before)
        let request = fastn_account::AccountToAccountMessage::Email {
            raw_message: email.raw_message.clone(),
            envelope_from: email.envelope_from.clone(),
            envelope_to: email.envelope_to.clone(),
        };

        // Use the new type-safe fastn-p2p call instead of manual stream handling
        let result: Result<fastn_account::EmailDeliveryResponse, fastn_rig::EmailDeliveryError> = 
            tokio::time::timeout(
                std::time::Duration::from_secs(30),
                fastn_p2p::call(
                    our_secret_key.clone(),
                    peer_id52,
                    RigProtocol::EmailDelivery,
                    request,
                )
            ).await
            .map_err(|_| fastn_rig::EmailDeliveryError::PendingDeliveriesQueryFailed {
                source: fastn_mail::GetPendingDeliveriesError::DatabaseQueryFailed {
                    source: rusqlite::Error::InvalidColumnName("P2P call timeout".to_string()),
                },
            })?
            .map_err(|e| fastn_rig::EmailDeliveryError::PendingDeliveriesQueryFailed {
                source: fastn_mail::GetPendingDeliveriesError::DatabaseQueryFailed {
                    source: rusqlite::Error::InvalidColumnName(format!("P2P call failed: {e}")),
                },
            })?;

        match result {
            Ok(response) => {
                println!("✅ DEBUG: Email {} delivered successfully", email.email_id);
                responses.push(response);
            }
            Err(delivery_error) => {
                println!("❌ DEBUG: Email {} delivery failed", email.email_id);
                // Create error response
                responses.push(fastn_account::EmailDeliveryResponse {
                    email_id: email.email_id.clone(),
                    status: fastn_account::DeliveryStatus::Rejected {
                        reason: format!("Delivery failed: {delivery_error:?}"),
                    },
                });
            }
        }
    }

    println!("🎯 DEBUG: Completed delivery of {} emails via fastn-p2p", responses.len());
    Ok(responses)
}