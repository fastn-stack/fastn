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
    
    /// Loads an entity by its ID52.
    ///
    /// The entity folder must be named by its ID52.
    ///
    /// # Arguments
    ///
    /// * `id52` - The ID52 of the entity to load
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Entity folder doesn't exist
    /// - Entity cannot be loaded (invalid keys, missing database, etc.)
    pub async fn load_entity(&self, id52: &str) -> eyre::Result<fastn_entity::Entity> {
        use eyre::WrapErr;
        
        let entity_path = self.path.join(id52);
        
        if !entity_path.exists() {
            return Err(eyre::eyre!("Entity folder for '{}' not found", id52));
        }
        
        if !entity_path.is_dir() {
            return Err(eyre::eyre!("Path for '{}' is not a directory", id52));
        }
        
        fastn_entity::Entity::load(&entity_path).await
            .wrap_err_with(|| format!("Failed to load entity '{}' from {entity_path:?}", id52))
    }
    
    /// Creates a new entity.
    ///
    /// The entity folder will be named by its generated ID52.
    /// Updates config.json with the new entity's online status.
    ///
    /// # Returns
    ///
    /// The newly created Entity.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Entity creation fails
    /// - Config update fails
    pub async fn create_entity(&mut self) -> eyre::Result<fastn_entity::Entity> {
        use eyre::WrapErr;
        
        // Create the entity (it creates its own folder named by ID52)
        let entity = fastn_entity::Entity::create(&self.path).await
            .wrap_err("Failed to create new entity")?;
        
        let id52 = entity.id52();
        
        // Add to online status map (new entities start online)
        self.online_status.insert(id52.clone(), true);
        
        // Update last to the newly created entity
        self.last = id52.clone();
        
        // Save updated config
        self.save_config()
            .wrap_err("Failed to update config.json after creating entity")?;
        
        tracing::info!("Created new entity: {}", id52);
        
        Ok(entity)
    }
    
    /// Sets the online status for an entity.
    ///
    /// Updates the in-memory map and persists to config.json.
    pub fn set_online(&mut self, id52: &str, online: bool) -> eyre::Result<()> {
        use eyre::WrapErr;
        
        self.online_status.insert(id52.to_string(), online);
        self.save_config()
            .wrap_err("Failed to save config after updating online status")?;
        
        tracing::info!("Set entity {} to {}", id52, if online { "online" } else { "offline" });
        Ok(())
    }
    
    /// Gets the online status for an entity.
    ///
    /// Returns false if the entity is not in the map.
    pub fn is_online(&self, id52: &str) -> bool {
        self.online_status.get(id52).copied().unwrap_or(false)
    }
    
    /// Sets the last used entity.
    ///
    /// Updates the in-memory field and persists to config.json.
    pub fn set_last(&mut self, id52: &str) -> eyre::Result<()> {
        use eyre::WrapErr;
        
        self.last = id52.to_string();
        self.save_config()
            .wrap_err("Failed to save config after updating last entity")?;
        
        tracing::info!("Set last entity to {}", id52);
        Ok(())
    }
    
    /// Gets the last used entity's ID52.
    pub fn last(&self) -> &str {
        &self.last
    }
    
    /// Save the current configuration to config.json.
    fn save_config(&self) -> eyre::Result<()> {
        use eyre::WrapErr;
        
        let config_path = self.path.join("config.json");
        let config_json = serde_json::to_string_pretty(&self)?;
        std::fs::write(&config_path, config_json)
            .wrap_err("Failed to write config.json")?;
        
        Ok(())
    }
}
