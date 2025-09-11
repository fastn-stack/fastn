#!/usr/bin/osascript
-- Apple Script to automate fastn email account setup in Apple Mail
-- Run with: osascript setup-apple-mail.scpt

on run
    -- Alice account details (secure .fastn format)
    set aliceEmail to "alice@71es6evsls5l9l4g3ksjnsp6v8s07fldg8e2ufvu2ohjq5tthus0.fastn"
    set alicePassword to "DI*l4qHBjdGl$J7t"
    set aliceFullName to "Alice Test (fastn P2P)"
    
    -- Server configuration
    set imapServer to "localhost"
    set imapPort to 8143
    set smtpServer to "localhost"  
    set smtpPort to 8587
    
    display dialog "🍎 Setting up Alice's fastn email account in Apple Mail

Account Details:
Email: " & aliceEmail & "
Full Name: " & aliceFullName & "
IMAP: " & imapServer & ":" & imapPort & "
SMTP: " & smtpServer & ":" & smtpPort & "

⚠️ IMPORTANT: You'll need to manually accept the self-signed certificate when prompted.

Click OK to start the setup process..." buttons {"Cancel", "OK"} default button 2
    
    if button returned of result = "Cancel" then
        return
    end if
    
    -- Open Mail app
    tell application "Mail"
        activate
        delay 2
        
        -- Open Mail preferences
        tell application "System Events"
            tell process "Mail"
                -- Open preferences with Cmd+,
                key code 44 using command down
                delay 3
                
                -- Click Accounts tab
                try
                    click button "Accounts" of toolbar 1 of window 1
                    delay 2
                on error
                    display dialog "Could not find Accounts tab. Please manually navigate to Mail > Settings > Accounts and click the + button to add an account."
                    return
                end try
                
                -- Click + button to add account
                try
                    click button 1 of scroll area 1 of splitter group 1 of window 1
                    delay 2
                on error
                    display dialog "Could not find + button. Please manually click the + button in the accounts list."
                    return
                end try
            end tell
        end tell
        
        delay 3
        
        -- Handle account setup dialog
        tell application "System Events"
            tell process "Mail"
                -- Look for "Other Mail Account" option
                try
                    click button "Other Mail Account..." of sheet 1 of window 1
                    delay 2
                on error
                    display dialog "Please select 'Other Mail Account...' option in the dialog."
                    return
                end try
                
                -- Fill in account information
                try
                    -- Full Name field
                    set value of text field 1 of sheet 1 of window 1 to aliceFullName
                    
                    -- Email field  
                    set value of text field 2 of sheet 1 of window 1 to aliceEmail
                    
                    -- Password field
                    set value of text field 3 of sheet 1 of window 1 to alicePassword
                    
                    delay 1
                    
                    -- Click Sign In button
                    click button "Sign In" of sheet 1 of window 1
                    delay 5
                    
                on error
                    display dialog "Could not fill in account fields. Please manually enter:
Full Name: " & aliceFullName & "
Email: " & aliceEmail & "
Password: " & alicePassword
                    return
                end try
            end tell
        end tell
        
        -- Wait for account verification and manual server setup
        display dialog "✅ Account information entered!

Apple Mail will now try to automatically configure the account.
This will likely FAIL because fastn uses custom ports.

NEXT STEPS:
1. Click 'Cancel' or 'Manual Setup' when automatic setup fails
2. In Manual Setup, configure:

IMAP Server:
- Server: localhost
- Port: 8143
- Security: None
- Username: " & aliceEmail & "

SMTP Server:  
- Server: localhost
- Port: 8587
- Security: None
- Username: " & aliceEmail & "

3. Accept the self-signed certificate when prompted

The script will wait while you complete the manual setup..." buttons {"Continue"} default button 1
        
        display dialog "🎉 Alice's fastn account setup complete!

To test:
1. Start fastn-rig servers (see SETUP_SUMMARY.md)
2. Compose a new email to: bob@2kuos0orl2tu40st5oiasb6dip9ojefv9ob072khncvi7gooahd0.fastn
3. Send email and verify P2P delivery

Next: Run setup-apple-mail-bob.scpt for Bob's account" buttons {"Done"} default button 1
        
    end tell
end run