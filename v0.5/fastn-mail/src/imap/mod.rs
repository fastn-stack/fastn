//! IMAP functionality for fastn-mail
//!
//! This module provides both:
//! 1. IMAP server-side storage functions (for fastn-rig IMAP server implementation)  
//! 2. IMAP client functionality (for testing and dual verification)

// IMAP client functionality (network-based)
pub mod client;
pub mod commands;

// IMAP server-side storage functions (filesystem-based)
// These modules extend the Store impl with IMAP functionality
pub mod fetch;
pub mod list_folders;
pub mod search;
pub mod select_folder;  
pub mod store_flags;
pub mod thread;

pub use client::ImapClient;
pub use commands::{
    imap_connect_command,
    imap_fetch_command,
    imap_list_command,
    imap_test_pipeline_command,
};

/// IMAP client configuration
#[derive(Debug, Clone)]
pub struct ImapConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub starttls: bool,
}

impl ImapConfig {
    pub fn new(host: String, port: u16, username: String, password: String) -> Self {
        Self {
            host,
            port,
            username,
            password,
            starttls: false,
        }
    }

    pub fn with_starttls(mut self, enable: bool) -> Self {
        self.starttls = enable;
        self
    }
}