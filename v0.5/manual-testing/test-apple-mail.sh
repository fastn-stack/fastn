#!/bin/bash

# FASTN Email Apple Mail Automation  
# Automates Apple Mail account setup and email testing

set -euo pipefail

FASTN_EMAIL_DIR="$HOME/fastn-email"

if [ ! -f "$FASTN_EMAIL_DIR/SETUP_SUMMARY.md" ]; then
    echo "❌ Setup summary not found. Run setup-fastn-email.sh first."
    exit 1
fi

echo "🍎 FASTN Email Apple Mail Testing"  
echo "================================="

# Source account information
ALICE_ACCOUNT=$(ls "$FASTN_EMAIL_DIR/alice/accounts/" | head -1)
BOB_ACCOUNT=$(ls "$FASTN_EMAIL_DIR/bob/accounts/" | head -1)

# Extract SMTP passwords (simplified - assume they're in summary file)
ALICE_SMTP_PASS=$(grep "SMTP.*Password:" "$FASTN_EMAIL_DIR/SETUP_SUMMARY.md" | head -1 | grep -o "\`[^']*\`" | tr -d '`' | head -1)
BOB_SMTP_PASS=$(grep "SMTP.*Password:" "$FASTN_EMAIL_DIR/SETUP_SUMMARY.md" | head -2 | tail -1 | grep -o "\`[^']*\`" | tr -d '`' | head -1)

if [ -z "$ALICE_SMTP_PASS" ] || [ -z "$BOB_SMTP_PASS" ]; then
    echo "❌ Could not extract SMTP passwords from summary file"
    echo "🔍 Please check ~/fastn-email/SETUP_SUMMARY.md manually"
    exit 1
fi

echo "📋 Test Configuration:"
echo "Alice: alice@$ALICE_ACCOUNT.com (Password: $ALICE_SMTP_PASS)"
echo "Bob: bob@$BOB_ACCOUNT.com (Password: $BOB_SMTP_PASS)" 
echo ""

echo "⚠️  This script will configure Apple Mail accounts for FASTN testing."
echo "🛑 This will modify your Apple Mail settings."
echo ""
read -p "Continue? (y/N): " -r
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Cancelled."
    exit 0
fi

echo ""
echo "🚀 Configuring Apple Mail accounts..."

# Apple Mail account configuration via AppleScript
osascript << EOF
tell application "Mail"
    activate
    
    -- Remove existing FASTN test accounts if they exist
    try
        set existingAccount to account "Alice FASTN Test"
        delete existingAccount
    end try
    
    try  
        set existingAccount to account "Bob FASTN Test"
        delete existingAccount
    end try
    
    -- Wait for Mail to be ready
    delay 2
    
    -- Create Alice account
    display dialog "Setting up Alice account..." with title "FASTN Email Setup" buttons {"Continue"} default button 1
    
    -- Note: Apple Mail account creation via AppleScript is limited
    -- We'll provide manual setup instructions instead
end tell

tell application "System Events"
    -- Open Mail preferences
    tell application "Mail"
        activate
    end tell
    
    delay 1
    key code 44 using command down -- Cmd+,
    
    delay 2
    
    -- Click Accounts tab
    click button "Accounts" of toolbar 1 of window 1 of application process "Mail"
    
    delay 1
    
    -- Instructions for manual setup
    display dialog "Apple Mail Preferences opened.

ALICE SETUP:
1. Click '+' to add account
2. Choose 'Other Mail Account'
3. Name: Alice FASTN Test  
4. Email: alice@$ALICE_ACCOUNT.com
5. Password: $ALICE_SMTP_PASS

IMAP Settings:
- Server: localhost
- Port: 8143
- Username: alice

SMTP Settings:  
- Server: localhost
- Port: 8587
- Username: alice
- Password: $ALICE_SMTP_PASS

Click OK when Alice account is set up." with title "Alice Account Setup" buttons {"OK"} default button 1
    
    -- Bob account setup
    display dialog "BOB SETUP:
1. Click '+' to add another account
2. Choose 'Other Mail Account'  
3. Name: Bob FASTN Test
4. Email: bob@$BOB_ACCOUNT.com
5. Password: $BOB_SMTP_PASS

IMAP Settings:
- Server: localhost  
- Port: 8144
- Username: bob

SMTP Settings:
- Server: localhost
- Port: 8588  
- Username: bob
- Password: $BOB_SMTP_PASS

Click OK when Bob account is set up." with title "Bob Account Setup" buttons {"OK"} default button 1
end tell
EOF

echo "✅ Apple Mail preferences opened with setup instructions"
echo ""
echo "📧 After setting up accounts, test email sending:"
echo ""
echo "1. In Apple Mail, compose new email"
echo "2. From: Alice FASTN Test"  
echo "3. To: bob@$BOB_ACCOUNT.com"
echo "4. Subject: Apple Mail Test"
echo "5. Body: Testing FASTN email via Apple Mail"
echo "6. Send email"
echo ""
echo "7. Check if email arrives in Bob's inbox"
echo "8. Reply from Bob to Alice"
echo "9. Verify bidirectional communication"
echo ""
echo "🔍 Monitor server logs for any errors:"
echo "tail -f ~/fastn-email/manual-testing-logs/alice_test.log"
echo "tail -f ~/fastn-email/manual-testing-logs/bob_test.log"
echo ""

# Keep script running to monitor
echo "📊 Email monitoring active. Press Ctrl+C to stop."
echo "Watching for new emails..."

# Monitor email directories for changes
while true; do
    ALICE_INBOX_COUNT=$(find "$FASTN_EMAIL_DIR/alice/accounts/$ALICE_ACCOUNT/mails/default/INBOX" -name "*.eml" 2>/dev/null | wc -l)
    BOB_INBOX_COUNT=$(find "$FASTN_EMAIL_DIR/bob/accounts/$BOB_ACCOUNT/mails/default/INBOX" -name "*.eml" 2>/dev/null | wc -l)
    
    echo "$(date): Alice INBOX: $ALICE_INBOX_COUNT, Bob INBOX: $BOB_INBOX_COUNT"
    sleep 10
done