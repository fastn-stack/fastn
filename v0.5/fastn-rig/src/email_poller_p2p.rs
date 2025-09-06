//! Email delivery poller using fastn-p2p

/// Start email delivery poller using fastn-p2p
pub async fn start_email_delivery_poller(
    account_manager: std::sync::Arc<fastn_account::AccountManager>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("📬 Starting email delivery poller with fastn-p2p...");
    
    loop {
        tokio::select! {
            _ = fastn_p2p::cancelled() => {
                println!("📬 Email delivery poller shutting down...");
                return Ok(());
            }
            _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
                // Scan for pending emails and deliver using fastn-p2p
                if let Err(e) = scan_and_deliver_emails(&account_manager).await {
                    eprintln!("❌ Email delivery scan failed: {}", e);
                } else {
                    println!("✅ Email delivery scan completed");
                }
            }
        }
    }
}

/// Scan for pending emails and deliver them using fastn-p2p
async fn scan_and_deliver_emails(
    account_manager: &fastn_account::AccountManager,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("📭 Scanning for pending emails...");
    
    // Get all accounts to check for pending emails
    let all_endpoints = account_manager.get_all_endpoints().await?;
    
    for (endpoint_id52, _secret_key, account_path) in all_endpoints {
        println!("📧 Checking account {} for pending emails", endpoint_id52);
        
        // Load the account to access its mail system
        let _account = fastn_account::Account::load(&account_path).await?;
        
        // Get pending emails using mail system API
        println!("📭 Scanning account {} for pending emails", endpoint_id52);
        
        // TODO: Need public API to access mail.store - currently private field
        // The old system was broken, so this is a net improvement even incomplete
        println!("📭 Email scanning not implemented yet - need public mail API");
        
        // This is the structure we need once Account provides public mail access:
        /*
        let pending_deliveries = account.get_pending_deliveries_public_api().await?;
        if pending_deliveries.is_empty() {
            return Ok(()); // Early return - no emails
        }
        
        println!("📤 Found {} peer deliveries pending in {}", pending_deliveries.len(), endpoint_id52);
        
        for delivery in pending_deliveries {
            let peer_id52 = delivery.peer_id52; // Proper field access!
            println!("📤 Processing {} emails for peer {}", delivery.email_count, peer_id52.id52());
            
            let emails = account.get_emails_for_peer_public_api(&peer_id52).await?;
            
            match crate::email_delivery_p2p::deliver_emails_to_peer_v2(
                &emails, &secret_key.public_key(), &peer_id52, account_manager
            ).await {
                Ok(delivered_ids) => println!("✅ Delivered {} emails", delivered_ids.len()),
                Err(e) => eprintln!("❌ Delivery failed: {}", e),
            }
        }
        */
    }
    
    Ok(())
}