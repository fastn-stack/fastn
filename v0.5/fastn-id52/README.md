# fastn-id52

[![Crates.io](https://img.shields.io/crates/v/fastn-id52.svg)](https://crates.io/crates/fastn-id52)
[![Documentation](https://docs.rs/fastn-id52/badge.svg)](https://docs.rs/fastn-id52)
[![License](https://img.shields.io/crates/l/fastn-id52.svg)](LICENSE)

ID52 entity identity and cryptographic key management for the fastn P2P network.

## Overview

`fastn-id52` provides entity identity for the fastn P2P network. Each fastn instance
(called an "entity") is identified by an ID52 - a 52-character encoded Ed25519 public
key that uniquely identifies the entity on the network.

### What is ID52?

ID52 is the identity of an entity on the fastn peer-to-peer network. It's a
52-character encoding format using BASE32_DNSSEC that represents the entity's
Ed25519 public key. This format is designed to be:

- Unique identifier for each fastn entity
- Human-readable and copyable
- DNS-compatible (can be used in subdomains)
- URL-safe without encoding
- Fixed length (always 52 characters)

## Features

- **Entity Identity**: ID52 uniquely identifies fastn entities on the P2P network
- **ID52 Encoding**: 52-character BASE32_DNSSEC encoded public keys
- **Ed25519 Cryptography**: Industry-standard elliptic curve signatures
- **Key Generation**: Secure random entity key generation
- **Signature Operations**: Sign and verify messages between entities
- **Type Safety**: Strongly typed keys and signatures

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
fastn-id52 = "0.1"
```

## Usage

### Generating Keys

```rust
use fastn_id52::SecretKey;

// Generate a new random key pair
let secret_key = SecretKey::generate();

// Get the public key
let public_key = secret_key.public_key();

// Get the ID52 representation
let id52 = secret_key.id52();
println!("ID52: {}", id52);
// Output: i66fo538lfl5ombdf6tcdbrabp4hmp9asv7nrffuc2im13ct4q60
```

### Parsing ID52 Strings

```rust
use fastn_id52::PublicKey;
use std::str::FromStr;

let id52 = "i66fo538lfl5ombdf6tcdbrabp4hmp9asv7nrffuc2im13ct4q60";
let public_key = PublicKey::from_str(id52) ?;

// Convert back to ID52
assert_eq!(public_key.to_string(), id52);
```

### Signing and Verification

```rust
use fastn_id52::{SecretKey, Signature};

let secret_key = SecretKey::generate();
let message = b"Hello, world!";

// Sign a message
let signature = secret_key.sign(message);

// Verify the signature
let public_key = secret_key.public_key();
assert!(public_key.verify(message, &signature).is_ok());

// Verification fails with wrong message
assert!(public_key.verify(b"Wrong message", &signature).is_err());
```

### Working with Raw Bytes

```rust
use fastn_id52::{PublicKey, SecretKey};

// Secret key from bytes
let secret_bytes: [u8; 32] = [/* ... */];
let secret_key = SecretKey::from_bytes( & secret_bytes) ?;

// Public key from bytes
let public_bytes: [u8; 32] = [/* ... */];
let public_key = PublicKey::from_bytes( & public_bytes) ?;

// Export to bytes
let secret_bytes = secret_key.as_bytes();
let public_bytes = public_key.as_bytes();
```

### Serialization

All key types implement `Display` and `FromStr` for easy serialization:

```rust
use fastn_id52::{SecretKey, PublicKey};
use std::str::FromStr;

let secret_key = SecretKey::generate();

// Secret keys use hexadecimal encoding
let secret_hex = secret_key.to_string();
let secret_key2 = SecretKey::from_str( & secret_hex) ?;

// Public keys use ID52 encoding
let public_id52 = secret_key.public_key().to_string();
let public_key = PublicKey::from_str( & public_id52) ?;
```

## Error Handling

The crate provides detailed error types for all operations:

- `ParseId52Error`: Invalid ID52 string format
- `InvalidKeyBytesError`: Invalid key byte length or format
- `ParseSecretKeyError`: Invalid secret key string
- `InvalidSignatureBytesError`: Invalid signature bytes
- `SignatureVerificationError`: Signature verification failed

All errors implement `std::error::Error` and provide descriptive messages.

## Security Considerations

- **Secret Keys**: Never expose secret keys. They should be stored securely and
  never logged or transmitted.
- **Random Generation**: Uses `rand::rngs::OsRng` for cryptographically secure
  randomness
- **Constant Time**: Ed25519 operations are designed to be constant-time to
  prevent timing attacks
- **Key Derivation**: Each secret key deterministically derives its public key

## Examples

### Creating a Key Pair and Saving to Files

```rust
use fastn_id52::SecretKey;
use std::fs;

let secret_key = SecretKey::generate();
let public_key = secret_key.public_key();

// Save secret key (hex format)
fs::write("secret.key", secret_key.to_string()) ?;

// Save public key (ID52 format)
fs::write("public.id52", public_key.to_string()) ?;
```

### Loading Keys from Files

```rust
use fastn_id52::{SecretKey, PublicKey};
use std::fs;
use std::str::FromStr;

// Load secret key
let secret_hex = fs::read_to_string("secret.key") ?;
let secret_key = SecretKey::from_str( & secret_hex) ?;

// Load public key
let public_id52 = fs::read_to_string("public.id52") ?;
let public_key = PublicKey::from_str( & public_id52) ?;
```

## License

This project is licensed under the UPL-1.0 License - see the LICENSE file for
details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Acknowledgments

This crate is part of the fastn ecosystem and was migrated from the original
`kulfi-id52` implementation.
