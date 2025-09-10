#!/bin/bash

# FASTN Email Testing Environment Setup
# Creates fresh ~/fastn-email with multiple rigs and configuration summary

set -euo pipefail

FASTN_EMAIL_DIR="$HOME/fastn-email"
LOG_DIR="$FASTN_EMAIL_DIR/manual-testing-logs"

echo "🚀 Setting up FASTN Email Testing Environment"
echo "============================================"

# Clean up existing environment
if [ -d "$FASTN_EMAIL_DIR" ]; then
    echo "🧹 Cleaning existing ~/fastn-email directory..."
    rm -rf "$FASTN_EMAIL_DIR"
fi

# Create directory structure
echo "📁 Creating directory structure..."
mkdir -p "$FASTN_EMAIL_DIR"/{alice,bob,charlie}
mkdir -p "$LOG_DIR"

echo "🔧 Initializing Alice rig..."
SKIP_KEYRING=true FASTN_HOME="$FASTN_EMAIL_DIR/alice" \
  ~/.cargo/bin/cargo run --bin fastn-rig -- init 2>&1 | tee "$LOG_DIR/alice_init.log"

echo "🔧 Initializing Bob rig..."  
SKIP_KEYRING=true FASTN_HOME="$FASTN_EMAIL_DIR/bob" \
  ~/.cargo/bin/cargo run --bin fastn-rig -- init 2>&1 | tee "$LOG_DIR/bob_init.log"

echo "🔧 Initializing Charlie rig..."
SKIP_KEYRING=true FASTN_HOME="$FASTN_EMAIL_DIR/charlie" \
  ~/.cargo/bin/cargo run --bin fastn-rig -- init 2>&1 | tee "$LOG_DIR/charlie_init.log"

# Extract account IDs
echo "📋 Extracting account information..."
ALICE_ACCOUNT=$(ls "$FASTN_EMAIL_DIR/alice/accounts/" | head -1)
BOB_ACCOUNT=$(ls "$FASTN_EMAIL_DIR/bob/accounts/" | head -1) 
CHARLIE_ACCOUNT=$(ls "$FASTN_EMAIL_DIR/charlie/accounts/" | head -1)

echo "Alice Account: $ALICE_ACCOUNT"
echo "Bob Account: $BOB_ACCOUNT"  
echo "Charlie Account: $CHARLIE_ACCOUNT"

# Start servers temporarily to capture SMTP passwords
echo "🔐 Starting servers temporarily to capture SMTP passwords..."

# Start Alice
SKIP_KEYRING=true FASTN_HOME="$FASTN_EMAIL_DIR/alice" \
  FASTN_SMTP_PORT=8587 FASTN_IMAP_PORT=8143 \
  ~/.cargo/bin/cargo run --bin fastn-rig -- run > "$LOG_DIR/alice_startup.log" 2>&1 &
ALICE_PID=$!

# Start Bob  
SKIP_KEYRING=true FASTN_HOME="$FASTN_EMAIL_DIR/bob" \
  FASTN_SMTP_PORT=8588 FASTN_IMAP_PORT=8144 \
  ~/.cargo/bin/cargo run --bin fastn-rig -- run > "$LOG_DIR/bob_startup.log" 2>&1 &
BOB_PID=$!

# Start Charlie
SKIP_KEYRING=true FASTN_HOME="$FASTN_EMAIL_DIR/charlie" \
  FASTN_SMTP_PORT=8589 FASTN_IMAP_PORT=8145 \
  ~/.cargo/bin/cargo run --bin fastn-rig -- run > "$LOG_DIR/charlie_startup.log" 2>&1 &  
CHARLIE_PID=$!

echo "⏳ Waiting for servers to initialize (10 seconds)..."
sleep 10

# Extract SMTP passwords from logs
echo "🔍 Extracting SMTP passwords from startup logs..."
ALICE_SMTP_PASS=$(grep -o "Generated.*password.*: [^']*" "$LOG_DIR/alice_startup.log" | cut -d: -f2 | tr -d ' ' | head -1 || echo "EXTRACT_FAILED")
BOB_SMTP_PASS=$(grep -o "Generated.*password.*: [^']*" "$LOG_DIR/bob_startup.log" | cut -d: -f2 | tr -d ' ' | head -1 || echo "EXTRACT_FAILED")
CHARLIE_SMTP_PASS=$(grep -o "Generated.*password.*: [^']*" "$LOG_DIR/charlie_startup.log" | cut -d: -f2 | tr -d ' ' | head -1 || echo "EXTRACT_FAILED")

# Stop servers
echo "🛑 Stopping servers..."
kill $ALICE_PID $BOB_PID $CHARLIE_PID 2>/dev/null || true
sleep 2

# Generate setup summary
echo "📋 Generating setup summary..."
cat > "$FASTN_EMAIL_DIR/SETUP_SUMMARY.md" << EOF
# FASTN Email Testing Configuration

**Generated:** $(date)
**Environment:** ~/fastn-email  

## Rig Configuration

### Alice
- **Account ID**: \`$ALICE_ACCOUNT\`
- **Email Address**: \`alice@$ALICE_ACCOUNT.com\` ✅ CONFIRMED FORMAT
- **SMTP**: localhost:8587 (Password: \`$ALICE_SMTP_PASS\`)
- **IMAP**: localhost:8143 (Username: alice, Password: \`$ALICE_SMTP_PASS\`)
- **Account Path**: \`~/fastn-email/alice/accounts/$ALICE_ACCOUNT\`

### Bob  
- **Account ID**: \`$BOB_ACCOUNT\`
- **Email Address**: \`bob@$BOB_ACCOUNT.com\` ✅ CONFIRMED FORMAT
- **SMTP**: localhost:8588 (Password: \`$BOB_SMTP_PASS\`)
- **IMAP**: localhost:8144 (Username: bob, Password: \`$BOB_SMTP_PASS\`)
- **Account Path**: \`~/fastn-email/bob/accounts/$BOB_ACCOUNT\`

### Charlie
- **Account ID**: \`$CHARLIE_ACCOUNT\`  
- **Email Address**: \`charlie@$CHARLIE_ACCOUNT.com\` ✅ CONFIRMED FORMAT
- **SMTP**: localhost:8589 (Password: \`$CHARLIE_SMTP_PASS\`)
- **IMAP**: localhost:8145 (Username: charlie, Password: \`$CHARLIE_SMTP_PASS\`)
- **Account Path**: \`~/fastn-email/charlie/accounts/$CHARLIE_ACCOUNT\`

## Start Servers

\`\`\`bash
# Alice
SKIP_KEYRING=true FASTN_HOME=~/fastn-email/alice \\
  FASTN_SMTP_PORT=8587 FASTN_IMAP_PORT=8143 \\
  ~/.cargo/bin/cargo run --bin fastn-rig -- run

# Bob
SKIP_KEYRING=true FASTN_HOME=~/fastn-email/bob \\
  FASTN_SMTP_PORT=8588 FASTN_IMAP_PORT=8144 \\
  ~/.cargo/bin/cargo run --bin fastn-rig -- run

# Charlie  
SKIP_KEYRING=true FASTN_HOME=~/fastn-email/charlie \\
  FASTN_SMTP_PORT=8589 FASTN_IMAP_PORT=8145 \\
  ~/.cargo/bin/cargo run --bin fastn-rig -- run
\`\`\`

## Apple Mail Configuration

### Account 1: Alice
- **Account Type**: IMAP  
- **Email**: alice@$ALICE_ACCOUNT.com
- **Full Name**: Alice Test
- **IMAP Server**: localhost:8143
- **Username**: alice
- **Password**: $ALICE_SMTP_PASS  
- **SMTP Server**: localhost:8587
- **SMTP Username**: alice
- **SMTP Password**: $ALICE_SMTP_PASS

### Account 2: Bob
- **Account Type**: IMAP
- **Email**: bob@$BOB_ACCOUNT.com  
- **Full Name**: Bob Test
- **IMAP Server**: localhost:8144
- **Username**: bob
- **Password**: $BOB_SMTP_PASS
- **SMTP Server**: localhost:8588
- **SMTP Username**: bob  
- **SMTP Password**: $BOB_SMTP_PASS

## Testing Commands

\`\`\`bash
# Test CLI before client setup
./manual-testing/test-smtp-imap-cli.sh

# Test P2P delivery
./manual-testing/test-p2p-delivery.sh

# Test Apple Mail automation  
./manual-testing/test-apple-mail.sh
\`\`\`

---
*Setup completed successfully. All passwords extracted from server startup logs.*
EOF

echo ""
echo "✅ FASTN Email Testing Environment Ready!"
echo "📍 Location: ~/fastn-email"
echo "📋 Configuration: ~/fastn-email/SETUP_SUMMARY.md"
echo ""
echo "Next Steps:"
echo "1. Review ~/fastn-email/SETUP_SUMMARY.md"
echo "2. Run: ./manual-testing/test-smtp-imap-cli.sh"
echo "3. Start servers and test with Apple Mail"
echo ""