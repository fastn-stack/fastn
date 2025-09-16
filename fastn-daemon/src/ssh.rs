pub async fn ssh(fastn_home: &std::path::Path, target: &str) {
    use std::str::FromStr;
    // Load our SSH key from FASTN_HOME
    let ssh_dir = fastn_home.join("ssh");

    if !ssh_dir.exists() {
        eprintln!("Error: SSH not initialized. Run 'fastn init' first.");
        std::process::exit(1);
    }

    let (our_id52, our_key) = match fastn_id52::SecretKey::load_from_dir(&ssh_dir, "ssh") {
        Ok((id52, key)) => (id52, key),
        Err(e) => {
            eprintln!("Error: Failed to load SSH key: {}", e);
            std::process::exit(1);
        }
    };

    // TODO: Parse config.toml to resolve target (alias → ID52)
    // TODO: Validate target is in allowed list

    // For now, treat target as direct ID52
    let target_key = match fastn_id52::PublicKey::from_str(target) {
        Ok(key) => key,
        Err(_) => {
            eprintln!("Error: Invalid target ID52: {}", target);
            eprintln!("TODO: Add alias resolution from config.toml");
            std::process::exit(1);
        }
    };

    println!("Connecting to SSH server...");
    println!("  Our ID52: {}", our_id52);
    println!("  Target: {}", target_key);

    // Default to TTY mode for interactive shell
    fastn_remote::tty(our_key, target_key).await;
}
