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

    match lock_file.try_lock() {
        Ok(()) => {
            println!("🔒 Lock acquired: {}", lock_path.display());
        }
        Err(e) => {
            eprintln!("❌ Another instance of fastn is already running at {}", fastn_home.display());
            return Err(fastn_rig::RunError::LockAcquisitionFailed);
        }
    };

    let _lock_guard = lock_file;

    println!("🚀 Starting fastn at {}", fastn_home.display());

    // Load Rig and AccountManager
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

    // Use fastn-p2p global graceful
    let graceful = fastn_p2p::graceful();

    // Get all endpoints from all accounts
    let all_endpoints = account_manager
        .get_all_endpoints()
        .await
        .map_err(|e| fastn_rig::RunError::EndpointEnumerationFailed { source: e })?;

    // Start fastn-p2p listeners for all online endpoints
    let mut total_endpoints = 0;
    for (id52, secret_key, _account_path) in all_endpoints {
        if rig.is_entity_online(&id52).await
            .map_err(|e| fastn_rig::RunError::EntityOnlineStatusFailed { source: e })? 
        {
            let account_manager_clone = account_manager.clone();
            
            fastn_p2p::spawn(async move {
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

    // Start fastn-p2p listener for rig endpoint  
    let rig_id52 = rig.id52();
    if rig.is_entity_online(&rig_id52).await
        .map_err(|e| fastn_rig::RunError::EntityOnlineStatusFailed { source: e })? 
    {
        let account_manager_clone = account_manager.clone();
        let rig_secret = rig.secret_key().clone();
        
        fastn_p2p::spawn(async move {
            if let Err(e) = crate::p2p_server::start_p2p_listener(
                rig_secret,
                account_manager_clone,
            ).await {
                eprintln!("❌ Rig P2P listener failed for {}: {}", rig_id52, e);
            }
        });
        total_endpoints += 1;
        println!("✅ Rig endpoint online");
    }

    println!("📡 Started {} P2P listeners using fastn-p2p", total_endpoints);

    // Start email delivery poller using fastn-p2p
    let enable_poller = std::env::var("ENABLE_EMAIL_POLLER")
        .unwrap_or_else(|_| "true".to_string())
        .parse::<bool>()
        .unwrap_or(true);
    
    if enable_poller {
        println!("📬 Starting email delivery poller with fastn-p2p");
        let account_manager_clone = account_manager.clone();
        
        fastn_p2p::spawn(async move {
            if let Err(e) = crate::email_poller_p2p::start_email_delivery_poller(
                account_manager_clone,
            ).await {
                eprintln!("❌ Email delivery poller failed: {}", e);
            }
        });
        println!("✅ Email delivery poller started");
    }

    // Start SMTP server
    let smtp_port = std::env::var("FASTN_SMTP_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(2525);
    println!("📮 SMTP server listening on port {}", smtp_port);
    
    let smtp_server = crate::smtp::SmtpServer::new(
        account_manager.clone(),
        ([0, 0, 0, 0], smtp_port).into(),
        graceful.clone(),
    );
    let _smtp_handle = fastn_p2p::spawn(async move {
        if let Err(e) = smtp_server.start().await {
            tracing::error!("SMTP server error: {}", e);
        }
    });

    // Start HTTP server
    let http_port = std::env::var("FASTN_HTTP_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(0);
    println!("🌐 HTTP server starting on port {}", if http_port == 0 { "auto".to_string() } else { http_port.to_string() });
    
    crate::http_server::start_http_server(
        account_manager.clone(),
        rig.clone(), 
        graceful.clone(),
        Some(http_port),
    )
    .await?;

    println!("\n📨 fastn is running with fastn-p2p. Press Ctrl+C to stop.");
    
    // Wait for graceful shutdown
    graceful.shutdown().await.map_err(|e| fastn_rig::RunError::ShutdownFailed {
        source: Box::new(std::io::Error::other(format!("Shutdown failed: {e}")))
    })?;

    println!("👋 Goodbye!");
    Ok(())
}