//! Email delivery poller using fastn-p2p

/// Start email delivery poller using fastn-p2p
pub async fn start_email_delivery_poller(
    account_manager: std::sync::Arc<fastn_account::AccountManager>,
    graceful: fastn_p2p::Graceful,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("📬 Starting email delivery poller with fastn-p2p...");
    
    // TODO: Implement email scanning and delivery using fastn_p2p::call
    // This will replace the old complex email_delivery.rs
    
    loop {
        tokio::select! {
            _ = graceful.cancelled() => {
                println!("📬 Email delivery poller shutting down...");
                break;
            }
            _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
                // TODO: Scan for pending emails and use crate::email_delivery_p2p::deliver_emails_to_peer_v2
                println!("📭 Email delivery poller tick (TODO: implement scanning)");
            }
        }
    }
    
    Ok(())
}