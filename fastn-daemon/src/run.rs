pub async fn run(fastn_home: &std::path::Path) {
    println!("Starting fastn daemon...");

    let lock_file = fastn_home.join("fastn.lock");

    // Check if daemon was initialized first
    if !lock_file.exists() {
        eprintln!("Error: fastn daemon not initialized. Run 'fastn init' first.");
        eprintln!("Expected directory: {}", fastn_home.display());
        std::process::exit(1);
    }

    // Create/open lock file for locking
    let lock_file_handle = match std::fs::OpenOptions::new()
        .write(true)
        .create(false)
        .open(&lock_file)
    {
        Ok(file) => file,
        Err(e) => {
            eprintln!(
                "Error: Failed to open lock file {}: {}",
                lock_file.display(),
                e
            );
            std::process::exit(1);
        }
    };

    if let Err(e) = lock_file_handle.try_lock() {
        eprintln!(
            "Error: Failed to acquire exclusive lock on {}",
            lock_file.display()
        );
        eprintln!("Another fastn daemon instance is already running: {}", e);
        std::process::exit(1);
    }

    println!("Lock acquired: {}", lock_file.display());
    println!("fastn daemon started. Press Ctrl+C to stop.");

    // TODO: Handle cleanup on exit (signals, graceful shutdown, etc.)

    // Call fastn-ssh::run() to start SSH services
    fastn_ssh::run(fastn_home).await;

    // Main daemon loop - keep running until interrupted
    // Note: lock_file_handle must stay alive to maintain the lock
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}
