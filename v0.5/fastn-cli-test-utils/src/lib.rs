//! Comprehensive fastn CLI testing utilities
//!
//! This crate makes testing fastn commands pleasant by handling all the drudgery:
//! - Automatic binary discovery (workspace/home target flexibility)
//! - Process lifecycle with RAII cleanup (no more forgotten processes)
//! - All fastn command argument patterns (rig, mail, automerge, etc.)
//! - Peer management, SMTP ports, keyring handling
//! - Fluent API for readable test scenarios

use std::path::PathBuf;
use std::time::Duration;

pub mod simple;
pub mod fastn_rig;
pub mod fastn_mail;
pub mod test_env;

pub use simple::{
    get_binary_path, 
    get_fastn_rig_binary,
    get_fastn_mail_binary,
    detect_target_dir,
    ensure_built,
    CommandOutput,
};

pub use test_env::{FastnTestEnv, PeerHandle, EmailBuilder, EmailResult};
pub use fastn_rig::FastnRigCommand;
pub use fastn_mail::{FastnMailCommand, FastnMailSendBuilder};

/// fastn-specific test configuration
#[derive(Debug, Clone)]
pub struct FastnCliConfig {
    pub pre_build: bool,
    pub cleanup_on_drop: bool, 
    pub default_timeout: Duration,
    pub skip_keyring: bool,
    pub smtp_port_range: std::ops::Range<u16>,
}

impl Default for FastnCliConfig {
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