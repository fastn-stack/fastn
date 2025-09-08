//! IMAP client functionality for fastn-mail
//!
//! This module provides IMAP client capabilities for testing fastn email servers
//! and other IMAP servers. It includes dual verification testing to ensure
//! protocol results match filesystem reality.

pub mod client;
pub mod commands;

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