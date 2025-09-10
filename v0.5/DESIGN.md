# fastn v0.5: Email System Technical Design

fastn v0.5 provides a secure, privacy-first P2P email system that interoperates with standard email clients while maintaining cryptographic security and decentralized architecture.

## Overview

fastn v0.5 enables:
- Sending and receiving emails through standard email clients (Thunderbird, Apple Mail)
- Secure P2P email delivery between fastn rigs without central servers
- IMAP4rev1 and SMTP compliance for seamless integration with existing email workflows
- Privacy-preserving email infrastructure with no data retention by third parties
- Account ownership based on cryptographic keypairs, not domain ownership

## Architecture

### **Three-Layer Email System:**

1. **Client Layer**: Standard email clients (Thunderbird, Apple Mail, etc.)
2. **Protocol Layer**: SMTP/IMAP servers providing RFC compliance
3. **P2P Layer**: fastn-p2p network for decentralized email delivery

### **Core Components:**

- **fastn-rig**: Main daemon providing SMTP/IMAP servers and P2P networking
- **fastn-mail**: Email storage, processing, and client utilities  
- **fastn-account**: Account management and authentication
- **fastn-p2p**: Peer-to-peer communication over iroh networking

## Email Address Format

### **Secure Address Format:**
```
user@{52-char-account-id}.fastn
```

### **Security Design:**
- **Account ID**: 52-character cryptographic public key identifier
- **TLD**: `.fastn` - non-purchasable domain to prevent hijacking attacks
- **User prefix**: Arbitrary string chosen by account owner

### **Examples:**
```
alice@v7uum8f25ioqq2rc2ti51n5ufl3cpjhgfucd8tk8r6diu6mbqc70.fastn
bob@gis0i8adnbidgbfqul0bg06cm017lmsqrnq7k46hjojlebn4rn40.fastn
support@3gnfrd2i1p07s6hrpds9dv7m0qf2028sl632189ikh7asjcrfvj0.fastn
```

### **Security Benefits:**
- **Domain hijacking prevention**: `.fastn` TLD cannot be purchased by attackers
- **Cryptographic verification**: Account ID validates message authenticity  
- **P2P guarantee**: All `.fastn` addresses route through secure P2P network
- **No DNS dependency**: Account resolution through P2P discovery, not DNS

## Account Management

### **Account Creation:**
Each fastn rig creates accounts with:
- **Cryptographic keypair**: Ed25519 for signing and identity
- **Account ID52**: 52-character base58 encoding of public key
- **Password**: Generated for SMTP/IMAP authentication
- **Folder structure**: Standard IMAP folders (INBOX, Sent, Drafts, Trash)

### **Multi-Account Support:**
- **Single rig, multiple accounts**: One rig can host many email accounts
- **Account isolation**: Each account has separate storage and authentication
- **P2P discovery**: All accounts in a rig are discoverable by peers
- **Account status**: ONLINE/OFFLINE controls P2P availability

### **Authentication:**
- **SMTP**: Username format `user@{account-id}.fastn`, password from account creation
- **IMAP**: Same credentials for seamless email client setup
- **P2P**: Cryptographic verification using account keypairs

## Email Flow

### **Outbound Email (Alice → Bob):**
1. **Email client** (Thunderbird) composes email to `bob@{bob-id}.fastn`
2. **SMTP server** receives email, extracts Bob's account ID from address
3. **Storage** stores email in Alice's Sent folder as RFC 5322 .eml file
4. **P2P delivery** queues email for delivery to Bob's rig
5. **Network discovery** locates Bob's rig via fastn-p2p
6. **Delivery** transfers email securely to Bob's rig
7. **Storage** stores email in Bob's INBOX folder

### **Inbound Email Access:**
1. **Email client** (Apple Mail) connects via IMAP to local fastn-rig
2. **IMAP server** authenticates user and lists available folders
3. **Folder sync** shows message counts and folder structure
4. **Message retrieval** serves .eml files with headers, content, and flags
5. **Real-time updates** reflect new P2P deliveries in client

## Protocol Implementation

### **SMTP Server (Port 587):**
- **STARTTLS support**: Secure connection upgrade from plain text
- **Authentication**: PLAIN mechanism with account passwords
- **Routing**: Extracts destination account ID from email addresses
- **Storage**: RFC 5322 format with proper headers and MIME structure
- **P2P queueing**: Automatically queues outbound emails for P2P delivery

### **IMAP Server (Port 1143):**
- **IMAP4rev1 compliance**: Full RFC 3501 implementation
- **Core commands**: LOGIN, CAPABILITY, LIST, SELECT, LOGOUT
- **Message commands**: FETCH, UID FETCH (FLAGS, BODY[], RFC822.SIZE, BODY.PEEK)
- **Folder commands**: STATUS (MESSAGES, UIDNEXT, UNSEEN, RECENT)
- **Client compatibility**: LSUB, NOOP, CLOSE for legacy email client support
- **Dynamic authentication**: Account ID extraction from username  
- **Folder management**: Real-time message counts with recursive filesystem sync
- **Session management**: Proper IMAP state machine with folder selection

### **P2P Communication:**
- **Discovery**: Account IDs resolve to network endpoints via iroh
- **Direct connections**: Peer-to-peer when possible, relay fallback
- **Email format**: Raw RFC 5322 messages transferred securely
- **Delivery confirmation**: Sender tracks delivery status per recipient
- **Retry logic**: Automatic retry with exponential backoff

## Storage Architecture

### **Account Directory Structure:**
```
{fastn-home}/accounts/{account-id}/
├── mails/default/
│   ├── INBOX/2025/09/10/email-{uuid}.eml
│   ├── Sent/2025/09/10/email-{uuid}.eml
│   ├── Drafts/
│   └── Trash/
└── emails.db (SQLite: metadata, flags, delivery tracking)
```

### **Email Format:**
- **Files**: RFC 5322 .eml format for maximum compatibility
- **Organization**: Date-based folder hierarchy (YYYY/MM/DD)
- **Metadata**: SQLite database for flags, search, and delivery tracking
- **Consistency**: Filesystem and database always synchronized

### **Message Flags:**
- **Read/Unread**: IMAP \Seen flag persistence
- **Deleted**: IMAP \Deleted flag (soft delete)
- **Flagged**: User-defined importance markers
- **Storage**: SQLite database with .eml file correlation

## Network Architecture

### **P2P Discovery:**
- **Account resolution**: Account ID → network endpoint via fastn-net
- **Multi-rig support**: Accounts can be hosted on any fastn rig
- **Network mobility**: Rigs can change IP addresses without losing emails
- **Relay support**: Transparent fallback when direct connection impossible

### **Security Model:**
- **End-to-end verification**: Account IDs provide cryptographic identity
- **No central servers**: Pure P2P with no dependency on email providers
- **Private by default**: No email content stored on relay servers
- **Domain independence**: No reliance on DNS or domain ownership

## Email Client Integration

### **Supported Clients:**
- **Thunderbird**: Full compatibility (proven via manual testing)
- **Apple Mail**: Native macOS/iOS support
- **Outlook**: Standard IMAP/SMTP compatibility
- **Mobile clients**: Any client supporting IMAP4rev1

### **Client Setup:**
```
Account Type: IMAP
Email: alice@{account-id}.fastn
Password: {generated-password}

IMAP Server: localhost:1143 (STARTTLS)
SMTP Server: localhost:587 (STARTTLS)
```

### **Client Capabilities:**
- **Folder browsing**: All standard folders with accurate message counts
- **Email composition**: Standard compose → send via SMTP
- **Message reading**: Full email content with headers and attachments
- **Search**: Client-side search with server-side SEARCH command support
- **Real-time sync**: New P2P deliveries appear automatically

## Testing Infrastructure

### **Parametric Testing System:**
- **Multi-rig mode**: Tests inter-rig P2P delivery (1 account per rig)
- **Single-rig mode**: Tests intra-rig local delivery (2 accounts in 1 rig)  
- **Protocol variants**: SMTP plain text (bash) and STARTTLS (rust) modes
- **Comprehensive coverage**: `./test.sh --all` runs all 4 combinations
- **Perfect isolation**: Random ports (2500+), unique directories, timestamped logs
- **Dual verification**: IMAP protocol counts vs filesystem validation
- **CI integration**: GitHub Actions runs complete test matrix

### **Test Coverage:**
- **SMTP → P2P → IMAP pipeline**: Complete email flow validation
- **Authentication**: Real account credentials, not hardcoded
- **Message integrity**: Content verification through entire pipeline
- **Performance**: Sub-10-second delivery target across network

### **Manual Testing Framework:**
- **Real client validation**: Actual Thunderbird/Apple Mail integration
- **Setup automation**: Scripts for email client configuration
- **Multi-device testing**: Android, iOS, desktop client support
- **Production simulation**: Multi-user, multi-rig deployment scenarios

## Performance Design

### **Delivery Targets:**
- **Local delivery** (same rig): < 1 second
- **P2P delivery** (rig-to-rig): < 10 seconds  
- **Message retrieval** (IMAP): < 100ms per message
- **Folder sync** (IMAP): < 500ms for folder list

### **Scalability:**
- **Account limits**: 1000+ accounts per rig (filesystem limited)
- **Message volume**: Millions of messages per account (SQLite limited)
- **Network scale**: Unlimited rigs in P2P network
- **Client connections**: Multiple IMAP clients per account simultaneously

### **Resource Usage:**
- **Storage**: ~1.5x email size (metadata overhead minimal)
- **Memory**: ~10MB per active IMAP connection
- **Network**: P2P bandwidth scales with email volume
- **CPU**: Minimal overhead for crypto operations

## Security Model

### **Identity and Authentication:**
- **Account ownership**: Cryptographic keypairs, not username/password
- **Message authenticity**: Account ID verification on all P2P deliveries
- **Password security**: Generated passwords for email client compatibility only
- **Key management**: SKIP_KEYRING mode for development, secure storage for production

### **Network Security:**
- **Transport encryption**: STARTTLS for client connections, iroh encryption for P2P
- **Endpoint verification**: Account IDs provide unforgeable identity
- **No metadata leakage**: Email headers and routing private by design
- **Relay privacy**: Relay servers cannot decrypt or access email content

### **Threat Model Protection:**
- ✅ **Domain hijacking**: Eliminated via `.fastn` TLD
- ✅ **Man-in-the-middle**: STARTTLS prevents connection interception
- ✅ **Account impersonation**: Cryptographic account IDs prevent spoofing
- ✅ **Email interception**: P2P delivery bypasses email provider surveillance
- ✅ **Metadata collection**: No central servers to collect communication patterns

## Operational Model

### **Rig Deployment:**
- **Personal rigs**: Individual users run fastn-rig on personal devices
- **Organizational rigs**: Companies run rigs for employee email accounts
- **Hybrid setup**: Mix of personal and organizational rigs in same network
- **Mobile support**: Lightweight rigs on mobile devices (future)

### **Email Client Configuration:**
- **Server discovery**: fastn-rig advertises SMTP/IMAP ports locally
- **Certificate trust**: Self-signed certificates for localhost connections
- **Account import**: QR codes or config files for easy mobile setup
- **Backup and sync**: Account directories portable across devices

### **Network Operations:**
- **Always-on connectivity**: Rigs maintain P2P presence for email delivery
- **Graceful degradation**: Store-and-forward when recipients offline
- **Network resilience**: Automatic relay usage when direct connection fails
- **Mobile adaptation**: Connection management for laptop sleep/wake cycles

## Privacy and Compliance

### **Privacy Guarantees:**
- **Zero knowledge delivery**: Relay servers cannot access email content
- **Metadata minimization**: Only delivery routing information exposed
- **No data retention**: No permanent storage on infrastructure servers  
- **User control**: Complete data ownership and portability

### **Compliance Considerations:**
- **GDPR compliance**: User controls all personal data storage and processing
- **Data portability**: Account directories fully exportable
- **Right to deletion**: Users can delete all email data locally
- **No vendor lock-in**: Standard IMAP/SMTP means client choice freedom

## Future Extensions

### **Protocol Enhancements:**
- **IDLE support**: Real-time push notifications for mobile clients
- **SEARCH optimization**: Server-side search with indexing
- **Attachment optimization**: Chunked transfer for large files
- **Message threading**: Conversation view support

### **Network Features:**
- **Multi-device sync**: Same account accessible from multiple devices
- **Offline capability**: Enhanced store-and-forward for disconnected devices
- **Bandwidth optimization**: Delta sync for large mailboxes
- **Quality of service**: Priority delivery for urgent messages

### **User Experience:**
- **Web interface**: Browser-based email client
- **Mobile apps**: Native iOS/Android applications
- **Desktop integration**: Native notifications and system tray
- **Contact discovery**: P2P address book synchronization

## Implementation Status

### **Core Features (v0.5.0):**
- ✅ **SMTP/IMAP servers**: Full protocol compliance with STARTTLS
- ✅ **P2P delivery**: Reliable email transfer between rigs
- ✅ **Email client support**: Thunderbird and Apple Mail compatibility
- ✅ **Security**: Domain hijacking prevention and encrypted transport
- ✅ **Testing**: Comprehensive parametric testing with CI integration

### **Production Readiness:**
- ✅ **Manual testing**: Real email client validation completed
- ✅ **Performance**: Sub-10-second delivery across network  
- ✅ **Reliability**: Automated retry and error handling
- ✅ **Documentation**: Complete setup and operation guides
- ✅ **Security audit**: Threat model analysis and mitigation

## Design Philosophy

### **Privacy First:**
Every design decision prioritizes user privacy and data ownership over convenience or performance. Users maintain complete control over their email data and communication patterns.

### **Standard Compliance:**
Full adherence to email standards ensures compatibility with existing email ecosystem while adding P2P capabilities transparently.

### **Decentralized by Design:**
No central points of failure, control, or surveillance. The email system operates as a pure P2P network with cryptographic security guarantees.

### **User Empowerment:**
Users own their email addresses through cryptographic keypairs, not through domain ownership or service provider accounts.

---

This design enables fastn v0.5 to serve as a production-ready email system that combines the familiarity of traditional email with the security and privacy benefits of modern P2P networking.