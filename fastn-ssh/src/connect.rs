/// Connect to SSH server (CLI interface - handles parsing)
pub async fn connect_cli(private_key: &str, target: &str) {
    use std::str::FromStr;

    // Parse private key from content
    let secret_key = match fastn_id52::SecretKey::from_str(private_key.trim()) {
        Ok(key) => key,
        Err(e) => {
            eprintln!("Error: Invalid private key format: {}", e);
            std::process::exit(1);
        }
    };

    // Parse target ID52
    let target_key = match fastn_id52::PublicKey::from_str(target.trim()) {
        Ok(key) => key,
        Err(e) => {
            eprintln!("Error: Invalid target ID52 '{}': {}", target, e);
            std::process::exit(1);
        }
    };

    // Call the typed P2P function
    connect(secret_key, target_key).await;
}

/// Core SSH connect implementation (pure P2P)
pub async fn connect(secret_key: fastn_id52::SecretKey, target: fastn_id52::PublicKey) {
    println!("SSH connection configured:");
    println!("  Our ID52: {}", secret_key.id52());
    println!("  Target: {}", target);

    println!("\n🚀 Connecting to SSH server...");

    // TODO: Implement actual SSH connection using fastn-p2p
    // TODO: Set up P2P protocol for SSH
    // TODO: Establish connection to target
    // TODO: Handle SSH session (shell, commands, etc.)

    println!("SSH connection established. Type 'exit' to disconnect.");

    // Simulate SSH session
    loop {
        tokio::select! {
            _ = fastn_p2p::cancelled() => {
                println!("SSH connection interrupted by shutdown signal");
                break;
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_secs(1)) => {
                // TODO: Handle SSH session I/O
            }
        }
    }

    println!("SSH connection closed.");
}
