#!/usr/bin/osascript
-- Apple Script to set up Bob's fastn email account in Apple Mail

on run
    -- Bob account details (secure .fastn format)
    set bobEmail to "bob@2kuos0orl2tu40st5oiasb6dip9ojefv9ob072khncvi7gooahd0.fastn"
    set bobPassword to "Q0MilZ7sjJB74mDq"
    set bobFullName to "Bob Test (fastn P2P)"
    
    -- Server configuration
    set imapServer to "localhost"
    set imapPort to 8144
    set smtpServer to "localhost"
    set smtpPort to 8588
    
    display dialog "🍎 Setting up Bob's fastn email account in Apple Mail

Account Details:
Email: " & bobEmail & "
Full Name: " & bobFullName & "
IMAP: " & imapServer & ":" & imapPort & "
SMTP: " & smtpServer & ":" & smtpPort & "

This will add a SECOND account to Apple Mail for testing P2P email between Alice and Bob.

Click OK to continue..." buttons {"Cancel", "OK"} default button 2
    
    if button returned of result = "Cancel" then
        return
    end if
    
    -- Follow same process as Alice setup
    tell application "Mail"
        activate
        delay 2
        
        tell application "System Events"
            tell process "Mail"
                -- Open preferences
                key code 44 using command down
                delay 3
                
                -- Navigate to Accounts
                try
                    click button "Accounts" of toolbar 1 of window 1
                    delay 2
                    
                    -- Add new account
                    click button 1 of scroll area 1 of splitter group 1 of window 1
                    delay 3
                    
                    -- Select Other Mail Account
                    click button "Other Mail Account..." of sheet 1 of window 1
                    delay 2
                    
                    -- Fill account details
                    set value of text field 1 of sheet 1 of window 1 to bobFullName
                    set value of text field 2 of sheet 1 of window 1 to bobEmail
                    set value of text field 3 of sheet 1 of window 1 to bobPassword
                    
                    delay 1
                    click button "Sign In" of sheet 1 of window 1
                    delay 5
                    
                on error theError
                    display dialog "Automated setup failed: " & theError & "

Please manually add Bob's account:
1. Mail > Settings > Accounts > +
2. Other Mail Account
3. Enter:
   - Name: " & bobFullName & "
   - Email: " & bobEmail & "
   - Password: " & bobPassword & "

Then configure servers manually:
IMAP: localhost:" & imapPort & " (No Security)
SMTP: localhost:" & smtpPort & " (No Security)"
                end try
            end tell
        end tell
        
        display dialog "✅ Bob's account setup initiated!

Manual Configuration Required:
IMAP Server: localhost:" & imapPort & "
SMTP Server: localhost:" & smtpPort & "
Username: " & bobEmail & "
Security: None for both servers

🧪 Testing Instructions:
1. Start both fastn-rig servers
2. Send email from Alice to Bob  
3. Send email from Bob to Alice
4. Verify P2P delivery in both directions

Both accounts ready for fastn P2P email testing!" buttons {"Done"} default button 1
    end tell
end run