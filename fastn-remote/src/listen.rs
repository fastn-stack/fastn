/// Start SSH listener (CLI interface - handles parsing)
pub async fn listen_cli(private_key: &str, allowed: &str) {
    use std::str::FromStr;

    // Parse private key from content
    let secret_key = match fastn_id52::SecretKey::from_str(private_key.trim()) {
        Ok(key) => key,
        Err(e) => {
            eprintln!("Error: Invalid private key format: {e}");
            std::process::exit(1);
        }
    };

    // Parse allowed ID52 list
    let allowed_keys: Vec<fastn_id52::PublicKey> = allowed
        .split(',')
        .map(|id52| id52.trim())
        .filter(|id52| !id52.is_empty())
        .map(|id52| {
            fastn_id52::PublicKey::from_str(id52).unwrap_or_else(|e| {
                eprintln!("Error: Invalid ID52 '{id52}': {e}");
                std::process::exit(1);
            })
        })
        .collect();

    // Call the typed P2P function
    listen(secret_key, allowed_keys).await;
}

/// Core SSH listener implementation (pure P2P)
pub async fn listen(secret_key: fastn_id52::SecretKey, allowed_keys: Vec<fastn_id52::PublicKey>) {
    println!("SSH listener configured:");
    println!("  Our ID52: {}", secret_key.id52());
    println!("  Allowed remotes: {} ID52s", allowed_keys.len());
    for (i, key) in allowed_keys.iter().enumerate() {
        println!("    {}: {key}", i + 1);
    }

    println!("\n🚀 SSH listener started. Press Ctrl+C to stop.");

    // TODO: Implement actual SSH listener using fastn-p2p
    // TODO: Set up P2P protocol for SSH
    // TODO: Handle incoming connections and validate against allowed_keys

    // Keep running until interrupted
    loop {
        tokio::select! {
            _ = fastn_p2p::cancelled() => {
                println!("SSH listener shutting down gracefully...");
                break;
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_secs(1)) => {
                // Keep running
            }
        }
    }

    println!("SSH listener stopped.");
}
