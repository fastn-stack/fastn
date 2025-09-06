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
    println!("📬 Starting email delivery poller...");

    // Test the function immediately to see if it works
    println!("🧪 Testing email delivery function before spawning task...");
    match check_and_deliver_emails(&account_manager, &peer_stream_senders, &graceful).await {
        Ok(()) => println!("✅ Email delivery function test successful"),
        Err(e) => {
            println!("❌ Email delivery function test failed: {}", e);
            return Err(fastn_rig::RunError::ShutdownFailed {
                source: Box::new(e),
            });
        }
    }

    // Spawn background task for email delivery polling
    let graceful_clone = graceful.clone();
    let task_handle = graceful.spawn(async move {
        println!("🚀 Email delivery poller task spawned successfully");
        email_delivery_poller_loop(account_manager, graceful_clone, peer_stream_senders).await;
        println!("🏁 Email delivery poller task finished");
    });

    println!(
        "✅ Email delivery poller started with handle: {:?}",
        task_handle.is_finished()
    );
    Ok(())
}

/// Main email delivery poller loop
async fn email_delivery_poller_loop(
    account_manager: std::sync::Arc<AccountManager>,
    graceful: fastn_net::Graceful,
    peer_stream_senders: fastn_net::PeerStreamSenders,
) {
    println!("🚀 Email delivery poller loop started");
    tracing::info!("Email delivery poller loop started");

    // Add initial delay before first poll
    println!("⏳ Waiting 2 seconds before first poll cycle...");
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    println!("🔄 Starting first email delivery poll cycle...");

    loop {
        tokio::select! {
            // Check for shutdown signal
            _ = graceful.cancelled() => {
                println!("🛑 Email delivery poller shutting down");
                tracing::info!("📬 Email delivery poller shutting down");
                break;
            }

            // Wait before next poll cycle
            _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
                println!("⏰ Email delivery poll cycle starting ({})", chrono::Utc::now().format("%H:%M:%S"));

                // Execute poll cycle with detailed error handling
                match check_and_deliver_emails(&account_manager, &peer_stream_senders, &graceful).await {
                    Ok(()) => {
                        println!("✅ Email delivery poll cycle completed successfully");
                    }
                    Err(e) => {
                        println!("❌ Email delivery poll failed: {}", e);
                        tracing::error!("📬 Email delivery poll failed: {e}");
                        // Continue polling despite errors
                    }
                }
            }
        }
    }

    println!("🏁 Email delivery poller loop ended");
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
    tracing::info!("📬 Starting email delivery poll cycle");
    println!("🔄 Email delivery poller: checking all accounts for pending deliveries");

    // Step 1: Collect all pending deliveries across accounts with alias tracking
    let delivery_tasks = match collect_pending_deliveries(account_manager).await {
        Ok(tasks) => tasks,
        Err(e) => {
            println!("❌ Failed to collect pending deliveries: {}", e);
            tracing::error!("📬 Failed to collect pending deliveries: {}", e);
            return Ok(()); // Continue polling despite error
        }
    };

    if delivery_tasks.is_empty() {
        println!("📭 No pending deliveries found across all accounts");
        tracing::debug!("📬 No pending deliveries across all accounts");
        return Ok(());
    }

    println!(
        "📬 Found {} delivery tasks across all accounts",
        delivery_tasks.len()
    );
    tracing::info!(
        "📬 Found {} delivery tasks across all accounts",
        delivery_tasks.len()
    );

    // Step 2: Process each delivery task with correct alias pairing
    for task in delivery_tasks {
        println!(
            "📤 Processing {} emails to {} using alias {}",
            task.email_count, task.peer_id52, task.our_alias
        );
        tracing::info!(
            "📤 Processing deliveries: {} emails to {} using alias {}",
            task.email_count,
            task.peer_id52,
            task.our_alias
        );

        match attempt_delivery_to_peer(&task, peer_stream_senders, graceful, account_manager).await
        {
            Ok(delivered_count) => {
                println!(
                    "✅ Successfully delivered {} emails to {}",
                    delivered_count, task.peer_id52
                );
                tracing::info!(
                    "✅ Successfully delivered {} emails to {}",
                    delivered_count,
                    task.peer_id52
                );
            }
            Err(e) => {
                println!("❌ Failed to deliver emails to {}: {}", task.peer_id52, e);
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
    println!("🔍 Getting all endpoints from account manager...");
    let all_endpoints = account_manager
        .get_all_endpoints()
        .await
        .map_err(|e| fastn_rig::EmailDeliveryError::EndpointEnumerationFailed { source: e })?;

    println!(
        "🔍 Found {} endpoints to check for pending emails",
        all_endpoints.len()
    );

    // Group endpoints by account path to avoid checking same account multiple times
    let mut account_paths = std::collections::HashSet::new();

    for (alias_id52, _secret_key, account_path) in all_endpoints {
        println!(
            "🔍 Checking endpoint: {} at path {}",
            alias_id52,
            account_path.display()
        );

        // Only check each account once (accounts may have multiple aliases)
        if account_paths.contains(&account_path) {
            println!(
                "⏭️  Skipping duplicate account path: {}",
                account_path.display()
            );
            continue;
        }
        account_paths.insert(account_path.clone());

        println!("📂 Loading account at path: {}", account_path.display());
        match collect_account_deliveries(&account_path, &alias_id52).await {
            Ok(mut tasks) => {
                println!(
                    "✅ Found {} delivery tasks from account {}",
                    tasks.len(),
                    account_path.display()
                );
                delivery_tasks.append(&mut tasks);
            }
            Err(e) => {
                println!(
                    "❌ Failed to check account {}: {}",
                    account_path.display(),
                    e
                );
                tracing::warn!(
                    "📬 Failed to check account {}: {}",
                    account_path.display(),
                    e
                );
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
    println!(
        "📧 Loading mail store for account: {}",
        account_path.display()
    );
    let mail_store = fastn_mail::Store::load(account_path)
        .await
        .map_err(|e| fastn_rig::EmailDeliveryError::MailStoreLoadFailed { source: e })?;

    // Get pending deliveries for this account
    println!(
        "🔍 Querying pending deliveries for account: {}",
        account_path.display()
    );
    let pending_deliveries = mail_store
        .get_pending_deliveries()
        .await
        .map_err(|e| fastn_rig::EmailDeliveryError::PendingDeliveriesQueryFailed { source: e })?;

    println!(
        "📊 Found {} pending deliveries for account {}",
        pending_deliveries.len(),
        account_path.display()
    );

    if pending_deliveries.is_empty() {
        return Ok(vec![]);
    }

    let mut tasks = Vec::new();

    // For each peer with pending deliveries, we need to determine the correct alias pairing
    for delivery in pending_deliveries {
        // Get the actual alias used in the From address for emails to this peer
        let our_alias = match get_sender_alias_for_peer(&mail_store, &delivery.peer_id52).await {
            Ok(alias) => alias,
            Err(e) => {
                tracing::warn!(
                    "Failed to get sender alias for peer {}: {}",
                    delivery.peer_id52,
                    e
                );
                println!(
                    "⚠️  Skipping delivery to {} due to alias lookup failure: {}",
                    delivery.peer_id52, e
                );
                continue; // Skip this delivery and continue with others
            }
        };

        let task = DeliveryTask {
            peer_id52: delivery.peer_id52,
            account_path: account_path.to_path_buf(),
            our_alias,
            email_count: delivery.email_count,
        };

        tracing::debug!(
            "📤 Added delivery task: {} emails to {} using alias {}",
            task.email_count,
            task.peer_id52,
            task.our_alias
        );

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
        .map_err(
            |e| fastn_rig::EmailDeliveryError::PendingDeliveriesQueryFailed {
                source: fastn_mail::GetPendingDeliveriesError::DatabaseQueryFailed { source: e },
            },
        )?;

    // Handle case where our_alias_used might be NULL (shouldn't happen but be safe)
    if our_alias.is_empty() {
        return Err(fastn_rig::EmailDeliveryError::NoSenderAliasFound {
            peer_id52: peer_id52_str,
        });
    }

    // Parse the alias string to PublicKey for type safety
    let our_alias_key: fastn_id52::PublicKey = our_alias
        .parse()
        .map_err(|_| fastn_rig::EmailDeliveryError::InvalidAliasFormat { alias: our_alias })?;

    Ok(our_alias_key)
}

/// Attempt to deliver all pending emails to a specific peer
async fn attempt_delivery_to_peer(
    task: &DeliveryTask,
    peer_stream_senders: &fastn_net::PeerStreamSenders,
    graceful: &fastn_net::Graceful,
    account_manager: &AccountManager,
) -> Result<usize, fastn_rig::EmailDeliveryError> {
    tracing::debug!(
        "📤 Attempting delivery to {} using alias {}",
        task.peer_id52,
        task.our_alias
    );

    // Step 1: Load mail store for this account
    let mail_store = fastn_mail::Store::load(&task.account_path)
        .await
        .map_err(|e| fastn_rig::EmailDeliveryError::MailStoreLoadFailed { source: e })?;

    // Step 2: Get all emails queued for this peer
    println!(
        "🔍 DEBUG: About to call get_emails_for_peer for {}",
        task.peer_id52
    );
    let emails = match mail_store.get_emails_for_peer(&task.peer_id52).await {
        Ok(emails) => {
            println!(
                "✅ DEBUG: get_emails_for_peer returned {} emails",
                emails.len()
            );
            emails
        }
        Err(e) => {
            println!("❌ DEBUG: get_emails_for_peer failed: {}", e);
            return Err(fastn_rig::EmailDeliveryError::EmailsForPeerQueryFailed { source: e });
        }
    };

    if emails.is_empty() {
        tracing::debug!(
            "📬 No emails found for peer {} (may have been delivered by another process)",
            task.peer_id52
        );
        return Ok(0);
    }

    tracing::info!(
        "📧 Found {} emails to deliver to {}",
        emails.len(),
        task.peer_id52
    );

    // Step 3: Deliver all emails via single P2P connection (our_alias is already PublicKey)
    println!(
        "🚀 DEBUG: About to deliver {} emails to peer {}",
        emails.len(),
        task.peer_id52
    );
    let delivered_emails = match deliver_emails_to_peer(
        &emails,
        &task.our_alias,
        &task.peer_id52,
        peer_stream_senders,
        graceful,
        &task.account_path,
        account_manager,
    )
    .await
    {
        Ok(emails) => {
            println!(
                "✅ DEBUG: deliver_emails_to_peer succeeded with {} delivered emails",
                emails.len()
            );
            emails
        }
        Err(e) => {
            println!("❌ DEBUG: deliver_emails_to_peer failed: {}", e);
            return Err(e);
        }
    };

    // Step 5: Mark all successfully delivered emails in database
    let mut delivered_count = 0;
    for email_id in delivered_emails {
        mail_store
            .mark_delivered_to_peer(&email_id, &task.peer_id52)
            .await
            .map_err(|e| fastn_rig::EmailDeliveryError::MarkDeliveredFailed { source: e })?;

        delivered_count += 1;
        tracing::debug!(
            "✅ Marked email {} as delivered to {}",
            email_id,
            task.peer_id52
        );
    }

    Ok(delivered_count)
}

/// Deliver multiple emails to a peer via single P2P connection
async fn deliver_emails_to_peer(
    emails: &[fastn_mail::EmailForDelivery],
    our_alias: &fastn_id52::PublicKey,
    peer_id52: &fastn_id52::PublicKey,
    peer_stream_senders: &fastn_net::PeerStreamSenders,
    graceful: &fastn_net::Graceful,
    _account_path: &std::path::Path,
    account_manager: &AccountManager,
) -> Result<Vec<String>, fastn_rig::EmailDeliveryError> {
    if emails.is_empty() {
        return Ok(vec![]);
    }

    tracing::info!(
        "📧 Establishing P2P connection from {} to {} for {} emails",
        our_alias,
        peer_id52,
        emails.len()
    );

    // Get the account that owns our_alias and extract its secret key
    let account = match account_manager.find_account_by_alias(our_alias).await {
        Ok(account) => {
            println!("✅ DEBUG: Found account for alias {}", our_alias.id52());
            account
        }
        Err(e) => {
            println!(
                "❌ DEBUG: Failed to find account for alias {}: {}",
                our_alias.id52(),
                e
            );
            return Err(fastn_rig::EmailDeliveryError::NoSenderAliasFound {
                peer_id52: our_alias.id52(),
            });
        }
    };

    // Get the secret key for our_alias from the account
    let our_secret_key = match get_secret_key_for_alias(&account, our_alias).await {
        Ok(key) => {
            println!("✅ DEBUG: Got secret key for alias {}", our_alias.id52());
            key
        }
        Err(e) => {
            println!(
                "❌ DEBUG: Failed to get secret key for alias {}: {}",
                our_alias.id52(),
                e
            );
            return Err(fastn_rig::EmailDeliveryError::NoSenderAliasFound {
                peer_id52: our_alias.id52(),
            });
        }
    };

    // Create endpoint using the real secret key for our_alias
    let our_endpoint = match fastn_net::get_endpoint(our_secret_key).await {
        Ok(endpoint) => {
            println!(
                "✅ DEBUG: Created P2P endpoint for alias {}",
                our_alias.id52()
            );
            endpoint
        }
        Err(e) => {
            println!(
                "❌ DEBUG: Failed to create P2P endpoint for alias {}: {}",
                our_alias.id52(),
                e
            );
            return Err(
                fastn_rig::EmailDeliveryError::PendingDeliveriesQueryFailed {
                    source: fastn_mail::GetPendingDeliveriesError::DatabaseQueryFailed {
                        source: rusqlite::Error::InvalidColumnName(format!(
                            "Failed to create endpoint: {e}"
                        )),
                    },
                },
            );
        }
    };

    let mut delivered_email_ids = Vec::new();
    let total_bytes: usize = emails.iter().map(|e| e.size_bytes).sum();

    tracing::info!(
        "📤 Simulating P2P email delivery: {} emails ({} bytes total) from {} to {}",
        emails.len(),
        total_bytes,
        our_alias,
        peer_id52
    );

    // Use proper fastn-net get_stream (same as working test and http_proxy.rs)
    println!(
        "🔗 DEBUG: About to establish P2P stream via fastn-net to {}",
        peer_id52.id52()
    );

    let (mut send, mut recv) = match tokio::time::timeout(
        std::time::Duration::from_secs(30), // 30 second timeout
        fastn_net::get_stream(
            our_endpoint,
            fastn_net::Protocol::AccountToAccount.into(),
            peer_id52,
            peer_stream_senders.clone(),
            graceful.clone(),
        ),
    )
    .await
    {
        Ok(Ok(stream)) => {
            println!("✅ DEBUG: fastn-net P2P stream established successfully");
            stream
        }
        Ok(Err(e)) => {
            println!("❌ DEBUG: fastn-net P2P stream establishment failed: {}", e);
            return Err(
                fastn_rig::EmailDeliveryError::PendingDeliveriesQueryFailed {
                    source: fastn_mail::GetPendingDeliveriesError::DatabaseQueryFailed {
                        source: rusqlite::Error::InvalidColumnName(format!(
                            "P2P connection failed: {e}"
                        )),
                    },
                },
            );
        }
        Err(_timeout) => {
            println!("⏰ DEBUG: fastn-net P2P stream establishment timed out after 30 seconds");
            return Err(
                fastn_rig::EmailDeliveryError::PendingDeliveriesQueryFailed {
                    source: fastn_mail::GetPendingDeliveriesError::DatabaseQueryFailed {
                        source: rusqlite::Error::InvalidColumnName(
                            "P2P connection timeout".to_string(),
                        ),
                    },
                },
            );
        }
    };

    tracing::info!(
        "🔗 P2P stream established from {} to {}",
        our_alias,
        peer_id52
    );

    // Send each email over the established stream with request-response
    for email in emails {
        println!(
            "📧 DEBUG: Processing email {} for P2P delivery",
            email.email_id
        );

        // Create AccountToAccountMessage for P2P delivery with envelope data
        let p2p_message = fastn_account::AccountToAccountMessage::new_email(
            email.raw_message.clone(),
            email.envelope_from.clone(),
            email.envelope_to.clone(),
        );

        println!(
            "📦 DEBUG: Created P2P message for email {} ({} bytes)",
            email.email_id,
            p2p_message.size()
        );

        tracing::debug!(
            "📦 Sending email {} ({} bytes) over P2P stream",
            email.email_id,
            p2p_message.size()
        );

        // Send email message as JSON
        println!("📤 DEBUG: About to serialize P2P message to JSON");
        let message_json = match serde_json::to_string(&p2p_message) {
            Ok(json) => {
                println!(
                    "✅ DEBUG: P2P message serialized successfully ({} chars)",
                    json.len()
                );
                json
            }
            Err(e) => {
                println!("❌ DEBUG: P2P message serialization failed: {}", e);
                return Err(
                    fastn_rig::EmailDeliveryError::PendingDeliveriesQueryFailed {
                        source: fastn_mail::GetPendingDeliveriesError::DatabaseQueryFailed {
                            source: rusqlite::Error::InvalidColumnName(format!(
                                "Failed to serialize message: {e}"
                            )),
                        },
                    },
                );
            }
        };

        println!("📡 DEBUG: About to write email message to P2P stream");
        match send.write_all(message_json.as_bytes()).await {
            Ok(_) => println!("✅ DEBUG: Email message written to stream"),
            Err(e) => {
                println!("❌ DEBUG: Failed to write email message to stream: {}", e);
                return Err(
                    fastn_rig::EmailDeliveryError::PendingDeliveriesQueryFailed {
                        source: fastn_mail::GetPendingDeliveriesError::DatabaseQueryFailed {
                            source: rusqlite::Error::InvalidColumnName(format!(
                                "Failed to send email: {e}"
                            )),
                        },
                    },
                );
            }
        }

        println!("📡 DEBUG: About to write newline to P2P stream");
        match send.write_all(b"\n").await {
            Ok(_) => println!("✅ DEBUG: Newline written to stream"),
            Err(e) => {
                println!("❌ DEBUG: Failed to write newline to stream: {}", e);
                return Err(
                    fastn_rig::EmailDeliveryError::PendingDeliveriesQueryFailed {
                        source: fastn_mail::GetPendingDeliveriesError::DatabaseQueryFailed {
                            source: rusqlite::Error::InvalidColumnName(format!(
                                "Failed to send newline: {e}"
                            )),
                        },
                    },
                );
            }
        }

        println!("⏳ DEBUG: Waiting for delivery response from peer (proper confirmation)");

        // Wait for actual response from peer (proper delivery confirmation)
        let response =
            match fastn_net::next_json::<fastn_account::EmailDeliveryResponse>(&mut recv).await {
                Ok(response) => {
                    println!(
                        "✅ DEBUG: Received response from peer: {:?}",
                        response.status
                    );
                    response
                }
                Err(e) => {
                    println!("❌ DEBUG: Failed to receive response from peer: {}", e);
                    return Err(
                        fastn_rig::EmailDeliveryError::PendingDeliveriesQueryFailed {
                            source: fastn_mail::GetPendingDeliveriesError::DatabaseQueryFailed {
                                source: rusqlite::Error::InvalidColumnName(format!(
                                    "Failed to receive response: {e}"
                                )),
                            },
                        },
                    );
                }
            };

        // Handle delivery response properly
        match response.status {
            fastn_account::DeliveryStatus::Accepted => {
                delivered_email_ids.push(email.email_id.clone());
                println!(
                    "✅ Email {} delivered successfully (confirmed by peer)",
                    email.email_id
                );
                tracing::info!(
                    "📧 Email {} delivered to {} (confirmed)",
                    email.email_id,
                    peer_id52
                );
            }
            fastn_account::DeliveryStatus::Rejected { reason } => {
                println!("❌ Email {} rejected by peer: {}", email.email_id, reason);
                tracing::warn!(
                    "📧 Email {} rejected by {}: {}",
                    email.email_id,
                    peer_id52,
                    reason
                );
                // Don't add to delivered_email_ids - will remain queued for retry
            }
        }
    }

    tracing::info!(
        "✅ P2P delivery completed: {} of {} emails delivered to {}",
        delivered_email_ids.len(),
        emails.len(),
        peer_id52
    );

    Ok(delivered_email_ids)
}

/// Get secret key for a specific alias from an account
async fn get_secret_key_for_alias(
    account: &fastn_account::Account,
    alias_key: &fastn_id52::PublicKey,
) -> Result<fastn_id52::SecretKey, fastn_rig::EmailDeliveryError> {
    // Get all aliases for this account
    let aliases = account.aliases().await;

    // Find the alias that matches our target
    for alias in aliases {
        if *alias.public_key() == *alias_key {
            return Ok(alias.secret_key().clone());
        }
    }

    // If we get here, the alias wasn't found in the account
    Err(fastn_rig::EmailDeliveryError::NoSenderAliasFound {
        peer_id52: alias_key.id52(),
    })
}
