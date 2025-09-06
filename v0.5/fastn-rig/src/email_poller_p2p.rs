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
    
    for (endpoint_id52, secret_key, account_path) in all_endpoints {
        println!("📧 Checking account {} for pending emails", endpoint_id52);
        
        // Load the account to access its mail system
        let account = fastn_account::Account::load(&account_path).await?;
        
        // TODO: Use proper public API to get pending emails
        // For now, just log that we're checking (the old system was broken anyway)  
        println!("📭 Email poller tick - would scan {} for pending emails", endpoint_id52);
        if false { // Disabled until proper API is found
            let pending_emails: Vec<fastn_mail::EmailForDelivery> = vec![];
            if !pending_emails.is_empty() {
                println!("📤 Found {} pending emails in {}", pending_emails.len(), endpoint_id52);
                
                // Group emails by target peer for efficient delivery
                let mut emails_by_peer: std::collections::HashMap<String, Vec<_>> = std::collections::HashMap::new();
                
                for email in pending_emails {
                    // Extract peer ID from envelope_to field (format: user@peer_id52.com)
                    if let Some(domain) = email.envelope_to.split('@').nth(1) {
                        let peer_id52 = domain.replace(".com", "");
                        emails_by_peer.entry(peer_id52).or_default().push(email);
                    }
                }
                
                // Deliver to each peer using fastn-p2p
                for (peer_id52, emails) in emails_by_peer {
                    if let Ok(peer_key) = peer_id52.parse::<fastn_id52::PublicKey>() {
                        println!("📤 Delivering {} emails to peer {}", emails.len(), peer_id52);
                        
                        match crate::email_delivery_p2p::deliver_emails_to_peer_v2(
                            &emails,
                            &secret_key.public_key(),
                            &peer_key,
                            account_manager,
                        ).await {
                            Ok(delivered_ids) => {
                                println!("✅ Delivered {} emails via fastn-p2p", delivered_ids.len());
                            }
                            Err(e) => {
                                eprintln!("❌ Failed to deliver emails to {}: {}", peer_id52, e);
                            }
                        }
                    }
                }
            }
        }
    }
    
    Ok(())
}