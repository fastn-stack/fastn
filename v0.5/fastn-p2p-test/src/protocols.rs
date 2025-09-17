//! Shared Protocol Definitions
//!
//! This module contains all the P2P protocols used by the example binaries.
//! By centralizing protocol definitions, we eliminate duplication and ensure
//! consistency across all examples.

use serde::{Deserialize, Serialize};

// ================================================================================================
// Echo Protocol - For basic P2P testing
// ================================================================================================

/// Simple echo protocol for basic P2P communication testing
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EchoProtocol {
    Echo,
}

impl std::fmt::Display for EchoProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "echo")
    }
}

/// Request message for echo protocol
#[derive(Debug, Serialize, Deserialize)]
pub struct EchoRequest {
    pub message: String,
}

/// Response message for echo protocol
#[derive(Debug, Serialize, Deserialize)]
pub struct EchoResponse {
    pub echo: String,
}

/// Error type for echo protocol
#[derive(Debug, Serialize, Deserialize)]
pub struct EchoError {
    pub error: String,
}

// ================================================================================================
// Helper Functions - Common operations across binaries
// ================================================================================================

/// Parse a secret key from string or generate a new one
pub fn get_or_generate_key(key_str: Option<&str>) -> eyre::Result<fastn_id52::SecretKey> {
    match key_str {
        Some(s) => s.parse().map_err(|e| eyre::eyre!("Invalid key: {}", e)),
        None => Ok(fastn_id52::SecretKey::generate()),
    }
}

/// Parse a public key (ID52) from string
pub fn parse_id52(id52_str: &str) -> eyre::Result<fastn_id52::PublicKey> {
    id52_str.parse().map_err(|e| eyre::eyre!("Invalid ID52: {}", e))
}

/// Initialize minimal logging for examples
pub fn init_logging() {
    tracing_subscriber::fmt()
        .with_env_filter("fastn_p2p=info")
        .with_target(false)
        .without_time()
        .init();
}