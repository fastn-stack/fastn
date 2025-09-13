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
                let cache_dir = dirs::cache_dir().ok_or_else(|| {
                    CacheError::DirectoryCreation(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "No system cache directory",
                    ))
                })?;

                // TODO: Move project ID generation from keys.rs
                cache_dir.join("fastn-project-specific")
            }
        };

        if !base_dir.exists() {
            std::fs::create_dir_all(&base_dir).map_err(CacheError::DirectoryCreation)?;
        }

        Ok(Self { base_dir })
    }

    /// Read cached value for given key
    pub fn get<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        let cache_file = self.get_cache_file_path(key);

        // Robust cache reading with error handling
        let cache_content = match std::fs::read_to_string(&cache_file) {
            Ok(content) => content,
            Err(_) => return Ok(None), // Cache miss
        };

        match serde_json::from_str::<T>(&cache_content) {
            Ok(value) => Ok(Some(value)),
            Err(e) => {
                // Cache corruption - remove and return miss
                eprintln!(
                    "Warning: Corrupted cache file {}, removing: {}",
                    cache_file.display(),
                    e
                );
                std::fs::remove_file(&cache_file).ok();
                Ok(None)
            }
        }
    }

    /// Write value to cache with given key
    pub fn set<T>(&self, key: &str, value: &T) -> Result<()>
    where
        T: serde::Serialize,
    {
        let cache_file = self.get_cache_file_path(key);

        // Create parent directory if needed
        if let Some(parent) = cache_file.parent() {
            std::fs::create_dir_all(parent).map_err(CacheError::DirectoryCreation)?;
        }

        // Serialize and write
        let content = serde_json::to_string(value).map_err(CacheError::Serialization)?;

        std::fs::write(&cache_file, content).map_err(CacheError::FileIO)?;

        Ok(())
    }

    pub fn clear_all(&self) -> Result<()> {
        if self.base_dir.exists() {
            std::fs::remove_dir_all(&self.base_dir).map_err(CacheError::FileIO)?;
            std::fs::create_dir_all(&self.base_dir).map_err(CacheError::DirectoryCreation)?;
        }
        Ok(())
    }

    /// Get cache file path for given key
    fn get_cache_file_path(&self, key: &str) -> PathBuf {
        self.base_dir.join(sanitize_cache_key(key))
    }

    /// Get cache directory for debugging
    pub fn cache_dir(&self) -> &PathBuf {
        &self.base_dir
    }
}

/// Sanitize cache key for filesystem safety
fn sanitize_cache_key(key: &str) -> String {
    key.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_")
}
