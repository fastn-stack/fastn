//! Clean fastn-rig run function using fastn-p2p

/// Main run function using fastn-p2p (replaces old run.rs)
pub async fn run(home: Option<std::path::PathBuf>) -> Result<(), fastn_rig::RunError> {
    // Resolve fastn_home path
    let fastn_home = fastn_rig::resolve_fastn_home(home)?;

    // Check if already initialized
    let is_initialized = fastn_rig::Rig::is_initialized(&fastn_home);
    if !is_initialized {
        eprintln!("❌ fastn_home not initialized at {}", fastn_home.display());
        eprintln!("   Run 'fastn-rig init' first to initialize the rig");
        return Err(fastn_rig::RunError::FastnHomeResolution);
    }

    // Acquire exclusive lock for runtime
    let lock_path = fastn_home.join(".fastn.lock");
    let lock_file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(&lock_path)
        .map_err(|e| fastn_rig::RunError::LockFileOpen {
            path: lock_path.clone(),
            source: e,
        })?;

    match lock_file.try_lock() {
        Ok(()) => {
            println!("🔒 Lock acquired: {}", lock_path.display());
        }
        Err(_e) => {
            eprintln!(
                "❌ Another instance of fastn is already running at {}",
                fastn_home.display()
            );
            return Err(fastn_rig::RunError::LockAcquisition);
        }
    };

    let _lock_guard = lock_file;

    println!("🚀 Starting fastn at {}", fastn_home.display());

    // Load Rig and AccountManager
    println!("📂 Loading existing fastn_home...");
    let rig = fastn_rig::Rig::load(fastn_home.clone())
        .map_err(|e| fastn_rig::RunError::RigLoading { source: e })?;
    let account_manager = std::sync::Arc::new(
        fastn_account::AccountManager::load(fastn_home.clone())
            .await
            .map_err(|e| fastn_rig::RunError::AccountManagerLoad { source: e })?,
    );

    println!("🔑 Rig ID52: {}", rig.id52());
    println!("👤 Owner: {}", rig.owner());

    // Use fastn-p2p global singletons - no more graceful variables needed!

    // Get all endpoints from all accounts
    let all_endpoints = account_manager
        .get_all_endpoints()
        .await
        .map_err(|e| fastn_rig::RunError::EndpointEnumeration { source: e })?;

    // Start fastn-p2p listeners for all online endpoints
    let mut total_endpoints = 0;
    for (id52, secret_key, _account_path) in all_endpoints {
        if rig
            .is_entity_online(&id52)
            .await
            .map_err(|e| fastn_rig::RunError::EntityOnlineStatus { source: e })?
        {
            let account_manager_clone = account_manager.clone();

            fastn_p2p::spawn(async move {
                if let Err(e) =
                    crate::p2p_server::start_p2p_listener(secret_key, account_manager_clone).await
                {
                    eprintln!("❌ Account P2P listener failed for {id52}: {e}");
                }
            });
            total_endpoints += 1;
        }
    }

    // Start fastn-p2p listener for rig endpoint
    let rig_id52 = rig.id52();
    if rig
        .is_entity_online(&rig_id52)
        .await
        .map_err(|e| fastn_rig::RunError::EntityOnlineStatus { source: e })?
    {
        let account_manager_clone = account_manager.clone();
        let rig_secret = rig.secret_key().clone();

        fastn_p2p::spawn(async move {
            if let Err(e) =
                crate::p2p_server::start_p2p_listener(rig_secret, account_manager_clone).await
            {
                eprintln!("❌ Rig P2P listener failed for {rig_id52}: {e}");
            }
        });
        total_endpoints += 1;
        println!("✅ Rig endpoint online");
    }

    println!("📡 Started {total_endpoints} P2P listeners using fastn-p2p");

    // Start email delivery poller using fastn-p2p
    println!("📬 Starting email delivery poller...");
    let account_manager_clone = account_manager.clone();

    let _poller_handle = fastn_p2p::spawn(async move {
        if let Err(e) = crate::email_poller_p2p::start_email_delivery_poller(account_manager_clone).await {
            tracing::error!("Email delivery poller failed: {e}");
        }
    });
    println!("✅ Email delivery poller started");

    // Start SMTP server with STARTTLS support
    let smtp_port = std::env::var("FASTN_SMTP_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(587);  // Default to port 587 (standard email submission port)
    println!("📮 SMTP server listening on port {smtp_port} (supports both plain text and STARTTLS)");

    // Create SMTP server with STARTTLS support (MUST succeed)
    let smtp_server = crate::smtp::SmtpServer::new(
        account_manager.clone(), 
        ([0, 0, 0, 0], smtp_port).into(),
        &fastn_home,
        rig.secret_key().clone(),
    ).map_err(|e| {
        eprintln!("❌ CRITICAL: Failed to create SMTP server: {e}");
        eprintln!("   This indicates a serious problem that must be fixed:");
        eprintln!("   - Certificate storage creation failed");
        eprintln!("   - Directory permissions issues");  
        eprintln!("   - TLS/crypto library problems");
        eprintln!("   fastn_home: {}", fastn_home.display());
        eprintln!("   Certificate dir would be: {}", fastn_home.parent().map(|p| p.join("certs").display().to_string()).unwrap_or_else(|| "UNKNOWN".to_string()));
        fastn_rig::RunError::FastnHomeResolution // Stop execution to force debugging
    })?;
    
    println!("✅ SMTP server created with STARTTLS certificate support");
    let _smtp_handle = fastn_p2p::spawn(async move {
        if let Err(e) = smtp_server.start().await {
            tracing::error!("SMTP server error: {}", e);
        }
    });

    // Start IMAP server  
    let imap_port = std::env::var("FASTN_IMAP_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(1143);  // Default to unprivileged port 1143
    println!("📨 Starting IMAP server on port {imap_port}...");
    
    let imap_account_manager = account_manager.clone();
    let _imap_handle = fastn_p2p::spawn(async move {
        if let Err(e) = crate::imap::start_imap_server(imap_account_manager, imap_port).await {
            tracing::error!("IMAP server error: {}", e);
        }
    });
    
    println!("✅ IMAP server started on port {imap_port}");

    // Start HTTP server
    let http_port = std::env::var("FASTN_HTTP_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(0);
    println!(
        "🌐 HTTP server starting on port {}",
        if http_port == 0 {
            "auto".to_string()
        } else {
            http_port.to_string()
        }
    );

    crate::http_server::start_http_server(account_manager.clone(), rig.clone(), Some(http_port))
        .await?;

    println!("\n📨 fastn is running with fastn-p2p. Press Ctrl+C to stop.");

    // Wait for graceful shutdown
    fastn_p2p::globals::graceful()
        .shutdown()
        .await
        .map_err(|e| fastn_rig::RunError::Shutdown {
            source: Box::new(std::io::Error::other(format!("Shutdown failed: {e}"))),
        })?;

    println!("👋 Goodbye!");
    Ok(())
}
