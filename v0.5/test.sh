#!/bin/bash
# 🎯 FASTN CRITICAL EMAIL SYSTEM TESTS
#
# This script runs the most important tests in fastn - the complete email pipeline tests.
# If these tests pass, the entire fastn email system is operational.
#
# 🎯 TESTING PHILOSOPHY:
# - This script runs exactly ONE comprehensive end-to-end test (with rust/bash variants)
# - The existing tests are ENHANCED with additional verification (like IMAP)
# - DO NOT add new separate tests here - enhance the existing ones
# - The goal is ONE test that validates EVERYTHING: SMTP + P2P + IMAP + filesystem
# - Each test should use dual verification where possible (protocol vs filesystem)
#
# Usage:
#   ./test.sh           # Run bash plain text test with multi-rig (default, fastest)
#   ./test.sh --rust    # Run only Rust STARTTLS test with multi-rig
#   ./test.sh --both    # Run both Rust and bash tests with multi-rig
#   ./test.sh --single  # Run bash test with single rig, two accounts
#   ./test.sh --multi   # Run bash tests with both single and multi-rig modes
#   ./test.sh --all     # Run all tests: both single/multi rig modes AND both rust/bash
#   ./test.sh --help    # Show this help

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
SINGLE_RIG=false
BOTH_MODES=false

# Parse all arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --rust)
            RUN_BASH=false
            RUN_RUST=true
            ;;
        --both)
            RUN_RUST=true
            RUN_BASH=true
            ;;
        --single)
            SINGLE_RIG=true
            ;;
        --multi)
            BOTH_MODES=true
            ;;
        --all)
            RUN_RUST=true
            RUN_BASH=true
            BOTH_MODES=true
            ;;
        --help)
            echo "🎯 FASTN CRITICAL EMAIL SYSTEM TESTS"
            echo
            echo "Usage:"
            echo "  ./test.sh           # Run bash plain text test with multi-rig (default, fastest)"
            echo "  ./test.sh --rust    # Run only Rust STARTTLS test with multi-rig"
            echo "  ./test.sh --both    # Run both Rust and bash tests with multi-rig"
            echo "  ./test.sh --single  # Run bash test with single rig, two accounts"
            echo "  ./test.sh --multi   # Run bash tests with both single and multi-rig modes"
            echo "  ./test.sh --all     # Run all tests: both single/multi rig modes AND both rust/bash"
            echo "  ./test.sh --help    # Show this help"
            echo
            echo "These tests validate the complete fastn email pipeline:"
            echo "  - SMTP server functionality (plain text and STARTTLS)"
            echo "  - fastn-p2p email delivery between rigs (or accounts within single rig)"
            echo "  - Email storage in Sent and INBOX folders"  
            echo "  - End-to-end email system integration"
            echo
            echo "Test modes:"
            echo "  Multi-rig: Tests inter-rig communication (1 account per rig)"
            echo "  Single-rig: Tests intra-rig communication (2 accounts in 1 rig)"
            echo "  --multi: Runs both single and multi-rig to find different bugs"
            echo "  --all: Comprehensive testing for CI/release validation"
            exit 0
            ;;
        "")
            # No arguments - default behavior (two-rigs bash test)
            ;;
        *)
            error "Unknown option: $1. Use --help for usage information."
            ;;
    esac
    shift
done

# Log what we're running
if [[ "$BOTH_MODES" == true ]]; then
    if [[ "$RUN_RUST" == true && "$RUN_BASH" == true ]]; then
        log "Running ALL TESTS: bash+rust × single+multi-rig (comprehensive CI mode)"
    else
        log "Running bash tests in MULTI-MODE: single + multi-rig"
    fi
elif [[ "$SINGLE_RIG" == true ]]; then
    if [[ "$RUN_RUST" == true && "$RUN_BASH" == true ]]; then
        log "Running both critical email tests in SINGLE-RIG mode"
    elif [[ "$RUN_RUST" == true ]]; then
        log "Running only Rust STARTTLS test in SINGLE-RIG mode"
    else
        log "Running bash plain text test in SINGLE-RIG mode"
    fi
else
    if [[ "$RUN_RUST" == true && "$RUN_BASH" == true ]]; then
        log "Running both critical email tests (multi-rig mode)"
    elif [[ "$RUN_RUST" == true ]]; then
        log "Running only Rust STARTTLS test (multi-rig mode)" 
    else
        log "Running bash plain text test (multi-rig mode, fastest)"
    fi
fi

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
    local mode_desc="Multi-rig: Encrypted STARTTLS SMTP → fastn-p2p → INBOX"
    if [[ "${1:-}" == "single-rig" ]]; then
        mode_desc="Single-rig: STARTTLS SMTP → local delivery → INBOX (2 accounts)"
    fi
    
    header "🔐 CRITICAL TEST #1: Rust STARTTLS Integration"
    log "Test: email_end_to_end_starttls.rs"
    log "Mode: $mode_desc"
    echo
    
    local test_env=""
    if [[ "${1:-}" == "single-rig" ]]; then
        test_env="FASTN_TEST_SINGLE_RIG=1"
    fi
    
    if eval "$test_env cargo test -p fastn-rig email_end_to_end_starttls -- --nocapture"; then
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
    local mode_desc="Multi-rig: Plain text SMTP → fastn-p2p → INBOX"
    if [[ "${1:-}" == "single-rig" ]]; then
        mode_desc="Single-rig: Plain text SMTP → local delivery → INBOX (2 accounts)"
    fi
    
    header "📧 CRITICAL TEST #2: Bash Plain Text Integration"
    log "Test: email_end_to_end_plaintext.sh"
    log "Mode: $mode_desc"
    echo
    
    cd fastn-rig
    local script_args=""
    if [[ "${1:-}" == "single-rig" ]]; then
        script_args="--single"
    fi
    
    if bash tests/email_end_to_end_plaintext.sh $script_args; then
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
if [[ "$BOTH_MODES" == true ]]; then
    # Run both single-rig and multi-rig modes
    if [[ "$RUN_BASH" == true ]]; then
        header "📧 Running bash test in MULTI-RIG mode..."
        run_bash_test
        echo
        
        # Clean up between test modes to prevent state interference
        log "🧹 Cleaning up between multi-rig and single-rig tests..."
        rm -rf /tmp/fastn-complete-test 2>/dev/null || true
        sleep 2
        
        header "📧 Running bash test in SINGLE-RIG mode..."
        run_bash_test "single-rig"
    fi
    
    if [[ "$RUN_RUST" == true ]]; then
        echo
        header "🔐 Running Rust test in MULTI-RIG mode..."
        run_rust_test
        echo
        header "🔐 Running Rust test in SINGLE-RIG mode..."
        run_rust_test "single-rig"
    fi
else
    # Run single mode
    if [ "$RUN_RUST" = true ]; then
        if [ "$SINGLE_RIG" = true ]; then
            run_rust_test "single-rig"
        else
            run_rust_test
        fi
    fi

    if [ "$RUN_BASH" = true ]; then
        if [ "$SINGLE_RIG" = true ]; then
            run_bash_test "single-rig"
        else
            run_bash_test
        fi
    fi
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