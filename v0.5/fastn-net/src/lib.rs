//! # fastn-net
//!
//! Network utilities and P2P communication between fastn entities.
//!
//! This crate provides P2P networking capabilities for fastn entities using Iroh.
//! Each fastn instance is called an "entity" in the P2P network, identified by
//! its unique ID52 (a 52-character encoded Ed25519 public key).
//!
//! ## Example
//!
//! ```ignore
//! use fastn_net::{global_iroh_endpoint, ping};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Get the global Iroh endpoint for this entity
//! let endpoint = global_iroh_endpoint().await;
//!
//! // Connect to another entity (requires valid entity ID52)
//! let entity_id = "entity_id52_here";
//! let connection = /* establish connection to entity */;
//!
//! // Ping the connection
//! ping(&connection).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Supported Protocols
//!
//! - [`Protocol::Ping`] - Test connectivity between entities
//! - [`Protocol::Http`] - Proxy HTTP requests through entities
//! - [`Protocol::Tcp`] - Tunnel TCP connections between entities
//! - [`Protocol::Socks5`] - SOCKS5 proxy support

extern crate self as fastn_net;

pub mod dot_fastn;
pub mod get_endpoint;
mod get_stream;
mod graceful;
pub mod http;
mod http_connection_manager;
mod http_to_peer;
mod peer_to_http;
mod ping;
pub mod protocol;
mod secret;
mod tcp;
mod utils;
mod utils_iroh;
pub mod errors;

pub use get_endpoint::get_endpoint;
pub use get_stream::{PeerStreamSenders, get_stream};
pub use graceful::Graceful;
pub use http::ProxyResult;
pub use http_connection_manager::{HttpConnectionManager, HttpConnectionPool, HttpConnectionPools};
pub use http_to_peer::{http_to_peer, http_to_peer_non_streaming};
pub use peer_to_http::peer_to_http;
pub use ping::{PONG, ping};
pub use protocol::{APNS_IDENTITY, Protocol, ProtocolHeader};
pub use secret::{
    SECRET_KEY_FILE, generate_and_save_key, generate_secret_key, get_secret_key, read_or_create_key,
};
pub use tcp::{peer_to_tcp, pipe_tcp_stream_over_iroh, tcp_to_peer};
pub use utils::mkdir;
pub use utils_iroh::{
    accept_bi, accept_bi_with, get_remote_id52, global_iroh_endpoint, next_json, next_string,
};

// Deprecated helper functions - use fastn_id52 directly
pub use utils::{id52_to_public_key, public_key_to_id52};

/// Map of entity IDs to their port and endpoint.
///
/// Stores tuples of (ID52 prefix, (port, Iroh endpoint)) for entity lookup.
/// Uses a Vec instead of HashMap because:
/// - We need prefix matching for shortened IDs in subdomains
/// - The number of entities is typically small per instance
/// - Linear search with prefix matching is fast enough
///
/// The ID52 strings may be truncated when used in DNS subdomains due to the
/// 63-character limit, so prefix matching allows finding the correct entity.
pub type IDMap = std::sync::Arc<tokio::sync::Mutex<Vec<(String, (u16, iroh::Endpoint))>>>;

/// Acknowledgment message used in protocol handshakes.
pub const ACK: &str = "ack";
