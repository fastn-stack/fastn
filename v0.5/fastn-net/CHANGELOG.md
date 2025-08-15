# Changelog

All notable changes to the fastn-net crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres
to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.2] - 2025-08-15

### Fixed

- Fixed broken doctests in lib.rs and graceful.rs
- Updated graceful.rs documentation to use correct `cancelled()` method instead
  of non-existent `is_cancelled()`
- Fixed example code to use proper `tokio::select!` patterns for cancellation
  handling
- Corrected `endpoint.accept()` usage in examples (returns `Option` not
  `Result`)

### Changed

- Updated dependency: fastn-id52 from 0.1.0 to 0.1.1 (adds CLI tool for key
  generation)

### Removed

- Removed outdated test files that were no longer relevant:
    - `tests/baseline_compat.rs`
    - `tests/baseline_compatibility.rs`
    - `tests/baseline_key_compatibility.rs`
    - `tests/smoke_test.rs`

## [0.1.1] - 2025-08-15

### Added

- Initial release of fastn-net crate
- P2P networking support via Iroh 0.91
- HTTP and TCP proxying over P2P connections
- Connection pooling with bb8 for HTTP clients
- Protocol multiplexing support (HTTP, TCP, SOCKS5, Ping)
- Global Iroh endpoint management
- Bidirectional stream utilities
- Identity management integration with fastn-id52

### Features

- `global_iroh_endpoint()` - Singleton Iroh endpoint for P2P connections
- `ping()` and `PONG` - Connectivity testing between peers
- `http_to_peer()` and `peer_to_http()` - HTTP proxying functions
- `tcp_to_peer()` and `peer_to_tcp()` - TCP tunneling functions
- `HttpConnectionManager` - Connection pooling for HTTP clients
- `Protocol` enum - Supported protocol types
- `accept_bi()` and `accept_bi_with()` - Accept incoming streams
- `next_json()` and `next_string()` - Stream reading utilities

### Technical Details

- Based on Iroh 0.91 for P2P networking
- Uses hyper 1.6 for HTTP handling
- Connection pooling with bb8 0.9
- Async runtime with tokio
- Migrated from kulfi-utils to fastn ecosystem

[0.1.2]: https://github.com/fastn-stack/fastn/releases/tag/fastn-net-v0.1.2
[0.1.1]: https://github.com/fastn-stack/fastn/releases/tag/fastn-net-v0.1.1
