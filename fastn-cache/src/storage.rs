//! Storage implementation - moved from fastn-core/src/utils.rs

use crate::{CacheConfig, CacheError, Result};
use std::path::PathBuf;

pub struct CacheStorage {
    base_dir: PathBuf,
}

impl CacheStorage {
    pub fn new(config: &CacheConfig) -> Result<Self> {
        let base_dir = match &config.cache_dir {
            Some(dir) => dir.clone(),
            None => {
                // Use system cache directory with project-specific naming
                let cache_dir = dirs::cache_dir()
                    .ok_or_else(|| CacheError::DirectoryCreation(
                        std::io::Error::new(std::io::ErrorKind::NotFound, "No system cache directory")
                    ))?;
                
                // TODO: Move project ID generation from keys.rs
                cache_dir.join("fastn-project-specific")
            }
        };
        
        if !base_dir.exists() {
            std::fs::create_dir_all(&base_dir)
                .map_err(CacheError::DirectoryCreation)?;
        }
        
        Ok(Self { base_dir })
    }
    
    pub fn clear_all(&self) -> Result<()> {
        if self.base_dir.exists() {
            std::fs::remove_dir_all(&self.base_dir)
                .map_err(CacheError::FileIO)?;
            std::fs::create_dir_all(&self.base_dir)
                .map_err(CacheError::DirectoryCreation)?;
        }
        Ok(())
    }
}