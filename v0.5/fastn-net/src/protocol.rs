//! Protocol multiplexing over P2P connections.
//!
//! This module implements a custom protocol multiplexing system over Iroh P2P
//! connections, deliberately deviating from Iroh's recommended ALPN-per-protocol
//! approach.
//!
//! # Why Not Use Iroh's Built-in ALPN Feature?
//!
//! Iroh [recommends using different ALPNs](https://docs.rs/iroh/latest/iroh/endpoint/struct.Builder.html#method.alpns)
//! for different protocols. However, this approach has a significant limitation:
//! **each protocol requires a separate connection**.
//!
//! ## The Problem with Multiple Connections
//!
//! Consider a typical P2P session where an entity might:
//! - Send periodic pings to check connection health
//! - Proxy HTTP requests through another entity
//! - Tunnel TCP connections simultaneously
//! - Stream real-time data (e.g., during a call while browsing shared files)
//!
//! With Iroh's approach, each protocol would need its own connection, requiring
//! a full TLS handshake for each. ALPN is negotiated during the TLS handshake:
//!
//! ```text
//! Client Hello Message Structure:
//! ┌─────────────────────────────────────┐
//! │ Handshake Type: Client Hello (1)    │
//! │ Version: TLS 1.2 (0x0303)          │
//! │ Random: dd67b5943e5efd07...        │
//! │ Cipher Suites: [...]                │
//! │ Extensions:                         │
//! │   ALPN Extension:                   │
//! │     - h2                           │
//! │     - http/1.1                     │
//! └─────────────────────────────────────┘
//! ```
//!
//! Creating additional connections means additional:
//! - TLS handshakes (expensive cryptographic operations)
//! - Network round trips
//! - Memory overhead for connection state
//! - Complexity in connection management
//!
//! ## Our Solution: Application-Layer Multiplexing
//!
//! We use a single ALPN (`/fastn/entity/0.1`) and multiplex different protocols
//! over [bidirectional streams](https://docs.rs/iroh/latest/iroh/endpoint/struct.Connection.html#method.open_bi)
//! within that connection:
//!
//! ```text
//! Single Connection between Entities
//!     ├── Stream 1: HTTP Proxy
//!     ├── Stream 2: Ping
//!     ├── Stream 3: TCP Tunnel
//!     └── Stream N: ...
//! ```
//!
//! Each stream starts with a JSON protocol header identifying its type.
//!
//! # The Protocol "Protocol"
//!
//! ## Stream Lifecycle
//!
//! 1. **Client entity** opens a bidirectional stream
//! 2. **Client** sends a JSON protocol header (newline-terminated)
//! 3. **Server entity** sends ACK to confirm protocol support
//! 4. Protocol-specific communication begins
//!
//! ## Protocol Header
//!
//! The first message on each stream is a JSON-encoded [`ProtocolHeader`] containing:
//! - The [`Protocol`] type (Ping, Http, Tcp, etc.)
//! - Optional protocol-specific metadata
//!
//! This allows protocol handlers to receive all necessary information upfront
//! without additional negotiation rounds.
//!
//! # Future Considerations
//!
//! This multiplexing approach may not be optimal for all use cases. Real-time
//! protocols (RTP/RTCP for audio/video) might benefit from dedicated connections
//! to avoid head-of-line blocking. This design decision will be re-evaluated
//! based on performance requirements.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum Protocol {
    /// client can send this message to check if the connection is open / healthy.
    Ping,
    /// client may not be using NTP, or may only have p2p access and no other internet access, in
    /// which case it can ask for the time from the peers and try to create a consensus.
    WhatTimeIsIt,
    /// client wants to make an HTTP request to a device whose ID is specified. note that the exact
    /// ip:port is not known to peers, they only the "device id" for the service. server will figure
    /// out the ip:port from the device id.
    Http,
    HttpProxy,
    /// if the client wants their traffic to route via this server, they can send this. for this to
    /// work, the person owning the device must have created a SOCKS5 device, and allowed this peer
    /// to access it.
    Socks5,
    Tcp,
    // TODO: RTP/"RTCP" for audio video streaming

    // Fastn-specific protocols for entity communication
    /// Messages from Device to Account (sync requests, status reports, etc.)
    DeviceToAccount,
    /// Messages between Accounts (email, file sharing, automerge sync, etc.)
    AccountToAccount,
    /// Messages from Account to Device (commands, config updates, notifications, etc.)
    AccountToDevice,
    /// Control messages for Rig management (bring online/offline, set current, etc.)
    RigControl,
}

impl std::fmt::Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Protocol::Ping => write!(f, "Ping"),
            Protocol::WhatTimeIsIt => write!(f, "WhatTimeIsIt"),
            Protocol::Http => write!(f, "Http"),
            Protocol::HttpProxy => write!(f, "HttpProxy"),
            Protocol::Socks5 => write!(f, "Socks5"),
            Protocol::Tcp => write!(f, "Tcp"),
            Protocol::DeviceToAccount => write!(f, "DeviceToAccount"),
            Protocol::AccountToAccount => write!(f, "AccountToAccount"),
            Protocol::AccountToDevice => write!(f, "AccountToDevice"),
            Protocol::RigControl => write!(f, "RigControl"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_display() {
        assert_eq!(format!("{}", Protocol::Ping), "Ping");
        assert_eq!(format!("{}", Protocol::AccountToAccount), "AccountToAccount");
        assert_eq!(format!("{}", Protocol::HttpProxy), "HttpProxy");
    }
}

/// Single ALPN protocol identifier for all fastn entity connections.
///
/// Each fastn instance is called an "entity" in the P2P network. Unlike Iroh's
/// recommended approach of using different ALPNs for different protocols, we use
/// a single ALPN and multiplex protocols at the application layer. This avoids
/// the overhead of multiple TLS handshakes when entities need to use multiple
/// protocols (e.g., HTTP proxy + TCP tunnel + ping).
///
/// See module documentation for detailed rationale.
pub const APNS_IDENTITY: &[u8] = b"/fastn/entity/0.1";

/// Protocol header with optional metadata.
///
/// Sent at the beginning of each bidirectional stream to identify
/// the protocol and provide any protocol-specific metadata.
#[derive(Debug)]
pub struct ProtocolHeader {
    pub protocol: Protocol,
    pub extra: Option<String>,
}

impl From<Protocol> for ProtocolHeader {
    fn from(protocol: Protocol) -> Self {
        Self {
            protocol,
            extra: None,
        }
    }
}
