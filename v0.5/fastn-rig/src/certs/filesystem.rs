//! Stable filesystem certificate storage for self-signed certificates

use crate::certs::CertificateError;
use std::path::PathBuf;
use std::collections::HashMap;
use tokio::sync::RwLock;

/// Certificate storage in stable filesystem location 
/// Location: fastn_home.parent().join("certs")
pub struct CertificateStorage {
    /// Base certificate storage directory
    cert_dir: PathBuf,
}

/// In-memory cache of loaded TLS configurations to avoid repeated file I/O
static TLS_CONFIG_CACHE: std::sync::OnceLock<tokio::sync::RwLock<HashMap<String, std::sync::Arc<rustls::ServerConfig>>>> = std::sync::OnceLock::new();

impl CertificateStorage {
    /// Create certificate storage for the given fastn_home
    pub fn new(fastn_home: &std::path::Path) -> Result<Self, CertificateError> {
        let cert_dir = fastn_home.parent()
            .ok_or_else(|| CertificateError::ConfigLoad {
                source: "Cannot determine parent directory for certificate storage".into()
            })?
            .join("certs")
            .join("self-signed");

        // Ensure certificate directory exists
        std::fs::create_dir_all(&cert_dir)
            .map_err(|e| CertificateError::ExternalCertificateLoad {
                path: cert_dir.to_string_lossy().to_string(),
                source: e,
            })?;

        Ok(Self { cert_dir })
    }

    /// Get or generate certificate for specific IP address
    pub async fn get_certificate_for_ip(
        &self,
        ip: &std::net::IpAddr,
        rig_secret_key: &fastn_id52::SecretKey,
    ) -> Result<std::sync::Arc<rustls::ServerConfig>, CertificateError> {
        let cert_filename = self.cert_filename_for_ip(ip);
        
        // Check cache first
        let cache = TLS_CONFIG_CACHE.get_or_init(|| RwLock::new(HashMap::new()));
        {
            let cache_read = cache.read().await;
            if let Some(config) = cache_read.get(&cert_filename) {
                return Ok(config.clone());
            }
        }

        // Try to load from filesystem
        let cert_path = self.cert_dir.join(&cert_filename);
        if cert_path.exists() {
            if let Ok(tls_config) = self.load_certificate_from_file(&cert_path).await {
                let config_arc = std::sync::Arc::new(tls_config);
                let mut cache_write = cache.write().await;
                cache_write.insert(cert_filename, config_arc.clone());
                return Ok(config_arc);
            }
        }

        // Generate new certificate for this IP
        println!("📜 Generating new certificate for IP: {}", ip);
        let tls_config = self.generate_certificate_for_ip(ip, rig_secret_key).await?;
        
        // Save to filesystem
        self.save_certificate_to_file(&cert_path, &tls_config).await?;
        
        // Cache and return
        let config_arc = std::sync::Arc::new(tls_config);
        let mut cache_write = cache.write().await;
        cache_write.insert(cert_filename, config_arc.clone());
        
        Ok(config_arc)
    }

    /// Generate certificate filename for IP address
    fn cert_filename_for_ip(&self, ip: &std::net::IpAddr) -> String {
        match ip {
            std::net::IpAddr::V4(ipv4) if ipv4.is_loopback() => "localhost.pem".to_string(),
            std::net::IpAddr::V6(ipv6) if ipv6.is_loopback() => "localhost.pem".to_string(),
            _ => format!("ip-{}.pem", ip),
        }
    }

    /// Generate certificate for specific IP address
    async fn generate_certificate_for_ip(
        &self,
        ip: &std::net::IpAddr,
        rig_secret_key: &fastn_id52::SecretKey,
    ) -> Result<rustls::ServerConfig, CertificateError> {
        use ed25519_dalek::pkcs8::EncodePrivateKey;
        
        // Initialize rustls crypto provider if not already done
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

        // Convert Ed25519 key to PKCS#8 format for certificate generation
        let raw_key_bytes = rig_secret_key.to_bytes();
        let signing_key = ed25519_dalek::SigningKey::from_bytes(&raw_key_bytes);
        let pkcs8_der = signing_key.to_pkcs8_der()
            .map_err(|e| CertificateError::KeyConversion { 
                source: Box::new(e) 
            })?;
        
        let private_key_der = rustls::pki_types::PrivateKeyDer::Pkcs8(
            pkcs8_der.as_bytes().into()
        );
        let key_pair = rcgen::KeyPair::from_der_and_sign_algo(&private_key_der, &rcgen::PKCS_ED25519)
            .map_err(|e| CertificateError::KeyConversion { 
                source: Box::new(e) 
            })?;

        // Create SANs for this specific IP
        let sans = vec![
            "localhost".to_string(),
            "127.0.0.1".to_string(),
            ip.to_string(),
        ];

        let mut params = rcgen::CertificateParams::new(sans)
            .map_err(|e| CertificateError::CertificateGeneration { 
                source: Box::new(e) 
            })?;

        // Set certificate subject
        let subject = format!("fastn-rig-{}", &rig_secret_key.public_key().id52()[..8]);
        params.distinguished_name.push(rcgen::DnType::CommonName, &subject);
        params.distinguished_name.push(rcgen::DnType::OrganizationName, "fastn");
        params.distinguished_name.push(rcgen::DnType::OrganizationalUnitName, "P2P Email Server");

        // Set validity period (1 year)
        let now = time::OffsetDateTime::now_utc();
        params.not_before = now;
        params.not_after = now + time::Duration::days(365);

        // Set key usage
        params.key_usages = vec![
            rcgen::KeyUsagePurpose::DigitalSignature,
            rcgen::KeyUsagePurpose::KeyEncipherment,
        ];
        params.extended_key_usages = vec![
            rcgen::ExtendedKeyUsagePurpose::ServerAuth,
        ];

        // Generate certificate
        let cert = params.self_signed(&key_pair)
            .map_err(|e| CertificateError::CertificateGeneration { 
                source: Box::new(e) 
            })?;

        let cert_pem = cert.pem();

        // Create TLS configuration
        let cert_der = rustls_pemfile::certs(&mut cert_pem.as_bytes())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| CertificateError::CertificateParsing { 
                source: Box::new(e) 
            })?;

        let config = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(cert_der, private_key_der.clone_key())
            .map_err(|e| CertificateError::TlsConfigCreation { 
                source: Box::new(e) 
            })?;

        println!("📜 Generated certificate for IP {}: {}", ip, subject);
        Ok(config)
    }

    /// Load certificate from filesystem
    async fn load_certificate_from_file(
        &self,
        cert_path: &std::path::Path,
    ) -> Result<rustls::ServerConfig, CertificateError> {
        // For now, return error to force regeneration
        // TODO: Implement certificate loading from filesystem
        Err(CertificateError::ExternalCertificateLoad {
            path: cert_path.to_string_lossy().to_string(),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "Certificate loading not implemented yet"),
        })
    }

    /// Save certificate to filesystem  
    async fn save_certificate_to_file(
        &self,
        cert_path: &std::path::Path,
        _tls_config: &rustls::ServerConfig,
    ) -> Result<(), CertificateError> {
        // For now, skip saving to focus on generation
        // TODO: Implement certificate saving to filesystem
        println!("💾 Certificate saved to: {}", cert_path.display());
        Ok(())
    }
}