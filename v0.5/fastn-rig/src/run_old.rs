/// Main run function for fastn
pub async fn run(home: Option<std::path::PathBuf>) -> Result<(), fastn_rig::RunError> {
    // Resolve fastn_home path
    let fastn_home = fastn_rig::resolve_fastn_home(home)?;

    // Check if already initialized
    let is_initialized = fastn_rig::Rig::is_initialized(&fastn_home);

    // If not initialized, run should fail - user must run init first
    if !is_initialized {
        eprintln!("❌ fastn_home not initialized at {}", fastn_home.display());
        eprintln!("   Run 'fastn-rig init' first to initialize the rig");
        return Err(fastn_rig::RunError::FastnHomeResolutionFailed);
    }

    // Acquire exclusive lock for runtime
    let lock_path = fastn_home.join(".fastn.lock");
    let lock_file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(&lock_path)
        .map_err(|e| fastn_rig::RunError::LockFileOpenFailed {
            path: lock_path.clone(),
            source: e,
        })?;

    // Acquire exclusive lock using standard library API
    match lock_file.try_lock() {
        Ok(()) => {
            println!("🔒 Lock acquired: {}", lock_path.display());
        }
        Err(e) => {
            eprintln!(
                "❌ Another instance of fastn is already running at {}",
                fastn_home.display()
            );
            eprintln!("   Lock file: {}", lock_path.display());
            eprintln!("   Error: {e}");
            eprintln!(
                "\nIf you're sure no other instance is running, delete the lock file and try again."
            );
            return Err(fastn_rig::RunError::LockAcquisitionFailed);
        }
    };

    // Keep the file handle alive to maintain the lock
    let _lock_guard = lock_file;

    println!("🚀 Starting fastn at {}", fastn_home.display());

    // Load Rig and AccountManager (we know it's initialized)
    println!("📂 Loading existing fastn_home...");
    let rig = fastn_rig::Rig::load(fastn_home.clone())
        .map_err(|e| fastn_rig::RunError::RigLoadingFailed { source: e })?;
    let account_manager = std::sync::Arc::new(
        fastn_account::AccountManager::load(fastn_home.clone())
            .await
            .map_err(|e| fastn_rig::RunError::AccountManagerLoadFailed { source: e })?,
    );

    println!("🔑 Rig ID52: {}", rig.id52());
    println!("👤 Owner: {}", rig.owner());
    match rig.get_current().await {
        Ok(current) => println!("📍 Current entity: {current}"),
        Err(e) => {
            eprintln!("⚠️  Failed to get current entity: {e}");
            return Err(fastn_rig::RunError::CurrentEntityFailed { source: e });
        }
    }

    // Use fastn-p2p global graceful instead of local one
    let graceful = fastn_p2p::graceful();

    // Get all endpoints from all accounts
    let all_endpoints = account_manager
        .get_all_endpoints()
        .await
        .map_err(|e| fastn_rig::RunError::EndpointEnumerationFailed { source: e })?;

    // Start endpoints that are marked as online
    let mut total_endpoints = 0;
    for (id52, secret_key, account_path) in all_endpoints {
        // Check if this endpoint is online in the rig database
        if rig
            .is_entity_online(&id52)
            .await
            .map_err(|e| fastn_rig::RunError::EntityOnlineStatusFailed { source: e })?
        {
            // Start fastn-p2p listener for this account endpoint
            let account_manager_clone = account_manager.clone();
            
            tokio::spawn(async move {
                if let Err(e) = crate::p2p_server::start_p2p_listener(
                    secret_key,
                    account_manager_clone,
                ).await {
                    eprintln!("❌ Account P2P listener failed for {}: {}", id52, e);
                }
            });
            total_endpoints += 1;
        }
    }

    // Bring the Rig's own endpoint online if it's marked as online
    let rig_id52 = rig.id52();
    if rig
        .is_entity_online(&rig_id52)
        .await
        .map_err(|e| fastn_rig::RunError::EntityOnlineStatusFailed { source: e })?
    {
        // Start fastn-p2p listener for rig endpoint
        let account_manager_clone = account_manager.clone();
        let rig_secret = rig.secret_key().clone();
        
        tokio::spawn(async move {
            if let Err(e) = crate::p2p_server::start_p2p_listener(
                rig_secret,
                account_manager_clone,
            ).await {
                eprintln!("❌ Rig P2P listener failed for {}: {}", rig_id52, e);
            }
        });
        total_endpoints += 1;
        println!("✅ Rig endpoint online");
    } else {
        println!("⏸️  Rig endpoint offline (use 'set-online' to bring online)");
    }

    // Display service information
    println!("\n📨 fastn is running. Press Ctrl+C to stop.");
    println!("   P2P: active on {total_endpoints} endpoints");
    println!("   Email Delivery: polling every 5 seconds");

    // SMTP server configuration
    let smtp_port = std::env::var("FASTN_SMTP_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(2525); // Default to 2525 for development
    println!("   SMTP: listening on port {smtp_port}");

    println!("   IMAP: planned (port 1143)");
    let http_port = std::env::var("FASTN_HTTP_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(0); // Default 0 = auto-select free port
    if http_port == 0 {
        println!("   HTTP: auto-selecting free port");
    } else {
        println!("   HTTP: listening on port {http_port}");
    }

    // Email delivery poller now uses fastn-p2p::call - much simpler!
    let enable_poller = std::env::var("ENABLE_EMAIL_POLLER")
        .unwrap_or_else(|_| "true".to_string())
        .parse::<bool>()
        .unwrap_or(true);
    
    if enable_poller {
        println!("📬 Email poller enabled - using clean fastn-p2p::call for delivery");
        // Start email delivery poller using clean fastn-p2p APIs!
        crate::email_poller_p2p::start_email_delivery_poller(
            account_manager.clone(),
            graceful.clone(),
        )
        .await
        .map_err(|e| fastn_rig::RunError::ShutdownFailed {
            source: e,
        })?;
        println!("✅ Email delivery poller started with fastn-p2p");
    } else {
        println!("📭 Email delivery poller disabled");
    }

    // Start SMTP server for email reception
    let smtp_server = crate::smtp::SmtpServer::new(
        account_manager.clone(),
        ([0, 0, 0, 0], smtp_port).into(),
        graceful.clone(),
    );
    let _smtp_handle = graceful.spawn(async move {
        if let Err(e) = smtp_server.start().await {
            tracing::error!("SMTP server error: {}", e);
        }
    });

    // Start HTTP server for web interface
    crate::http_server::start_http_server(
        account_manager.clone(),
        rig.clone(),
        graceful.clone(),
        Some(http_port),
    )
    .await?;

    // P2P messages handled directly by fastn-p2p listeners - no complex infrastructure needed!
    
    // Wait for graceful shutdown
    graceful.shutdown().await.map_err(|e| fastn_rig::RunError::ShutdownFailed {
        source: Box::new(std::io::Error::other(format!("Shutdown failed: {e}")))
    })?;

    println!("👋 Goodbye!");
    Ok(())
}
    let p2p_endpoint_manager = std::sync::Arc::new(tokio::sync::Mutex::new(endpoint_manager));
    let p2p_graceful = graceful.clone();
    let _p2p_handle = graceful.spawn(async move {
        println!("\n📬 P2P handler started...");
        loop {
            tokio::select! {
                Some(p2p_msg) = message_rx.recv() => {
                    println!("📨 DEBUG: Received P2P message on endpoint {} from {} ({} bytes)", 
                             p2p_msg.our_endpoint.id52(), p2p_msg.peer_id52.id52(), p2p_msg.message.len());
                    tracing::info!(
                        "Received message on endpoint {} from {} (type: {:?})",
                        p2p_msg.our_endpoint.id52(),
                        p2p_msg.peer_id52.id52(),
                        p2p_msg.owner_type
                    );

                    // Route message based on owner type
                    let result = match p2p_msg.owner_type {
                        fastn_rig::OwnerType::Account => {
                            process_account_message(&account_manager, p2p_msg).await
                        }
                        fastn_rig::OwnerType::Device => {
                            process_device_message(&p2p_msg.our_endpoint, p2p_msg.message).await
                        }
                        fastn_rig::OwnerType::Rig => process_rig_message(&p2p_msg.our_endpoint, p2p_msg.message).await,
                    };

                    if let Err(e) = result {
                        tracing::error!("Failed to process message: {}", e);
                    }
                }
                _ = p2p_graceful.cancelled() => {
                    tracing::info!("P2P handler shutting down");
                    break;
                }
            }
        }
    });

    // Future: spawn other servers here
    // let smtp_graceful = graceful.clone();
    // let smtp_handle = graceful.spawn(async move {
    //     smtp_server(smtp_graceful).await
    // });
    //
    // let http_graceful = graceful.clone();
    // let http_handle = graceful.spawn(async move {
    //     http_server(http_graceful).await
    // });
    //
    // let imap_graceful = graceful.clone();
    // let imap_handle = graceful.spawn(async move {
    //     imap_server(imap_graceful).await
    // });

    // Use graceful shutdown to wait for Ctrl+C and manage all tasks
    graceful
        .shutdown()
        .await
        .map_err(|e| fastn_rig::RunError::ShutdownFailed {
            source: Box::new(std::io::Error::other(format!(
                "Graceful shutdown failed: {e}"
            ))) as Box<dyn std::error::Error + Send + Sync>,
        })?;

    // Clean shutdown of all endpoints (but don't change their online status in DB)
    println!("Stopping all endpoints...");
    let mut endpoint_manager = p2p_endpoint_manager.lock().await;
    endpoint_manager
        .shutdown_all()
        .await
        .map_err(|e| fastn_rig::RunError::EndpointOnlineFailed { source: e })?;

    println!("🔓 Releasing lock...");
    println!("👋 Goodbye!");

    Ok(())
}

/// Process a message received on an account endpoint
async fn process_account_message(
    account_manager: &fastn_account::AccountManager,
    p2p_msg: fastn_rig::P2PMessage,
) -> Result<(), fastn_rig::MessageProcessingError> {
    // All connections are pre-authorized, so we only handle actual messages
    println!(
        "📨 Processing message from {} to {} ({} bytes)",
        p2p_msg.peer_id52.id52(),
        p2p_msg.our_endpoint.id52(),
        p2p_msg.message.len()
    );

    let account_message =
        serde_json::from_slice::<fastn_account::AccountToAccountMessage>(&p2p_msg.message)
            .map_err(
                |e| fastn_rig::MessageProcessingError::MessageDeserializationFailed { source: e },
            )?;

    // Delegate to AccountManager for proper handling
    account_manager
        .handle_account_message(&p2p_msg.peer_id52, &p2p_msg.our_endpoint, account_message)
        .await
        .map_err(
            |e| fastn_rig::MessageProcessingError::AccountMessageHandlingFailed { source: e },
        )?;

    println!("✅ Account message handled successfully");
    Ok(())
}

/// Process a message received on a device endpoint
async fn process_device_message(
    endpoint: &fastn_id52::PublicKey,
    message: Vec<u8>,
) -> Result<(), fastn_rig::MessageProcessingError> {
    // Device message processing (connection already authorized)
    println!("📱 Device message on {endpoint}: {} bytes", message.len());
    // TODO: Handle device sync messages
    Ok(())
}

/// Process a message received on the rig endpoint
async fn process_rig_message(
    endpoint: &fastn_id52::PublicKey,
    message: Vec<u8>,
) -> Result<(), fastn_rig::MessageProcessingError> {
    // Rig control message processing (connection already authorized)
    println!("⚙️ Rig message on {endpoint}: {} bytes", message.len());
    // TODO: Handle rig control messages
    Ok(())
}
