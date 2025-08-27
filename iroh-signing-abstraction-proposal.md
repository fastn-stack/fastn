# Proposal: Abstract signing operations to support external key management services

## Background: malai service key management challenges

We're building the **malai service** - a service that needs to manage multiple
ID52 identities (Ed25519-based) for peer-to-peer operations using iroh. Each
identity requires secure private key access for signing operations.

## Current Implementation

We've implemented a comprehensive key management system in `fastn-id52`:

1. **System keyring integration**: Stores keys securely in OS credential
   managers
2. **Environment variable fallback**:
    - `FASTN_SECRET_KEYS_FILE`: Path to file containing keys
    - `FASTN_SECRET_KEYS`: Keys directly in environment
3. **File-based storage**: Direct key files with `.id52`/`.private-key` formats
4. **Fallback chain**: keyring → env file → env var → fail

**Format**: `prefix: hexkey` where prefix matches ID52 start (e.g.,
`i66fo538: abc123...`)

## The Problem

**Keyring fails on headless Linux servers** - the most common deployment
environment for malai. System keyrings (GNOME Keyring, KWallet) require desktop
environments that don't exist on servers.

This forces us to use less secure approaches:

- Environment variables (visible in process lists)
- Files on disk (persistent, discoverable)
- Both approaches expose private key material

## Why ssh-agent as a Solution

ssh-agent solves our key security problems:

- **Memory-only storage**: Keys never hit disk
- **Process isolation**: Unix socket access control
- **Battle-tested**: Most sensitive credentials (SSH keys) use this model
- **Standard tooling**: Available on all Linux servers
- **Audit trail**: ssh-agent logging for security compliance

## The iroh Integration Challenge

iroh currently requires direct private key access (`SecretKey.sign()`)
throughout:

- pkarr discovery service needs raw keys for DNS record signing
- Document/replica operations require immediate signing capability
- Connection authentication expects synchronous key access

This prevents using ssh-agent, which only provides signing services without
exposing private key material.

## Proposed Solution

Abstract iroh's signing operations to support external signing services:

```rust
#[async_trait]
pub trait SigningService: Send + Sync {
    async fn sign(&self, message: &[u8]) -> Result<Signature, SigningError>;
    fn public_key(&self) -> PublicKey;
}

// Implementations:
struct LocalSigner(SecretKey);        // Current behavior  
struct SshAgentSigner {
    /* ssh-agent client */
};
struct HsmSigner {
    /* hardware security */
};
```

## Benefits Beyond Our Use Case

- **Enterprise HSM support**: Hardware security module integration
- **Distributed signing**: Multi-party/threshold signatures (building on
  existing FROST work)
- **Key rotation**: External key lifecycle management
- **Compliance**: Audit trails and secure key handling

## Existing Foundation

- **FROST threshold signatures**: Already proves iroh can work with distributed
  signing
- **Modular plugin architecture**: Discovery trait shows pluggability is
  possible
- **Ed25519 compatibility**: Any Ed25519 signature source works with existing
  verification

Would the maintainers be interested in this direction? We're willing to
contribute the implementation work as it directly serves our malai service
requirements while benefiting the broader iroh ecosystem.
