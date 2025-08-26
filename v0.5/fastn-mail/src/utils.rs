//! # Email Utilities
//!
//! Common utility functions and trait implementations for email handling.

/// Display implementation for EmailAddress
impl std::fmt::Display for crate::EmailAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.name {
            Some(name) => write!(f, "{name} <{}>", self.address),
            None => write!(f, "{}", self.address),
        }
    }
}
