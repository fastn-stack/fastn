//! Unified CLI testing utilities for fastn workspace
//!
//! This crate provides common functionality for testing CLI tools across the fastn workspace,
//! eliminating duplication of binary discovery, process management, and test environment setup.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;

pub mod binary;
pub mod environment; 
pub mod process;
pub mod fluent;
pub mod examples;

pub use binary::BinaryManager;
pub use environment::{TestEnvironment, PeerHandle};
pub use process::{ProcessManager, ProcessHandle};
pub use fluent::{TestScenario, TestRunner, EmailBuilder, EmailResult};

/// Standard test configuration for fastn CLI tools
#[derive(Debug, Clone)]
pub struct CliConfig {
    pub pre_build: bool,
    pub cleanup_on_drop: bool, 
    pub default_timeout: Duration,
    pub skip_keyring: bool,
    pub smtp_port_range: std::ops::Range<u16>,
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            pre_build: true,
            cleanup_on_drop: true,
            default_timeout: Duration::from_secs(30),
            skip_keyring: true,
            smtp_port_range: 2525..2600,
        }
    }
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
    pub fn expect_success(self) -> Result<Self, eyre::Error> {
        if self.success {
            Ok(self)
        } else {
            Err(eyre::eyre!(
                "Command failed with exit code {:?}\nstdout: {}\nstderr: {}",
                self.exit_code,
                self.stdout,
                self.stderr
            ))
        }
    }
    
    pub fn expect_failure(self) -> Result<Self, eyre::Error> {
        if !self.success {
            Ok(self)
        } else {
            Err(eyre::eyre!(
                "Command unexpectedly succeeded\nstdout: {}",
                self.stdout
            ))
        }
    }
    
    /// Extract account ID from fastn-rig init output
    pub fn extract_account_id(&self) -> Result<String, eyre::Error> {
        for line in self.stdout.lines() {
            if line.contains("Account ID:") {
                if let Some(id) = line.split_whitespace().nth(2) {
                    return Ok(id.to_string());
                }
            }
        }
        Err(eyre::eyre!("Account ID not found in output: {}", self.stdout))
    }
    
    /// Extract password from fastn-rig init output  
    pub fn extract_password(&self) -> Result<String, eyre::Error> {
        for line in self.stdout.lines() {
            if line.contains("Password:") {
                if let Some(password) = line.split_whitespace().nth(1) {
                    return Ok(password.to_string());
                }
            }
        }
        Err(eyre::eyre!("Password not found in output: {}", self.stdout))
    }
    
    pub fn contains_output(&self, text: &str) -> bool {
        self.stdout.contains(text) || self.stderr.contains(text)
    }
}

/// Parameters for send-mail command
#[derive(Debug, Clone)]
pub struct SendMailParams {
    pub from: String,
    pub to: String,
    pub subject: String,
    pub body: String,
    pub smtp_port: u16,
    pub password: String,
    pub fastn_home: PathBuf,
}