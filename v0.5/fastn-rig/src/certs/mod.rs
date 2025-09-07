//! Email certificate management for STARTTLS support
//!
//! This module handles:
//! - Self-signed certificate generation using rig's Ed25519 key
//! - External certificate configuration (nginx/Let's Encrypt integration)
//! - Certificate storage in RigConfig automerge document
//! - TLS configuration for STARTTLS SMTP server

mod errors;
mod self_signed;
mod storage;

pub use errors::CertificateError;

use crate::automerge::EmailCertificateConfig;
use std::path::Path;
use std::sync::Arc;

/// Main certificate manager for email protocols
pub struct CertificateManager {
    /// Reference to automerge database for RigConfig access
    automerge_db: Arc<fastn_automerge::Db>,
    /// Rig's public key for RigConfig document access
    rig_id52: fastn_id52::PublicKey,
}

impl CertificateManager {
    /// Create new certificate manager
    pub fn new(
        automerge_db: Arc<fastn_automerge::Db>,
        rig_id52: fastn_id52::PublicKey,
    ) -> Result<Self, CertificateError> {
        Ok(Self {
            automerge_db,
            rig_id52,
        })
    }

    /// Get or create TLS configuration for STARTTLS server
    /// 
    /// This is the main entry point - it handles:
    /// - Loading existing certificate from RigConfig
    /// - Generating new self-signed certificate if needed  
    /// - Loading external certificate if configured
    /// - Converting to rustls::ServerConfig for TLS server
    pub async fn get_or_create_tls_config(&self) -> Result<rustls::ServerConfig, CertificateError> {
        // Load current rig config
        let rig_config = crate::automerge::RigConfig::load(&self.automerge_db, &self.rig_id52)
            .map_err(|e| CertificateError::ConfigLoad { 
                source: Box::new(e) 
            })?;

        match &rig_config.email_certificate {
            EmailCertificateConfig::SelfSigned { cert_pem, expires_at, .. } => {
                // Check if this is a placeholder or expired certificate
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64;
                
                if cert_pem == "PLACEHOLDER" || *expires_at <= now {
                    // Placeholder or expired certificate, generate new one
                    self.generate_and_store_self_signed_certificate().await
                } else {
                    // Valid certificate, use existing
                    self.create_tls_config_from_self_signed(&rig_config, cert_pem).await
                }
            }
            EmailCertificateConfig::External { cert_path, key_path, .. } => {
                // Load external certificate
                self.load_external_certificate(cert_path, key_path).await
            }
        }
    }

    /// Generate new self-signed certificate and store in RigConfig
    async fn generate_and_store_self_signed_certificate(&self) -> Result<rustls::ServerConfig, CertificateError> {
        // Implementation in self_signed.rs
        self_signed::generate_and_store_certificate(
            &self.automerge_db,
            &self.rig_id52,
        ).await
    }

    /// Create TLS config from existing self-signed certificate
    async fn create_tls_config_from_self_signed(
        &self,
        rig_config: &crate::automerge::RigConfig,
        cert_pem: &str,
    ) -> Result<rustls::ServerConfig, CertificateError> {
        // Implementation in self_signed.rs
        self_signed::create_tls_config_from_stored_cert(rig_config, cert_pem).await
    }

    /// Load external certificate and create TLS config
    async fn load_external_certificate(
        &self,
        cert_path: &str,
        key_path: &str,
    ) -> Result<rustls::ServerConfig, CertificateError> {
        // Implementation in storage.rs
        storage::load_external_certificate(cert_path, key_path).await
    }
}