#!/bin/bash
# 🎯 FASTN CRITICAL EMAIL SYSTEM TESTS
#
# This script runs the most important tests in fastn - the complete email pipeline tests.
# If these tests pass, the entire fastn email system is operational.
#
# Usage:
#   ./test.sh            # Run bash plain text test (default, fastest)
#   ./test.sh --rust     # Run only Rust STARTTLS test (slower due to cargo)
#   ./test.sh --both     # Run both critical tests (comprehensive validation)
#   ./test.sh --help     # Show this help

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m' 
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
BOLD='\033[1m'
NC='\033[0m'

log() { echo -e "${BLUE}[$(date +'%H:%M:%S')] $1${NC}"; }
success() { echo -e "${GREEN}✅ $1${NC}"; }
error() { echo -e "${RED}❌ $1${NC}"; exit 1; }
warn() { echo -e "${YELLOW}⚠️  $1${NC}"; }
header() { echo -e "${BOLD}${BLUE}$1${NC}"; }

# Parse command line arguments  
RUN_RUST=false
RUN_BASH=true

case "${1:-}" in
    --rust)
        RUN_BASH=false
        RUN_RUST=true
        log "Running only Rust STARTTLS test"
        ;;
    --both)
        RUN_RUST=true
        RUN_BASH=true
        log "Running both critical email tests"
        ;;
    --help)
        echo "🎯 FASTN CRITICAL EMAIL SYSTEM TESTS"
        echo
        echo "Usage:"
        echo "  ./test.sh            # Run bash plain text test (default, fastest)"
        echo "  ./test.sh --rust     # Run only Rust STARTTLS test (slower due to cargo)"
        echo "  ./test.sh --both     # Run both critical tests (comprehensive validation)"
        echo "  ./test.sh --help     # Show this help"
        echo
        echo "These tests validate the complete fastn email pipeline:"
        echo "  - SMTP server functionality (plain text and STARTTLS)"
        echo "  - fastn-p2p email delivery between rigs"
        echo "  - Email storage in Sent and INBOX folders"  
        echo "  - End-to-end email system integration"
        exit 0
        ;;
    "")
        log "Running bash plain text test (default, fastest)"
        ;;
    *)
        error "Unknown option: $1. Use --help for usage information."
        ;;
esac

header "🎯 🎯 FASTN CRITICAL EMAIL SYSTEM TESTS 🎯 🎯"
header "============================================="
log "These are the most important tests in fastn"
log "If these pass, the entire email system is operational"
echo

# Track test results
RUST_RESULT=""
BASH_RESULT=""
TESTS_RUN=0
TESTS_PASSED=0

# Function to run Rust STARTTLS test
run_rust_test() {
    header "🔐 CRITICAL TEST #1: Rust STARTTLS Integration"
    log "Test: email_end_to_end_starttls.rs"
    log "Mode: Encrypted STARTTLS SMTP → fastn-p2p → INBOX"
    echo
    
    if cargo test -p fastn-rig email_end_to_end_starttls -- --nocapture; then
        success "Rust STARTTLS test PASSED"
        RUST_RESULT="✅ PASSED"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        if [ "$RUN_BASH" = false ]; then
            # Running only Rust test - exit immediately on failure
            error "Rust STARTTLS test FAILED"
        else
            # Running both tests - continue to show final results
            warn "Rust STARTTLS test FAILED"
            RUST_RESULT="❌ FAILED"
        fi
    fi
    TESTS_RUN=$((TESTS_RUN + 1))
    echo
}

# Function to run bash plain text test
run_bash_test() {
    header "📧 CRITICAL TEST #2: Bash Plain Text Integration"
    log "Test: email_end_to_end_plaintext.sh"
    log "Mode: Plain text SMTP → fastn-p2p → INBOX"
    echo
    
    cd fastn-rig
    if bash tests/email_end_to_end_plaintext.sh; then
        success "Bash plain text test PASSED"
        BASH_RESULT="✅ PASSED"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        if [ "$RUN_RUST" = false ]; then
            # Running only bash test - exit immediately on failure
            cd ..
            error "Bash plain text test FAILED"
        else
            # Running both tests - continue to show final results
            warn "Bash plain text test FAILED"
            BASH_RESULT="❌ FAILED"
        fi
    fi
    cd ..
    TESTS_RUN=$((TESTS_RUN + 1))
    echo
}

# Run selected tests
if [ "$RUN_RUST" = true ]; then
    run_rust_test
fi

if [ "$RUN_BASH" = true ]; then
    run_bash_test
fi

# Final results
header "🎯 CRITICAL EMAIL TESTS SUMMARY"
header "================================"

if [ "$RUN_RUST" = true ]; then
    echo -e "🔐 STARTTLS Test (Rust): $RUST_RESULT"
fi

if [ "$RUN_BASH" = true ]; then
    echo -e "📧 Plain Text Test (Bash): $BASH_RESULT" 
fi

echo
if [ $TESTS_PASSED -eq $TESTS_RUN ]; then
    success "🎉 ALL CRITICAL TESTS PASSED ($TESTS_PASSED/$TESTS_RUN)"
    success "🎉 fastn email system is FULLY OPERATIONAL"
    echo
    echo -e "${BOLD}${GREEN}🚀 READY FOR PRODUCTION EMAIL DEPLOYMENT 🚀${NC}"
    exit 0
else
    error "❌ CRITICAL TESTS FAILED ($TESTS_PASSED/$TESTS_RUN passed)"
    error "❌ fastn email system has issues - investigate failures above"
    exit 1
fi