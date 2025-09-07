//! Email delivery using fastn-p2p for type-safe, clean P2P communication

use crate::protocols::RigProtocol;
use serde::{Deserialize, Serialize};

/// Simple email delivery response for P2P communication
#[derive(Debug, Serialize, Deserialize)]
pub struct EmailDeliveryResponse {
    pub email_id: String,
    pub status: String,
}

/// Simple email delivery error for P2P communication
#[derive(Debug, Serialize, Deserialize)]
pub struct EmailDeliveryError {
    pub message: String,
    pub code: String,
}

/// Deliver emails to a peer using fastn-p2p
///
/// This is the new implementation using the locked-down fastn-p2p API
/// instead of the low-level fastn-net get_stream approach.
pub async fn deliver_emails_to_peer_v2(
    emails: &[fastn_mail::EmailForDelivery],
    _our_alias: &fastn_id52::PublicKey,
    peer_id52: &fastn_id52::PublicKey,
    _account_manager: &fastn_account::AccountManager,
) -> Result<Vec<String>, fastn_rig::EmailDeliveryError> {
    if emails.is_empty() {
        println!("📭 DEBUG: No emails to deliver via fastn-p2p");
        return Ok(Vec::new());
    }

    println!(
        "🔗 DEBUG: About to deliver {} emails via fastn-p2p to {}",
        emails.len(),
        peer_id52.id52()
    );

    // For now, use a placeholder secret key - we'll fix the account integration later
    let our_secret_key = fastn_id52::SecretKey::generate(); // TODO: Get from account
    println!("✅ DEBUG: Using placeholder secret key for testing");

    let mut delivered_email_ids = Vec::new();

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
        println!(
            "📧 DEBUG: Calling fastn_p2p::call for email {}",
            email.email_id
        );

        let call_result = tokio::time::timeout(
            std::time::Duration::from_secs(30),
            fastn_p2p::call::<RigProtocol, _, EmailDeliveryResponse, EmailDeliveryError>(
                our_secret_key.clone(),
                peer_id52,
                RigProtocol::EmailDelivery,
                request,
            ),
        )
        .await;

        match call_result {
            Ok(Ok(Ok(_response))) => {
                println!("✅ DEBUG: Email {} delivered successfully", email.email_id);
                delivered_email_ids.push(email.email_id.clone());
            }
            Ok(Ok(Err(_delivery_error))) => {
                println!(
                    "❌ DEBUG: Email {} delivery rejected by peer",
                    email.email_id
                );
                // Skip adding to delivered list - will be retried later
            }
            Ok(Err(call_error)) => {
                println!(
                    "❌ DEBUG: Email {} fastn-p2p call failed: {}",
                    email.email_id, call_error
                );
                // Skip adding to delivered list
            }
            Err(_timeout) => {
                println!(
                    "⏰ DEBUG: Email {} delivery timed out after 30 seconds",
                    email.email_id
                );
                // Skip adding to delivered list
            }
        }
    }

    println!(
        "🎯 DEBUG: Completed delivery of {} emails via fastn-p2p",
        delivered_email_ids.len()
    );
    Ok(delivered_email_ids)
}
