//! Certificate management error types

use thiserror::Error;

/// Error type for certificate operations
#[derive(Error, Debug)]
pub enum CertificateError {
    #[error("Failed to load rig config from automerge")]
    ConfigLoad { 
        #[source]
        source: Box<dyn std::error::Error + Send + Sync> 
    },

    #[error("Failed to save rig config to automerge")]
    ConfigSave { 
        #[source]
        source: Box<dyn std::error::Error + Send + Sync> 
    },

    #[error("Failed to generate self-signed certificate")]
    CertificateGeneration { 
        #[source]
        source: Box<dyn std::error::Error + Send + Sync> 
    },

    #[error("Failed to load rig secret key")]
    RigKeyLoad { 
        #[source]
        source: Box<dyn std::error::Error + Send + Sync> 
    },

    #[error("Failed to convert Ed25519 key for certificate use")]
    KeyConversion { 
        #[source]
        source: Box<dyn std::error::Error + Send + Sync> 
    },

    #[error("Failed to create rustls TLS configuration")]
    TlsConfigCreation { 
        #[source]
        source: Box<dyn std::error::Error + Send + Sync> 
    },

    #[error("Failed to load external certificate: {path}")]
    ExternalCertificateLoad {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to parse certificate PEM data")]
    CertificateParsing { 
        #[source]
        source: Box<dyn std::error::Error + Send + Sync> 
    },

    #[error("Certificate has expired")]
    CertificateExpired {
        expired_at: i64,
    },

    #[error("Public IP detection failed")]
    PublicIpDetection { 
        #[source]
        source: Box<dyn std::error::Error + Send + Sync> 
    },
}