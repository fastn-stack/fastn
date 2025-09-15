#!/usr/bin/osascript
-- Smart Apple Mail setup that automates everything possible and asks for help when stuck

on run
    -- Account details
    set aliceEmail to "alice@71es6evsls5l9l4g3ksjnsp6v8s07fldg8e2ufvu2ohjq5tthus0.fastn"
    set alicePassword to "DI*l4qHBjdGl$J7t"
    set aliceFullName to "Alice Test (fastn P2P)"
    
    display dialog "🤖 SMART APPLE MAIL SETUP

I'll automate everything I can and ask for help only when needed.

Setting up: " & aliceEmail & "

Let's start!" buttons {"Cancel", "Start Automation"} default button 2
    
    if button returned of result = "Cancel" then
        return
    end if
    
    -- Step 1: Open Mail (automated)
    try
        tell application "Mail"
            activate
            delay 2
        end tell
    on error
        display dialog "❌ Could not open Apple Mail. Please open it manually, then click OK." buttons {"OK"}
    end try
    
    -- Step 2: Open Preferences (automated)
    try
        tell application "System Events"
            tell process "Mail"
                key code 44 using command down -- Cmd+,
                delay 3
            end tell
        end tell
    on error
        display dialog "🙋‍♂️ I need help!

I couldn't open Mail settings automatically.

Please do this manually:
1. In Apple Mail, click 'Mail' menu
2. Click 'Settings' (or 'Preferences')

Then click OK to continue." buttons {"OK"}
    end try
    
    -- Step 3: Click Accounts tab (automated)
    try
        tell application "System Events"
            tell process "Mail"
                click button "Accounts" of toolbar 1 of window 1
                delay 2
            end tell
        end tell
    on error
        display dialog "🙋‍♂️ I need help!

I couldn't click the Accounts tab.

Please manually:
1. Click the 'Accounts' tab in the settings window

Then click OK to continue." buttons {"OK"}
    end try
    
    -- Step 4: Add new account (automated)
    try
        tell application "System Events"
            tell process "Mail"
                click button 1 of scroll area 1 of splitter group 1 of window 1 -- + button
                delay 3
            end tell
        end tell
    on error
        display dialog "🙋‍♂️ I need help!

I couldn't click the + button to add an account.

Please manually:
1. Click the '+' button (bottom left of accounts list)

Then click OK to continue." buttons {"OK"}
    end try
    
    -- Step 5: Select Other Mail Account (automated)
    try
        tell application "System Events"
            tell process "Mail"
                click button "Other Mail Account…" of sheet 1 of window 1
                delay 2
            end tell
        end tell
    on error
        display dialog "🙋‍♂️ I need help!

I couldn't select the account type.

Please manually:
1. Click 'Other Mail Account...' in the dialog

Then click OK to continue." buttons {"OK"}
    end try
    
    -- Step 6: Fill account information (automated)
    try
        tell application "System Events"
            tell process "Mail"
                -- Full Name
                set value of text field 1 of sheet 1 of window 1 to aliceFullName
                delay 0.5
                
                -- Email  
                set value of text field 2 of sheet 1 of window 1 to aliceEmail
                delay 0.5
                
                -- Password
                set value of text field 3 of sheet 1 of window 1 to alicePassword
                delay 1
                
                -- Click Sign In
                click button "Sign In" of sheet 1 of window 1
                delay 5
            end tell
        end tell
    on error
        display dialog "🙋‍♂️ I need help!

I couldn't fill in the account information automatically.

Please manually enter:
- Full Name: " & aliceFullName & "
- Email: " & aliceEmail & "  
- Password: " & alicePassword & "

Then click 'Sign In' and click OK when done." buttons {"OK"}
    end try
    
    -- Step 7: Handle failed auto-setup (manual guidance)
    display dialog "🤖 Auto-setup will fail (expected)

Apple Mail will try to auto-configure and fail because fastn uses custom ports.

When you see the error/failure:
1. Click 'Manual Setup' or 'Configure Manually'

Click OK when you've done this." buttons {"OK"}
    
    -- Step 8: Manual server configuration (provide exact settings)
    display dialog "🙋‍♂️ Manual Server Configuration Required

Apple Mail should now show server configuration fields.

IMAP (Incoming Mail):
- Account Type: IMAP
- Mail Server: localhost  
- Port: 8143
- Use SSL: NO (uncheck!)
- Username: " & aliceEmail & "
- Password: " & alicePassword & "

SMTP (Outgoing Mail):
- SMTP Server: localhost
- Port: 8587  
- Use SSL: NO (uncheck!)
- Username: " & aliceEmail & "
- Password: " & alicePassword & "

Click OK when servers are configured." buttons {"OK"}
    
    -- Step 9: Certificate acceptance
    display dialog "🔒 Certificate Acceptance

You'll see certificate warnings for localhost.

When prompted:
1. Click 'Trust' or 'Accept' or 'Connect'  
2. You may see this twice (IMAP + SMTP)

Click OK when certificates are accepted." buttons {"OK"}
    
    -- Final verification
    display dialog "🎉 Setup Complete!

Alice's account should now be working in Apple Mail.

🧪 TEST NOW:
1. Compose new email in Apple Mail
2. Send TO: bob@2kuos0orl2tu40st5oiasb6dip9ojefv9ob072khncvi7gooahd0.fastn
3. Check Bob's INBOX in ~/fastn-email for delivery

📊 Or run: osascript check-mail-status.scpt" buttons {"Test Status", "Done"} default button 1
    
    if button returned of result = "Test Status" then
        do shell script "osascript " & quoted form of ((path to me as text) & "../check-mail-status.scpt")
    end if
    
end run