//! DNS resolution functionality for fastn ID52 public keys.
//!
//! This module provides DNS TXT record lookup to resolve public keys from domain names.
//! For example, given a TXT record "malai=abc123def456..." on domain "fifthtry.com",
//! we can resolve the public key for scope "malai".

use crate::{PublicKey, errors::ResolveError};
use hickory_resolver::{TokioAsyncResolver, config::*};
use std::str::FromStr;

/// Resolves a public key from DNS TXT records.
///
/// Looks for TXT records on the given domain in the format "{scope}={public_key_id52}".
/// For example, if the domain "fifthtry.com" has a TXT record "malai=abc123def456...",
/// calling resolve("fifthtry.com", "malai") will return the public key.
///
/// # Arguments
///
/// * `domain` - The domain to query for TXT records
/// * `scope` - The scope/prefix to look for in TXT records
///
/// # Returns
///
/// Returns the resolved `PublicKey` on success, or a `ResolveError` on failure.
///
/// # Examples
///
/// ```no_run
/// use fastn_id52::dns::resolve;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let public_key = resolve("fifthtry.com", "malai").await?;
/// println!("Resolved public key: {}", public_key.id52());
/// # Ok(())
/// # }
/// ```
pub async fn resolve(domain: &str, scope: &str) -> Result<PublicKey, ResolveError> {
    let resolver = TokioAsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default());

    let response = resolver
        .txt_lookup(domain)
        .await
        .map_err(|e| ResolveError {
            domain: domain.to_string(),
            scope: scope.to_string(),
            reason: format!("DNS TXT lookup failed: {}", e),
        })?;

    let scope_prefix = format!("{}=", scope);

    for record in response.iter() {
        for txt_data in record.iter() {
            let txt_string = String::from_utf8_lossy(txt_data);

            if let Some(id52_part) = txt_string.strip_prefix(&scope_prefix) {
                return PublicKey::from_str(id52_part).map_err(|e| ResolveError {
                    domain: domain.to_string(),
                    scope: scope.to_string(),
                    reason: format!("Invalid ID52 in DNS record '{}': {}", txt_string, e),
                });
            }
        }
    }

    Err(ResolveError {
        domain: domain.to_string(),
        scope: scope.to_string(),
        reason: format!("No TXT record found with prefix '{}'", scope_prefix),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_resolve_nonexistent_domain() {
        let result = resolve("nonexistent-domain-12345.com", "test").await;
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.domain, "nonexistent-domain-12345.com");
        assert_eq!(err.scope, "test");
        assert!(err.reason.contains("DNS TXT lookup failed"));
    }

    #[tokio::test]
    async fn test_resolve_existing_domain_no_matching_scope() {
        // Using a real domain that likely doesn't have our specific TXT record
        let result = resolve("google.com", "fastn-test-nonexistent").await;
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert_eq!(err.domain, "google.com");
        assert_eq!(err.scope, "fastn-test-nonexistent");
        // Could be either no TXT records or no matching scope
        assert!(
            err.reason.contains("No TXT record found with prefix")
                || err.reason.contains("DNS TXT lookup failed")
        );
    }

    // Note: We can't easily test successful resolution without setting up a real DNS record
    // or using a mock resolver, which would require additional dependencies
}
