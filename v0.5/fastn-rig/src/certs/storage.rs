//! External certificate storage and loading (nginx/Let's Encrypt integration)

use crate::certs::CertificateError;
use std::path::Path;

/// Load external certificate and create TLS configuration
/// 
/// Used for nginx/Let's Encrypt certificate integration where
/// certificates are managed externally and fastn reads from file paths
pub async fn load_external_certificate(
    cert_path: &str,
    key_path: &str,
) -> Result<rustls::ServerConfig, CertificateError> {
    println!("📁 Loading external certificate from: {}", cert_path);
    println!("🔑 Loading external private key from: {}", key_path);

    // Load certificate file
    let cert_pem = tokio::fs::read_to_string(cert_path).await
        .map_err(|e| CertificateError::ExternalCertificateLoad {
            path: cert_path.to_string(),
            source: e,
        })?;

    // Load private key file  
    let key_pem = tokio::fs::read_to_string(key_path).await
        .map_err(|e| CertificateError::ExternalCertificateLoad {
            path: key_path.to_string(),
            source: e,
        })?;

    // Parse certificate from PEM
    let cert_der = rustls_pemfile::certs(&mut cert_pem.as_bytes())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| CertificateError::CertificateParsing { 
            source: Box::new(e) 
        })?;

    if cert_der.is_empty() {
        return Err(CertificateError::CertificateParsing { 
            source: "No certificates found in PEM file".into() 
        });
    }

    // Parse private key from PEM  
    let private_key = rustls_pemfile::private_key(&mut key_pem.as_bytes())
        .map_err(|e| CertificateError::CertificateParsing { 
            source: Box::new(e) 
        })?
        .ok_or_else(|| CertificateError::CertificateParsing { 
            source: "No private key found in PEM file".into() 
        })?;

    // Create TLS configuration
    let config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert_der, private_key)
        .map_err(|e| CertificateError::TlsConfigCreation { 
            source: Box::new(e) 
        })?;

    println!("🔐 External certificate loaded successfully");
    Ok(config)
}

/// Validate external certificate files exist and are readable
pub async fn validate_external_certificate_paths(
    cert_path: &str,
    key_path: &str,
) -> Result<(), CertificateError> {
    // Check certificate file
    if !Path::new(cert_path).exists() {
        return Err(CertificateError::ExternalCertificateLoad {
            path: cert_path.to_string(),
            source: std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Certificate file not found"
            ),
        });
    }

    // Check private key file
    if !Path::new(key_path).exists() {
        return Err(CertificateError::ExternalCertificateLoad {
            path: key_path.to_string(), 
            source: std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Private key file not found"
            ),
        });
    }

    // Try to read both files to check permissions
    let _ = tokio::fs::read_to_string(cert_path).await
        .map_err(|e| CertificateError::ExternalCertificateLoad {
            path: cert_path.to_string(),
            source: e,
        })?;

    let _ = tokio::fs::read_to_string(key_path).await
        .map_err(|e| CertificateError::ExternalCertificateLoad {
            path: key_path.to_string(),
            source: e,
        })?;

    println!("✅ External certificate files validated");
    Ok(())
}

/// Extract certificate metadata from external certificate file
pub async fn get_external_certificate_info(
    cert_path: &str,
) -> Result<(String, i64), CertificateError> {
    let cert_pem = tokio::fs::read_to_string(cert_path).await
        .map_err(|e| CertificateError::ExternalCertificateLoad {
            path: cert_path.to_string(),
            source: e,
        })?;

    // Parse certificate to extract subject and expiry
    // This is a simplified implementation - in production you might want
    // to use x509-parser for more detailed certificate inspection
    
    let subject = "External Certificate".to_string(); // TODO: Parse actual subject
    let expires_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64 + (365 * 24 * 60 * 60); // TODO: Parse actual expiry

    Ok((subject, expires_at))
}

/// Create TLS configuration from certificate and key PEM strings (stored in automerge)
pub async fn create_tls_config_from_pem_strings(
    cert_pem: &str,
    key_pem: &str,
) -> Result<rustls::ServerConfig, CertificateError> {
    println!("🔐 Creating TLS config from certificate content in automerge");

    // Parse certificate from PEM string
    let cert_der = rustls_pemfile::certs(&mut cert_pem.as_bytes())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| CertificateError::CertificateParsing { 
            source: Box::new(e) 
        })?;

    if cert_der.is_empty() {
        return Err(CertificateError::CertificateParsing { 
            source: "No certificates found in PEM content".into() 
        });
    }

    // Parse private key from PEM string
    let private_key = rustls_pemfile::private_key(&mut key_pem.as_bytes())
        .map_err(|e| CertificateError::CertificateParsing { 
            source: Box::new(e) 
        })?
        .ok_or_else(|| CertificateError::CertificateParsing { 
            source: "No private key found in PEM content".into() 
        })?;

    // Create TLS configuration
    let config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert_der, private_key)
        .map_err(|e| CertificateError::TlsConfigCreation { 
            source: Box::new(e) 
        })?;

    println!("🔐 TLS configuration created from automerge certificate content");
    Ok(config)
}