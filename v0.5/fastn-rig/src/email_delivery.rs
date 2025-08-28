//! # Email Delivery Module  
//!
//! Handles P2P email delivery from accounts to peers.
//!
//! ## Responsibilities
//! - Poll each account for pending email deliveries
//! - Connect to peers and deliver emails via P2P
//! - Mark emails as delivered or failed with retry logic  
//! - Handle delivery errors and backoff strategies

use fastn_account::AccountManager;

/// Response from peer for individual email delivery
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct EmailDeliveryResponse {
    /// Email ID being responded to
    pub email_id: String,
    /// Delivery result
    pub status: DeliveryStatus,
}

/// Status of individual email delivery
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum DeliveryStatus {
    /// Email accepted and stored in recipient's INBOX
    Accepted,
    /// Email rejected with reason (permanent failure)
    Rejected { reason: String },
}

/// Start the email delivery poller task
///
/// This function spawns a background task that periodically:
/// 1. Checks all accounts for pending P2P email deliveries
/// 2. Attempts to deliver emails to each peer
/// 3. Updates delivery status (delivered/failed)
/// 4. Handles retry logic for failed deliveries
pub async fn start_email_delivery_poller(
    account_manager: std::sync::Arc<AccountManager>,
    graceful: fastn_net::Graceful,
    peer_stream_senders: fastn_net::PeerStreamSenders,
) -> Result<(), fastn_rig::RunError> {
    tracing::info!("📬 Starting email delivery poller...");
    
    // Spawn background task for email delivery polling
    let graceful_clone = graceful.clone();
    graceful.spawn(async move {
        email_delivery_poller_loop(account_manager, graceful_clone, peer_stream_senders).await;
    });
    
    Ok(())
}

/// Main email delivery poller loop
async fn email_delivery_poller_loop(
    account_manager: std::sync::Arc<AccountManager>,
    graceful: fastn_net::Graceful,
    peer_stream_senders: fastn_net::PeerStreamSenders,
) {
    tracing::debug!("Email delivery poller loop started");
    
    loop {
        tokio::select! {
            // Check for shutdown signal
            _ = graceful.cancelled() => {
                tracing::info!("📬 Email delivery poller shutting down");
                break;
            }
            
            // Wait before next poll cycle
            _ = tokio::time::sleep(std::time::Duration::from_secs(30)) => {
                if let Err(e) = check_and_deliver_emails(&account_manager, &peer_stream_senders, &graceful).await {
                    tracing::error!("📬 Email delivery poll failed: {e}");
                }
            }
        }
    }
    
    tracing::debug!("Email delivery poller loop ended");
}

/// Delivery task for a specific peer with correct alias pairing
#[derive(Debug, Clone)]
struct DeliveryTask {
    /// Peer ID52 to deliver to
    peer_id52: fastn_id52::PublicKey,
    /// Account path containing the emails
    account_path: std::path::PathBuf,
    /// Our alias to use for P2P connection (from From address)
    our_alias: fastn_id52::PublicKey,
    /// Number of emails pending for this peer
    email_count: usize,
}

/// Check all accounts for pending email deliveries and process them  
async fn check_and_deliver_emails(
    account_manager: &AccountManager,
    peer_stream_senders: &fastn_net::PeerStreamSenders,
    graceful: &fastn_net::Graceful,
) -> Result<(), fastn_rig::EmailDeliveryError> {
    tracing::debug!("📬 Checking all accounts for pending email deliveries");
    
    // Step 1: Collect all pending deliveries across accounts with alias tracking
    let delivery_tasks = collect_pending_deliveries(account_manager).await?;
    
    if delivery_tasks.is_empty() {
        tracing::debug!("📬 No pending deliveries across all accounts");
        return Ok(());
    }
    
    tracing::info!("📬 Found {} delivery tasks across all accounts", delivery_tasks.len());
    
    // Step 2: Process each delivery task with correct alias pairing
    for task in delivery_tasks {
        tracing::info!("📤 Processing deliveries: {} emails to {} using alias {}", 
            task.email_count, task.peer_id52, task.our_alias);
            
        match attempt_delivery_to_peer(&task, peer_stream_senders, graceful).await {
            Ok(delivered_count) => {
                tracing::info!("✅ Successfully delivered {} emails to {}", 
                    delivered_count, task.peer_id52);
            }
            Err(e) => {
                tracing::warn!("❌ Failed to deliver emails to {}: {}", task.peer_id52, e);
                // TODO: Update retry logic and backoff
            }
        }
    }
    
    Ok(())
}

/// Collect all pending deliveries across accounts into unified task list
async fn collect_pending_deliveries(
    account_manager: &AccountManager,
) -> Result<Vec<DeliveryTask>, fastn_rig::EmailDeliveryError> {
    let mut delivery_tasks = Vec::new();
    
    // Get all endpoints (each represents an account/alias pair)
    let all_endpoints = account_manager.get_all_endpoints().await
        .map_err(|e| fastn_rig::EmailDeliveryError::EndpointEnumerationFailed { source: e })?;
    
    // Group endpoints by account path to avoid checking same account multiple times
    let mut account_paths = std::collections::HashSet::new();
    
    for (alias_id52, _secret_key, account_path) in all_endpoints {
        // Only check each account once (accounts may have multiple aliases)
        if account_paths.contains(&account_path) {
            continue;
        }
        account_paths.insert(account_path.clone());
        
        match collect_account_deliveries(&account_path, &alias_id52).await {
            Ok(mut tasks) => {
                delivery_tasks.append(&mut tasks);
            }
            Err(e) => {
                tracing::warn!("📬 Failed to check account {}: {}", account_path.display(), e);
            }
        }
    }
    
    Ok(delivery_tasks)
}

/// Collect delivery tasks from a specific account with correct alias pairing
async fn collect_account_deliveries(
    account_path: &std::path::Path,
    _account_alias: &str,
) -> Result<Vec<DeliveryTask>, fastn_rig::EmailDeliveryError> {
    // Load the account's email store
    let mail_store = fastn_mail::Store::load(account_path).await
        .map_err(|e| fastn_rig::EmailDeliveryError::MailStoreLoadFailed { source: e })?;
    
    // Get pending deliveries for this account
    let pending_deliveries = mail_store.get_pending_deliveries().await
        .map_err(|e| fastn_rig::EmailDeliveryError::PendingDeliveriesQueryFailed { source: e })?;
    
    if pending_deliveries.is_empty() {
        return Ok(vec![]);
    }
    
    let mut tasks = Vec::new();
    
    // For each peer with pending deliveries, we need to determine the correct alias pairing
    for delivery in pending_deliveries {
        // Get the actual alias used in the From address for emails to this peer
        let our_alias = get_sender_alias_for_peer(&mail_store, &delivery.peer_id52).await?;
        
        let task = DeliveryTask {
            peer_id52: delivery.peer_id52,
            account_path: account_path.to_path_buf(),
            our_alias,
            email_count: delivery.email_count,
        };
        
        tracing::debug!("📤 Added delivery task: {} emails to {} using alias {}", 
            task.email_count, task.peer_id52, task.our_alias);
        
        tasks.push(task);
    }
    
    Ok(tasks)
}

/// Get the sender alias used for emails to a specific peer
/// 
/// This queries the fastn_emails table to find which of our aliases was used
/// in the From address for emails destined to this peer. This ensures we use
/// the correct alias pair for P2P connection authentication.
async fn get_sender_alias_for_peer(
    mail_store: &fastn_mail::Store,
    peer_id52: &fastn_id52::PublicKey,
) -> Result<fastn_id52::PublicKey, fastn_rig::EmailDeliveryError> {
    let conn = mail_store.connection().lock().await;
    let peer_id52_str = peer_id52.id52();
    
    // Query for the our_alias_used field from emails pending delivery to this peer
    // This tells us which alias was in the From address of emails to this peer
    let our_alias: String = conn
        .query_row(
            "SELECT e.our_alias_used 
             FROM fastn_emails e
             JOIN fastn_email_delivery d ON e.email_id = d.email_id
             WHERE d.recipient_id52 = ? AND d.delivery_status = 'queued'
             LIMIT 1",
            [&peer_id52_str],
            |row| row.get(0),
        )
        .map_err(|e| fastn_rig::EmailDeliveryError::PendingDeliveriesQueryFailed { 
            source: fastn_mail::GetPendingDeliveriesError::DatabaseQueryFailed { source: e }
        })?;
    
    // Handle case where our_alias_used might be NULL (shouldn't happen but be safe)
    if our_alias.is_empty() {
        return Err(fastn_rig::EmailDeliveryError::NoSenderAliasFound { 
            peer_id52: peer_id52_str 
        });
    }
    
    // Parse the alias string to PublicKey for type safety
    let our_alias_key: fastn_id52::PublicKey = our_alias.parse()
        .map_err(|_| fastn_rig::EmailDeliveryError::InvalidAliasFormat { 
            alias: our_alias 
        })?;
    
    Ok(our_alias_key)
}

/// Attempt to deliver all pending emails to a specific peer
async fn attempt_delivery_to_peer(
    task: &DeliveryTask,
    peer_stream_senders: &fastn_net::PeerStreamSenders, 
    graceful: &fastn_net::Graceful,
) -> Result<usize, fastn_rig::EmailDeliveryError> {
    tracing::debug!("📤 Attempting delivery to {} using alias {}", task.peer_id52, task.our_alias);
    
    // Step 1: Load mail store for this account
    let mail_store = fastn_mail::Store::load(&task.account_path).await
        .map_err(|e| fastn_rig::EmailDeliveryError::MailStoreLoadFailed { source: e })?;
    
    // Step 2: Get all emails queued for this peer
    let emails = mail_store.get_emails_for_peer(&task.peer_id52).await
        .map_err(|e| fastn_rig::EmailDeliveryError::EmailsForPeerQueryFailed { source: e })?;
    
    if emails.is_empty() {
        tracing::debug!("📬 No emails found for peer {} (may have been delivered by another process)", task.peer_id52);
        return Ok(0);
    }
    
    tracing::info!("📧 Found {} emails to deliver to {}", emails.len(), task.peer_id52);
    
    // Step 3: Deliver all emails via single P2P connection (our_alias is already PublicKey)
    let delivered_emails = deliver_emails_to_peer(&emails, &task.our_alias, &task.peer_id52, peer_stream_senders, graceful).await?;
    
    // Step 5: Mark all successfully delivered emails in database  
    let mut delivered_count = 0;
    for email_id in delivered_emails {
        mail_store.mark_delivered_to_peer(&email_id, &task.peer_id52).await
            .map_err(|e| fastn_rig::EmailDeliveryError::MarkDeliveredFailed { source: e })?;
        
        delivered_count += 1;
        tracing::debug!("✅ Marked email {} as delivered to {}", email_id, task.peer_id52);
    }
    
    Ok(delivered_count)
}

/// Deliver multiple emails to a peer via single P2P connection
async fn deliver_emails_to_peer(
    emails: &[fastn_mail::EmailForDelivery],
    our_alias: &fastn_id52::PublicKey,
    peer_id52: &fastn_id52::PublicKey,
    _peer_stream_senders: &fastn_net::PeerStreamSenders,
    _graceful: &fastn_net::Graceful,
) -> Result<Vec<String>, fastn_rig::EmailDeliveryError> {
    if emails.is_empty() {
        return Ok(vec![]);
    }
    
    tracing::info!("📧 Establishing P2P connection from {} to {} for {} emails", 
        our_alias, peer_id52, emails.len());
    
    // TODO: Get our endpoint for our_alias (need to find account that owns this alias)
    // For now, we need a way to map our_alias → iroh::Endpoint
    // This requires integration with EndpointManager to find the right endpoint
    
    // Placeholder until we implement endpoint lookup
    let our_endpoint = fastn_net::get_endpoint().await;
    
    let mut delivered_email_ids = Vec::new();
    let total_bytes: usize = emails.iter().map(|e| e.size_bytes).sum();
    
    tracing::info!("📤 Simulating P2P email delivery: {} emails ({} bytes total) from {} to {}", 
        emails.len(), total_bytes, our_alias, peer_id52);
    
    // Establish P2P stream using fastn-net infrastructure
    let (mut send, mut recv) = fastn_net::get_stream(
        our_endpoint,
        fastn_net::Protocol::AccountToAccount.into(),
        peer_id52.id52(),
        peer_stream_senders.clone(),
        graceful.clone(),
    ).await.map_err(|e| {
        fastn_rig::EmailDeliveryError::PendingDeliveriesQueryFailed {
            source: fastn_mail::GetPendingDeliveriesError::DatabaseQueryFailed {
                source: rusqlite::Error::InvalidColumnName(format!("P2P connection failed: {e}"))
            }
        }
    })?;
    
    tracing::info!("🔗 P2P stream established from {} to {}", our_alias, peer_id52);
    
    // Send each email over the established stream with request-response
    for email in emails {
        // Create AccountToAccountMessage for P2P delivery
        let p2p_message = fastn_account::AccountToAccountMessage::new_email(email.raw_message.clone());
        
        tracing::debug!("📦 Sending email {} ({} bytes) over P2P stream", 
            email.email_id, p2p_message.size());
        
        // Send email message
        serde_json::to_writer(&mut send, &p2p_message).await.map_err(|e| {
            fastn_rig::EmailDeliveryError::PendingDeliveriesQueryFailed {
                source: fastn_mail::GetPendingDeliveriesError::DatabaseQueryFailed {
                    source: rusqlite::Error::InvalidColumnName(format!("Failed to send message: {e}"))
                }
            }
        })?;
        send.write_all(b"\n").await.map_err(|e| {
            fastn_rig::EmailDeliveryError::PendingDeliveriesQueryFailed {
                source: fastn_mail::GetPendingDeliveriesError::DatabaseQueryFailed {
                    source: rusqlite::Error::InvalidColumnName(format!("Failed to send newline: {e}"))
                }
            }
        })?;
        
        // Wait for delivery response  
        let response = fastn_net::next_json::<EmailDeliveryResponse>(&mut recv).await.map_err(|e| {
            fastn_rig::EmailDeliveryError::PendingDeliveriesQueryFailed {
                source: fastn_mail::GetPendingDeliveriesError::DatabaseQueryFailed {
                    source: rusqlite::Error::InvalidColumnName(format!("Failed to receive response: {e}"))
                }
            }
        })?;
        
        // Handle individual email response
        match response.status {
            DeliveryStatus::Accepted => {
                delivered_email_ids.push(email.email_id.clone());
                tracing::debug!("✅ Email {} accepted by {}", email.email_id, peer_id52);
            }
            DeliveryStatus::Rejected { reason } => {
                tracing::warn!("❌ Email {} rejected by {}: {}", email.email_id, peer_id52, reason);
                
                // Create bounce message for rejected email
                if let Err(e) = create_bounce_message(&email.email_id, &reason, &task.account_path).await {
                    tracing::error!("Failed to create bounce message for {}: {}", email.email_id, e);
                }
                
                // Don't add to delivered list (it wasn't delivered)
            }
        }
    }
    
    tracing::info!("✅ P2P delivery completed: {} of {} emails delivered to {}", 
        delivered_email_ids.len(), emails.len(), peer_id52);
    
    Ok(delivered_email_ids)
}

/// Create bounce message for rejected email delivery
async fn create_bounce_message(
    original_email_id: &str,
    rejection_reason: &str,
    account_path: &std::path::Path,
) -> Result<(), fastn_rig::EmailDeliveryError> {
    tracing::info!("📝 Creating bounce message for rejected email {}: {}", 
        original_email_id, rejection_reason);
    
    // Load account's mail store
    let mail_store = fastn_mail::Store::load(account_path).await
        .map_err(|e| fastn_rig::EmailDeliveryError::MailStoreLoadFailed { source: e })?;
    
    // Create RFC 3464-style bounce message
    let bounce_subject = format!("Mail Delivery Failure: {}", original_email_id);
    let bounce_body = format!(
        "Your email could not be delivered to the recipient.\n\n\
        Original Email ID: {}\n\
        Failure Reason: {}\n\n\
        This is an automated message from the fastn mail delivery system.\n\
        The original message has been removed from the delivery queue.",
        original_email_id, rejection_reason
    );
    
    // Build bounce email in RFC 5322 format
    let timestamp = chrono::Utc::now().format("%a, %d %b %Y %H:%M:%S +0000");
    let bounce_message_id = format!("bounce-{}", uuid::Uuid::new_v4());
    
    let bounce_email = format!(
        "From: Mail Delivery System <mailer-daemon@system.local>\r\n\
        To: Original Sender\r\n\
        Subject: {}\r\n\
        Date: {}\r\n\
        Message-ID: <{}>\r\n\
        MIME-Version: 1.0\r\n\
        Content-Type: text/plain; charset=utf-8\r\n\
        \r\n\
        {}",
        bounce_subject, timestamp, bounce_message_id, bounce_body
    );
    
    // Store bounce message in sender's INBOX using p2p_receive_email
    // This puts the bounce in INBOX where the sender will see it
    let system_sender = fastn_id52::SecretKey::generate().public_key(); // System identity
    mail_store.p2p_receive_email(bounce_email.into_bytes(), &system_sender).await
        .map_err(|e| fastn_rig::EmailDeliveryError::PendingDeliveriesQueryFailed {
            source: fastn_mail::GetPendingDeliveriesError::DatabaseQueryFailed {
                source: rusqlite::Error::InvalidColumnName(format!("Failed to create bounce: {e}"))
            }
        })?;
    
    tracing::info!("📝 Bounce message created for {} in INBOX", original_email_id);
    Ok(())
}