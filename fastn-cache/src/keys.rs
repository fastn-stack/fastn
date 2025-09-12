//! Cache key generation - handles project identification and cache directory naming

use std::path::PathBuf;

/// Represents a cache key for FTD compilation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheKey {
    pub project_id: String,
    pub file_id: String,
}

impl CacheKey {
    /// Generate cache key for a specific FTD file
    pub fn for_file(file_id: &str) -> Self {
        Self {
            project_id: generate_project_id(),
            file_id: file_id.to_string(),
        }
    }
    
    /// Convert to filesystem-safe string
    pub fn to_string(&self) -> String {
        format!("{}+{}", 
            sanitize_for_filesystem(&self.project_id),
            sanitize_for_filesystem(&self.file_id)
        )
    }
}

/// Generate unique project identifier
fn generate_project_id() -> String {
    let current_dir = std::env::current_dir()
        .expect("Cannot read current directory");
    let fastn_ftd_path = current_dir.join("FASTN.ftd");
    
    if !fastn_ftd_path.exists() {
        // No FASTN.ftd - use directory name
        return format!("no-config-{}", 
            current_dir.file_name()
                .unwrap_or_default()
                .to_string_lossy()
        );
    }
    
    let fastn_content = std::fs::read_to_string(&fastn_ftd_path)
        .unwrap_or_default();
    
    // Extract package name
    let package_name = fastn_content
        .lines()
        .find(|line| line.trim_start().starts_with("-- fastn.package:"))
        .and_then(|line| line.split(':').nth(1))
        .map(|name| name.trim())
        .unwrap_or("unnamed");
    
    // Get git repo info for stable identification
    if let Ok(output) = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(&current_dir)
        .output()
    {
        if output.status.success() {
            let git_root = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let git_root_path = std::path::Path::new(&git_root);
            
            // Get repo name
            let repo_name = get_git_repo_name(&current_dir)
                .unwrap_or_else(|| {
                    git_root_path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string()
                });
            
            // Get relative path from git root
            let relative_path = current_dir
                .strip_prefix(git_root_path)
                .map(|rel| rel.join("FASTN.ftd"))
                .unwrap_or_else(|_| PathBuf::from("FASTN.ftd"));
            
            // Format: {repo}+{relative-path}+{package}
            return format!("{}+{}+{}", 
                repo_name,
                relative_path.to_string_lossy().replace(['/', '\\'], "_"),
                package_name
            );
        }
    }
    
    // Not a git repo - use directory name
    format!("{}+{}", 
        current_dir.file_name()
            .unwrap_or_default()
            .to_string_lossy(),
        package_name
    )
}

fn get_git_repo_name(current_dir: &std::path::Path) -> Option<String> {
    let output = std::process::Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(current_dir)
        .output()
        .ok()?;
    
    if output.status.success() {
        let url = String::from_utf8_lossy(&output.stdout);
        return url.trim()
            .split('/')
            .last()?
            .trim_end_matches(".git")
            .to_string()
            .into();
    }
    
    None
}

fn sanitize_for_filesystem(s: &str) -> String {
    s.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_")
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cache_key_generation() {
        let key = CacheKey::for_file("index.ftd");
        assert!(!key.project_id.is_empty());
        assert_eq!(key.file_id, "index.ftd");
    }
    
    #[test]
    fn test_filesystem_sanitization() {
        assert_eq!(sanitize_for_filesystem("path/with\\colon:"), "path_with_colon_");
    }
}