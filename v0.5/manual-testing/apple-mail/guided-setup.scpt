#!/usr/bin/osascript
-- Step-by-step guided Apple Mail setup with one instruction at a time

on run
    -- Alice account details
    set aliceEmail to "alice@71es6evsls5l9l4g3ksjnsp6v8s07fldg8e2ufvu2ohjq5tthus0.fastn"
    set alicePassword to "DI*l4qHBjdGl$J7t"
    set aliceFullName to "Alice Test (fastn P2P)"
    
    -- Step 1
    display dialog "📧 STEP 1: Open Apple Mail

Click 'Open Mail' and I'll guide you through each step.

We'll set up Alice's fastn account:
Email: " & aliceEmail & "

Ready?" buttons {"Cancel", "Open Mail"} default button 2
    
    if button returned of result = "Cancel" then
        return
    end if
    
    -- Open Mail
    tell application "Mail" to activate
    
    -- Step 2  
    display dialog "📧 STEP 2: Open Settings

In Apple Mail:
1. Click 'Mail' in the menu bar
2. Click 'Settings' (or 'Preferences')

Done? Click 'Next Step' when Settings window is open." buttons {"Cancel", "Next Step"} default button 2
    
    if button returned of result = "Cancel" then
        return
    end if
    
    -- Step 3
    display dialog "📧 STEP 3: Go to Accounts

In the Settings window:
1. Click the 'Accounts' tab at the top

Done? Click 'Next Step' when you're on Accounts tab." buttons {"Cancel", "Next Step"} default button 2
    
    if button returned of result = "Cancel" then
        return
    end if
    
    -- Step 4
    display dialog "📧 STEP 4: Add New Account

In the Accounts tab:
1. Click the '+' button (bottom left of accounts list)

Done? Click 'Next Step' when account type dialog appears." buttons {"Cancel", "Next Step"} default button 2
    
    if button returned of result = "Cancel" then
        return
    end if
    
    -- Step 5
    display dialog "📧 STEP 5: Select Account Type

In the account type dialog:
1. Click 'Other Mail Account...'

Done? Click 'Next Step' when account info form appears." buttons {"Cancel", "Next Step"} default button 2
    
    if button returned of result = "Cancel" then
        return
    end if
    
    -- Step 6 - Account Info  
    display dialog "📧 STEP 6: Enter Account Information

Fill in these EXACT values:

Full Name: " & aliceFullName & "
Email: " & aliceEmail & "  
Password: " & alicePassword & "

Then click 'Sign In' (it will try auto-setup and fail)

Done? Click 'Next Step' after clicking Sign In." buttons {"Cancel", "Next Step"} default button 2
    
    if button returned of result = "Cancel" then
        return
    end if
    
    -- Step 7
    display dialog "📧 STEP 7: Choose Manual Setup

Auto-setup will fail (expected). When prompted:
1. Click 'Manual Setup' or 'Configure Manually'

Done? Click 'Next Step' when manual setup dialog appears." buttons {"Cancel", "Next Step"} default button 2
    
    if button returned of result = "Cancel" then
        return
    end if
    
    -- Step 8 - IMAP
    display dialog "📧 STEP 8: Configure IMAP Server

In the manual setup dialog:

INCOMING MAIL SERVER (IMAP):
Account Type: IMAP
Mail Server: localhost
Port: 8143
Use SSL: NO (uncheck this!)
Username: " & aliceEmail & "
Password: " & alicePassword & "

Done? Click 'Next Step' when IMAP is configured." buttons {"Cancel", "Next Step"} default button 2
    
    if button returned of result = "Cancel" then
        return
    end if
    
    -- Step 9 - SMTP
    display dialog "📧 STEP 9: Configure SMTP Server  

OUTGOING MAIL SERVER (SMTP):
SMTP Server: localhost
Port: 8587
Use SSL: NO (uncheck this!)
Username: " & aliceEmail & "
Password: " & alicePassword & "

Done? Click 'Next Step' when SMTP is configured." buttons {"Cancel", "Next Step"} default button 2
    
    if button returned of result = "Cancel" then
        return
    end if
    
    -- Step 10 - Certificate
    display dialog "📧 STEP 10: Accept Certificate

When prompted about certificate:
1. Click 'Connect' or 'Trust' or 'Accept'
2. You may see this prompt twice (IMAP + SMTP)

Done? Click 'Next Step' after accepting certificates." buttons {"Cancel", "Next Step"} default button 2
    
    if button returned of result = "Cancel" then
        return
    end if
    
    -- Step 11 - Completion
    display dialog "🎉 SETUP COMPLETE!

Alice's fastn account should now be configured in Apple Mail.

🧪 NEXT STEPS:
1. Test Status: osascript check-mail-status.scpt
2. Send Test Email: osascript send-test-email.scpt  
3. Full Automation: osascript automated-mail-test.scpt

📧 TEST EMAIL ADDRESS:
Send email TO: bob@2kuos0orl2tu40st5oiasb6dip9ojefv9ob072khncvi7gooahd0.fastn

💡 TIP: Compose a new email and send to Bob to test P2P delivery!" buttons {"Run Status Check", "Done"} default button 2
    
    if button returned of result = "Run Status Check" then
        do shell script "osascript /Users/amitu/Projects/fastn-me/v0.5/manual-testing/apple-mail/check-mail-status.scpt"
    end if
    
end run