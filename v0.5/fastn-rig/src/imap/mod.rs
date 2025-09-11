//! IMAP server implementation for fastn-rig
//!
//! Provides IMAP4rev1 server functionality with STARTTLS support.

pub mod protocol;
pub mod server;
pub mod session;

pub use server::start_imap_server;

/// IMAP server configuration
pub struct ImapConfig {
    pub port: u16,
    pub max_connections: usize,
}

impl Default for ImapConfig {
    fn default() -> Self {
        Self {
            port: 1143, // Unprivileged default
            max_connections: 100,
        }
    }
}
