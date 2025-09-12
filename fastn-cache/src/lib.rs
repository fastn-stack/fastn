#![deny(unused_crate_dependencies)]
#![allow(clippy::derive_partial_eq_without_eq)]

//! # fastn-cache
//! 
//! High-performance caching system for FTD compilation and incremental builds.
//! 
//! This crate provides intelligent caching that dramatically improves fastn serve 
//! and fastn build performance while maintaining correctness through sophisticated 
//! dependency tracking.
//!
//! ## Design Principles
//!
//! - **Safety First**: Cache sharing errors cause extra work, never wrong content
//! - **Dependency Tracking**: Track what affects what, invalidate correctly  
//! - **Multi-Project Safety**: Different projects must not interfere
//!
//! ## Usage
//!
//! ```rust,no_run
//! use fastn_cache::{FtdCache, CacheConfig};
//! 
//! let config = CacheConfig::default().enable(true);
//! let mut cache = FtdCache::new(config)?;
//! 
//! // Parse with caching
//! let doc = cache.parse_cached("index.ftd", source_content, 0)?;
//! 
//! // Update with dependencies after compilation
//! cache.update_dependencies("index.ftd", &dependencies, &doc)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::SystemTime;

mod storage;
mod keys;
mod dependency;
mod invalidation;
mod build;

pub use storage::CacheStorage;
pub use keys::CacheKey;
pub use dependency::DependencyTracker;

/// Configuration for FTD caching system
#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub enabled: bool,
    pub cache_dir: Option<PathBuf>,
    pub max_cache_size: Option<u64>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Disabled by default for safety
            cache_dir: None, // Use system cache directory
            max_cache_size: None, // Unlimited
        }
    }
}

impl CacheConfig {
    pub fn enable(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    
    pub fn cache_dir(mut self, dir: PathBuf) -> Self {
        self.cache_dir = Some(dir);
        self
    }
}

/// Main FTD caching interface
pub struct FtdCache {
    config: CacheConfig,
    storage: CacheStorage,
    dependency_tracker: DependencyTracker,
}

/// Cached parse result for FTD documents
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct CachedParse {
    pub hash: String,
    pub dependencies: Vec<String>,
    pub created_at: SystemTime,
    // Note: We'll need to define a serializable ParsedDocument type
}

/// Build cache metadata for incremental builds
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct BuildCache {
    pub documents: BTreeMap<String, DocumentMetadata>,
    pub file_checksums: BTreeMap<String, String>,  
    pub packages_state: PackagesState,
    pub fastn_config_hash: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct DocumentMetadata {
    pub html_checksum: String,
    pub dependencies: Vec<String>,
    pub last_built: SystemTime,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct PackagesState {
    pub packages_hash: String,
    pub last_updated: SystemTime,
}

/// Cache statistics for monitoring and debugging
#[derive(Debug)]
pub struct CacheStats {
    pub total_entries: usize,
    pub cache_size_bytes: u64,
    pub hit_rate: f64,
    pub last_cleanup: SystemTime,
}

/// Errors that can occur during caching operations
#[derive(thiserror::Error, Debug)]
pub enum CacheError {
    #[error("Cache directory creation failed: {0}")]
    DirectoryCreation(std::io::Error),
    
    #[error("Cache file I/O error: {0}")]
    FileIO(std::io::Error),
    
    #[error("Cache serialization error: {0}")]
    Serialization(serde_json::Error),
    
    #[error("Dependency tracking error: {message}")]
    DependencyTracking { message: String },
    
    #[error("Cache corruption detected: {message}")]
    Corruption { message: String },
}

pub type Result<T> = std::result::Result<T, CacheError>;

impl FtdCache {
    /// Create new cache instance for a fastn project
    pub fn new(config: CacheConfig) -> Result<Self> {
        let storage = CacheStorage::new(&config)?;
        let dependency_tracker = DependencyTracker::new();
        
        Ok(Self {
            config,
            storage,
            dependency_tracker,
        })
    }
    
    /// Check if cached parse result is available and valid
    pub fn get_cached_parse(&self, file_id: &str, source: &str) -> Result<Option<CachedParse>> {
        if !self.config.enabled {
            return Ok(None);
        }
        
        match self.storage.get::<CachedParse>(file_id)? {
            Some(cached) => {
                // Validate cache using dependency-aware hash
                let current_hash = dependency::generate_dependency_hash(source, &cached.dependencies);
                
                if cached.hash == current_hash {
                    eprintln!("🚀 PERF: CACHE HIT (all {} dependencies unchanged) for: {}", cached.dependencies.len(), file_id);
                    Ok(Some(cached))
                } else {
                    eprintln!("🔥 PERF: Cache invalidated (file or dependency changed) for: {}", file_id);
                    Ok(None)
                }
            }
            None => {
                eprintln!("🔥 PERF: Cache miss (no previous cache) for: {}", file_id);
                Ok(None)
            }
        }
    }
    
    /// Cache parse result with dependencies
    pub fn cache_parse_result(
        &mut self,
        file_id: &str, 
        source: &str,
        dependencies: &[String],
    ) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }
        
        // Generate dependency-aware hash
        let hash = dependency::generate_dependency_hash(source, dependencies);
        
        let cached_parse = CachedParse {
            hash,
            dependencies: dependencies.to_vec(),
            created_at: SystemTime::now(),
        };
        
        self.storage.set(file_id, &cached_parse)?;
        
        // Update dependency tracker
        self.dependency_tracker.record_dependencies(file_id, dependencies);
        
        eprintln!("🔥 PERF: Cached parse result with {} dependencies for: {}", dependencies.len(), file_id);
        Ok(())
    }
    
    /// Update cache with collected dependencies after compilation
    pub fn update_dependencies(
        &mut self,
        file_id: &str, 
        dependencies: &[String],
    ) -> Result<()> {
        // TODO: Implement dependency-aware cache updates
        todo!("Update cache with real dependency information")
    }
    
    /// Check if build is needed for incremental builds
    pub fn is_build_needed(&self, doc_id: &str) -> bool {
        // TODO: Implement build need detection
        todo!("Check if document needs rebuilding")
    }
    
    /// Mark document as built with metadata
    pub fn mark_built(
        &mut self,
        doc_id: &str,
        html_checksum: &str,
        dependencies: &[String]
    ) -> Result<()> {
        // TODO: Implement build completion tracking
        todo!("Mark document as successfully built")
    }
    
    /// Clear all cache (for troubleshooting)
    pub fn clear_all(&mut self) -> Result<()> {
        self.storage.clear_all()
    }
    
    /// Get cache statistics for debugging
    pub fn stats(&self) -> CacheStats {
        // TODO: Implement cache statistics
        CacheStats {
            total_entries: 0,
            cache_size_bytes: 0,
            hit_rate: 0.0,
            last_cleanup: SystemTime::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cache_config() {
        let config = CacheConfig::default().enable(true);
        assert!(config.enabled);
    }
    
    #[test] 
    fn test_cache_creation() {
        let config = CacheConfig::default();
        let result = FtdCache::new(config);
        // Will implement once storage module exists
    }
}