use std::error::Error;
use std::fmt;

/// Error returned when parsing an invalid ID52 string.
///
/// This error occurs when attempting to parse a string that doesn't conform to
/// the ID52 format (52-character BASE32_DNSSEC encoding).
#[derive(Debug, Clone)]
pub struct ParseId52Error {
    pub input: String,
    pub reason: String,
}

impl fmt::Display for ParseId52Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid ID52 '{}': {}", self.input, self.reason)
    }
}

impl Error for ParseId52Error {}

/// Error returned when parsing an invalid secret key string.
///
/// This error occurs when attempting to parse a string that doesn't represent
/// a valid Ed25519 secret key in hexadecimal or base32 format.
#[derive(Debug, Clone)]
pub struct ParseSecretKeyError {
    pub reason: String,
}

impl fmt::Display for ParseSecretKeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid secret key: {}", self.reason)
    }
}

impl Error for ParseSecretKeyError {}

/// Error returned when creating keys from invalid byte arrays.
///
/// This error occurs when the provided byte array has the wrong length or
/// contains invalid key material.
#[derive(Debug, Clone)]
pub struct InvalidKeyBytesError {
    pub expected: usize,
    pub got: usize,
}

impl fmt::Display for InvalidKeyBytesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Invalid key length: expected {} bytes, got {}",
            self.expected, self.got
        )
    }
}

impl Error for InvalidKeyBytesError {}

/// Error returned when signature verification fails.
///
/// This error indicates that a signature is not valid for the given public key
/// and message combination.
#[derive(Debug, Clone)]
pub struct SignatureVerificationError;

impl fmt::Display for SignatureVerificationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Signature verification failed")
    }
}

impl Error for SignatureVerificationError {}

/// Error returned when creating a signature from invalid bytes.
///
/// This error occurs when attempting to create a signature from a byte array
/// that is not exactly 64 bytes long.
#[derive(Debug, Clone)]
pub struct InvalidSignatureBytesError {
    pub expected: usize,
    pub got: usize,
}

impl fmt::Display for InvalidSignatureBytesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Invalid signature length: expected {} bytes, got {}",
            self.expected, self.got
        )
    }
}

impl Error for InvalidSignatureBytesError {}

/// Error returned when DNS resolution fails.
///
/// This error occurs when attempting to resolve a public key from DNS but
/// the operation fails for various reasons.
#[derive(Debug, Clone)]
#[cfg(feature = "dns")]
pub struct ResolveError {
    pub domain: String,
    pub scope: String,
    pub reason: String,
}

#[cfg(feature = "dns")]
impl fmt::Display for ResolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Failed to resolve public key for domain '{}' with scope '{}': {}",
            self.domain, self.scope, self.reason
        )
    }
}

#[cfg(feature = "dns")]
impl Error for ResolveError {}
