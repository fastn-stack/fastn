//! Simple, generic utilities for CLI testing

use std::path::PathBuf;
use std::process::Command;

/// Get path to any binary with automatic building and flexible target directory support
pub fn get_binary_path(binary_name: &str) -> PathBuf {
    let target_dir = detect_target_dir();
    let binary_path = target_dir.join(binary_name);

    // If binary doesn't exist, try building the common package
    if !binary_path.exists() {
        let _ = ensure_built(&[binary_name]);
    }

    if !binary_path.exists() {
        panic!(
            "Binary {binary_name} not found at {}",
            binary_path.display()
        );
    }

    binary_path
}

/// Get path to fastn-rig binary (convenience helper for common case)
pub fn get_fastn_rig_binary() -> PathBuf {
    get_binary_path("fastn-rig")
}

/// Get path to fastn-mail binary (convenience helper for common case)  
pub fn get_fastn_mail_binary() -> PathBuf {
    // Always ensure fastn-mail is built with net feature for SMTP client support
    let _ = ensure_built(&["fastn-mail"]);
    get_binary_path("fastn-mail")
}

/// Build specified binaries using cargo
pub fn ensure_built(binary_names: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    for binary_name in binary_names {
        let output = match *binary_name {
            "fastn-rig" => Command::new("cargo")
                .args(["build", "-p", "fastn-rig", "--bin", "fastn-rig"])
                .output()?,
            "fastn-mail" => Command::new("cargo")
                .args(["build", "--package", "fastn-mail", "--features", "net"])
                .output()?,
            "test_utils" => Command::new("cargo")
                .args(["build", "--bin", "test_utils"])
                .output()?,
            _ => Command::new("cargo")
                .args(["build", "--bin", binary_name])
                .output()?,
        };

        if !output.status.success() {
            return Err(format!(
                "Failed to build {binary_name}: {}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }
    }
    Ok(())
}

/// Output from a CLI command execution  
#[derive(Debug)]
pub struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
    pub exit_code: Option<i32>,
}

impl CommandOutput {
    pub fn expect_success(self) -> Result<Self, Box<dyn std::error::Error>> {
        if self.success {
            Ok(self)
        } else {
            Err(format!(
                "Command failed with exit code {:?}\nstdout: {}\nstderr: {}",
                self.exit_code, self.stdout, self.stderr
            )
            .into())
        }
    }

    pub fn expect_failure(self) -> Result<Self, Box<dyn std::error::Error>> {
        if !self.success {
            Ok(self)
        } else {
            Err(format!("Command unexpectedly succeeded\nstdout: {}", self.stdout).into())
        }
    }

    pub fn contains_output(&self, text: &str) -> bool {
        self.stdout.contains(text) || self.stderr.contains(text)
    }

    /// Extract account ID from fastn-rig init output
    pub fn extract_account_id(&self) -> Result<String, Box<dyn std::error::Error>> {
        for line in self.stdout.lines() {
            // Check for "Primary account:" format first
            if line.contains("Primary account:") {
                if let Some(id) = line.split("Primary account:").nth(1) {
                    return Ok(id.trim().to_string());
                }
            }
            // Fallback to old "Account ID:" format
            if line.contains("Account ID:") {
                if let Some(id) = line.split_whitespace().nth(2) {
                    return Ok(id.to_string());
                }
            }
        }
        Err("Account ID not found in output".into())
    }

    /// Extract password from fastn-rig init output
    pub fn extract_password(&self) -> Result<String, Box<dyn std::error::Error>> {
        for line in self.stdout.lines() {
            if line.contains("Password:") {
                if let Some(password) = line.split_whitespace().nth(1) {
                    return Ok(password.to_string());
                }
            }
        }
        Err("Password not found in output".into())
    }
}

/// Detect target directory across multiple possible locations
pub fn detect_target_dir() -> PathBuf {
    // Strategy 1: CARGO_TARGET_DIR environment variable
    if let Ok(target_dir) = std::env::var("CARGO_TARGET_DIR") {
        let path = PathBuf::from(target_dir);
        if path.exists() {
            return path.join("debug");
        }
    }

    // Strategy 2: Workspace target (v0.5/target/debug)
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .ancestors()
        .find(|p| p.join("Cargo.toml").exists() && p.join("fastn-rig").exists())
        .expect("Could not find workspace root");
    let v0_5_target = workspace_root.join("target/debug");

    // Strategy 3: Home target (~/target/debug)
    let home_target = PathBuf::from(std::env::var("HOME").unwrap_or_default()).join("target/debug");

    // Strategy 4: Current directory target (./target/debug)
    let local_target = PathBuf::from("./target/debug");

    // Strategy 5: Hardcoded legacy path
    let legacy_target = PathBuf::from("/Users/amitu/target/debug");

    // Check in order of preference
    for candidate in [&v0_5_target, &home_target, &local_target, &legacy_target] {
        if candidate.exists() {
            return candidate.clone();
        }
    }

    // If none exist, default to workspace target (build will create it)
    v0_5_target
}
