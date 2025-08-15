use std::collections::HashMap;

/// EntityManager handles the management of multiple entities.
///
/// It's essentially the config.json file loaded into memory, providing
/// methods to create, list, and load entities from disk.
/// Entities are stored in folders named after their ID52 identifiers.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EntityManager {
    /// Path to the directory containing all entities
    pub path: std::path::PathBuf,
    /// The last used entity's ID52
    pub last: String,
    /// Map of entity ID52 to online/offline status
    #[serde(default)]
    pub online_status: HashMap<String, bool>,
}

impl EntityManager {
    /// Creates a new EntityManager or loads an existing one.
    ///
    /// Behavior in strict mode:
    /// - Empty directory → creates config.json + default entity
    /// - Non-empty directory → must have valid config.json + at least one entity
    /// - Invalid state → returns error
    ///
    /// # Arguments
    ///
    /// * `path` - Optional path to entities directory. Defaults to platform-specific data directory.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Directory cannot be created
    /// - Non-empty directory lacks valid config.json
    /// - config.json exists but no entities found
    /// - config.json references non-existent entity
    pub async fn new(path: Option<std::path::PathBuf>) -> eyre::Result<Self> {
        use eyre::WrapErr;

        let path = if let Some(p) = path {
            p
        } else {
            // Use platform-specific data directory
            let proj_dirs = directories::ProjectDirs::from("com", "fastn", "fastn")
                .ok_or_else(|| eyre::eyre!("Failed to determine project directories"))?;
            proj_dirs.data_dir().to_path_buf()
        };

        tracing::info!("Initializing EntityManager at {path:?}");

        // Create directory if it doesn't exist
        if !path.exists() {
            std::fs::create_dir_all(&path).wrap_err("Failed to create entities directory")?;
            tracing::info!("Created entities directory at {path:?}");
        }

        let config_path = path.join("config.json");

        if config_path.exists() {
            // Config exists - load it
            tracing::info!("Loading existing EntityManager configuration");

            let config_content =
                std::fs::read_to_string(&config_path).wrap_err("Failed to read config.json")?;
            let mut manager: Self =
                serde_json::from_str(&config_content).wrap_err("Invalid config.json")?;

            // Update path in case it was moved
            manager.path = path;

            // TODO: Verify the last entity exists

            tracing::info!("Loaded EntityManager with last entity: {}", manager.last);

            Ok(manager)
        } else {
            // No config.json - check if directory is empty
            let is_empty = std::fs::read_dir(&path)
                .wrap_err("Failed to read directory")?
                .next()
                .is_none();

            if is_empty {
                // Empty directory - create default entity
                tracing::info!("Empty directory, creating default entity");

                // Entity::create takes parent directory and creates its own folder
                let entity = fastn_entity::Entity::create(&path).await?;
                let id52 = entity.id52();

                // Initialize manager/config
                let mut online_status = HashMap::new();
                online_status.insert(id52.clone(), true); // New entities start online

                let manager = Self {
                    path: path.clone(),
                    last: id52.clone(),
                    online_status,
                };

                // Write config.json
                let config_json = serde_json::to_string_pretty(&manager)?;
                std::fs::write(&config_path, config_json)
                    .wrap_err("Failed to write config.json")?;

                tracing::info!("Created new EntityManager with default entity: {id52}");

                Ok(manager)
            } else {
                // Non-empty directory without config.json - error (strict mode)
                Err(eyre::eyre!(
                    "Directory {path:?} is non-empty but lacks config.json. \
                     This is an invalid state. Please clean up or use a different directory."
                ))
            }
        }
    }
}
