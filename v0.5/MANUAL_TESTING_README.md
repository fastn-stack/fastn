# FASTN Email Manual Testing Guide

This guide provides a comprehensive testing framework for FASTN email functionality including P2P delivery, SMTP, and IMAP compatibility with real email clients.

## Quick Start

```bash
# 1. Setup fresh testing environment
./manual-testing/setup-fastn-email.sh

# 2. Run automated CLI tests
./manual-testing/test-smtp-imap-cli.sh

# 3. Test email delivery between rigs
./manual-testing/test-p2p-delivery.sh

# 4. Configure real email clients (manual step)
# Follow instructions in ~/fastn-email/SETUP_SUMMARY.md
```

## Directory Structure

```
~/fastn-email/
├── SETUP_SUMMARY.md          # Generated config summary with passwords
├── alice/                    # First rig
├── bob/                      # Second rig  
├── charlie/                  # Third rig (optional)
└── manual-testing-logs/      # Test results and logs
```

## Testing Scripts

### 1. Environment Setup
- `setup-fastn-email.sh` - Creates fresh ~/fastn-email with multiple rigs
- Generates `SETUP_SUMMARY.md` with all connection details
- Captures SMTP passwords from rig initialization

### 2. Automated CLI Testing
- `test-smtp-imap-cli.sh` - Tests SMTP/IMAP using fastn-mail CLI
- Validates email address formats match working examples
- Confirms server connectivity before manual client testing

### 3. P2P Delivery Testing
- `test-p2p-delivery.sh` - Tests direct P2P email between all rigs
- Monitors delivery times and success rates
- Validates filesystem and database consistency

### 4. Email Client Automation (Future)
- `test-apple-mail.sh` - Automates Apple Mail configuration and testing
- `test-thunderbird.sh` - Automates Thunderbird testing
- Uses AppleScript/osascript for macOS integration

## Manual Testing Workflow

### Phase 1: Automated Validation
1. Run setup script to create fresh environment
2. Execute CLI tests to ensure servers working
3. Test P2P delivery to confirm core functionality
4. Review `SETUP_SUMMARY.md` for client configuration

### Phase 2: Email Client Testing
1. Configure Thunderbird/Apple Mail using summary file
2. Send test emails through client SMTP
3. Verify IMAP folder sync and message retrieval
4. Test bidirectional communication

### Phase 3: Multi-Device Testing
1. Copy account configs to other devices
2. Test Android email clients
3. Validate cross-platform compatibility
4. Monitor performance under load

## Email Address Format Standard

**SECURE FORMAT** (prevents domain hijacking attacks):
```
user@[52-char-account-id].fastn
```

**Examples:**
- `test@v7uum8f25ioqq2rc2ti51n5ufl3cpjhgfucd8tk8r6diu6mbqc70.fastn`
- `inbox@gis0i8adnbidgbfqul0bg06cm017lmsqrnq7k46hjojlebn4rn40.fastn`

**Security Notes:**
- ✅ `.fastn` TLD cannot be purchased - prevents domain hijacking
- ❌ `.com/.org/.net` domains rejected - could be purchased by attackers
- ❌ `test@localhost` (wrong domain)
- ❌ `test@fastn.dev` (not account-specific)

## Server Configuration

Each rig runs with isolated ports:
- **Alice**: SMTP 8587, IMAP 8143
- **Bob**: SMTP 8588, IMAP 8144  
- **Charlie**: SMTP 8589, IMAP 8145

## Testing Requirements

### Automated Tests Must Pass
- ✅ Rig initialization with account creation
- ✅ SMTP server responds to auth attempts
- ✅ IMAP server responds to capability queries
- ✅ P2P delivery within 10 seconds
- ✅ Email format validation against working examples

### Manual Client Tests
- ✅ Thunderbird/Apple Mail SMTP sending
- ✅ IMAP folder synchronization  
- ✅ Email content preservation
- ✅ Bidirectional communication
- ✅ Multiple concurrent clients

## Troubleshooting

### Common Issues
1. **"Invalid domain format"** - Check email uses `.com` suffix
2. **"Authentication failed"** - Verify SMTP password from summary file
3. **"Connection refused"** - Ensure rig servers are running
4. **Empty IMAP folders** - Check P2P delivery completed first

### Debug Commands
```bash
# Check rig status
ps aux | grep fastn-rig

# Verify email delivery
find ~/fastn-email/*/accounts/*/mails/default/INBOX/ -name "*.eml" -mtime -1

# Test IMAP connectivity
fastn-mail imap-connect --host localhost --port 8143 --username test --password [FROM_SUMMARY]
```

## Production Readiness Checklist

- [ ] All automated tests pass
- [ ] Real email client compatibility verified
- [ ] Multi-device testing completed
- [ ] Performance under load tested
- [ ] Security audit passed
- [ ] Documentation complete

---

**Next Steps:**
1. Create setup and testing scripts
2. Test fresh environment from scratch
3. Automate email client configuration
4. Expand to multi-device testing

*This testing framework ensures consistent, reliable FASTN email functionality across all platforms and clients.*