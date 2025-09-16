/// Initialize SSH configuration and setup
///
/// This function sets up SSH-related configuration files, directories,
/// and initial key management for the fastn daemon.
pub async fn init(fastn_home: &std::path::Path) {
    let remote_dir = fastn_home.join("remote-access");

    // Check if SSH is already initialized
    if remote_dir.exists() {
        eprintln!("Error: SSH already initialized at {}", remote_dir.display());
        std::process::exit(1);
    }

    // Create ssh directory
    if let Err(e) = std::fs::create_dir_all(&remote_dir) {
        eprintln!(
            "Error: Failed to create SSH directory {}: {}",
            remote_dir.display(),
            e
        );
        std::process::exit(1);
    }

    // Generate new SSH secret key
    let secret_key = fastn_id52::SecretKey::generate();

    // Store secret key using the standard format
    if let Err(e) = secret_key.save_to_dir(&remote_dir, "remote") {
        eprintln!("Error: Failed to save SSH secret key: {}", e);
        std::process::exit(1);
    }

    let config_path = remote_dir.join("config.toml");

    // Create default config.toml
    let default_config = r#"# fastn SSH Configuration
#
# Configure remote machines that can access this fastn daemon via SSH.
# Each section defines an allowed remote with explicit permissions.
#
# Example configuration:
# [amitu]
# id52 = "your-remote-id52-here"
# allow-ssh = true

# Uncomment and configure your remotes:
# [my-remote]
# id52 = "remote-machine-id52"
# allow-ssh = true  # Enables SSH access for this remote
"#;

    if let Err(e) = std::fs::write(&config_path, default_config) {
        eprintln!(
            "Error: Failed to write SSH config to {}: {}",
            config_path.display(),
            e
        );
        std::process::exit(1);
    }

    // Get the public key for display
    let public_key = secret_key.public_key();

    println!("SSH configuration initialized successfully!");
    println!("SSH directory: {}", remote_dir.display());
    println!("SSH ID52 (public key): {public_key}");
    println!(
        "Secret key stored at: {}",
        remote_dir.join("remote.private-key").display()
    );
    println!("Configuration file: {}", config_path.display());
    println!();
    println!("Next steps:");
    println!("1. Share your SSH ID52 with remote machines: {public_key}");
    println!("2. Configure allowed remotes in: {}", config_path.display());
    println!("3. Run 'fastn daemon' to start the SSH service");
}
