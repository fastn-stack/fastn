use fastn_entity::EntityManager;

#[tokio::test]
async fn test_entity_manager_lifecycle() -> eyre::Result<()> {
    // Use file storage for tests (not keyring)
    unsafe {
        std::env::set_var("SKIP_KEYRING", "true");
    }

    // Create a temporary directory for testing
    let temp_dir = tempfile::tempdir()?;
    let path = temp_dir.path().to_path_buf();

    // Create EntityManager (should create default entity)
    let mut manager = EntityManager::new(Some(path.clone())).await?;

    // Verify default entity was created
    let last_id52 = manager.last().to_string();
    assert!(!last_id52.is_empty());
    assert_eq!(last_id52.len(), 52); // ID52 should be 52 characters

    // Verify entity folder exists
    let entity_path = path.join(&last_id52);
    assert!(entity_path.exists());
    assert!(entity_path.is_dir());

    // Verify config.json was created
    let config_path = path.join("config.json");
    assert!(config_path.exists());

    // Load the default entity
    let entity = manager.default_entity().await?;
    assert_eq!(entity.id52(), last_id52);

    // Create another entity
    let new_entity = manager.create_entity().await?;
    let new_id52 = new_entity.id52();
    assert_ne!(new_id52, last_id52);

    // Verify the new entity became the last
    assert_eq!(manager.last(), new_id52);

    // Check online status
    assert!(manager.is_online(&new_id52));
    assert!(manager.is_online(&last_id52));

    // Set one offline
    manager.set_online(&last_id52, false)?;
    assert!(!manager.is_online(&last_id52));
    assert!(manager.is_online(&new_id52));

    // Reload EntityManager from same directory
    let manager2 = EntityManager::new(Some(path)).await?;
    assert_eq!(manager2.last(), new_id52);
    assert!(!manager2.is_online(&last_id52));
    assert!(manager2.is_online(&new_id52));

    Ok(())
}

#[tokio::test]
async fn test_entity_manager_strict_mode() -> eyre::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let path = temp_dir.path().to_path_buf();

    // Create a non-empty directory without config.json
    std::fs::create_dir_all(&path)?;
    std::fs::write(path.join("random_file.txt"), "some content")?;

    // Should fail in strict mode
    let result = EntityManager::new(Some(path)).await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("non-empty but lacks config.json")
    );

    Ok(())
}
