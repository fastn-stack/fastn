# Changelog

All notable changes to the fastn-id52 crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres
to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **Automerge CRDT support** (optional feature: `automerge`)
  - `PublicKey`, `SecretKey`, and `Signature` now implement `autosurgeon::Reconcile` and `autosurgeon::Hydrate`
  - Enables type-safe storage in Automerge CRDT documents
  - Keys stored as ID52/hex strings and automatically converted back to typed objects
  - Feature-gated to avoid unnecessary dependencies
  - Usage: `fastn-id52 = { workspace = true, features = ["automerge"] }`
- **PublicKey convenience method**
  - Added `id52()` method to `PublicKey` for consistency with `SecretKey`
  - Returns the ID52 string representation directly
- `Clone` trait implementation for `SecretKey`
  - Allows copying secret keys when needed in structs
  - Clones by reconstructing from bytes
- `Debug` trait implementation for `SecretKey`
  - Shows only the public ID52 in debug output
  - Omits the actual 32-byte secret key material for security
  - Format: `SecretKey { id52: "..." }`
- New `SecretKey` helper methods for key loading:
  - `load_from_dir(dir, prefix)`: Comprehensive key loading from directory
    - Checks for `{prefix}.id52` and `{prefix}.private-key` files
    - Enforces strict mode (errors if both files exist)
    - Returns tuple of (ID52, SecretKey)
  - `load_for_id52(id52)`: Load key with automatic fallback
    - Tries system keyring first
    - Falls back to `FASTN_SECRET_KEYS` environment variable
- Environment variable support for secret keys:
  - `FASTN_SECRET_KEYS`: Keys directly in environment variable
  - `FASTN_SECRET_KEYS_FILE`: Path to file containing keys (more secure)
  - Cannot have both set (strict mode enforcement)
  - Format: `prefix1: hexkey1\nprefix2: hexkey2` (spaces around `:` are optional)
  - Files support comments (lines starting with `#`) and empty lines
  - Flexible prefix matching using `starts_with`
  - Use as many or few characters as needed for unique identification

### Changed

- `SecretKey` now derives `Clone` and `Debug` for better ergonomics
- Debug output for `SecretKey` no longer exposes sensitive key material

## [0.1.2] - 2025-08-15

### Added

- System keyring integration for secure secret key storage
  - Default storage now uses system keyring (password manager)
  - `SecretKey::store_in_keyring()` method to save keys
  - `SecretKey::from_keyring(id52)` method to load keys
  - `SecretKey::delete_from_keyring()` method to remove keys
  - `KeyringError` type for keyring operation failures
- CLI improvements
  - Keyring storage is now the default behavior
  - `--keyring` / `-k` flag for explicit keyring storage
  - `--short` / `-s` flag for minimal output (only ID52)
  - Support for `-` as filename to output to stdout
  - Improved argument parsing with structured `Cli` type

### Changed

- **BREAKING**: CLI default behavior now stores in keyring instead of requiring flags
- **BREAKING**: Removed `--print` option (use `--file -` or `-f -` instead)
- CLI now requires explicit `--file` flag for file storage (security improvement)
- Refactored CLI parsing with proper command structure
- Keys stored in keyring as hex strings for password manager compatibility
- Keyring service name: "fastn", account: ID52 of the entity

### Security

- No automatic fallback from keyring to file storage
- File storage requires explicit user consent via `--file` flag
- Clear error messages when keyring is unavailable
- Support for legacy keyring format (raw bytes) while preferring hex format

## [0.1.1] - 2025-08-15

### Added

- CLI binary `fastn-id52` for entity key generation
  - `generate` command to create new entity identities
  - `--file` option to save keys to files (default: `.fastn.secret-key`)
  - `--print` option to output keys to stdout
  - Safety checks to prevent accidental key overwriting

### Security

- CLI requires explicit flags (`--print` or `--file`) to output secret keys
- File operations check for existing files to prevent accidental overwriting

## [0.1.0] - 2025-08-15

### Added

- Initial release of fastn-id52 crate
- Entity identity for fastn P2P network
- ID52 encoding/decoding for entity public keys (52-character BASE32_DNSSEC format)
- Ed25519 public/private key pair support for entity authentication
- Key generation and serialization
- Digital signature creation and verification
- Hexadecimal encoding for secret keys
- Comprehensive error types for key and signature operations
- Full test coverage for core functionality

### Features

- `PublicKey`: 52-character ID52 encoded public keys
- `SecretKey`: Ed25519 secret keys with hex encoding
- `Signature`: Ed25519 signature support with hex encoding (128 characters)
- Key generation with `SecretKey::generate()`
- String parsing and serialization for all key types
    - `PublicKey`: Display/FromStr using ID52 format
    - `SecretKey`: Display/FromStr using hex format (64 chars)
    - `Signature`: Display/FromStr using hex format (128 chars)
- Serde support with automatic serialization/deserialization
- Secure signature verification

### Technical Details

- Based on ed25519-dalek v2.1.1 for cryptographic operations
- Uses data-encoding for BASE32_DNSSEC encoding
- No external dependencies beyond core cryptographic libraries
- Migrated from kulfi-id52 to fastn ecosystem
- Intentional Copy trait design:
    - `PublicKey` and `Signature` derive Copy for convenience
    - `SecretKey` deliberately does not derive Copy to encourage explicit
      cloning of sensitive data

[0.1.2]: https://github.com/fastn-stack/fastn/releases/tag/fastn-id52-v0.1.2
[0.1.1]: https://github.com/fastn-stack/fastn/releases/tag/fastn-id52-v0.1.1
[0.1.0]: https://github.com/fastn-stack/fastn/releases/tag/fastn-id52-v0.1.0
