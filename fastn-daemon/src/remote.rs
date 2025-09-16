pub async fn rshell(fastn_home: &std::path::Path, target: &str, command: Option<&str>) {
    use std::str::FromStr;
    // Load our remote key from FASTN_HOME
    let remote_dir = fastn_home.join("remote");

    if !remote_dir.exists() {
        eprintln!("Error: Remote access not initialized. Run 'fastn init' first.");
        std::process::exit(1);
    }

    let (our_id52, our_key) = match fastn_id52::SecretKey::load_from_dir(&remote_dir, "remote") {
        Ok((id52, key)) => (id52, key),
        Err(e) => {
            eprintln!("Error: Failed to load remote key: {}", e);
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

    println!("Connecting to remote shell...");
    println!("  Our ID52: {}", our_id52);
    println!("  Target: {}", target_key);

    // Call fastn-remote rshell function
    fastn_remote::rshell(our_key, target_key, command).await;
}

pub async fn rexec(fastn_home: &std::path::Path, target: &str, command: &str) {
    use std::str::FromStr;
    // Load our remote key from FASTN_HOME
    let remote_dir = fastn_home.join("remote");

    if !remote_dir.exists() {
        eprintln!("Error: Remote access not initialized. Run 'fastn init' first.");
        std::process::exit(1);
    }

    let (our_id52, our_key) = match fastn_id52::SecretKey::load_from_dir(&remote_dir, "remote") {
        Ok((id52, key)) => (id52, key),
        Err(e) => {
            eprintln!("Error: Failed to load remote key: {}", e);
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

    println!("Executing remote command...");
    println!("  Our ID52: {}", our_id52);
    println!("  Target: {}", target_key);
    println!("  Command: {}", command);

    // Call fastn-remote rexec function
    fastn_remote::rexec(our_key, target_key, command).await;
}
