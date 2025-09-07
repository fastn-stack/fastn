//! Email delivery poller using fastn-p2p

/// Start email delivery poller using fastn-p2p
pub async fn start_email_delivery_poller(
    account_manager: std::sync::Arc<fastn_account::AccountManager>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    eprintln!("🔧 DEBUG POLLER: email_delivery_poller STARTING with fastn-p2p");
    eprintln!("🔧 DEBUG POLLER: account_manager has {} accounts", 
        account_manager.get_all_endpoints().await.map(|v| v.len()).unwrap_or(0));
    
    let mut tick_count = 0;
    loop {
        tokio::select! {
            _ = fastn_p2p::cancelled() => {
                println!("📬 Email delivery poller shutting down...");
                return Ok(());
            }
            _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
                tick_count += 1;
                eprintln!("🔧 DEBUG POLLER: TICK #{tick_count} - starting scan");
                
                // Scan for pending emails and deliver using fastn-p2p
                if let Err(e) = scan_and_deliver_emails(&account_manager).await {
                    eprintln!("❌ Email delivery scan failed: {e}");
                } else {
                    println!("✅ Email delivery scan #{tick_count} completed");
                }
            }
        }
    }
}

/// Scan for pending emails and deliver them using fastn-p2p
async fn scan_and_deliver_emails(
    account_manager: &fastn_account::AccountManager,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    eprintln!("🔧 DEBUG SCAN: scan_and_deliver_emails() CALLED");
    
    // Get all accounts to check for pending emails
    eprintln!("🔧 DEBUG SCAN: About to call account_manager.get_all_endpoints()");
    let all_endpoints = match account_manager.get_all_endpoints().await {
        Ok(endpoints) => {
            println!("🔧 DEBUG: get_all_endpoints() SUCCESS - found {} endpoints", endpoints.len());
            endpoints
        }
        Err(e) => {
            println!("❌ DEBUG: get_all_endpoints() FAILED: {e}");
            return Err(Box::new(e));
        }
    };
    
    for (endpoint_id52, _secret_key, account_path) in all_endpoints {
        println!("🔧 DEBUG: Checking account {} at path {}", endpoint_id52, account_path.display());
        
        // Load the mail store directly for now
        println!("🔧 DEBUG: Loading mail store from {}", account_path.display());
        let mail_store = match fastn_mail::Store::load(&account_path).await {
            Ok(store) => {
                println!("🔧 DEBUG: Mail store load SUCCESS for {endpoint_id52}");
                store
            }
            Err(e) => {
                println!("❌ DEBUG: Mail store load FAILED for {endpoint_id52}: {e}");
                continue; // Skip this account and try next
            }
        };
        
        // Get pending P2P deliveries using the mail store's API
        println!("📭 Scanning account {endpoint_id52} for pending P2P deliveries");
        
        // Get pending deliveries from the fastn_email_delivery table
        let pending_deliveries = match mail_store.get_pending_deliveries().await {
            Ok(deliveries) => {
                println!("📧 Found {} pending P2P deliveries", deliveries.len());
                deliveries
            }
            Err(e) => {
                println!("❌ Failed to get pending deliveries: {e}");
                continue;
            }
        };
        
        // Process each pending delivery
        for delivery in pending_deliveries {
            println!("📤 Processing delivery to peer: {}", delivery.peer_id52.id52());
            
            // Get emails for this peer
            let emails = match mail_store.get_emails_for_peer(&delivery.peer_id52).await {
                Ok(emails) => {
                    println!("📧 Found {} emails for peer {}", emails.len(), delivery.peer_id52.id52());
                    emails
                }
                Err(e) => {
                    println!("❌ Failed to get emails for peer: {e}");
                    continue;
                }
            };
            
            // Actually deliver via P2P
            println!("🚀 Starting P2P delivery of {} emails to peer {}", emails.len(), delivery.peer_id52.id52());
            
            match crate::email_delivery_p2p::deliver_emails_to_peer_v2(
                &emails, 
                &_secret_key.public_key(), 
                &delivery.peer_id52, 
                account_manager
            ).await {
                Ok(delivered_ids) => {
                    println!("✅ Successfully delivered {} emails via P2P", delivered_ids.len());
                    
                    // Mark delivered emails as completed in the database
                    for email_id in delivered_ids {
                        if let Err(e) = mail_store.mark_delivered_to_peer(&email_id, &delivery.peer_id52).await {
                            println!("❌ Failed to mark email {email_id} as delivered: {e}");
                        } else {
                            println!("✅ Marked email {email_id} as delivered in database");
                        }
                    }
                }
                Err(e) => {
                    println!("❌ P2P delivery failed: {e}");
                    // Emails remain in pending state for retry
                }
            }
        }
        
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