#!/bin/bash

# FASTN Email CLI Testing
# Tests SMTP/IMAP functionality using fastn-mail CLI before client setup

set -euo pipefail

FASTN_EMAIL_DIR="$HOME/fastn-email"
LOG_DIR="$FASTN_EMAIL_DIR/manual-testing-logs"

if [ ! -f "$FASTN_EMAIL_DIR/SETUP_SUMMARY.md" ]; then
    echo "❌ Setup summary not found. Run setup-fastn-email.sh first."
    exit 1
fi

echo "🧪 FASTN Email CLI Testing"
echo "========================="

# Source account information
ALICE_ACCOUNT=$(ls "$FASTN_EMAIL_DIR/alice/accounts/" | head -1)
BOB_ACCOUNT=$(ls "$FASTN_EMAIL_DIR/bob/accounts/" | head -1)
CHARLIE_ACCOUNT=$(ls "$FASTN_EMAIL_DIR/charlie/accounts/" | head -1)

ALICE_PATH="$FASTN_EMAIL_DIR/alice/accounts/$ALICE_ACCOUNT"
BOB_PATH="$FASTN_EMAIL_DIR/bob/accounts/$BOB_ACCOUNT"
CHARLIE_PATH="$FASTN_EMAIL_DIR/charlie/accounts/$CHARLIE_ACCOUNT"

echo "📋 Test Configuration:"
echo "Alice: alice@$ALICE_ACCOUNT.com"
echo "Bob: bob@$BOB_ACCOUNT.com"  
echo "Charlie: charlie@$CHARLIE_ACCOUNT.com"
echo ""

# Start servers
echo "🚀 Starting test servers..."

SKIP_KEYRING=true FASTN_HOME="$FASTN_EMAIL_DIR/alice" \
  FASTN_SMTP_PORT=8587 FASTN_IMAP_PORT=8143 \
  ~/.cargo/bin/cargo run --bin fastn-rig -- run > "$LOG_DIR/alice_test.log" 2>&1 &
ALICE_PID=$!

SKIP_KEYRING=true FASTN_HOME="$FASTN_EMAIL_DIR/bob" \
  FASTN_SMTP_PORT=8588 FASTN_IMAP_PORT=8144 \
  ~/.cargo/bin/cargo run --bin fastn-rig -- run > "$LOG_DIR/bob_test.log" 2>&1 &
BOB_PID=$!

SKIP_KEYRING=true FASTN_HOME="$FASTN_EMAIL_DIR/charlie" \
  FASTN_SMTP_PORT=8589 FASTN_IMAP_PORT=8145 \
  ~/.cargo/bin/cargo run --bin fastn-rig -- run > "$LOG_DIR/charlie_test.log" 2>&1 &
CHARLIE_PID=$!

# Cleanup function
cleanup() {
    echo "🛑 Stopping servers..."
    kill $ALICE_PID $BOB_PID $CHARLIE_PID 2>/dev/null || true
    sleep 2
}
trap cleanup EXIT

echo "⏳ Waiting for servers to start (10 seconds)..."
sleep 10

# Test server connectivity
echo "🔌 Testing server connectivity..."

# Test SMTP port connectivity
for port in 8587 8588 8589; do
    if nc -z localhost $port; then
        echo "✅ SMTP port $port: Connected"
    else  
        echo "❌ SMTP port $port: Failed"
        exit 1
    fi
done

# Test IMAP port connectivity  
for port in 8143 8144 8145; do
    if nc -z localhost $port; then
        echo "✅ IMAP port $port: Connected"
    else
        echo "❌ IMAP port $port: Failed"  
        exit 1
    fi
done

echo ""
echo "📧 Testing P2P Email Delivery..."

# Test Alice → Bob
echo "📤 Testing Alice → Bob..."
FASTN_HOME="$FASTN_EMAIL_DIR/alice" ~/.cargo/bin/cargo run --package fastn-mail --features net --bin fastn-mail -- \
  --account-path "$ALICE_PATH" \
  send-mail --direct \
  --from "alice@$ALICE_ACCOUNT.com" \
  --to "bob@$BOB_ACCOUNT.com" \
  --subject "CLI Test: Alice to Bob" \
  --body "Testing P2P delivery from Alice to Bob via CLI"

sleep 5

# Verify delivery
BOB_INBOX="$BOB_PATH/mails/default/INBOX"
if find "$BOB_INBOX" -name "*.eml" -newer "$BOB_PATH" | grep -q eml; then
    echo "✅ Alice → Bob: Email delivered"
else
    echo "❌ Alice → Bob: Delivery failed"
    exit 1
fi

# Test Bob → Charlie  
echo "📤 Testing Bob → Charlie..."
FASTN_HOME="$FASTN_EMAIL_DIR/bob" ~/.cargo/bin/cargo run --package fastn-mail --features net --bin fastn-mail -- \
  --account-path "$BOB_PATH" \
  send-mail --direct \
  --from "bob@$BOB_ACCOUNT.com" \
  --to "charlie@$CHARLIE_ACCOUNT.com" \
  --subject "CLI Test: Bob to Charlie" \
  --body "Testing P2P delivery from Bob to Charlie via CLI"

sleep 5

# Verify delivery  
CHARLIE_INBOX="$CHARLIE_PATH/mails/default/INBOX"
if find "$CHARLIE_INBOX" -name "*.eml" -newer "$CHARLIE_PATH" | grep -q eml; then
    echo "✅ Bob → Charlie: Email delivered"
else
    echo "❌ Bob → Charlie: Delivery failed" 
    exit 1
fi

# Test Charlie → Alice
echo "📤 Testing Charlie → Alice..."
FASTN_HOME="$FASTN_EMAIL_DIR/charlie" ~/.cargo/bin/cargo run --package fastn-mail --features net --bin fastn-mail -- \
  --account-path "$CHARLIE_PATH" \
  send-mail --direct \
  --from "charlie@$CHARLIE_ACCOUNT.com" \
  --to "alice@$ALICE_ACCOUNT.com" \
  --subject "CLI Test: Charlie to Alice" \
  --body "Testing P2P delivery from Charlie to Alice via CLI"

sleep 5

# Verify delivery
ALICE_INBOX="$ALICE_PATH/mails/default/INBOX"  
if find "$ALICE_INBOX" -name "*.eml" -newer "$ALICE_PATH" | grep -q eml; then
    echo "✅ Charlie → Alice: Email delivered"
else
    echo "❌ Charlie → Alice: Delivery failed"
    exit 1
fi

echo ""
echo "📬 Testing IMAP Connectivity..."

# Test IMAP connections (basic connectivity test)
echo "🔍 Testing IMAP server responses..."

# Alice IMAP
if timeout 10 bash -c "</dev/tcp/localhost/8143"; then
    echo "✅ Alice IMAP: Server responding"
else
    echo "❌ Alice IMAP: Connection failed"
    exit 1
fi

# Bob IMAP
if timeout 10 bash -c "</dev/tcp/localhost/8144"; then
    echo "✅ Bob IMAP: Server responding" 
else
    echo "❌ Bob IMAP: Connection failed"
    exit 1
fi

# Charlie IMAP
if timeout 10 bash -c "</dev/tcp/localhost/8145"; then
    echo "✅ Charlie IMAP: Server responding"
else
    echo "❌ Charlie IMAP: Connection failed"
    exit 1  
fi

echo ""
echo "📊 Email Count Summary:"
ALICE_SENT=$(find "$ALICE_PATH/mails/default/Sent" -name "*.eml" 2>/dev/null | wc -l)
ALICE_INBOX=$(find "$ALICE_INBOX" -name "*.eml" 2>/dev/null | wc -l)
BOB_SENT=$(find "$BOB_PATH/mails/default/Sent" -name "*.eml" 2>/dev/null | wc -l)  
BOB_INBOX=$(find "$BOB_INBOX" -name "*.eml" 2>/dev/null | wc -l)
CHARLIE_SENT=$(find "$CHARLIE_PATH/mails/default/Sent" -name "*.eml" 2>/dev/null | wc -l)
CHARLIE_INBOX=$(find "$CHARLIE_INBOX" -name "*.eml" 2>/dev/null | wc -l)

echo "Alice: Sent=$ALICE_SENT, INBOX=$ALICE_INBOX"
echo "Bob: Sent=$BOB_SENT, INBOX=$BOB_INBOX"  
echo "Charlie: Sent=$CHARLIE_SENT, INBOX=$CHARLIE_INBOX"

# Verify expected counts
if [ "$ALICE_SENT" -eq 1 ] && [ "$ALICE_INBOX" -eq 1 ] && \
   [ "$BOB_SENT" -eq 1 ] && [ "$BOB_INBOX" -eq 1 ] && \
   [ "$CHARLIE_SENT" -eq 1 ] && [ "$CHARLIE_INBOX" -eq 1 ]; then
    echo "✅ Email counts match expected values"
else
    echo "❌ Email counts don't match expected values"
    echo "Expected: Each rig should have 1 sent and 1 received email"
    exit 1
fi

echo ""
echo "🎉 All CLI Tests Passed!"
echo "✅ Server connectivity confirmed"
echo "✅ P2P delivery working (full triangle)"
echo "✅ IMAP servers responding"  
echo "✅ Email counts validated"
echo ""
echo "📋 Ready for Apple Mail configuration!"
echo "📍 Configuration file: ~/fastn-email/SETUP_SUMMARY.md"
echo ""