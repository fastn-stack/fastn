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

    let _lock_guard = match file_guard::lock(&lock_file, file_guard::Lock::Exclusive, 0, 1) {
        Ok(guard) => guard,
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

    println!("🚀 Starting fastn at {}", fastn_home.display());
    println!("🔒 Lock acquired: {}", lock_path.display());

    // Load Rig and AccountManager (we know it's initialized)
    println!("📂 Loading existing fastn_home...");
    let rig = fastn_rig::Rig::load(fastn_home.clone())
        .map_err(|e| fastn_rig::RunError::RigLoadingFailed { source: e })?;
    let account_manager = fastn_account::AccountManager::load(fastn_home.clone())
        .await
        .map_err(|e| fastn_rig::RunError::AccountManagerLoadFailed { source: e })?;

    println!("🔑 Rig ID52: {}", rig.id52());
    println!("👤 Owner: {}", rig.owner());
    match rig.get_current().await {
        Ok(current) => println!("📍 Current entity: {current}"),
        Err(e) => {
            eprintln!("⚠️  Failed to get current entity: {e}");
            return Err(fastn_rig::RunError::CurrentEntityFailed { source: e });
        }
    }

    // Create graceful shutdown handler
    let graceful = fastn_net::Graceful::new();

    // Create EndpointManager with graceful
    let (mut endpoint_manager, mut message_rx) = fastn_rig::EndpointManager::new(graceful.clone());

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
            endpoint_manager
                .bring_online(
                    id52,
                    secret_key,
                    fastn_rig::OwnerType::Account,
                    account_path,
                )
                .await
                .map_err(|e| fastn_rig::RunError::EndpointOnlineFailed { source: e })?;
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
        endpoint_manager
            .bring_online(
                rig_id52,
                rig.secret_key().to_bytes().to_vec(),
                fastn_rig::OwnerType::Rig,
                fastn_home.clone(),
            )
            .await
            .map_err(|e| fastn_rig::RunError::EndpointOnlineFailed { source: e })?;
        total_endpoints += 1;
        println!("✅ Rig endpoint online");
    } else {
        println!("⏸️  Rig endpoint offline (use 'set-online' to bring online)");
    }

    // Display service information
    println!("\n📨 fastn is running. Press Ctrl+C to stop.");
    println!("   P2P: active on {total_endpoints} endpoints");
    println!("   SMTP: planned (port 2525)");
    println!("   IMAP: planned (port 1143)");
    println!("   HTTP: planned (port 8000)");

    // Spawn P2P message handler as a background task
    let p2p_endpoint_manager = std::sync::Arc::new(tokio::sync::Mutex::new(endpoint_manager));
    let p2p_graceful = graceful.clone();
    let _p2p_handle = graceful.spawn(async move {
        println!("\n📬 P2P handler started...");
        loop {
            tokio::select! {
                Some((endpoint_id52, owner_type, message)) = message_rx.recv() => {
                    tracing::info!(
                        "Received message on endpoint {} (type: {:?})",
                        endpoint_id52,
                        owner_type
                    );

                    // Route message based on owner type
                    let result = match owner_type {
                        fastn_rig::OwnerType::Account => {
                            process_account_message(&account_manager, &endpoint_id52, message).await
                        }
                        fastn_rig::OwnerType::Device => {
                            process_device_message(&endpoint_id52, message).await
                        }
                        fastn_rig::OwnerType::Rig => process_rig_message(&endpoint_id52, message).await,
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
    _account_manager: &fastn_account::AccountManager,
    endpoint_id52: &str,
    message: Vec<u8>,
) -> Result<(), fastn_rig::MessageProcessingError> {
    // TODO: Find which account owns this endpoint and process the email
    println!(
        "📨 Account message on {}: {} bytes",
        endpoint_id52,
        message.len()
    );

    // In the future:
    // 1. Find which account owns this endpoint
    // 2. Call account.process_email(endpoint_id52, message)

    Ok(())
}

/// Process a message received on a device endpoint
async fn process_device_message(
    endpoint_id52: &str,
    message: Vec<u8>,
) -> Result<(), fastn_rig::MessageProcessingError> {
    // TODO: Handle device sync messages
    println!(
        "📱 Device message on {}: {} bytes",
        endpoint_id52,
        message.len()
    );

    Ok(())
}

/// Process a message received on the rig endpoint
async fn process_rig_message(
    endpoint_id52: &str,
    message: Vec<u8>,
) -> Result<(), fastn_rig::MessageProcessingError> {
    // TODO: Handle rig control messages
    println!(
        "⚙️ Rig message on {}: {} bytes",
        endpoint_id52,
        message.len()
    );

    Ok(())
}
