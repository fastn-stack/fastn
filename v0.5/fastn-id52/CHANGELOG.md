# Changelog

All notable changes to the fastn-id52 crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-08-15

### Added
- Initial release of fastn-id52 crate
- ID52 encoding/decoding for public keys (52-character BASE32_DNSSEC format)
- Ed25519 public/private key pair support
- Key generation and serialization
- Digital signature creation and verification
- Hexadecimal encoding for secret keys
- Comprehensive error types for key and signature operations
- Full test coverage for core functionality

### Features
- `PublicKey`: 52-character ID52 encoded public keys
- `SecretKey`: Ed25519 secret keys with hex encoding
- `Signature`: Ed25519 signature support (64 bytes)
- Key generation with `SecretKey::generate()`
- String parsing and serialization for all key types
- Secure signature verification

### Technical Details
- Based on ed25519-dalek v2.1.1 for cryptographic operations
- Uses data-encoding for BASE32_DNSSEC encoding
- No external dependencies beyond core cryptographic libraries
- Migrated from kulfi-id52 to fastn ecosystem

[0.1.0]: https://github.com/fastn-stack/fastn/releases/tag/fastn-id52-v0.1.0