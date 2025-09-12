//! Dependency tracking for cache invalidation

use std::collections::{HashMap, HashSet};

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
    
    /// Check if any dependencies of a file have changed
    pub fn dependencies_changed(&self, file_id: &str) -> bool {
        if let Some(deps) = self.dependencies.get(file_id) {
            for dep_path in deps {
                if file_changed_since_cache(dep_path) {
                    return true;
                }
            }
        }
        false
    }
}

/// Check if a file has changed since it was cached
fn file_changed_since_cache(file_path: &str) -> bool {
    // TODO: Implement file modification time checking
    // Compare against cached modification time
    false
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