#!/usr/bin/osascript
-- Apple Script to send test emails between fastn accounts

on run
    -- Account details
    set aliceEmail to "alice@71es6evsls5l9l4g3ksjnsp6v8s07fldg8e2ufvu2ohjq5tthus0.fastn"
    set bobEmail to "bob@2kuos0orl2tu40st5oiasb6dip9ojefv9ob072khncvi7gooahd0.fastn"
    
    tell application "Mail"
        activate
        delay 1
        
        -- Get fastn accounts
        set aliceAccount to null
        set bobAccount to null
        
        repeat with currentAccount in (every account)
            try
                if name of currentAccount contains "71es6evsls5l9l4g3ksjnsp6v8s07fldg8e2ufvu2ohjq5tthus0" then
                    set aliceAccount to currentAccount
                end if
                if name of currentAccount contains "2kuos0orl2tu40st5oiasb6dip9ojefv9ob072khncvi7gooahd0" then
                    set bobAccount to currentAccount
                end if
            end try
        end repeat
        
        -- Check if accounts are set up
        if aliceAccount is null and bobAccount is null then
            display dialog "❌ No fastn accounts found in Apple Mail!

Please run the setup scripts first:
1. osascript ~/fastn-email/setup-apple-mail.scpt
2. osascript ~/fastn-email/setup-apple-mail-bob.scpt

Then try this script again." buttons {"OK"} default button 1
            return
        end if
        
        -- Choose sending direction
        set dialogResult to display dialog "📧 fastn P2P Email Test

Choose test direction:" buttons {"Alice → Bob", "Bob → Alice", "Cancel"} default button 1 cancel button "Cancel"
        
        set buttonChoice to button returned of dialogResult
        
        if buttonChoice = "Alice → Bob" then
            if aliceAccount is null then
                display dialog "❌ Alice's account not found! Please set up Alice's account first."
                return
            end if
            set fromAccount to aliceAccount
            set fromEmail to aliceEmail
            set toEmail to bobEmail
            set testDirection to "Alice → Bob"
        else if buttonChoice = "Bob → Alice" then
            if bobAccount is null then
                display dialog "❌ Bob's account not found! Please set up Bob's account first."
                return
            end if
            set fromAccount to bobAccount
            set fromEmail to bobEmail
            set toEmail to aliceEmail
            set testDirection to "Bob → Alice"
        else
            return
        end if
        
        -- Create and send test email
        try
            set timeStamp to (current date) as string
            set testSubject to "🧪 fastn P2P Test: " & testDirection & " at " & timeStamp
            set testBody to "This is a test email sent via fastn P2P network.

From: " & fromEmail & "
To: " & toEmail & "
Direction: " & testDirection & "
Timestamp: " & timeStamp & "
Security: Using secure .fastn addresses

If you receive this email, the fastn P2P email system is working correctly!

🚀 fastn v0.5 - Privacy-first P2P email"
            
            -- Create new message
            set newMessage to make new outgoing message with properties {sender:fromAccount, subject:testSubject, content:testBody}
            
            -- Set recipient
            tell newMessage
                make new to recipient at end of to recipients with properties {address:toEmail}
            end tell
            
            -- Send the message
            send newMessage
            
            display dialog "✅ Test email sent successfully!

Direction: " & testDirection & "
From: " & fromEmail & "
To: " & toEmail & "
Subject: " & testSubject & "

⏳ Check recipient's INBOX in 5-10 seconds for P2P delivery.
📊 Monitor server logs: tail -f ~/fastn-email/*_server.log" buttons {"OK"} default button 1
            
        on error sendError
            display dialog "❌ Failed to send test email: " & sendError & "

Troubleshooting:
1. Check servers are running: ps aux | grep fastn-rig
2. Check account configuration in Mail settings
3. Check server logs for errors" buttons {"OK"} default button 1
        end try
    end tell
end run