//! Self-signed certificate generation using rig's Ed25519 key

use crate::automerge::{EmailCertificate, RigConfig};
use crate::certs::CertificateError;
use ed25519_dalek::pkcs8::EncodePrivateKey;

/// Generate new self-signed certificate using rig's existing Ed25519 key
pub async fn generate_and_store_certificate(
    automerge_db: &fastn_automerge::Db,
    rig_id52: &fastn_id52::PublicKey,
) -> Result<rustls::ServerConfig, CertificateError> {
    println!(
        "🔐 Generating self-signed certificate for rig: {}",
        rig_id52.id52()
    );

    // 1. Load rig's secret key (we'll reuse this for certificate)
    let rig_secret_key = load_rig_secret_key(rig_id52)?;

    // 2. Generate Subject Alternative Names based on deployment environment
    let sans = generate_certificate_sans(rig_id52).await?;

    // 3. Generate certificate using rig's Ed25519 key
    let cert_pem = generate_certificate_with_rig_key(&rig_secret_key, &sans)?;

    // 4. Store certificate configuration in RigConfig
    store_certificate_in_rig_config(automerge_db, rig_id52, &cert_pem, &sans).await?;

    // 5. Create rustls TLS configuration
    create_tls_config_from_rig_key(&cert_pem, &rig_secret_key).await
}

/// Create TLS config from existing stored certificate
pub async fn create_tls_config_from_stored_cert(
    rig_config: &RigConfig,
    cert_pem: &str,
) -> Result<rustls::ServerConfig, CertificateError> {
    // Load rig's secret key to create TLS config
    let rig_secret_key = load_rig_secret_key(&rig_config.rig)?;
    create_tls_config_from_rig_key(cert_pem, &rig_secret_key).await
}

/// Load rig's secret key from storage (keyring or filesystem)
fn load_rig_secret_key(
    rig_id52: &fastn_id52::PublicKey,
) -> Result<fastn_id52::SecretKey, CertificateError> {
    let id52_string = rig_id52.id52();
    fastn_id52::SecretKey::load_for_id52(&id52_string).map_err(|e| CertificateError::RigKeyLoad {
        source: Box::new(e),
    })
}

/// Generate Subject Alternative Names based on deployment environment
async fn generate_certificate_sans(
    rig_id52: &fastn_id52::PublicKey,
) -> Result<Vec<String>, CertificateError> {
    let mut sans = vec!["localhost".to_string(), "127.0.0.1".to_string()];

    // Add public IP if detectable
    if let Ok(public_ip) = detect_public_ip().await {
        sans.push(public_ip.clone());
        println!("🌐 Added public IP to certificate: {}", public_ip);
    }

    // Add hostname if configured
    if let Ok(hostname) = std::env::var("FASTN_HOSTNAME") {
        sans.push(hostname.clone());
        println!("🏠 Added hostname to certificate: {}", hostname);
    }

    // Add domain if configured
    if let Ok(domain) = std::env::var("FASTN_DOMAIN") {
        sans.push(domain.clone());
        println!("🌍 Added domain to certificate: {}", domain);
    }

    // Add rig-specific local discovery name
    let rig_local = format!("{}.local", rig_id52.id52());
    sans.push(rig_local);

    println!("📜 Certificate will be valid for: {:?}", sans);
    Ok(sans)
}

/// Generate certificate using rig's Ed25519 key
fn generate_certificate_with_rig_key(
    rig_secret_key: &fastn_id52::SecretKey,
    sans: &[String],
) -> Result<String, CertificateError> {
    // Convert rig's Ed25519 key to PKCS#8 format for rcgen
    let raw_key_bytes = rig_secret_key.to_bytes();
    let signing_key = ed25519_dalek::SigningKey::from_bytes(&raw_key_bytes);
    let pkcs8_der = signing_key
        .to_pkcs8_der()
        .map_err(|e| CertificateError::KeyConversion {
            source: Box::new(e),
        })?;

    let private_key_der = rustls::pki_types::PrivateKeyDer::Pkcs8(pkcs8_der.as_bytes().into());
    let key_pair = rcgen::KeyPair::from_der_and_sign_algo(&private_key_der, &rcgen::PKCS_ED25519)
        .map_err(|e| CertificateError::KeyConversion {
        source: Box::new(e),
    })?;

    // Create certificate parameters
    let mut params = rcgen::CertificateParams::new(sans.to_vec()).map_err(|e| {
        CertificateError::CertificateGeneration {
            source: Box::new(e),
        }
    })?;

    // Set certificate subject
    let subject = format!("fastn-rig-{}", &rig_secret_key.public_key().id52()[..8]);
    params
        .distinguished_name
        .push(rcgen::DnType::CommonName, &subject);
    params
        .distinguished_name
        .push(rcgen::DnType::OrganizationName, "fastn");
    params
        .distinguished_name
        .push(rcgen::DnType::OrganizationalUnitName, "P2P Email Server");

    // Set validity period (1 year)
    let now = time::OffsetDateTime::now_utc();
    params.not_before = now;
    params.not_after = now + time::Duration::days(365);

    // Set key usage for email protocols
    params.key_usages = vec![
        rcgen::KeyUsagePurpose::DigitalSignature,
        rcgen::KeyUsagePurpose::KeyEncipherment,
    ];
    params.extended_key_usages = vec![rcgen::ExtendedKeyUsagePurpose::ServerAuth];

    // Generate certificate with our key pair
    let cert =
        params
            .self_signed(&key_pair)
            .map_err(|e| CertificateError::CertificateGeneration {
                source: Box::new(e),
            })?;

    let cert_pem = cert.pem();

    println!("📜 Generated self-signed certificate: {}", subject);
    Ok(cert_pem)
}

/// Store certificate in RigConfig automerge document
async fn store_certificate_in_rig_config(
    automerge_db: &fastn_automerge::Db,
    rig_id52: &fastn_id52::PublicKey,
    cert_pem: &str,
    sans: &[String],
) -> Result<(), CertificateError> {
    // Load current rig config
    let mut rig_config =
        RigConfig::load(automerge_db, rig_id52).map_err(|e| CertificateError::ConfigLoad {
            source: Box::new(e),
        })?;

    // Create certificate configuration
    let now_unix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    // TODO: Store certificate in stable filesystem location instead of automerge
    println!("💾 Certificate generation complete (storage to be implemented)");
    Ok(())
}

/// Create rustls TLS configuration from certificate and rig key
async fn create_tls_config_from_rig_key(
    cert_pem: &str,
    rig_secret_key: &fastn_id52::SecretKey,
) -> Result<rustls::ServerConfig, CertificateError> {
    // Parse certificate from PEM
    let cert_der = rustls_pemfile::certs(&mut cert_pem.as_bytes())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| CertificateError::CertificateParsing {
            source: Box::new(e),
        })?;

    if cert_der.is_empty() {
        return Err(CertificateError::CertificateParsing {
            source: "No certificates found in PEM data".into(),
        });
    }

    // Convert Ed25519 key to rustls format
    let raw_key_bytes = rig_secret_key.to_bytes();
    let signing_key = ed25519_dalek::SigningKey::from_bytes(&raw_key_bytes);
    let pkcs8_der = signing_key
        .to_pkcs8_der()
        .map_err(|e| CertificateError::KeyConversion {
            source: Box::new(e),
        })?;

    let private_key = rustls::pki_types::PrivateKeyDer::Pkcs8(pkcs8_der.as_bytes().into());

    // Create TLS configuration
    let config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert_der, private_key.clone_key())
        .map_err(|e| CertificateError::TlsConfigCreation {
            source: Box::new(e),
        })?;

    println!("🔐 TLS configuration created successfully");
    Ok(config)
}

/// Detect public IP address for certificate SANs
async fn detect_public_ip() -> Result<String, CertificateError> {
    // Check if public IP is explicitly configured
    if let Ok(configured_ip) = std::env::var("FASTN_PUBLIC_IP") {
        return Ok(configured_ip);
    }

    // For now, skip automatic detection to keep module simple
    // TODO: Add HTTP client for public IP detection
    Err(CertificateError::PublicIpDetection {
        source: "Public IP auto-detection not implemented yet".into(),
    })
}
