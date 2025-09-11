#!/usr/bin/osascript
-- Provide clear Apple Mail setup instructions without requiring accessibility permissions

on run
    -- Alice account details
    set aliceEmail to "alice@71es6evsls5l9l4g3ksjnsp6v8s07fldg8e2ufvu2ohjq5tthus0.fastn"
    set alicePassword to "DI*l4qHBjdGl$J7t"
    set aliceFullName to "Alice Test (fastn P2P)"
    
    set setupInstructions to "🍎 APPLE MAIL SETUP INSTRUCTIONS

📧 ALICE'S ACCOUNT CONFIGURATION:

1. Open Apple Mail
2. Go to Mail > Settings (or Mail > Preferences)
3. Click 'Accounts' tab
4. Click the '+' button to add new account
5. Select 'Other Mail Account...'

6. Enter Account Information:
   Full Name: " & aliceFullName & "
   Email: " & aliceEmail & "
   Password: " & alicePassword & "

7. Click 'Sign In' (auto-setup will fail - this is expected)

8. When prompted, choose 'Manual Setup'

9. Configure IMAP (Incoming Mail):
   Account Type: IMAP
   Mail Server: localhost
   Port: 8143  
   Use SSL: NO (uncheck)
   Username: " & aliceEmail & "
   Password: " & alicePassword & "

10. Configure SMTP (Outgoing Mail):
    SMTP Server: localhost
    Port: 8587
    Use SSL: NO (uncheck)  
    Username: " & aliceEmail & "
    Password: " & alicePassword & "

11. ✅ IMPORTANT: Accept self-signed certificate when prompted

🧪 TESTING AFTER SETUP:
- Send email to: bob@2kuos0orl2tu40st5oiasb6dip9ojefv9ob072khncvi7gooahd0.fastn
- Check delivery in Bob's INBOX (should arrive in 5-10 seconds)
- Use automation scripts for advanced testing

📊 MONITORING:
- Server logs: tail -f ~/fastn-email/*_server.log
- Status check: osascript check-mail-status.scpt
- Automated tests: osascript automated-mail-test.scpt"
    
    display dialog setupInstructions buttons {"Copy Instructions", "Open Mail", "Done"} default button 2
    
    if button returned of result = "Copy Instructions" then
        set the clipboard to setupInstructions
        display notification "Setup instructions copied to clipboard" with title "fastn Mail Setup"
    else if button returned of result = "Open Mail" then
        tell application "Mail"
            activate
        end tell
        
        -- Show follow-up dialog
        display dialog "📧 Apple Mail opened!

Follow the setup instructions to add Alice's account.

After setup, test with:
📤 Send email to Bob: bob@2kuos0orl2tu40st5oiasb6dip9ojefv9ob072khncvi7gooahd0.fastn
🔍 Check status: osascript check-mail-status.scpt  
🧪 Run full test: osascript automated-mail-test.scpt" buttons {"OK"} default button 1
    end if
    
end run