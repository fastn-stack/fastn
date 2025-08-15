# fastn-net

[![Crates.io](https://img.shields.io/crates/v/fastn-net.svg)](https://crates.io/crates/fastn-net)
[![Documentation](https://docs.rs/fastn-net/badge.svg)](https://docs.rs/fastn-net)
[![License](https://img.shields.io/crates/l/fastn-net.svg)](LICENSE)

Network utilities and P2P communication for the fastn ecosystem.

## Overview

`fastn-net` provides P2P networking capabilities for fastn entities using [Iroh](https://github.com/n0-computer/iroh). Each fastn instance is called an "entity" in the P2P network, identified by a unique ID52 (52-character encoded Ed25519 public key).

## Features

- P2P networking between fastn entities with NAT traversal
- HTTP and TCP proxying between entities
- Connection pooling for HTTP clients
- Protocol multiplexing (HTTP, TCP, SOCKS5, Ping) over single connections
- Entity identification via ID52 encoding

## Installation

```toml
[dependencies]
fastn-net = "0.1"
```

## Usage

```rust
use fastn_net::{global_iroh_endpoint, ping, PONG, Protocol};

// Get the global Iroh endpoint for entity connections
let endpoint = global_iroh_endpoint().await;

// Connect to another entity and open a stream
let connection = endpoint.connect(entity_node_id, "").await?;
let (mut send, mut recv) = connection.open_bi().await?;

// Send a ping to test connectivity between entities
ping(&mut send).await?;
let response = fastn_net::next_string(&mut recv).await?;
assert_eq!(response, PONG);
```

## Supported Protocols

- `Protocol::Ping` - Connectivity testing
- `Protocol::Http` - HTTP request proxying
- `Protocol::Tcp` - TCP tunneling
- `Protocol::Socks5` - SOCKS5 proxy
- `Protocol::HttpProxy` - HTTP proxy protocol

## License

UPL-1.0