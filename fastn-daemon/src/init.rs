pub async fn init(fastn_home: &std::path::Path) {
    println!("Initializing fastn daemon at: {}", fastn_home.display());

    // Create FASTN_HOME directory (mkdir -p equivalent)
    if let Err(e) = std::fs::create_dir_all(fastn_home) {
        eprintln!(
            "Error: Failed to create directory {}: {}",
            fastn_home.display(),
            e
        );
        std::process::exit(1);
    }

    let lock_file = fastn_home.join("fastn.lock");

    // Check if lock file already exists - fail if it does
    if lock_file.exists() {
        eprintln!(
            "Error: fastn daemon already initialized at {}",
            fastn_home.display()
        );
        eprintln!("Lock file exists: {}", lock_file.display());
        std::process::exit(1);
    }

    // TODO: Call fastn-ssh::init() here when fastn-ssh crate is ready

    println!("fastn daemon initialized successfully!");
    println!("Home directory: {}", fastn_home.display());
}
