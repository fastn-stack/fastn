# SMTP STARTTLS Implementation Design

## Overview

This document outlines the design for adding STARTTLS support to fastn's SMTP server to enable secure email client compatibility. STARTTLS (port 587) is prioritized over SSL (port 465) for better code reuse and modern standard compliance.

## Current State

- ✅ **Working SMTP server** - fastn-rig runs SMTP on port 2525 (configurable)
- ✅ **Multi-account routing** - Routes emails to correct fastn accounts
- ✅ **P2P delivery** - Emails delivered via fastn-p2p to remote inboxes
- ✅ **Plain text only** - No encryption support yet

## Goal

Enable standard email clients to connect securely to fastn's SMTP server using industry-standard SSL/TLS encryption.

## SSL vs STARTTLS Decision

### Recommendation: **Implement Both**

**Explicit SSL (Port 465):**
- Connection starts encrypted from the beginning
- Simpler implementation - wrap entire connection in TLS
- Legacy standard, but still widely used

**STARTTLS (Port 587/25):**
- Connection starts plain text, upgrades to TLS after STARTTLS command
- More complex - need to handle upgrade process
- Modern standard, preferred by most clients

### Implementation Priority
1. **Phase 1**: Explicit SSL (Port 465) - easier to implement and test
2. **Phase 2**: STARTTLS (Port 587) - more complex upgrade mechanism

## Technical Architecture

### TLS Library Choice

**Recommendation: `rustls`**
- Already in dependencies (used by iroh, reqwest, etc.)
- Pure Rust, memory-safe
- Good performance and security track record
- Avoids OpenSSL dependency complexity

### Certificate Management Strategy

#### **The P2P Reality: Why Self-Signed is the Only Option for Most Users**

**Core Issue: P2P vs Domain-Based Certificates**
- **P2P nature**: Anyone can generate an ID52 and start sending emails without domains/IPs
- **CA Requirements**: Let's Encrypt and all CAs require domain ownership validation
- **Fundamental conflict**: P2P privacy model vs. CA verification model

**Certificate Strategy by Use Case:**

**For P2P-Only Users (Majority):**
- **Self-signed certificates ONLY** - No domain required
- **Privacy-first approach** - No external validation needed
- **Client recommendations** - We guide users to compatible email clients

**For Domain Owners (Future Enhancement):**
- **FNAME DNS mapping** - Map domain to ID52
- **Let's Encrypt integration** - CA-signed certificates for domain owners
- **Hybrid approach** - Support both certificate types

#### **Design Decision: Single Rig Certificate (Networking and Privacy Reality)**

**Why One Certificate Per Rig (Not Per Account):**
- **Network constraints**: Can't require `{account_id52}.com` domains (no DNS)
- **Port limitations**: Corporate firewalls restrict available ports (465, 587 standard)  
- **Port management**: Account-specific ports would create networking complexity
- **IP address reality**: Single rig = single public IP, so account hosting is already visible

**Privacy Analysis:**
- **Certificate doesn't decrease privacy** - Rig ID52 in certificate doesn't reveal more than IP address already does
- **IP-level visibility**: If 10 accounts run on same rig, they share public IP anyway
- **SMTP AUTH privacy**: Individual account access still protected by authentication
- **Certificate scope**: Enables encryption, doesn't expose account details

**Certificate Scope:**
- **Subject**: Uses rig ID52 as identifier (`CN=fastn-rig-{rig_id52_prefix}`)
- **SANs**: `localhost`, `127.0.0.1`, `{rig_id52}.local`
- **Protocol scope**: Covers SMTP, IMAP, and future email protocols
- **Reusable**: Same certificate for all email protocols on this rig

**Phase 1: Self-Signed Rig Certificates**
- Generate one certificate per rig using rig's ID52
- Support standard SMTP SSL ports (465)
- Support compatible email clients (Thunderbird, FairEmail, Apple Mail)  
- Clear documentation about client limitations

**Future Phase: Domain Integration (For Rig Operators with Domains)**
- FNAME support for domain → rig mapping  
- Let's Encrypt integration for rig operators with domains
- Domain-based certificates (`mail.example.com`)
- **Certificate reuse**: Same certificate for SMTP, IMAP, and all email protocols
- **External certificate support**: Import existing Let's Encrypt certificates
- Choice between privacy (self-signed) vs compatibility (domain-based)

### Port Configuration (STARTTLS-First Strategy)

```
Port 2525: Plain text SMTP (development/testing)
Port 587:  STARTTLS SMTP (primary production port)
Port 465:  SSL-wrapped SMTP (no current plans to implement)
```

**Environment Variables:**
```bash
# Core SMTP Configuration (works for both deployment modes)
FASTN_SMTP_PORT=2525           # Plain text (localhost/public)
FASTN_SMTP_STARTTLS_PORT=587   # STARTTLS (primary secure port)
# FASTN_SMTP_SSL_PORT=465      # SSL (not planned unless email client analysis shows need)

# Deployment Mode Configuration  
FASTN_PUBLIC_IP=203.0.113.42   # Override public IP detection
FASTN_HOSTNAME=mail.example.com # Hostname for certificate SANs
FASTN_DOMAIN=example.com        # Domain for certificate SANs
```

**Port Strategy:**
- **Port 587 (STARTTLS)**: Primary secure email submission port (modern standard)
- **Port 2525**: Development and testing (existing)
- **Port 465 (SSL)**: Not implemented unless email client compatibility analysis shows critical need

**Port Accessibility:**
- **Localhost deployment**: `127.0.0.1:587` for secure local clients
- **Public deployment**: `0.0.0.0:587` for internet email clients
- **Firewall considerations**: Port 587 is standard email submission port, widely allowed

## Implementation Plan

### Phase 1: STARTTLS (Port 587) - Primary Implementation

#### 1. Certificate Generation Module (`fastn-rig/src/certs/`)

**Files:**
- `mod.rs` - Public API
- `self_signed.rs` - Self-signed certificate generation
- `storage.rs` - Certificate storage and loading
- `errors.rs` - Certificate-specific error types

**Certificate Generation Details:**
```rust
pub struct CertificateManager {
    cert_dir: PathBuf,           // {fastn_home}/certs/
    rig_id52: String,           // Used for keyring entries
    skip_keyring: bool,         // From SKIP_KEYRING env var
}

impl CertificateManager {
    pub fn new(fastn_home: &Path) -> Result<Self, CertError> {
        let cert_dir = fastn_home.join("certs");
        let rig_id52 = Self::load_rig_id52(fastn_home)?;
        let skip_keyring = std::env::var("SKIP_KEYRING").unwrap_or_default() == "true";
        
        Ok(Self { cert_dir, rig_id52, skip_keyring })
    }
    
    /// Generate or load TLS configuration for SMTP server
    pub async fn get_or_create_tls_config(&self) -> Result<rustls::ServerConfig, CertError>;
    
    /// Generate new self-signed certificate
    async fn generate_certificate(&self) -> Result<(String, String), CertError>; // (cert_pem, key_pem)
    
    /// Load existing certificate from storage
    async fn load_certificate(&self) -> Result<(String, String), CertError>;
    
    /// Check if certificate exists and is valid
    pub fn certificate_exists(&self) -> bool;
    
    /// Get certificate metadata for display
    pub fn certificate_info(&self) -> Result<CertInfo, CertError>;
}
```

**Certificate Generation Specification:**
```rust
async fn generate_certificate(&self) -> Result<(String, String), CertError> {
    use rcgen::{Certificate, CertificateParams, DistinguishedName};
    
    let mut params = CertificateParams::new(vec![
        "localhost".to_string(),
        "127.0.0.1".to_string(),
        format!("{}.local", self.rig_id52), // e.g., abc123...xyz.local
    ]);
    
    // Certificate subject uses rig ID52 (networking reality)
    let mut dn = DistinguishedName::new();
    dn.push(rcgen::DnType::CommonName, &format!("fastn-rig-{}", &self.rig_id52[..8]));
    dn.push(rcgen::DnType::OrganizationName, "fastn");
    dn.push(rcgen::DnType::OrganizationalUnitName, "P2P Email Server");
    params.distinguished_name = dn;
    
    // Validity period: 1 year
    params.not_before = time::OffsetDateTime::now_utc();
    params.not_after = time::OffsetDateTime::now_utc() + time::Duration::days(365);
    
    // Key usage
    params.key_usages = vec![
        rcgen::KeyUsagePurpose::DigitalSignature,
        rcgen::KeyUsagePurpose::KeyEncipherment,
    ];
    params.extended_key_usages = vec![
        rcgen::ExtendedKeyUsagePurpose::ServerAuth,
    ];
    
    let cert = Certificate::from_params(params)?;
    let cert_pem = cert.serialize_pem()?;
    let key_pem = cert.serialize_private_key_pem();
    
    Ok((cert_pem, key_pem))
}
```

**Certificate Storage:**
```
~/.fastn/
├── certs/
│   ├── fastn.crt     # Self-signed certificate
│   ├── fastn.key     # Private key
│   └── cert.info     # Metadata (creation date, expiry, etc.)
```

#### 2. SSL SMTP Server (`fastn-rig/src/smtp/ssl.rs`)

**Key Components:**
```rust
pub struct SslSmtpServer {
    tls_acceptor: TlsAcceptor,
    smtp_handler: SmtpHandler, // Reuse existing SMTP logic
}

pub async fn start_ssl_smtp_server(
    account_manager: Arc<AccountManager>,
    port: u16,
    cert_manager: CertificateManager,
) -> Result<(), SmtpSslError>;
```

**Architecture:**
- Wrap existing `SmtpHandler` with TLS layer
- Reuse all SMTP protocol logic (authentication, routing, etc.)
- Handle TLS handshake before SMTP conversation

#### 3. Integration with Run Command

**Modify `fastn-rig run`:**
- Start both plain and SSL SMTP servers concurrently
- Log which ports are listening
- Handle certificate generation/loading

```rust
// In run.rs
async fn start_smtp_servers(
    account_manager: Arc<AccountManager>,
) -> Result<(), RunError> {
    // Start plain text server (existing)
    let plain_port = std::env::var("FASTN_SMTP_PORT")
        .unwrap_or("2525".to_string())
        .parse()?;
    
    // Start SSL server (new)
    let ssl_port = std::env::var("FASTN_SMTP_SSL_PORT")
        .unwrap_or("465".to_string())
        .parse()?;
    
    let cert_manager = CertificateManager::new(&fastn_home);
    
    tokio::try_join!(
        start_smtp_server(account_manager.clone(), plain_port),
        start_ssl_smtp_server(account_manager, ssl_port, cert_manager)
    )?;
    
    Ok(())
}
```

### Phase 2: STARTTLS (Port 587)

**Additional complexity:**
- Parse STARTTLS command in SMTP protocol
- Upgrade existing connection to TLS mid-stream
- Handle state machine for upgrade process

## Testing Strategy

### Unit Tests
- Certificate generation and loading
- TLS handshake simulation
- SSL SMTP command processing

### Integration Tests

**Test 1: Self-Signed Certificate Generation**
```rust
#[tokio::test]
async fn test_certificate_generation() {
    let temp_dir = TempDir::new()?;
    let cert_manager = CertificateManager::new(temp_dir.path());
    let (cert, key) = cert_manager.ensure_certificates().await?;
    // Validate certificate format, expiry, etc.
}
```

**Test 2: SSL SMTP Connection**
```rust
#[tokio::test]
async fn test_ssl_smtp_connection() {
    let mut test_env = FastnTestEnv::new("ssl-smtp")?;
    let peer = test_env.create_peer("peer1").await?;
    
    // Start with SSL enabled
    test_env.start_peer_with_ssl("peer1").await?;
    
    // Connect with TLS client and send email
    let result = test_env.email_ssl()
        .from("peer1") 
        .to("peer1") // Self-send for testing
        .subject("SSL Test")
        .send()
        .await?;
        
    result.expect_success()?;
}
```

**Test 3: Real Email Client Compatibility**
- Document how to configure Thunderbird/other clients
- Manual testing checklist
- Connection parameters documentation

### Error Handling

**New Error Types:**
```rust
#[derive(thiserror::Error, Debug)]
pub enum SmtpSslError {
    #[error("Certificate generation failed")]
    CertificateGeneration { source: Box<dyn std::error::Error> },
    
    #[error("TLS handshake failed")]
    TlsHandshake { source: rustls::Error },
    
    #[error("SSL acceptor creation failed")]
    AcceptorCreation { source: Box<dyn std::error::Error> },
}
```

## Configuration

### Environment Variables
```bash
# Existing
FASTN_SMTP_PORT=2525              # Plain text SMTP

# New
FASTN_SMTP_SSL_PORT=465           # SSL SMTP  
FASTN_CERT_PATH=/custom/cert/dir  # Optional custom cert location
FASTN_SKIP_SSL=true              # Disable SSL server for testing
```

### Certificate Lifecycle and Generation

#### When Certificates Are Generated

**Trigger: First SMTP SSL Server Start**
- Certificates generated lazily when SSL SMTP server starts
- NOT during `fastn-rig init` (keeps init lightweight)
- Generation happens in `start_ssl_smtp_server()` function

**Generation Flow:**
```rust
// In start_ssl_smtp_server()
let cert_manager = CertificateManager::new(&fastn_home);

// This call generates if missing, loads if exists
let tls_config = cert_manager.get_or_create_tls_config().await?;
```

#### Certificate Storage Architecture: Automerge-First Design

**Two Certificate Modes:**

### Mode 1: Self-Signed Certificates (fastn-managed)
**Storage**: In rig's automerge document
**Benefits**: 
- Remote certificate management via automerge sync
- No custom certificate sync protocol needed
- Account managing rig from another location can update certificates
- Distributed certificate deployment

**RigConfig Integration:**
```rust
// Add to existing RigConfig structure
impl RigConfig {
    pub email_certificate: Option<EmailCertificateConfig>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum EmailCertificateConfig {
    SelfSigned {
        cert_pem: String,                    // Certificate stored in RigConfig
        generated_at: time::OffsetDateTime,
        expires_at: time::OffsetDateTime,
        subject: String,
        sans: Vec<String>,
        // Note: uses_rig_key is implicit - self-signed always uses rig key
    },
    External {
        cert_path: String,                   // Path to external certificate file
        key_path: String,                    // Path to external private key file
        domain: String,                      // Domain name for the certificate
        auto_reload: bool,                   // Watch for certificate file changes
        last_reload: time::OffsetDateTime,   // When certificate was last loaded
        fallback_to_self_signed: bool,       // Generate self-signed if external fails
    },
}
```

**Simplified Key Storage (Using Rig's Secret Key):**
- **Self-signed mode**: `uses_rig_key: true` - No separate private key storage needed
- **External mode**: Points to external certificate files managed by nginx/certbot

### Mode 2: External Certificates (nginx/Let's Encrypt path) 
**Storage**: File path configuration in RigConfig
**Benefits**:
- Reuse existing Let's Encrypt certificates from nginx
- No certificate duplication  
- Automatic updates when nginx certificates renew
- Shared certificate management across services

**RigConfig Structure for External Certificates:**
```rust
// In RigConfig.email_certificate
EmailCertificateConfig {
    mode: CertificateMode::External,
    cert_pem: None,  // Not stored in automerge
    cert_path: Some("/etc/letsencrypt/live/example.com/fullchain.pem"),
    key_path: Some("/etc/letsencrypt/live/example.com/privkey.pem"),
    uses_rig_key: false,  // Uses external private key
    domain: Some("mail.example.com"),
    auto_reload: true,
    generated_at: cert_file_creation_time,
    expires_at: cert_expiry_from_file,
    // ... other metadata
}
```

**File System State:**
```
{fastn_home}/
├── .fastn.lock
├── rig/
│   └── automerge.sqlite    # Contains certificate config/data
└── accounts/
    └── {account_id}/
```

**No dedicated certificate files** - Everything managed through RigConfig automerge or external paths!

## 🌐 **Dual Deployment Architecture**

### **Deployment Mode Detection and Certificate Adaptation**

**Challenge**: Same fastn-rig binary must work for both localhost and public IP deployments
**Solution**: Dynamic SAN generation based on deployment environment

**SAN Generation Logic:**
```rust
impl CertificateManager {
    async fn generate_sans(&self) -> Vec<String> {
        let mut sans = vec![
            "localhost".to_string(),
            "127.0.0.1".to_string(),
        ];
        
        // Detect deployment mode and add appropriate SANs
        
        // 1. Add public IP if detectable
        if let Ok(public_ip) = self.detect_public_ip().await {
            sans.push(public_ip);
            println!("🌐 Added public IP to certificate: {}", public_ip);
        }
        
        // 2. Add hostname if set
        if let Ok(hostname) = std::env::var("FASTN_HOSTNAME") {
            sans.push(hostname.clone());
            println!("🏠 Added hostname to certificate: {}", hostname);
        }
        
        // 3. Add domain if configured (for Let's Encrypt compat)
        if let Ok(domain) = std::env::var("FASTN_DOMAIN") {
            sans.push(domain.clone());
            println!("🌍 Added domain to certificate: {}", domain);
        }
        
        // 4. Add rig-specific mDNS name for local discovery
        sans.push(format!("{}.local", &self.rig_id52));
        
        println!("📜 Certificate will be valid for: {:?}", sans);
        sans
    }
}
```

### **Environment Configuration for Deployment Modes**

**Localhost Deployment (Privacy-Focused):**
```bash
# No additional environment variables needed
# Certificate automatically includes localhost, 127.0.0.1, {rig_id52}.local
FASTN_SMTP_SSL_PORT=465  # Default SSL port
```

**Public IP Deployment (P2P Network):**
```bash
# For public IP detection
FASTN_PUBLIC_IP=203.0.113.42        # Optional: override auto-detection
FASTN_HOSTNAME=mail.example.com      # Optional: if hostname available
FASTN_DOMAIN=example.com             # Optional: for future Let's Encrypt

# Standard ports for external access
FASTN_SMTP_SSL_PORT=465              # SSL SMTP
FASTN_SMTP_PORT=587                  # Plain/STARTTLS (future)
```

### **Certificate Validity Across Deployment Modes**

**Self-Signed Certificate Covers:**
- ✅ **Localhost connections** - Always included
- ✅ **Public IP connections** - Auto-detected and included  
- ✅ **Hostname connections** - If hostname provided
- ✅ **Local network discovery** - `{rig_id52}.local` for mDNS
- ✅ **Future domain** - If domain configured

**Single Certificate Works For:**
- ✅ **Local privacy users** - Connect via localhost
- ✅ **P2P network operators** - Connect via public IP
- ✅ **Domain owners** - Connect via hostname (self-signed) or Let's Encrypt (external)

**This means the same rig can serve:**
- Privacy-focused users connecting locally (`localhost:465`)
- Remote users connecting over internet (`{public_ip}:465`)
- Domain-based connections (`mail.example.com:465`)
- Multiple deployment scenarios with single certificate

### **Testing Strategy for Both Deployment Modes**

**Unit Tests (Fast, No Network):**
- Certificate generation with various SAN combinations
- RigConfig serialization/deserialization
- Mock certificate manager with in-memory certificates

**Integration Tests (Localhost Mode):**
- SSL SMTP server startup on `localhost:465`
- Thunderbird connection test via `localhost:465`
- Self-signed certificate trust verification
- P2P delivery through localhost SSL

**Integration Tests (Public IP Mode):**
- Certificate includes detected public IP in SANs
- SSL server accessible from external IP
- Remote email client connection testing
- P2P delivery between rigs over public internet

**Manual Testing Scenarios:**
1. **Localhost privacy setup** - Developer running fastn-rig locally
2. **VPS deployment** - Public rig with static IP
3. **Domain deployment** - Public rig with domain name
4. **Mixed clients** - Local + remote clients connecting to same rig

#### Automerge + Keyring Integration Strategy

**With Keyring (SKIP_KEYRING=false):**
- Certificate stored in automerge document
- Private key stored in system keyring  
- Automerge contains keyring reference: `{"key_storage": "keyring", "keyring_id": "{rig_id52}"}`
- Keyring entry: `fastn.email.{rig_id52}.privatekey`

**Without Keyring (SKIP_KEYRING=true):**
- Both certificate and private key stored directly in automerge document
- Private key encrypted in automerge using rig's secret key
- Automerge contains: `{"key_storage": "automerge", "key_pem": "encrypted_private_key"}`

**Implementation:**
```rust
impl CertificateManager {
    async fn generate_and_store_certificate(&self) -> Result<(), CertError> {
        // Generate certificate using rig's existing Ed25519 key (no new key needed!)
        let rig_secret_key = self.rig_document.get_secret_key().await?;
        let cert_pem = self.generate_certificate_with_rig_key(&rig_secret_key).await?;
        
        let cert_config = EmailCertificateConfig::SelfSigned {
            cert_pem,
            generated_at: time::OffsetDateTime::now_utc(),
            expires_at: time::OffsetDateTime::now_utc() + time::Duration::days(365),
            subject: format!("CN=fastn-rig-{}", &self.rig_id52[..8]),
            sans: vec!["localhost".to_string(), "127.0.0.1".to_string()],
        };
        
        // Store in RigConfig
        let mut rig_config = self.rig_document.get_config().await?;
        rig_config.email_certificate = Some(cert_config);
        self.rig_document.set_config(rig_config).await?;
        
        println!("🔐 Self-signed certificate generated and stored in RigConfig");
        println!("🔑 Using rig's secret key - no additional key storage needed");
        Ok(())
    }

    async fn store_external_certificate_config(
        &self,
        cert_path: String,
        key_path: String,
        domain: Option<String>
    ) -> Result<(), CertError> {
        let cert_config = EmailCertificateConfig {
            mode: CertificateMode::External,
            cert_path: Some(cert_path),
            key_path: Some(key_path), 
            domain,
            auto_reload: true,
            fallback_to_self_signed: true,
            last_reload: Some(time::OffsetDateTime::now_utc()),
            // ... other fields
        };
        
        self.rig_document.set("email_certificate", cert_config).await?;
        println!("📁 External certificate configuration stored in automerge");
        Ok(())
    }
}
```

## Security Considerations

### Self-Signed Certificates
- **Pros**: No external dependencies, works offline, automatic setup
- **Cons**: Browser/client warnings, manual trust required
- **Mitigation**: Clear documentation for users on accepting certificates

### Private Key Security
- Store private keys with appropriate file permissions (600)
- Consider keyring integration for key storage
- Generate strong keys (RSA 2048+ or ECDSA P-256+)

### TLS Configuration
- Use modern TLS versions only (1.2+)
- Strong cipher suites
- Proper certificate validation

## User Experience

### First-Time Setup
1. User runs `fastn-rig run`
2. System auto-generates self-signed certificate
3. Displays connection instructions:
   ```
   📧 SMTP Servers Started:
   ┌─────────────────────────────────────┐
   │ Plain Text: localhost:2525          │
   │ SSL:        localhost:465           │
   │                                     │
   │ For email clients:                  │
   │ Server: localhost                   │
   │ Port: 465                          │
   │ Security: SSL/TLS                   │
   │ Auth: account@id52.com / password   │
   └─────────────────────────────────────┘
   ```

### Email Client STARTTLS + Self-Signed Certificate Support (2024 Analysis)

| Email Client | STARTTLS Port 587 | Self-Signed Cert Support | Configuration Method | Notes |
|--------------|------------------|-------------------------|---------------------|-------|
| **Thunderbird** | ✅ **Excellent** | ✅ **Full Support** | Certificate Manager → Add Exception | Best overall support for port 587 + self-signed |
| **Apple Mail (macOS)** | ✅ **Good** | ⚠️ **Manual Trust** | Keychain Access → Trust Settings | STARTTLS preferred over SSL on macOS |
| **Apple Mail (iOS)** | ✅ **Preferred** | ⚠️ **Complex Setup** | iOS Certificate Trust Settings | iOS demands STARTTLS over SSL |
| **FairEmail** | ✅ **Excellent** | ✅ **Configurable** | "Allow insecure connections" setting | Best Android option for STARTTLS |
| **K-9 Mail** | ✅ **Standard** | ⚠️ **Becoming Restrictive** | Certificate exceptions | STARTTLS support maintained |
| **Outlook Desktop** | ✅ **Standard** | ⚠️ **Manual Import** | certmgr.msc → Trusted Root CAs | Works with STARTTLS |
| **Outlook/Office 365** | ✅ **For External** | ❌ **CA Required** | OAuth complications | STARTTLS works but needs CA certs |
| **Gmail App** | ❌ **No Support** | ❌ **CA Required** | Google blocks self-signed | Cannot work with self-signed |

### **Key Findings for STARTTLS**

✅ **STARTTLS is preferred by mobile clients** - iOS specifically "demands STARTTLS over SSL"  
✅ **Better compatibility** - Port 587 is modern standard, more widely supported  
✅ **Same certificate challenges** - Self-signed certificate issues persist but STARTTLS doesn't make them worse

### Detailed Configuration Guides

#### Thunderbird (Primary Target - Excellent STARTTLS Support)
**Why**: Best support for STARTTLS + self-signed certificates
```
1. Account Setup:
   - Server: localhost (or public IP/domain)
   - Port: 587
   - Security: STARTTLS
   - Authentication: Normal password

2. Certificate Trust:
   - Tools → Settings → Privacy & Security → Certificates → View Certificates
   - Servers tab → Add Exception
   - Enter: localhost:587 → Get Certificate → Confirm Security Exception

3. Alternative (Manual):
   - Edit cert_override.txt in Thunderbird profile
   - Add line for port 587 with certificate fingerprint
```

#### Apple Mail (Secondary Target - Prefers STARTTLS)
**Why**: iOS "demands STARTTLS over SSL", good compatibility with port 587
```
Account Setup (Both macOS and iOS):
1. Server: localhost or public IP/domain
2. Port: 587  
3. Security: STARTTLS
4. Authentication: Password

Certificate Trust (macOS):
1. First connection shows certificate warning
2. Click "Show Certificate" → "Always Trust" 
3. Enter admin password to confirm

Certificate Trust (iOS):
1. Email certificate to device → Install profile
2. Settings → General → About → Certificate Trust Settings  
3. Enable "Full Trust" for the certificate
```

#### FairEmail (Android Primary - Excellent STARTTLS Support)
**Why**: Best Android option for STARTTLS + self-signed certificates
```
Account Setup:
1. Server: public IP or localhost
2. Port: 587
3. Security: STARTTLS
4. Authentication: Password

Self-Signed Certificate Handling:
1. Advanced settings → "Allow insecure connections" (enable)
2. Connection settings → Enable Bouncy Castle provider
3. Will prompt for certificate acceptance on first connection
```

### Client Testing Priority for STARTTLS

**Tier 1: Excellent STARTTLS + Self-Signed Support**
1. **Thunderbird** - Best overall compatibility (all platforms)
2. **FairEmail** - Best Android option, configurable security

**Tier 2: Good STARTTLS, Manual Certificate Setup**  
3. **Apple Mail** - iOS prefers STARTTLS, but complex cert trust
4. **K-9 Mail** - Standard STARTTLS, but restrictive cert policies

**Tier 3: STARTTLS Works, Limited Self-Signed Support**
5. **Outlook Desktop** - STARTTLS standard, manual cert import required

**Cannot Support (Regardless of STARTTLS):**
- ❌ **Gmail App** - Google blocks all self-signed certificates
- ❌ **Outlook/Office 365** - Microsoft requires CA-signed certificates

### Clients We Cannot Support (Due to P2P/Self-Signed Limitations)
- ❌ **Gmail App** - Google blocks self-signed certificates completely
- ❌ **Outlook/Office 365** - Microsoft requires CA-signed certificates 
- ❌ **Most modern mobile email apps** - Corporate security policies block self-signed
- ❌ **Enterprise email clients** - Company policies often block self-signed certs

### Our Strategy: Privacy-First Email Client Recommendations

**For fastn users, we recommend these compatible clients:**
1. **Thunderbird** (Windows/Mac/Linux) - Primary recommendation
2. **Apple Mail** (macOS/iOS) - With setup guidance  
3. **FairEmail** (Android) - Privacy-focused, configurable security
4. **Manual SMTP clients** - For developers (telnet, custom tools)

**User Communication:**
```
📧 fastn Email Setup Complete!

Your fastn email server supports privacy-first P2P delivery.
For the best experience, we recommend these email clients:

🥇 Thunderbird (easiest setup): https://thunderbird.net
🥈 Apple Mail (requires cert trust): [setup guide]
🥉 FairEmail (Android): [setup guide]

⚠️  Corporate email clients (Gmail, Outlook) require domain certificates
   and cannot connect to P2P email servers.
```

#### Certificate Initialization Flow

**Complete Flow in `start_ssl_smtp_server()`:**
```rust
pub async fn start_ssl_smtp_server(
    account_manager: Arc<AccountManager>,
    port: u16,
    fastn_home: &Path,
) -> Result<(), SmtpSslError> {
    println!("🔐 Initializing SSL SMTP server on port {port}...");
    
    // 1. Create certificate manager
    let cert_manager = CertificateManager::new(fastn_home)?;
    
    // 2. Display current certificate status
    if cert_manager.certificate_exists() {
        let info = cert_manager.certificate_info()?;
        println!("📜 Found existing SSL certificate:");
        println!("   Subject: {}", info.subject);
        println!("   Expires: {}", info.not_after);
        println!("   SANs: {:?}", info.subject_alt_names);
        
        // Check if certificate is near expiry (< 30 days)
        if info.expires_soon() {
            println!("⚠️  Certificate expires soon, consider regenerating");
        }
    } else {
        println!("📜 No SSL certificate found, generating new one...");
    }
    
    // 3. Get or create TLS configuration (this handles generation/loading)
    let tls_config = cert_manager.get_or_create_tls_config().await?;
    
    // 4. Create TLS acceptor
    let acceptor = TlsAcceptor::from(Arc::new(tls_config));
    
    // 5. Start SSL SMTP server with existing SMTP handler
    let ssl_server = SslSmtpServer::new(acceptor, account_manager);
    ssl_server.start(port).await?;
    
    Ok(())
}
```

**Error Scenarios and Handling:**

1. **Certificate Generation Fails:**
   - Fallback to plain text SMTP only
   - Log detailed error with troubleshooting steps
   - Continue serving on plain text port

2. **Keyring Access Fails:**
   - If `SKIP_KEYRING=false` but keyring unavailable
   - Prompt user to either enable SKIP_KEYRING or fix keyring
   - Don't fall back silently to filesystem storage

3. **File Permission Issues:**
   - Clear error messages about directory permissions
   - Suggest `chmod` commands to fix permissions
   - Validate certificate directory is writable

4. **Certificate Loading Fails:**
   - If existing certificate is corrupted
   - Backup old certificate and generate new one
   - Log the backup location for recovery

#### Integration with Existing Systems

**Rig Initialization (`fastn-rig init`):**
- **NO certificate generation during init** - keeps init fast
- Certificate directory creation happens during init
- Sets up proper directory structure only

**Rig Run (`fastn-rig run`):**
- Certificate generation happens here (lazy initialization)
- Both plain text and SSL servers start concurrently
- Clear status display about which ports are active

**Testing Integration:**
- `FastnTestEnv` supports SSL peer creation
- Tests can specify SSL vs plain text mode
- Integration tests cover both keyring modes

## Potential Implementation Questions

### 1. Certificate Subject/SAN
- What should be the certificate Common Name?
- Should we support multiple Subject Alternative Names?
- How to handle `localhost` vs actual hostname?

### 2. TLS Version Support
- Which TLS versions to support? (1.2, 1.3 only?)
- How to handle client compatibility?
- Cipher suite selection strategy?

### 3. Error Handling
- How to handle certificate expiry?
- What if certificate generation fails?
- Fallback behavior when SSL server fails to start?

### 4. Multi-Port Management
- Should SSL and plain text servers share the same SMTP handler code?
- How to handle graceful shutdown of multiple servers?
- Port conflict detection and handling?

### 5. Testing with Real Clients
- How to automate testing with actual email clients?
- Should we have CI tests that connect via SSL?
- How to handle certificate trust in automated tests?

### 6. Certificate Rotation
- When to regenerate certificates?
- How to handle certificate updates without downtime?
- Should we support hot-reloading of certificates?

### 7. Performance Considerations
- TLS handshake overhead on connection
- Certificate loading and caching strategy
- Connection pooling implications?

## Success Criteria

### Phase 1 Complete When:
- [ ] Self-signed certificates auto-generate on first run
- [ ] SSL SMTP server starts on port 465
- [ ] Can connect from Thunderbird with manual certificate trust
- [ ] Emails send successfully through SSL connection
- [ ] P2P delivery works end-to-end via SSL SMTP
- [ ] Integration tests cover SSL path
- [ ] Documentation for email client setup

## 📜 **Let's Encrypt Certificate Reuse (Future Enhancement)**

### Certificate Protocol Compatibility

**✅ Let's Encrypt certificates work across ALL protocols:**
- **Email protocols**: SMTP (ports 25, 587, 465), IMAP (port 993), POP3 (port 995)
- **Web protocols**: HTTP/HTTPS (ports 80, 443)
- **Any TLS-based protocol**: The certificate is protocol-agnostic

**Key insight from research**: Standard TLS certificates are protocol-agnostic. The same certificate file can secure multiple services.

### External Certificate Support: nginx/Let's Encrypt Integration

**Primary Use Case: nginx + Let's Encrypt Coexistence**
- **Rig operator has domain** - Already uses nginx with Let's Encrypt
- **Certificate sharing** - fastn reuses nginx's certificates
- **No duplication** - Single certificate file managed by nginx/certbot
- **Automatic renewal** - Certificate updates via nginx's certbot cron

**Common nginx + Let's Encrypt Certificate Locations:**
```bash
# Standard Let's Encrypt paths (managed by certbot)
/etc/letsencrypt/live/example.com/fullchain.pem  # Certificate
/etc/letsencrypt/live/example.com/privkey.pem    # Private key

# nginx configuration typically points to these same files
# fastn reads from same location - no copying needed
```

**fastn-rig External Certificate Commands:**
```bash
# Configure fastn to use nginx's Let's Encrypt certificate
fastn-rig cert external \
  --cert /etc/letsencrypt/live/example.com/fullchain.pem \
  --key /etc/letsencrypt/live/example.com/privkey.pem \
  --domain mail.example.com

# Show current certificate configuration
fastn-rig cert status

# Fallback to self-signed if external fails
fastn-rig cert fallback-self-signed
```

**Benefits of External Certificate Approach:**
- ✅ **No certificate duplication** - Single source of truth
- ✅ **Automatic updates** - nginx/certbot handles renewal
- ✅ **Broader client compatibility** - CA-signed certificates work everywhere
- ✅ **Shared infrastructure** - Reuse existing domain certificate setup

**Future fastn-rig Commands:**
```bash
# Import external certificate
fastn-rig cert import --cert /path/to/cert.pem --key /path/to/key.pem

# Configure to use Let's Encrypt certificate  
fastn-rig cert external --domain mail.example.com --letsencrypt

# Show current certificate info
fastn-rig cert info

# Generate new self-signed certificate
fastn-rig cert generate --force
```

**Implementation Strategy:**
- **Self-signed by default** - Works out of the box
- **External certificate detection** - Check for config.json on startup
- **Hot reloading** - Watch external certificate files for updates
- **Graceful fallback** - Use self-signed if external certificates fail

### Future Enhancements
- STARTTLS support (port 587)
- Let's Encrypt automatic integration (`fastn-rig cert letsencrypt --domain example.com`)
- Certificate management web UI
- Certificate monitoring and expiry alerts
- Multi-domain certificate support

## Dependencies

### New Dependencies Required

**Add to `fastn-rig/Cargo.toml`:**
```toml
[dependencies]
# Certificate generation
rcgen = "0.13"           # Self-signed certificate generation
time = "0.3"            # For certificate validity periods

# TLS (already available but need to ensure proper features)
rustls = "0.23"         # TLS implementation  
tokio-rustls = "0.26"   # Async TLS streams
rustls-pemfile = "2.0"  # PEM parsing utilities

# Optional: Certificate parsing for validation
x509-parser = "0.16"    # For certificate inspection/validation
```

**Cargo.toml Feature Flags:**
```toml
[features]
default = ["ssl"]
ssl = ["rcgen", "rustls", "tokio-rustls", "rustls-pemfile", "time"]
```

### Existing Dependencies to Leverage
- ✅ `keyring` - Already used for rig key storage
- ✅ `tokio` - Async runtime 
- ✅ `thiserror` - Error handling
- ✅ `serde` + `serde_json` - Certificate metadata storage

### Directory Structure Changes

**Update `fastn-rig init` to create certificate directory:**
```rust
// In rig.rs create() function
async fn create(fastn_home: PathBuf) -> Result<...> {
    // ... existing init logic ...
    
    // Create certificate directory structure
    let cert_dir = fastn_home.join("certs");
    std::fs::create_dir_all(&cert_dir).map_err(|e| {
        RigCreateError::CertDirectoryCreation {
            path: cert_dir.clone(),
            source: e,
        }
    })?;
    
    // Create backup directory
    std::fs::create_dir_all(cert_dir.join("backup"))?;
    
    println!("📁 Certificate directory created: {}", cert_dir.display());
    
    // ... rest of init ...
}
```

## 🔍 **Critical Implementation Design Decisions**

### **Decision 1: SMTP Session Stream Abstraction**

**Problem**: Current `SmtpSession` hardcoded to `TcpStream`, SSL needs `TlsStream`

**Current Code Structure:**
```rust
pub struct SmtpSession {
    stream: tokio::net::TcpStream,  // ← Problem: hardcoded type
    // ... other fields
}
```

**Solution Options:**

**Option A: Generic Stream (Recommended)**
```rust
use tokio::io::{AsyncRead, AsyncWrite};

pub struct SmtpSession<S: AsyncRead + AsyncWrite + Unpin + Send> {
    stream: S,
    state: SessionState,
    authenticated_account: Option<fastn_id52::PublicKey>,
    // ... other fields
}

// Then we can use:
// SmtpSession<TcpStream> for plain text
// SmtpSession<TlsStream<TcpStream>> for SSL
```

**Option B: Enum Wrapper (Simpler)**
```rust
#[derive(Debug)]
pub enum SmtpStream {
    Plain(tokio::net::TcpStream),
    Tls(tokio_rustls::server::TlsStream<tokio::net::TcpStream>),
}

impl AsyncRead for SmtpStream { /* delegate to inner */ }
impl AsyncWrite for SmtpStream { /* delegate to inner */ }

pub struct SmtpSession {
    stream: SmtpStream,  // Unified interface
    // ... unchanged fields
}
```

**Recommendation**: **Option A (Generic)** - More type-safe, zero-cost abstraction

### **Key STARTTLS Architectural Benefit: Maximum Code Reuse**

**STARTTLS Implementation Strategy:**
```rust
impl<S: AsyncRead + AsyncWrite + Unpin + Send> SmtpSession<S> {
    // ALL SMTP business logic is identical for plain text and encrypted!
    
    async fn handle_ehlo(&mut self) -> Result<(), SmtpError> {
        // Same logic for plain and encrypted
        let mut capabilities = vec![
            "AUTH PLAIN",
            "SIZE 52428800",  // 50MB limit
        ];
        
        // Add STARTTLS capability if TLS available and not already encrypted
        if self.tls_acceptor.is_some() && !self.is_encrypted() {
            capabilities.push("STARTTLS");
        }
        
        self.send_response(&format!("250-localhost\r\n250 {}", capabilities.join("\r\n250-"))).await
    }
    
    async fn handle_starttls(self) -> Result<SmtpSession<TlsStream<S>>, SmtpError> {
        // Upgrade existing connection to TLS
        self.send_response("220 Ready to start TLS").await?;
        let tls_stream = self.tls_acceptor.accept(self.stream).await?;
        
        // Return upgraded session - same state machine, encrypted stream
        Ok(SmtpSession::new(tls_stream, self.client_addr, None)) // No more STARTTLS after upgrade
    }
    
    // Same methods work for both plain and encrypted:
    // - handle_auth() - identical authentication logic
    // - handle_mail_from() - identical sender validation  
    // - handle_rcpt_to() - identical recipient validation
    // - handle_data() - identical email processing and P2P delivery
}
```

**Benefits:**
- ✅ **~95% code reuse** - Same business logic for plain and encrypted SMTP
- ✅ **Single test suite** - Test plain text, then add STARTTLS upgrade
- ✅ **Same P2P delivery** - Encrypted SMTP still uses same email processing
- ✅ **Gradual migration** - Plain text → STARTTLS upgrade path

### **Decision 2: Ed25519 Key Conversion for Certificate Generation**

**Problem**: Need to convert `fastn_id52::SecretKey` → certificate library formats

**Available**: `secret_key.to_bytes() -> [u8; 32]` (raw Ed25519 private key)
**Need**: Conversion to `rcgen::KeyPair` and `rustls::PrivateKey`

**Conversion Chain:**
```rust
impl CertificateManager {
    fn convert_ed25519_for_rcgen(raw_bytes: [u8; 32]) -> Result<rcgen::KeyPair, CertError> {
        // Option A: Use ed25519-dalek directly
        let signing_key = ed25519_dalek::SigningKey::from_bytes(&raw_bytes);
        
        // Option B: Use rcgen's built-in Ed25519 support
        rcgen::KeyPair::from_raw(&rcgen::PKCS_ED25519, &raw_bytes)
            .map_err(|e| CertError::KeyConversion { source: e })
    }
    
    fn convert_ed25519_for_rustls(raw_bytes: [u8; 32]) -> Result<rustls::PrivateKey, CertError> {
        // Convert to PKCS#8 DER format (rustls requirement)
        let signing_key = ed25519_dalek::SigningKey::from_bytes(&raw_bytes);
        let pkcs8_der = signing_key.to_pkcs8_der()
            .map_err(|e| CertError::KeyConversion { source: e })?;
        Ok(rustls::PrivateKey(pkcs8_der.as_bytes().to_vec()))
    }
}
```

**Dependencies needed**: `ed25519-dalek` for key format conversion (might already be available)

### **Decision 3: SSL Server Integration Pattern**

**Problem**: How do SSL and plain text servers coexist?

**Option A: Shared SMTP Handler (Recommended)**
```rust
async fn start_smtp_servers(
    account_manager: Arc<AccountManager>,
    fastn_home: &Path,
) -> Result<(), RunError> {
    let plain_port = env_port("FASTN_SMTP_PORT", 2525);
    let ssl_port = env_port("FASTN_SMTP_SSL_PORT", 465);
    
    // Load/generate certificate
    let cert_manager = CertificateManager::new(fastn_home)?;
    let tls_config = cert_manager.get_or_create_tls_config().await?;
    
    // Start both servers concurrently
    tokio::try_join!(
        start_plain_smtp_server(account_manager.clone(), plain_port),
        start_ssl_smtp_server(account_manager, ssl_port, tls_config)
    )?;
    
    Ok(())
}

async fn start_ssl_smtp_server(
    account_manager: Arc<AccountManager>,
    port: u16,
    tls_config: rustls::ServerConfig,
) -> Result<(), SmtpSslError> {
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;
    let acceptor = tokio_rustls::TlsAcceptor::from(Arc::new(tls_config));
    
    loop {
        let (stream, addr) = listener.accept().await?;
        let tls_stream = acceptor.accept(stream).await?;
        
        // Use same SMTP handler with generic stream
        let session = SmtpSession::new(tls_stream, addr, account_manager.clone());
        tokio::spawn(session.handle());
    }
}
```

## 🔍 Design Review: Potential Implementation Holes

### 1. Certificate Generation Edge Cases

**Question**: What if rcgen fails to generate certificate?
- **Risk**: No SSL server startup, unclear error messages
- **Solution**: Clear error handling with fallback to plain text only
- **Mitigation**: Test certificate generation in various environments

**Question**: What if rig ID52 changes after certificate generation?
- **Risk**: Certificate becomes invalid, keyring entries orphaned  
- **Solution**: Regenerate certificate if rig ID52 mismatch detected
- **Implementation**: Store rig ID52 in certificate metadata for validation

### 2. File System and Permissions

**Question**: What if certificate directory is not writable?
- **Risk**: Silent failures, certificates stored in wrong location
- **Solution**: Validate directory permissions before generation
- **Implementation**: Create directory with proper permissions during init

**Question**: What if certificate files get corrupted?
- **Risk**: SSL server fails to start, unclear error messages
- **Solution**: Certificate validation on load, auto-regeneration on corruption
- **Implementation**: Backup old certificates before regeneration

**Question**: What about Windows file permissions for private keys?
- **Risk**: Private keys readable by other users
- **Solution**: Use Windows ACLs to restrict access (similar to Unix 600)
- **Implementation**: Platform-specific permission setting

### 3. Keyring Integration Challenges

**Question**: What if keyring is available but rig ID52 is missing?
- **Risk**: Cannot create keyring entry, unclear failure
- **Solution**: Load rig ID52 from automerge database during certificate init
- **Implementation**: Add `CertificateManager::load_rig_id52()` method

**Question**: What if keyring storage succeeds but retrieval fails?
- **Risk**: Certificate exists but private key unavailable
- **Solution**: Detect inconsistent state, regenerate both cert and key
- **Implementation**: Validate cert/key pair consistency on load

**Question**: What if user switches SKIP_KEYRING setting?
- **Risk**: Private key in keyring but cert expects filesystem (or vice versa)
- **Solution**: Detect storage method mismatch, migrate key storage
- **Implementation**: Store storage method in certificate metadata

### 4. Multi-Server Coordination

**Question**: How do plain text and SSL servers share SMTP handler state?
- **Risk**: Race conditions, inconsistent authentication state
- **Solution**: Use Arc<SmtpHandler> shared between both servers
- **Implementation**: Factor out shared state management

**Question**: What if SSL server fails but plain text succeeds?
- **Risk**: Partial service, confusing user experience  
- **Solution**: Continue with plain text, log SSL failure prominently
- **Implementation**: Independent server startup with clear status reporting

**Question**: How to handle graceful shutdown of multiple servers?
- **Risk**: Zombie processes, incomplete shutdown
- **Solution**: Use fastn-p2p graceful shutdown for both servers
- **Implementation**: Coordinate shutdown signals

### 5. Certificate Lifecycle Management

**Question**: When should certificates be regenerated?
- **Risk**: Expired certificates, service interruption
- **Solution**: Check expiry on startup, warn user, auto-regenerate if <7 days
- **Implementation**: Certificate expiry monitoring in start flow

**Question**: How to handle certificate regeneration without downtime?
- **Risk**: SSL server restart required, dropped connections
- **Solution**: Hot certificate reload (if rustls supports it)
- **Implementation**: Certificate file watching and reload

### 6. Testing Complexity

**Question**: How to test SSL server without breaking plain text tests?
- **Risk**: Certificate generation makes tests slower/flakier
- **Solution**: Mock certificate manager for fast tests, real certs for integration
- **Implementation**: `TestCertificateManager` that uses in-memory certificates

**Question**: How to test both keyring and filesystem storage modes?
- **Risk**: Tests only cover one mode, bugs in other mode
- **Solution**: Parameterized tests covering both SKIP_KEYRING modes
- **Implementation**: Test suite runs each SSL test twice (keyring on/off)

### 7. User Experience Challenges

**Question**: How to guide users through client certificate trust process?
- **Risk**: Users can't connect, unclear instructions
- **Solution**: Interactive certificate trust wizard, client-specific guides
- **Implementation**: Detect failed SSL connections, show troubleshooting

**Question**: What if user connects from multiple devices?
- **Risk**: Certificate trust needed on each device
- **Solution**: Certificate export functionality, setup guides per platform
- **Implementation**: `fastn-rig export-cert` command for easy sharing

### 8. Security Considerations

**Question**: How to ensure private key security in filesystem mode?
- **Risk**: Private key readable by other users/processes
- **Solution**: Strict file permissions, key rotation capability
- **Implementation**: Regular permission validation, key storage auditing

**Question**: What about certificate fingerprint verification?
- **Risk**: Users might trust wrong certificates (MITM)
- **Solution**: Display certificate fingerprint for manual verification
- **Implementation**: Show fingerprint in startup messages and client setup

### 9. Error Recovery Scenarios

**Question**: What if both SSL and plain text servers fail to start?
- **Risk**: No SMTP service at all
- **Solution**: Clear error reporting, troubleshooting steps
- **Implementation**: Comprehensive error types with recovery suggestions

**Question**: What if certificate generation works but TLS config creation fails?
- **Risk**: Valid certificate but unusable SSL server
- **Solution**: Validate entire TLS config pipeline during generation
- **Implementation**: End-to-end validation before marking cert as ready

---

## ✅ Ready to Implement

This design now addresses:
- ✅ **P2P certificate reality** - Self-signed is the privacy-preserving choice
- ✅ **Client compatibility research** - Know exactly which clients work
- ✅ **Implementation edge cases** - Comprehensive error scenario planning
- ✅ **Future extensibility** - FNAME + Let's Encrypt path designed
- ✅ **User experience** - Clear client recommendations and setup guides

**Ready to start implementing self-signed SSL support?** The design is comprehensive and accounts for the P2P privacy model constraints.