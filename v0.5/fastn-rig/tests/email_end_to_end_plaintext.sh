#!/bin/bash
# 🎯 CRITICAL END-TO-END EMAIL TEST (PLAIN TEXT MODE)
# 
# This is one of the most important tests in fastn - validates complete email pipeline.
# Tests plain text SMTP → fastn-p2p → INBOX delivery.
# Companion test: email_end_to_end_starttls.rs (tests STARTTLS mode)
# Pre-compiles all binaries then uses them directly for precise timing
#
# Usage:
#   bash email_end_to_end_plaintext.sh          # Multi-rig mode: two rigs, one account each (default)
#   bash email_end_to_end_plaintext.sh --single # Single-rig mode: one rig, two accounts

set -euo pipefail

# Parse arguments
SINGLE_RIG=false
if [[ "${1:-}" == "--single" ]]; then
    SINGLE_RIG=true
    echo "🎯 SINGLE-RIG MODE: Testing 2 accounts within 1 rig"
else
    echo "🎯 MULTI-RIG MODE: Testing 1 account per rig (default)"
fi

# Configuration
export PATH="$PATH:$HOME/.cargo/bin"
# Use unique test directory and ports to allow parallel execution
TEST_SUFFIX=$(date +%s%N | tail -c 6)  # Last 6 digits of nanosecond timestamp
if [[ "$SINGLE_RIG" == true ]]; then
    TEST_DIR="/tmp/fastn-test-single-${TEST_SUFFIX}"
    SMTP_PORT1=${FASTN_TEST_SMTP_PORT:-$((2500 + RANDOM % 100))}
    SMTP_PORT2=""  # Single rig only uses one port
else
    TEST_DIR="/tmp/fastn-test-multi-${TEST_SUFFIX}"  
    SMTP_PORT1=${FASTN_TEST_SMTP_PORT:-$((2500 + RANDOM % 100))}
    SMTP_PORT2=${FASTN_TEST_SMTP_PORT2:-$((2600 + RANDOM % 100))}
fi

echo "🏗️  Test isolation: DIR=$TEST_DIR, SMTP_PORTS=$SMTP_PORT1,$SMTP_PORT2"

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

# Binary path detection (mirrors fastn-cli-test-utils::detect_target_dir logic)
detect_target_dir() {
    # This logic matches fastn-cli-test-utils for consistency
    # Check common binary locations (v0.5 target dir first since that's the new location)
    if [ -f "../target/debug/fastn-rig" ]; then
        echo "../target/debug"
    elif [ -f "./target/debug/fastn-rig" ]; then
        echo "./target/debug"
    elif [ -f "/Users/amitu/Projects/fastn-me/v0.5/target/debug/fastn-rig" ]; then
        echo "/Users/amitu/Projects/fastn-me/v0.5/target/debug"
    elif [ -f "$HOME/target/debug/fastn-rig" ]; then
        echo "$HOME/target/debug"
    elif [ -f "/Users/amitu/target/debug/fastn-rig" ]; then
        echo "/Users/amitu/target/debug"
    else
        error "Could not find fastn-rig binary in common locations"
    fi
}

# Global cleanup
cleanup() {
    log "🧹 Cleaning up processes (keeping test directory for debugging)..."
    pkill -f "FASTN_HOME.*fastn-complete-test" 2>/dev/null || true
    sleep 2
    pkill -9 -f "FASTN_HOME.*fastn-complete-test" 2>/dev/null || true
    # Keep test directory and log files for debugging
}
trap cleanup EXIT

log "🚀 🎯 CRITICAL: FASTN PLAIN TEXT EMAIL END-TO-END TEST 🎯"
log "=============================================="
log "Testing: Plain Text SMTP → fastn-p2p → INBOX delivery"
log "Companion: email_end_to_end_starttls.rs (STARTTLS mode)"

# Step 1: Build all binaries ONCE at the start (no compilation during test)
log "📦 Pre-compiling all required binaries (debug build for speed)..."
log "🔨 Building fastn-rig and test_utils..."
if ! cargo build --bin fastn-rig --bin test_utils 2>&1 | tail -10; then
    error "Failed to build fastn-rig binaries"
fi
log "🔨 Building fastn-mail..."
if ! cargo build --package fastn-mail --features net 2>&1 | tail -10; then
    error "Failed to build fastn-mail binary"
fi
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
# Clean up any leftover test directory to start fresh
rm -rf "$TEST_DIR" 2>/dev/null || true
cleanup

if [[ "$SINGLE_RIG" == true ]]; then
    mkdir -p "$TEST_DIR/rig1"
    success "Single rig directory created"
else
    mkdir -p "$TEST_DIR/peer1" "$TEST_DIR/peer2"  
    success "Dual rig directories created"
fi

# Step 3: Initialize peers/accounts
if [[ "$SINGLE_RIG" == true ]]; then
    log "🔧 Initializing single rig with first account..."
    SKIP_KEYRING=true FASTN_HOME="$TEST_DIR/rig1" "$FASTN_RIG" init > /tmp/rig1_init_${TEST_SUFFIX}.log 2>&1
    PEER1_CREDS=$("$TEST_UTILS" extract-account --file /tmp/rig1_init_${TEST_SUFFIX}.log --format json)
    ACCOUNT1_ID=$(echo "$PEER1_CREDS" | jq -r '.account_id')
    ACCOUNT1_PWD=$(echo "$PEER1_CREDS" | jq -r '.password')

    log "🔧 Creating second account in same rig..."
    SKIP_KEYRING=true FASTN_HOME="$TEST_DIR/rig1" "$FASTN_RIG" create-account > /tmp/rig1_account2_${TEST_SUFFIX}.log 2>&1
    PEER2_CREDS=$("$TEST_UTILS" extract-account --file /tmp/rig1_account2_${TEST_SUFFIX}.log --format json) 
    ACCOUNT2_ID=$(echo "$PEER2_CREDS" | jq -r '.account_id')
    ACCOUNT2_PWD=$(echo "$PEER2_CREDS" | jq -r '.password')

    log "🔧 Setting second account to ONLINE status..."
    SKIP_KEYRING=true FASTN_HOME="$TEST_DIR/rig1" "$FASTN_RIG" set-online "$ACCOUNT2_ID" true > /tmp/rig1_online_${TEST_SUFFIX}.log 2>&1

    success "Single Rig - Account 1: $ACCOUNT1_ID"
    success "Single Rig - Account 2: $ACCOUNT2_ID"
else
    log "🔧 Initializing peer 1..."
    SKIP_KEYRING=true FASTN_HOME="$TEST_DIR/peer1" "$FASTN_RIG" init > /tmp/peer1_init_${TEST_SUFFIX}.log 2>&1
    PEER1_CREDS=$("$TEST_UTILS" extract-account --file /tmp/peer1_init_${TEST_SUFFIX}.log --format json)
    ACCOUNT1_ID=$(echo "$PEER1_CREDS" | jq -r '.account_id')
    ACCOUNT1_PWD=$(echo "$PEER1_CREDS" | jq -r '.password')

    log "🔧 Initializing peer 2..."
    SKIP_KEYRING=true FASTN_HOME="$TEST_DIR/peer2" "$FASTN_RIG" init > /tmp/peer2_init_${TEST_SUFFIX}.log 2>&1
    PEER2_CREDS=$("$TEST_UTILS" extract-account --file /tmp/peer2_init_${TEST_SUFFIX}.log --format json)
    ACCOUNT2_ID=$(echo "$PEER2_CREDS" | jq -r '.account_id')
    ACCOUNT2_PWD=$(echo "$PEER2_CREDS" | jq -r '.password')

    success "Peer 1: $ACCOUNT1_ID"
    success "Peer 2: $ACCOUNT2_ID"
fi

# Validate
[ ${#ACCOUNT1_ID} -eq 52 ] || error "Invalid account 1 ID length: ${#ACCOUNT1_ID}"
[ ${#ACCOUNT2_ID} -eq 52 ] || error "Invalid account 2 ID length: ${#ACCOUNT2_ID}"

if [[ "$SINGLE_RIG" == true ]]; then
    [ -d "$TEST_DIR/rig1/accounts/$ACCOUNT1_ID" ] || error "Account 1 directory missing in single rig"
    [ -d "$TEST_DIR/rig1/accounts/$ACCOUNT2_ID" ] || error "Account 2 directory missing in single rig"
else
    [ -d "$TEST_DIR/peer1/accounts/$ACCOUNT1_ID" ] || error "Peer 1 account directory missing"
    [ -d "$TEST_DIR/peer2/accounts/$ACCOUNT2_ID" ] || error "Peer 2 account directory missing"
fi
success "Account validation passed"

# Step 4: Start rigs/peers (direct binary execution - no compilation delay)
if [[ "$SINGLE_RIG" == true ]]; then
    log "🚀 Starting single rig with 2 accounts (SMTP: $SMTP_PORT1)..."
    SKIP_KEYRING=true FASTN_HOME="$TEST_DIR/rig1" FASTN_SMTP_PORT="$SMTP_PORT1" \
        "$FASTN_RIG" run >/tmp/rig1_run_${TEST_SUFFIX}.log 2>&1 &
    PID1=$!
    PID2="" # No second rig in single-rig mode
else
    log "🚀 Starting peer 1 (SMTP: $SMTP_PORT1)..."
    SKIP_KEYRING=true FASTN_HOME="$TEST_DIR/peer1" FASTN_SMTP_PORT="$SMTP_PORT1" \
        "$FASTN_RIG" run >/tmp/peer1_run_${TEST_SUFFIX}.log 2>&1 &
    PID1=$!

    log "🚀 Starting peer 2 (SMTP: $SMTP_PORT2)..."  
    SKIP_KEYRING=true FASTN_HOME="$TEST_DIR/peer2" FASTN_SMTP_PORT="$SMTP_PORT2" \
        "$FASTN_RIG" run >/tmp/peer2_run_${TEST_SUFFIX}.log 2>&1 &
    PID2=$!
fi

# Enhanced cleanup for background processes
cleanup() {
    if [[ "$SINGLE_RIG" == true ]]; then
        log "🧹 Killing single rig process PID1=$PID1..."
        kill $PID1 2>/dev/null || true
        sleep 3
        kill -9 $PID1 2>/dev/null || true
    else
        log "🧹 Killing processes PID1=$PID1 PID2=$PID2..."
        kill $PID1 $PID2 2>/dev/null || true
        sleep 3
        kill -9 $PID1 $PID2 2>/dev/null || true
    fi
    wait 2>/dev/null || true
    # Keep test directory and log files for debugging
}
trap cleanup EXIT

# Wait for rigs/peers to fully start and verify they're listening
if [[ "$SINGLE_RIG" == true ]]; then
    log "⏳ Waiting for single rig to start (10 seconds for CI compatibility)..."
else
    log "⏳ Waiting for peers to start (10 seconds for CI compatibility)..."
fi
sleep 10

# Verify servers started successfully by checking logs (netstat not available on all systems)
log "🔍 Verifying servers started successfully..."

if [[ "$SINGLE_RIG" == true ]]; then
    # Check single rig server logs for successful startup
    if grep -q "SMTP server listening on.*${SMTP_PORT1}" /tmp/rig1_run_${TEST_SUFFIX}.log; then
        log "✅ Single rig SMTP server confirmed listening on port $SMTP_PORT1"
    else
        echo "❌ Single rig SMTP server startup failed"
        echo "📋 Single rig process logs (last 20 lines):"
        tail -20 /tmp/rig1_run_${TEST_SUFFIX}.log || echo "No rig1 log file"
        error "Single rig SMTP server not listening on port $SMTP_PORT1"
    fi
    success "Single rig SMTP server confirmed started successfully"
else
    # Check peer 1 server logs for successful startup
    if grep -q "SMTP server listening on.*${SMTP_PORT1}" /tmp/peer1_run_${TEST_SUFFIX}.log; then
        log "✅ Peer 1 SMTP server confirmed listening on port $SMTP_PORT1"
    else
        echo "❌ Peer 1 SMTP server startup failed"
        echo "📋 Peer 1 process logs (last 20 lines):"
        tail -20 /tmp/peer1_run_${TEST_SUFFIX}.log || echo "No peer1 log file"
        error "Peer 1 SMTP server not listening on port $SMTP_PORT1"
    fi

    # Check peer 2 server logs for successful startup  
    if grep -q "SMTP server listening on.*${SMTP_PORT2}" /tmp/peer2_run_${TEST_SUFFIX}.log; then
        log "✅ Peer 2 SMTP server confirmed listening on port $SMTP_PORT2"
    else
        echo "❌ Peer 2 SMTP server startup failed" 
        echo "📋 Peer 2 process logs (last 20 lines):"
        tail -20 /tmp/peer2_run_${TEST_SUFFIX}.log || echo "No peer2 log file"
        error "Peer 2 SMTP server not listening on port $SMTP_PORT2" 
    fi
    success "Both SMTP servers confirmed started successfully"
fi

# Check if processes are still running after startup wait
if ! kill -0 $PID1 2>/dev/null; then
    if [[ "$SINGLE_RIG" == true ]]; then
        echo "❌ Single rig process died during startup (PID $PID1)"
        echo "📋 Single rig logs:"
        cat /tmp/rig1_run.log || echo "No rig1 log file"
        error "Single rig process died"
    else
        echo "❌ Peer 1 process died during startup (PID $PID1)"
        echo "📋 Peer 1 logs:"
        cat /tmp/peer1_run.log || echo "No peer1 log file"
        error "Peer 1 process died"
    fi
fi

if [[ "$SINGLE_RIG" == true ]]; then
    success "Single rig running (PID: $PID1) with 2 accounts"
else
    if ! kill -0 $PID2 2>/dev/null; then
        echo "❌ Peer 2 process died during startup (PID $PID2)"
        echo "📋 Peer 2 logs:"
        cat /tmp/peer2_run.log || echo "No peer2 log file"
        error "Peer 2 process died"
    fi
    success "Both peers running (PIDs: $PID1, $PID2)"
fi

# Step 5: Send email (direct binary - no compilation)
log "📧 Sending email via SMTP (direct binary)..."
FROM="test@${ACCOUNT1_ID}.com"
TO="inbox@${ACCOUNT2_ID}.com"

log "📧 From: $FROM"  
log "📧 To: $TO"

# Use direct binary (no compilation delay during email send)
if [[ "$SINGLE_RIG" == true ]]; then
    FASTN_HOME_FOR_SEND="$TEST_DIR/rig1"
    log "📧 Sending from account 1 to account 2 within single rig..."
else
    FASTN_HOME_FOR_SEND="$TEST_DIR/peer1"
    log "📧 Sending from peer 1 to peer 2..."
fi

if FASTN_HOME="$FASTN_HOME_FOR_SEND" "$FASTN_MAIL" send-mail \
    --smtp "$SMTP_PORT1" --password "$ACCOUNT1_PWD" \
    --from "$FROM" --to "$TO" \
    --subject "Direct Binary Test" \
    --body "No compilation delays"; then
    success "Email sent via direct binary execution"
else
    error "SMTP email sending failed with direct binary"
fi

# Step 6: Monitor delivery with precise timing
if [[ "$SINGLE_RIG" == true ]]; then
    log "⏳ Monitoring local delivery within single rig (precise timing)..."
else
    log "⏳ Monitoring P2P delivery between rigs (precise timing)..."
fi

for attempt in $(seq 1 8); do
    sleep 3  # Shorter intervals since no compilation delays
    elapsed=$((attempt * 3))
    
    # Use direct binary for email counting (no compilation delay)
    if [[ "$SINGLE_RIG" == true ]]; then
        SENT_COUNT=$("$TEST_UTILS" count-emails -a "$TEST_DIR/rig1/accounts/$ACCOUNT1_ID" -f Sent | jq -r '.count')
        INBOX_COUNT=$("$TEST_UTILS" count-emails -a "$TEST_DIR/rig1/accounts/$ACCOUNT2_ID" -f INBOX | jq -r '.count')
    else
        SENT_COUNT=$("$TEST_UTILS" count-emails -a "$TEST_DIR/peer1/accounts/$ACCOUNT1_ID" -f Sent | jq -r '.count')
        INBOX_COUNT=$("$TEST_UTILS" count-emails -a "$TEST_DIR/peer2/accounts/$ACCOUNT2_ID" -f INBOX | jq -r '.count')
    fi
    
    log "📊 ${elapsed}s: Sent=$SENT_COUNT, INBOX=$INBOX_COUNT"
    
    if [ "$INBOX_COUNT" -gt 0 ]; then
        if [[ "$SINGLE_RIG" == true ]]; then
            success "🎉 Local delivery completed in ${elapsed}s within single rig!"
            log "✅ Local delivery validation: Email found in account 2 INBOX"
            log "✅ Single-rig pipeline validation: SMTP → local delivery → INBOX complete"
        else
            success "🎉 P2P delivery completed in ${elapsed}s with direct binaries!"
            log "✅ P2P delivery validation: Email found in receiver INBOX"
            log "✅ Email pipeline validation: SMTP → fastn-p2p → INBOX complete"
        fi
        
        success "🎉 COMPLETE SUCCESS with direct binary execution!"
        success "📊 SMTP→P2P→INBOX delivery working without compilation delays"
        exit 0
    fi
done

# Still failed - show debug info
warn "P2P delivery failed even with direct binaries and precise timing"
log "🐛 This suggests the issue is NOT compilation delays..."

if [[ "$SINGLE_RIG" == true ]]; then
    log "Recent single rig P2P logs:"
    grep -E "P2P|stream.*reply|deliver.*emails|DEBUG" /tmp/rig1_run_${TEST_SUFFIX}.log | tail -10 || warn "No P2P logs"
    
    log "📁 Debug artifacts preserved at:"
    log "   Test directory: $TEST_DIR"
    log "   Single rig run log: /tmp/rig1_run_${TEST_SUFFIX}.log"
    log "   Rig init log: /tmp/rig1_init_${TEST_SUFFIX}.log"
    log "   Account 2 create log: /tmp/rig1_account2_${TEST_SUFFIX}.log"
else
    log "Recent peer 1 P2P logs:"
    grep -E "P2P|stream.*reply|deliver.*emails|DEBUG" /tmp/peer1_run_${TEST_SUFFIX}.log | tail -10 || warn "No P2P logs"

    log "Recent peer 2 acceptance logs:"
    grep -E "Connection accepted|Account message|DEBUG" /tmp/peer2_run_${TEST_SUFFIX}.log | tail -10 || warn "No acceptance logs"

    log "📁 Debug artifacts preserved at:"
    log "   Test directory: $TEST_DIR"
    log "   Peer 1 run log: /tmp/peer1_run_${TEST_SUFFIX}.log"
    log "   Peer 2 run log: /tmp/peer2_run_${TEST_SUFFIX}.log"
    log "   Peer 1 init log: /tmp/peer1_init_${TEST_SUFFIX}.log" 
    log "   Peer 2 init log: /tmp/peer2_init_${TEST_SUFFIX}.log"
fi

error "Direct binary execution also timed out - check artifacts above for debugging"