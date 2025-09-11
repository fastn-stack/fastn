#!/bin/bash
# Start both Alice and Bob fastn-rig servers for Apple Mail testing

echo "🚀 Starting fastn email servers..."
echo "📧 Alice: SMTP 8587, IMAP 8143"  
echo "📧 Bob: SMTP 8588, IMAP 8144"
echo ""

# Start Alice server in background
echo "🍎 Starting Alice's server..."
SKIP_KEYRING=true FASTN_HOME=~/fastn-email/alice \
  FASTN_SMTP_PORT=8587 FASTN_IMAP_PORT=8143 \
  ~/.cargo/bin/cargo run --bin fastn-rig -- run > ~/fastn-email/alice_server.log 2>&1 &
ALICE_PID=$!

echo "🤖 Starting Bob's server..."  
SKIP_KEYRING=true FASTN_HOME=~/fastn-email/bob \
  FASTN_SMTP_PORT=8588 FASTN_IMAP_PORT=8144 \
  ~/.cargo/bin/cargo run --bin fastn-rig -- run > ~/fastn-email/bob_server.log 2>&1 &
BOB_PID=$!

echo "⏳ Waiting for servers to start..."
sleep 10

# Check if servers started successfully
if kill -0 $ALICE_PID 2>/dev/null && kill -0 $BOB_PID 2>/dev/null; then
    echo "✅ Both servers started successfully!"
    echo "📊 PIDs: Alice=$ALICE_PID, Bob=$BOB_PID"
    echo ""
    echo "🍎 Ready for Apple Mail configuration!"
    echo "📋 Run: osascript ~/fastn-email/setup-apple-mail.scpt"
    echo ""
    echo "📧 Test email addresses:"
    echo "   Alice: alice@71es6evsls5l9l4g3ksjnsp6v8s07fldg8e2ufvu2ohjq5tthus0.fastn"
    echo "   Bob:   bob@2kuos0orl2tu40st5oiasb6dip9ojefv9ob072khncvi7gooahd0.fastn"
    echo ""
    echo "🛑 To stop servers: kill $ALICE_PID $BOB_PID"
    echo ""
    echo "📊 Monitor logs:"
    echo "   tail -f ~/fastn-email/alice_server.log"
    echo "   tail -f ~/fastn-email/bob_server.log"
else
    echo "❌ Server startup failed!"
    echo "🔍 Check logs:"
    echo "   Alice: ~/fastn-email/alice_server.log"  
    echo "   Bob: ~/fastn-email/bob_server.log"
    exit 1
fi