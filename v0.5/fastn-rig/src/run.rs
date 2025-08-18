/// Main run function for fastn
pub async fn run(home: Option<std::path::PathBuf>) -> eyre::Result<()> {
    use eyre::WrapErr;
    // Determine the fastn home directory
    // Priority: 1. --home argument, 2. FASTN_HOME env var, 3. default via ProjectDirs
    let fastn_home = match home {
        Some(path) => path,
        None => match std::env::var("FASTN_HOME") {
            Ok(env_path) => std::path::PathBuf::from(env_path),
            Err(_) => {
                let proj_dirs = directories::ProjectDirs::from("com", "fastn", "fastn")
                    .ok_or_else(|| eyre::eyre!("Failed to determine project directories"))?;
                proj_dirs.data_dir().to_path_buf()
            }
        },
    };

    // Ensure fastn_home directory exists
    std::fs::create_dir_all(&fastn_home)
        .wrap_err_with(|| format!("Failed to create fastn_home directory at {fastn_home:?}"))?;

    // Check if fastn_home is already initialized
    let lock_path = fastn_home.join(".fastn.lock");
    let is_initialized = lock_path.exists();

    // Open or create lock file
    let lock_file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(&lock_path)
        .wrap_err_with(|| format!("Failed to open lock file at {lock_path:?}"))?;

    // Acquire exclusive lock to ensure only one instance runs
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
            return Err(eyre::eyre!("Failed to acquire lock"));
        }
    };

    println!("🚀 Starting fastn at {}", fastn_home.display());
    println!("🔒 Lock acquired: {}", lock_path.display());

    // Initialize or load Rig and AccountManager based on whether fastn_home is initialized
    let (rig, account_manager) = if is_initialized {
        println!("📂 Loading existing fastn_home...");
        let rig = fastn_rig::Rig::load(fastn_home.clone())
            .wrap_err("Failed to load Rig from existing fastn_home")?;
        let account_manager = fastn_account::AccountManager::load(fastn_home.clone())
            .await
            .wrap_err("Failed to load AccountManager from existing fastn_home")?;
        (rig, account_manager)
    } else {
        println!("🎉 Initializing new fastn_home...");
        let (rig, account_manager, primary_id52) = fastn_rig::Rig::create(fastn_home.clone())
            .await
            .wrap_err("Failed to create new Rig and first account")?;

        // Set the newly created account as current and online
        rig.set_endpoint_online(&primary_id52, true).await;
        rig.set_current(&primary_id52).await?;

        (rig, account_manager)
    };

    println!("🔑 Rig ID52: {}", rig.id52());
    println!("👤 Owner: {}", rig.owner());
    if let Some(current) = rig.get_current().await {
        println!("📍 Current entity: {current}");
    }

    // Create graceful shutdown handler
    let graceful = fastn_net::Graceful::new();

    // Create EndpointManager with graceful
    let (mut endpoint_manager, mut message_rx) = fastn_rig::EndpointManager::new(graceful.clone());

    // Get all endpoints from all accounts
    let all_endpoints = account_manager.get_all_endpoints().await?;

    // Start endpoints that are marked as online
    let mut total_endpoints = 0;
    for (id52, secret_key, account_path) in all_endpoints {
        // Check if this endpoint is online in the rig database
        if rig.is_endpoint_online(&id52).await {
            endpoint_manager
                .bring_online(
                    id52,
                    secret_key,
                    fastn_rig::OwnerType::Account,
                    account_path,
                )
                .await?;
            total_endpoints += 1;
        }
    }

    // Also bring the Rig's own endpoint online
    let rig_id52 = rig.id52();
    rig.set_endpoint_online(&rig_id52, true).await;
    endpoint_manager
        .bring_online(
            rig_id52,
            rig.secret_key().to_bytes().to_vec(),
            fastn_rig::OwnerType::Rig,
            fastn_home.clone(),
        )
        .await?;
    total_endpoints += 1;
    println!("✅ Rig endpoint online");

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
    graceful.shutdown().await?;

    // Clean shutdown of all endpoints (but don't change their online status in DB)
    println!("Stopping all endpoints...");
    let mut endpoint_manager = p2p_endpoint_manager.lock().await;
    endpoint_manager.shutdown_all().await?;

    println!("🔓 Releasing lock...");
    println!("👋 Goodbye!");

    Ok(())
}

/// Process a message received on an account endpoint
async fn process_account_message(
    _account_manager: &fastn_account::AccountManager,
    endpoint_id52: &str,
    message: Vec<u8>,
) -> eyre::Result<()> {
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
async fn process_device_message(endpoint_id52: &str, message: Vec<u8>) -> eyre::Result<()> {
    // TODO: Handle device sync messages
    println!(
        "📱 Device message on {}: {} bytes",
        endpoint_id52,
        message.len()
    );

    Ok(())
}

/// Process a message received on the rig endpoint
async fn process_rig_message(endpoint_id52: &str, message: Vec<u8>) -> eyre::Result<()> {
    // TODO: Handle rig control messages
    println!(
        "⚙️ Rig message on {}: {} bytes",
        endpoint_id52,
        message.len()
    );

    Ok(())
}
