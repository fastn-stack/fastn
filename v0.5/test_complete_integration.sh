#!/bin/bash
# Complete self-contained SMTP to P2P to INBOX integration test
# Pre-compiles all binaries then uses them directly for precise timing

set -euo pipefail

# Configuration
export PATH="$PATH:$HOME/.cargo/bin"
TEST_DIR="/tmp/fastn-complete-test"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m' 
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
NC='\033[0m'

log() { echo -e "${BLUE}[$(date +'%H:%M:%S')] $1${NC}"; }
success() { echo -e "${GREEN}✅ $1${NC}"; }
warn() { echo -e "${YELLOW}⚠️  $1${NC}"; }
error() { echo -e "${RED}❌ $1${NC}"; exit 1; }

# Binary path detection
detect_target_dir() {
    # Check common binary locations
    if [ -f "$HOME/target/debug/fastn-rig" ]; then
        echo "$HOME/target/debug"
    elif [ -f "./target/debug/fastn-rig" ]; then
        echo "./target/debug"
    elif [ -f "/Users/amitu/target/debug/fastn-rig" ]; then
        echo "/Users/amitu/target/debug"
    else
        error "Could not find fastn-rig binary in common locations"
    fi
}

# Global cleanup
cleanup() {
    log "🧹 Complete cleanup..."
    pkill -f "FASTN_HOME.*fastn-complete-test" 2>/dev/null || true
    sleep 2
    pkill -9 -f "FASTN_HOME.*fastn-complete-test" 2>/dev/null || true
    rm -rf "$TEST_DIR" /tmp/peer*.log 2>/dev/null || true
}
trap cleanup EXIT

log "🚀 FASTN SMTP→P2P→INBOX COMPLETE INTEGRATION TEST"
log "=============================================="

# Step 1: Build all binaries ONCE at the start (no compilation during test)
log "📦 Pre-compiling all required binaries (debug build for speed)..."
cargo build --bin fastn-rig --bin test_utils >/dev/null 2>&1 || error "Failed to build binaries"
cargo build --package fastn-mail --features net >/dev/null 2>&1 || error "Failed to build fastn-mail"
success "All binaries pre-compiled"

# Detect binary locations
TARGET_DIR=$(detect_target_dir)
FASTN_RIG="$TARGET_DIR/fastn-rig"
FASTN_MAIL="$TARGET_DIR/fastn-mail"
TEST_UTILS="$TARGET_DIR/test_utils"

log "🔍 Using binaries from: $TARGET_DIR"
[ -x "$FASTN_RIG" ] || error "fastn-rig binary not executable: $FASTN_RIG"
[ -x "$FASTN_MAIL" ] || error "fastn-mail binary not executable: $FASTN_MAIL"
[ -x "$TEST_UTILS" ] || error "test_utils binary not executable: $TEST_UTILS"
success "Binary paths validated"

# Step 2: Setup environment  
log "🏗️  Setting up test environment..."
cleanup
mkdir -p "$TEST_DIR/peer1" "$TEST_DIR/peer2"
success "Test directories created"

# Step 3: Initialize peers (no compilation - direct binary execution)
log "🔧 Initializing peer 1..."
SKIP_KEYRING=true FASTN_HOME="$TEST_DIR/peer1" "$FASTN_RIG" init > /tmp/peer1_init.log 2>&1
PEER1_CREDS=$("$TEST_UTILS" extract-account --file /tmp/peer1_init.log --format json)
ACCOUNT1_ID=$(echo "$PEER1_CREDS" | jq -r '.account_id')
ACCOUNT1_PWD=$(echo "$PEER1_CREDS" | jq -r '.password')

log "🔧 Initializing peer 2..."
SKIP_KEYRING=true FASTN_HOME="$TEST_DIR/peer2" "$FASTN_RIG" init > /tmp/peer2_init.log 2>&1
PEER2_CREDS=$("$TEST_UTILS" extract-account --file /tmp/peer2_init.log --format json)
ACCOUNT2_ID=$(echo "$PEER2_CREDS" | jq -r '.account_id')
ACCOUNT2_PWD=$(echo "$PEER2_CREDS" | jq -r '.password')

success "Peer 1: $ACCOUNT1_ID"
success "Peer 2: $ACCOUNT2_ID"

# Validate
[ ${#ACCOUNT1_ID} -eq 52 ] || error "Invalid peer 1 account ID length: ${#ACCOUNT1_ID}"
[ ${#ACCOUNT2_ID} -eq 52 ] || error "Invalid peer 2 account ID length: ${#ACCOUNT2_ID}"
[ -d "$TEST_DIR/peer1/accounts/$ACCOUNT1_ID" ] || error "Peer 1 account directory missing"
[ -d "$TEST_DIR/peer2/accounts/$ACCOUNT2_ID" ] || error "Peer 2 account directory missing"
success "Account validation passed"

# Step 4: Start peers (direct binary execution - no compilation delay)
log "🚀 Starting peer 1 (SMTP: 2525)..."
SKIP_KEYRING=true FASTN_HOME="$TEST_DIR/peer1" FASTN_SMTP_PORT=2525 \
    "$FASTN_RIG" run >/tmp/peer1_run.log 2>&1 &
PID1=$!

log "🚀 Starting peer 2 (SMTP: 2526)..."  
SKIP_KEYRING=true FASTN_HOME="$TEST_DIR/peer2" FASTN_SMTP_PORT=2526 \
    "$FASTN_RIG" run >/tmp/peer2_run.log 2>&1 &
PID2=$!

# Enhanced cleanup for background processes
cleanup() {
    log "🧹 Killing processes PID1=$PID1 PID2=$PID2..."
    kill $PID1 $PID2 2>/dev/null || true
    sleep 3
    kill -9 $PID1 $PID2 2>/dev/null || true
    wait 2>/dev/null || true
    rm -rf "$TEST_DIR" /tmp/peer*.log 2>/dev/null || true
}
trap cleanup EXIT

# Wait for peers to fully start (no compilation - should be faster)
log "⏳ Waiting for peers to start (6 seconds - no compilation delay)..."
sleep 6

# Verify processes are running
kill -0 $PID1 2>/dev/null || error "Peer 1 process died"
kill -0 $PID2 2>/dev/null || error "Peer 2 process died"
success "Both peers running (PIDs: $PID1, $PID2)"

# Step 5: Send email (direct binary - no compilation)
log "📧 Sending email via SMTP (direct binary)..."
FROM="test@${ACCOUNT1_ID}.com"
TO="inbox@${ACCOUNT2_ID}.com"

log "📧 From: $FROM"  
log "📧 To: $TO"

# Use direct binary (no compilation delay during email send)
if FASTN_HOME="$TEST_DIR/peer1" "$FASTN_MAIL" send-mail \
    --smtp 2525 --password "$ACCOUNT1_PWD" \
    --from "$FROM" --to "$TO" \
    --subject "Direct Binary Test" \
    --body "No compilation delays"; then
    success "Email sent via direct binary execution"
else
    error "SMTP email sending failed with direct binary"
fi

# Step 6: Monitor P2P delivery with precise timing
log "⏳ Monitoring P2P delivery (precise timing with direct binaries)..."

for attempt in $(seq 1 8); do
    sleep 3  # Shorter intervals since no compilation delays
    elapsed=$((attempt * 3))
    
    # Use direct binary for email counting (no compilation delay)
    SENT_COUNT=$("$TEST_UTILS" count-emails -a "$TEST_DIR/peer1/accounts/$ACCOUNT1_ID" -f Sent | jq -r '.count')
    INBOX_COUNT=$("$TEST_UTILS" count-emails -a "$TEST_DIR/peer2/accounts/$ACCOUNT2_ID" -f INBOX | jq -r '.count')
    
    log "📊 ${elapsed}s: Sent=$SENT_COUNT, INBOX=$INBOX_COUNT"
    
    if [ "$INBOX_COUNT" -gt 0 ]; then
        success "🎉 P2P delivery completed in ${elapsed}s with direct binaries!"
        
        # Validate using direct binary
        FINAL_CHECK=$("$TEST_UTILS" check-delivery \
            --sender-dir "$TEST_DIR/peer1/accounts/$ACCOUNT1_ID" \
            --receiver-dir "$TEST_DIR/peer2/accounts/$ACCOUNT2_ID")
        
        DELIVERY_OK=$(echo "$FINAL_CHECK" | jq -r '.delivery_complete')
        FOLDER_FIX_OK=$(echo "$FINAL_CHECK" | jq -r '.folder_fix_working')
        
        [ "$DELIVERY_OK" = "true" ] || error "Delivery validation failed"
        [ "$FOLDER_FIX_OK" = "true" ] || error "Folder fix validation failed"
        
        success "🎉 COMPLETE SUCCESS with direct binary execution!"
        success "📊 SMTP→P2P→INBOX delivery working without compilation delays"
        exit 0
    fi
done

# Still failed - show debug info
warn "P2P delivery failed even with direct binaries and precise timing"
log "🐛 This suggests the issue is NOT compilation delays..."

log "Recent peer 1 P2P logs:"
grep -E "P2P|stream.*reply|deliver.*emails" /tmp/peer1_run.log | tail -3 || warn "No P2P logs"

log "Recent peer 2 acceptance logs:"
grep -E "Connection accepted|Account message" /tmp/peer2_run.log | tail -3 || warn "No acceptance logs"

error "Direct binary execution also timed out - deeper issue confirmed"