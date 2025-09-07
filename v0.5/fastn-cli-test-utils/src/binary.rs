//! Binary discovery and build management

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::process::Command;

/// Manages discovery and building of fastn workspace binaries
#[derive(Debug)]
pub struct BinaryManager {
    target_dir: PathBuf,
    binaries: HashMap<String, PathBuf>,
}

impl BinaryManager {
    /// Create new binary manager with automatic target directory detection
    pub fn new() -> Result<Self, eyre::Error> {
        let target_dir = Self::detect_target_dir()?;
        tracing::info!("Using target directory: {}", target_dir.display());
        
        Ok(Self {
            target_dir,
            binaries: HashMap::new(),
        })
    }
    
    /// Detect target directory using multiple fallback strategies
    pub fn detect_target_dir() -> Result<PathBuf, eyre::Error> {
        // Strategy 1: CARGO_TARGET_DIR environment variable
        if let Ok(target_dir) = std::env::var("CARGO_TARGET_DIR") {
            let path = PathBuf::from(target_dir);
            if path.exists() {
                return Ok(path);
            }
        }
        
        // Strategy 2: Workspace target (v0.5/target/debug)
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let workspace_root = manifest_dir
            .ancestors()
            .find(|p| p.join("Cargo.toml").exists() && p.join("fastn-rig").exists())
            .ok_or_else(|| eyre::eyre!("Could not find workspace root"))?;
        let v0_5_target = workspace_root.join("target/debug");
        
        // Strategy 3: Home target (~/target/debug) 
        let home_target = PathBuf::from(std::env::var("HOME").unwrap_or_default())
            .join("target/debug");
        
        // Strategy 4: Current directory target (./target/debug)
        let local_target = PathBuf::from("./target/debug");
        
        // Check in order of preference
        for candidate in [&v0_5_target, &home_target, &local_target] {
            if candidate.exists() && candidate.join("fastn-rig").exists() {
                return Ok(candidate.clone());
            }
        }
        
        // If none exist, default to workspace target (build will create it)
        Ok(v0_5_target)
    }
    
    /// Ensure specified binaries are built and available
    pub async fn ensure_built(&mut self, specs: &[BinarySpec]) -> Result<(), eyre::Error> {
        tracing::info!("Pre-building {} binaries...", specs.len());
        
        for spec in specs {
            let mut cmd = Command::new("cargo");
            cmd.arg("build");
            
            match spec {
                BinarySpec::Binary { package, binary } => {
                    if let Some(pkg) = package {
                        cmd.args(["--package", pkg]);
                    }
                    cmd.args(["--bin", binary]);
                }
                BinarySpec::Package { package, features } => {
                    cmd.args(["--package", package]);
                    if let Some(features) = features {
                        cmd.args(["--features", features]);
                    }
                }
            }
            
            let output = cmd.output().await?;
            if !output.status.success() {
                return Err(eyre::eyre!(
                    "Failed to build {spec:?}: {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
            
            // Cache binary path
            let binary_name = match spec {
                BinarySpec::Binary { binary, .. } => binary,
                BinarySpec::Package { package, .. } => package,
            };
            let binary_path = self.target_dir.join(binary_name);
            self.binaries.insert(binary_name.to_string(), binary_path);
        }
        
        Ok(())
    }
    
    /// Get path to a specific binary
    pub fn get_binary(&mut self, name: &str) -> Result<PathBuf, eyre::Error> {
        if let Some(path) = self.binaries.get(name) {
            if path.exists() {
                Ok(path.clone())
            } else {
                Err(eyre::eyre!("Binary {name} not found at {}", path.display()))
            }
        } else {
            // Try default location
            let path = self.target_dir.join(name);
            if path.exists() {
                self.binaries.insert(name.to_string(), path.clone());
                Ok(path)
            } else {
                Err(eyre::eyre!("Binary {name} not built or not found"))
            }
        }
    }
}

/// Specification for building a binary
#[derive(Debug, Clone)]
pub enum BinarySpec {
    Binary { 
        package: Option<String>, 
        binary: String 
    },
    Package { 
        package: String, 
        features: Option<String> 
    },
}

impl BinarySpec {
    pub fn binary(name: &str) -> Self {
        Self::Binary { 
            package: None, 
            binary: name.to_string() 
        }
    }
    
    pub fn package_binary(package: &str, binary: &str) -> Self {
        Self::Binary { 
            package: Some(package.to_string()), 
            binary: binary.to_string() 
        }
    }
    
    pub fn package_with_features(package: &str, features: &str) -> Self {
        Self::Package { 
            package: package.to_string(), 
            features: Some(features.to_string()) 
        }
    }
}