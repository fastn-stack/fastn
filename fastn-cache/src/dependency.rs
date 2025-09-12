//! Dependency tracking for cache invalidation

use std::collections::{HashMap, HashSet};
use std::time::SystemTime;

/// Tracks file dependencies for intelligent cache invalidation
pub struct DependencyTracker {
    /// Maps file -> list of files it depends on
    dependencies: HashMap<String, Vec<String>>,
    /// Maps file -> list of files that depend on it (reverse index)
    dependents: HashMap<String, HashSet<String>>,
}

impl DependencyTracker {
    pub fn new() -> Self {
        Self {
            dependencies: HashMap::new(),
            dependents: HashMap::new(),
        }
    }
    
    /// Record that a file depends on other files
    pub fn record_dependencies(&mut self, file_id: &str, deps: &[String]) {
        // Store forward dependencies
        self.dependencies.insert(file_id.to_string(), deps.to_vec());
        
        // Update reverse dependencies (dependents)
        for dep in deps {
            self.dependents
                .entry(dep.clone())
                .or_default()
                .insert(file_id.to_string());
        }
    }
    
    /// Get all files that depend on the given file (directly or indirectly)
    pub fn get_affected_files(&self, changed_file: &str) -> Vec<String> {
        let mut affected = HashSet::new();
        let mut to_check = vec![changed_file.to_string()];
        
        while let Some(file) = to_check.pop() {
            if let Some(dependents) = self.dependents.get(&file) {
                for dependent in dependents {
                    if affected.insert(dependent.clone()) {
                        to_check.push(dependent.clone());
                    }
                }
            }
        }
        
        affected.into_iter().collect()
    }
    
    /// Check if any dependencies of a file have changed since given time
    pub fn dependencies_changed_since(&self, file_id: &str, cache_time: SystemTime) -> bool {
        if let Some(deps) = self.dependencies.get(file_id) {
            for dep_path in deps {
                if file_changed_since_cache(dep_path, cache_time) {
                    return true;
                }
            }
        }
        false
    }
    
    /// Generate dependency-aware hash for cache validation
    pub fn generate_cache_hash(&self, file_id: &str, source: &str) -> String {
        let dependencies = self.dependencies.get(file_id)
            .map(|deps| deps.as_slice())
            .unwrap_or(&[]);
        
        generate_dependency_hash(source, dependencies)
    }
}

/// Check if a file has changed by comparing modification times
fn file_changed_since_cache(file_path: &str, cached_time: SystemTime) -> bool {
    match std::fs::metadata(file_path) {
        Ok(metadata) => {
            match metadata.modified() {
                Ok(current_time) => current_time > cached_time,
                Err(_) => true, // If we can't get time, assume changed
            }
        }
        Err(_) => true, // If file doesn't exist, assume changed
    }
}

/// Generate hash that includes all dependency file contents
pub fn generate_dependency_hash(source: &str, dependencies: &[String]) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    source.hash(&mut hasher);
    
    // Include all dependency file contents in the hash
    for dep_path in dependencies {
        if let Ok(dep_content) = std::fs::read_to_string(dep_path) {
            dep_content.hash(&mut hasher);
        } else {
            // If dependency file doesn't exist, include its path in hash
            // This ensures cache invalidation if file appears/disappears
            dep_path.hash(&mut hasher);
        }
    }
    
    // CRITICAL: Include .packages directory state for fastn update resilience
    if let Ok(packages_dir) = std::fs::read_dir(".packages") {
        for entry in packages_dir.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // Include package directory modification time
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        format!("{:?}", modified).hash(&mut hasher);
                    }
                }
            }
        }
    }
    
    // Include FASTN.ftd content for configuration changes
    if let Ok(fastn_content) = std::fs::read_to_string("FASTN.ftd") {
        fastn_content.hash(&mut hasher);
    }
    
    format!("{:x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_dependency_tracking() {
        let mut tracker = DependencyTracker::new();
        
        // index.ftd depends on hero.ftd and banner.ftd
        tracker.record_dependencies("index.ftd", &["hero.ftd", "banner.ftd"]);
        
        // hero.ftd depends on common.ftd
        tracker.record_dependencies("hero.ftd", &["common.ftd"]);
        
        // If common.ftd changes, both hero.ftd and index.ftd are affected
        let affected = tracker.get_affected_files("common.ftd");
        assert!(affected.contains(&"hero.ftd".to_string()));
        assert!(affected.contains(&"index.ftd".to_string()));
    }
}