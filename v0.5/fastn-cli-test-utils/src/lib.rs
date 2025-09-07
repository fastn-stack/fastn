//! Generic CLI testing utilities for any Rust workspace
//!
//! This crate provides common functionality for testing CLI tools, focusing on:
//! - Binary discovery across flexible target directory configurations  
//! - Process lifecycle management with RAII cleanup
//! - Generic command execution and output parsing
//!
//! Application-specific logic (like fastn-rig's peers, SMTP ports, keyring) should be
//! built on top of these generic utilities rather than included here.

use std::path::PathBuf;
use std::time::Duration;

pub mod simple;

pub use simple::{
    get_binary_path, 
    get_fastn_rig_binary,
    get_fastn_mail_binary,
    detect_target_dir,
    ensure_built,
    CommandOutput,
};

/// Generic configuration for CLI testing
#[derive(Debug, Clone)]
pub struct CliConfig {
    pub pre_build: bool,
    pub cleanup_on_drop: bool, 
    pub default_timeout: Duration,
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            pre_build: true,
            cleanup_on_drop: true,
            default_timeout: Duration::from_secs(30),
        }
    }
}