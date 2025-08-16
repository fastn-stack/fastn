/// Default behavior when fastn is run without any arguments.
///
/// This function implements the main fastn runtime behavior.
pub async fn run(home: Option<std::path::PathBuf>) {
    // Determine the fastn home directory
    // Priority: 1. --home argument, 2. FASTN_HOME env var, 3. Let EntityManager use default
    let home_path = home.or_else(|| {
        std::env::var("FASTN_HOME")
            .ok()
            .map(std::path::PathBuf::from)
    });

    // Create EntityManager instance
    let entity_manager = match fastn_entity::EntityManager::new(home_path).await {
        Ok(em) => em,
        Err(e) => {
            eprintln!("Failed to initialize EntityManager: {e}");
            std::process::exit(1);
        }
    };

    // TODO: Implement the rest of the default fastn behavior
    println!(
        "fastn: initialized EntityManager at {}",
        entity_manager.path.display()
    );
    println!("Current entity: {}", entity_manager.last);
}
