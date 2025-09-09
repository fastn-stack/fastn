# Manual Email Testing Plan - Production fastn Email Setup

## Overview

This document provides a step-by-step guide for setting up and testing fastn email infrastructure with real email clients. This validates production readiness and provides a template for end users.

## Prerequisites

- **macOS/Linux machine** with fastn v0.5 built
- **Two email clients** (recommended: Thunderbird + Apple Mail)
- **Terminal access** for running fastn-rig instances
- **Network access** to localhost ports

## Recommended Email Clients

### **Primary: Thunderbird** (Excellent fastn support)
- ✅ **Best IMAP + STARTTLS support**
- ✅ **Easy certificate trust management**  
- ✅ **Cross-platform** (macOS/Windows/Linux)
- ✅ **Great debugging tools** for email issues
- **Download**: https://thunderbird.net

### **Secondary: Apple Mail** (Good native integration)
- ✅ **Native macOS integration**
- ✅ **STARTTLS support**
- ⚠️ **Certificate trust requires extra steps**
- ✅ **iOS compatibility** (for mobile testing)

### **Alternative: FairEmail (Android)**
- ✅ **Excellent Android support**
- ✅ **Privacy-focused with security settings**
- ✅ **Configurable certificate handling**
- **For mobile/Android testing**

## Test Plan Architecture

### **Two-Rig Setup on Single Machine**
```
┌─────────────────┐         ┌─────────────────┐
│   Rig 1 (Alice) │   P2P   │   Rig 2 (Bob)   │
│ SMTP: 2525      │ ←────→  │ SMTP: 2526      │
│ IMAP: 1143      │         │ IMAP: 1144      │
│                 │         │                 │
│ Thunderbird ←──┘         └──→ Apple Mail    │
└─────────────────┘         └─────────────────┘
```

**Benefits of this setup:**
- ✅ **Real P2P testing** - Two separate rig instances
- ✅ **Real email client testing** - Standard email programs  
- ✅ **Complete workflow validation** - Send from one, receive at other
- ✅ **Certificate testing** - Self-signed certificate trust workflow
- ✅ **Performance validation** - Real-world usage patterns

## Phase 1: Rig Setup (15 minutes)

### Step 1: Create Two Rig Instances

```bash
# Terminal 1: Create Alice's rig
cd /Users/amitu/Projects/fastn-me/v0.5
mkdir -p ~/fastn-email-test/alice
SKIP_KEYRING=true FASTN_HOME=~/fastn-email-test/alice cargo run --bin fastn-rig -- init

# Save Alice's credentials (important!)
# Account ID: alice_account_id52_will_be_shown  
# Password: alice_password_will_be_shown
```

```bash  
# Terminal 2: Create Bob's rig  
mkdir -p ~/fastn-email-test/bob
SKIP_KEYRING=true FASTN_HOME=~/fastn-email-test/bob cargo run --bin fastn-rig -- init

# Save Bob's credentials (important!)
# Account ID: bob_account_id52_will_be_shown
# Password: bob_password_will_be_shown
```

### Step 2: Start Both Rigs with Different Ports

```bash
# Terminal 1: Start Alice's rig
SKIP_KEYRING=true FASTN_HOME=~/fastn-email-test/alice \
  FASTN_SMTP_PORT=2525 FASTN_IMAP_PORT=1143 \
  cargo run --bin fastn-rig -- run

# Look for these success messages:
# ✅ SMTP server listening on 0.0.0.0:2525
# ✅ IMAP server listening on 0.0.0.0:1143
```

```bash
# Terminal 2: Start Bob's rig  
SKIP_KEYRING=true FASTN_HOME=~/fastn-email-test/bob \
  FASTN_SMTP_PORT=2526 FASTN_IMAP_PORT=1144 \
  cargo run --bin fastn-rig -- run
  
# Look for these success messages:
# ✅ SMTP server listening on 0.0.0.0:2526  
# ✅ IMAP server listening on 0.0.0.0:1144
```

### Step 3: Verify Server Status

```bash
# Terminal 3: Test server connectivity
echo "Testing Alice's servers:"
nc -zv localhost 2525  # SMTP should be open
nc -zv localhost 1143  # IMAP should be open

echo "Testing Bob's servers:"  
nc -zv localhost 2526  # SMTP should be open
nc -zv localhost 1144  # IMAP should be open
```

## Phase 2: Email Client Setup (20 minutes)

### Thunderbird Setup (Alice's Account)

**Step 1: Account Creation**
1. Open **Thunderbird**
2. **File** → **New** → **Existing Mail Account**  
3. **Manual Setup** (skip auto-configuration)

**Step 2: Server Configuration**
```
Name: Alice (fastn)
Email: alice@{alice_account_id52}.com  
Password: {alice_password_from_step1}

Incoming Mail (IMAP):
- Server: localhost
- Port: 1143  
- Security: None (we'll add STARTTLS later)
- Authentication: Normal password

Outgoing Mail (SMTP):
- Server: localhost
- Port: 2525
- Security: None (we'll add STARTTLS later)  
- Authentication: Normal password
- Username: alice@{alice_account_id52}.com
```

**Step 3: Test Connection**
1. Click **Re-test** → Should connect successfully
2. Click **Done** → Account should be created
3. **Send/Receive** → Should show folder structure (INBOX, Sent, Drafts, Trash)

### Apple Mail Setup (Bob's Account)  

**Step 1: Account Creation**
1. Open **Apple Mail**
2. **Mail** → **Add Account** → **Other Mail Account**

**Step 2: Server Configuration**
```
Name: Bob (fastn)
Email: bob@{bob_account_id52}.com
Password: {bob_password_from_step1}

Incoming Mail (IMAP):
- Mail Server: localhost
- Port: 1144
- Use SSL: No (plain text first)
- Authentication: Password  

Outgoing Mail (SMTP):
- Mail Server: localhost
- Port: 2526
- Use SSL: No (plain text first)
- Authentication: Password
- Username: bob@{bob_account_id52}.com
```

**Step 3: Test Connection**
1. **Save** → Should verify settings successfully
2. Check **Mailbox** → Should show INBOX, Sent, Drafts, Trash

## Phase 3: End-to-End Email Testing (15 minutes)

### Test 1: Alice → Bob Email

**Step 1: Send from Thunderbird (Alice)**
1. Click **Write** in Thunderbird
2. **To**: bob@{bob_account_id52}.com  
3. **Subject**: "Test Email from Alice via fastn"
4. **Body**: "This email tests the complete fastn email system: Thunderbird → SMTP → P2P → IMAP → Apple Mail"
5. Click **Send**

**Step 2: Verify Receipt in Apple Mail (Bob)**
1. **Check Apple Mail** → Should show new email in INBOX
2. **Open email** → Verify subject and content match
3. **Check headers** → Should show fastn routing information

### Test 2: Bob → Alice Email

**Step 1: Send from Apple Mail (Bob)**
1. Click **New Message** in Apple Mail  
2. **To**: alice@{alice_account_id52}.com
3. **Subject**: "Reply from Bob via fastn"  
4. **Body**: "This confirms bidirectional email delivery through fastn P2P network"
5. Click **Send**

**Step 2: Verify Receipt in Thunderbird (Alice)**
1. **Check Thunderbird** → Should show new email in INBOX
2. **Open email** → Verify subject and content match
3. **Check message source** → Verify fastn headers

### Test 3: Self-Send Test

**Step 3a: Alice sends to herself**
```
From: alice@{alice_account_id52}.com
To: alice@{alice_account_id52}.com  
Subject: Self-send test
```

**Step 3b: Bob sends to himself**
```
From: bob@{bob_account_id52}.com
To: bob@{bob_account_id52}.com
Subject: Self-send test  
```

## Phase 4: Advanced Testing (20 minutes)

### Test 4: STARTTLS Setup (Optional)

**Enable STARTTLS in Thunderbird:**
1. **Account Settings** → **Server Settings**
2. **Security**: Change to **STARTTLS**
3. **Accept certificate warning** → Trust self-signed certificate
4. **Test connection** → Should work with encryption

### Test 5: Multiple Folder Testing

**Create and test different folders:**
1. **Send to Drafts** → Test draft functionality  
2. **Delete emails** → Test Trash folder
3. **Flag/unflag emails** → Test message flags
4. **Search emails** → Test IMAP SEARCH

### Test 6: Performance Testing  

**High-volume testing:**
```bash
# Terminal 3: Send multiple emails via CLI
for i in {1..10}; do
  FASTN_HOME=~/fastn-email-test/alice cargo run -p fastn-mail --features net -- \
    --account-path ~/fastn-email-test/alice/accounts/{alice_account_id52} \
    send-mail --smtp 2525 --password "{alice_password}" \
    --from "alice@{alice_account_id52}.com" \
    --to "bob@{bob_account_id52}.com" \
    --subject "Bulk Test Email $i" \
    --body "Testing high-volume email delivery"
done
```

**Verify in email clients:**
- **Bob's Apple Mail** → Should receive 10 emails  
- **Check delivery timing** → Should be under 30 seconds total
- **Check email order** → Should maintain chronological order

## Phase 5: Validation Checklist

### ✅ **Core Email Functionality**
- [ ] Alice can send emails to Bob (Thunderbird → Apple Mail)
- [ ] Bob can send emails to Alice (Apple Mail → Thunderbird)  
- [ ] Self-send works for both accounts
- [ ] Email content integrity preserved (subject, body, headers)
- [ ] Folder placement correct (Sent for sender, INBOX for receiver)

### ✅ **IMAP Functionality**  
- [ ] Folder listing works (INBOX, Sent, Drafts, Trash visible)
- [ ] Message counts accurate (shows real number of emails)
- [ ] Email reading works (can open and read email content)
- [ ] Message flags work (read/unread status)
- [ ] Real-time updates (new emails appear without manual refresh)

### ✅ **P2P Delivery**
- [ ] Cross-rig delivery works (Alice rig → Bob rig)
- [ ] Delivery timing reasonable (under 10 seconds)  
- [ ] No message loss (all sent emails received)
- [ ] Proper routing (emails go to correct recipient accounts)

### ✅ **Production Readiness**
- [ ] Servers start reliably without errors
- [ ] Email clients connect without issues  
- [ ] Certificate trust workflow clear for users
- [ ] Error messages helpful when things go wrong
- [ ] Performance acceptable for daily use

## Troubleshooting Guide

### **Connection Issues**
```bash
# Check if servers are running
ps aux | grep fastn-rig

# Check if ports are listening  
lsof -i :2525  # Alice SMTP
lsof -i :1143  # Alice IMAP  
lsof -i :2526  # Bob SMTP
lsof -i :1144  # Bob IMAP
```

### **Certificate Issues**
- **Thunderbird**: Tools → Settings → Privacy & Security → Certificates → View Certificates → Servers → Add Exception
- **Apple Mail**: Keychain Access → Trust self-signed certificate

### **Email Not Appearing**
```bash
# Check email files directly
find ~/fastn-email-test/*/accounts/*/mails -name "*.eml" -ls

# Check server logs for errors
# Look at Terminal 1 and Terminal 2 output for errors
```

## Success Criteria

### **Minimal Success (Ready for Friends)**
- ✅ Both rigs start and stay running
- ✅ Both email clients connect and authenticate  
- ✅ Bidirectional email delivery works
- ✅ Emails readable in both clients

### **Complete Success (Production Ready)**
- ✅ All items in validation checklist pass
- ✅ Performance acceptable for daily use
- ✅ Error handling guides users to solutions
- ✅ Certificate trust workflow documented

### **Ultimate Success (Shareable)**
- ✅ Setup process takes under 1 hour
- ✅ Non-technical friends can follow tutorial
- ✅ Common issues have clear solutions  
- ✅ Email experience feels "normal" to users

## Friend Testing Distribution

### **Tutorial Package**
```
📧 fastn-email-setup/
├── README.md (this guide)
├── setup-thunderbird.md (detailed Thunderbird steps)  
├── setup-apple-mail.md (detailed Apple Mail steps)
├── troubleshooting.md (common issues and solutions)
├── fastn-rig (pre-built binary)
├── fastn-mail (pre-built binary)  
└── start-alice.sh (script to start Alice's rig)
└── start-bob.sh (script to start Bob's rig)
```

**Friend Test Goals:**
1. **Validate tutorial clarity** - Can non-technical users follow it?
2. **Find edge cases** - What breaks in real-world usage?  
3. **Performance feedback** - Is it fast enough for daily use?
4. **UX feedback** - Does it feel like normal email?

## Expected Timeline

- **Setup**: 30 minutes (both rigs + email clients)
- **Basic testing**: 15 minutes (send/receive validation)  
- **Advanced testing**: 15 minutes (folders, flags, bulk)
- **Documentation**: 15 minutes (record issues and successes)
- **Total**: ~75 minutes for complete validation

This manual testing will provide real-world validation that our automated tests can't capture - actual human interaction with the email system using standard email clients.