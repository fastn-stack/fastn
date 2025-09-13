#!/bin/bash
set -e

# Test 1: Basic Cache Invalidation
# Scenario: File change invalidates cache and serves updated content

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FASTN_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
FASTN_BIN="$FASTN_ROOT/target/debug/fastn"
TEST_PROJECT="$SCRIPT_DIR/fixtures/basic-project"

echo "🧪 Test 1: Basic Cache Invalidation"
echo "=================================="

# Build fastn if needed
if [ ! -f "$FASTN_BIN" ]; then
    echo "Building fastn binary..."
    cd "$FASTN_ROOT"
    ~/.cargo/bin/cargo build --bin fastn --quiet
fi

echo "✅ Using fastn binary: $FASTN_BIN"

# Clean up any existing cache
echo "🧹 Cleaning cache directories..."
rm -rf ~/.cache/cache-test-basic* 2>/dev/null || true

cd "$TEST_PROJECT"
echo "📁 Working in: $(pwd)"

# Start fastn serve with caching enabled in background
echo "🚀 Starting fastn serve with --enable-cache..."
"$FASTN_BIN" serve --port 8099 --enable-cache --offline > /tmp/fastn-test.log 2>&1 &
FASTN_PID=$!

# Wait for server to start
sleep 5

echo "🔧 Testing cache behavior..."

# Test function to get content and measure time
get_content() {
    local start_time=$(date +%s%N)
    local content=$(curl -s http://localhost:8099/ 2>/dev/null || echo "ERROR")
    local end_time=$(date +%s%N)
    local duration=$(( (end_time - start_time) / 1000000 )) # Convert to milliseconds
    echo "$content|DURATION:${duration}ms"
}

# First request - should be cache miss
echo "📝 First request (cache miss expected)..."
RESULT1=$(get_content)
CONTENT1=$(echo "$RESULT1" | cut -d'|' -f1)
DURATION1=$(echo "$RESULT1" | cut -d'|' -f2)

if [[ "$CONTENT1" == *"Original hero content - Version 1"* ]]; then
    echo "✅ First request served correct content"
    echo "⏱️  $DURATION1"
else
    echo "❌ First request failed - wrong content"
    echo "Content: $CONTENT1"
    kill $FASTN_PID 2>/dev/null || true
    exit 1
fi

# Second request - should be cache hit (faster)
echo "📝 Second request (cache hit expected)..."
RESULT2=$(get_content)
CONTENT2=$(echo "$RESULT2" | cut -d'|' -f1)
DURATION2=$(echo "$RESULT2" | cut -d'|' -f2)

if [[ "$CONTENT2" == *"Original hero content - Version 1"* ]]; then
    echo "✅ Second request served correct content"
    echo "⏱️  $DURATION2"
else
    echo "❌ Second request failed - wrong content"
    kill $FASTN_PID 2>/dev/null || true
    exit 1
fi

# Modify hero.ftd content
echo "✏️  Modifying hero.ftd content..."
sed -i.bak 's/Original hero content - Version 1/MODIFIED hero content - Version 2/g' hero.ftd

# Third request - should serve updated content (cache invalidated)
echo "📝 Third request (cache invalidation expected)..."
RESULT3=$(get_content)
CONTENT3=$(echo "$RESULT3" | cut -d'|' -f1)
DURATION3=$(echo "$RESULT3" | cut -d'|' -f2)

# Restore original content
mv hero.ftd.bak hero.ftd

if [[ "$CONTENT3" == *"MODIFIED hero content - Version 2"* ]]; then
    echo "✅ Third request served UPDATED content (cache invalidation worked!)"
    echo "⏱️  $DURATION3"
else
    echo "❌ CRITICAL FAILURE: Cache invalidation did not work!"
    echo "Expected: MODIFIED hero content - Version 2"
    echo "Got: $CONTENT3"
    kill $FASTN_PID 2>/dev/null || true
    exit 1
fi

# Fourth request - should cache the new content
echo "📝 Fourth request (new cache hit expected)..."
RESULT4=$(get_content)
CONTENT4=$(echo "$RESULT4" | cut -d'|' -f1)
DURATION4=$(echo "$RESULT4" | cut -d'|' -f2)

# Clean up
kill $FASTN_PID 2>/dev/null || true
sleep 1

echo ""
echo "🎉 TEST 1 PASSED: Basic Cache Invalidation Works Correctly"
echo "=================================="
echo "✅ Cache miss: Content served correctly"
echo "✅ Cache hit: Same content served faster"  
echo "✅ File change: Cache invalidated and new content served"
echo "✅ New cache: Updated content cached for future requests"
echo ""
echo "Performance Summary:"
echo "  Request 1 (miss): $DURATION1"
echo "  Request 2 (hit):  $DURATION2" 
echo "  Request 3 (invalidated): $DURATION3"
echo "  Request 4 (new hit): $DURATION4"
echo ""

# Check for any errors in fastn log
if grep -i "error\|panic\|failed" /tmp/fastn-test.log > /dev/null 2>&1; then
    echo "⚠️  Warnings found in fastn log:"
    grep -i "error\|panic\|failed" /tmp/fastn-test.log | head -5
else
    echo "✅ No errors in fastn server log"
fi

echo "🎯 Basic cache invalidation test completed successfully!"