#!/usr/bin/osascript
-- Comprehensive Apple Mail automation for fastn P2P email testing

on run
    set aliceEmail to "alice@71es6evsls5l9l4g3ksjnsp6v8s07fldg8e2ufvu2ohjq5tthus0.fastn"
    set bobEmail to "bob@2kuos0orl2tu40st5oiasb6dip9ojefv9ob072khncvi7gooahd0.fastn"
    
    tell application "Mail"
        activate
        delay 2
        
        -- Get initial status
        set initialStatus to my getMailStatus()
        
        display dialog "🧪 FASTN P2P EMAIL AUTOMATION TEST

This will:
1. 📊 Check current mail status  
2. 📤 Send test email Alice → Bob
3. ⏳ Wait for P2P delivery
4. 📬 Force mail refresh
5. ✅ Verify delivery

Initial Status:
" & initialStatus & "

Ready to start automated test?" buttons {"Cancel", "Start Test"} default button 2
        
        if button returned of result = "Cancel" then
            return
        end if
        
        -- Find accounts
        set aliceAccount to my findAccount("71es6evsls5l9l4g3ksjnsp6v8s07fldg8e2ufvu2ohjq5tthus0")
        set bobAccount to my findAccount("2kuos0orl2tu40st5oiasb6dip9ojefv9ob072khncvi7gooahd0")
        
        if aliceAccount is null then
            display dialog "❌ Alice's account not found! Please set up accounts first." buttons {"OK"}
            return
        end if
        
        -- Step 1: Send test email
        try
            set testSubject to "🤖 Automated fastn P2P Test #" & (random number from 1000 to 9999)
            set testBody to "This is an automated test email sent via Apple Script.

Testing fastn P2P email delivery with secure .fastn addresses.

Timestamp: " & (current date) & "
Test ID: " & (random number from 100000 to 999999) & "

If you receive this, the automation is working! 🎉"
            
            set newMessage to make new outgoing message with properties {sender:aliceAccount, subject:testSubject, content:testBody}
            tell newMessage
                make new to recipient at end of to recipients with properties {address:bobEmail}
            end tell
            send newMessage
            
            display notification "Test email sent: " & testSubject with title "fastn Test"
            
        on error sendError
            display dialog "❌ Failed to send test email: " & sendError buttons {"OK"}
            return
        end try
        
        -- Step 2: Wait for P2P delivery
        display dialog "📤 Test email sent to Bob!

⏳ Waiting 10 seconds for P2P delivery...
(fastn typically delivers in 5-10 seconds)" buttons {"Continue"} default button 1 giving up after 10
        
        -- Step 3: Force refresh
        check for new mail
        delay 5
        
        -- Step 4: Check results
        set finalStatus to my getMailStatus()
        
        set testResults to "🧪 AUTOMATED TEST RESULTS" & return & return
        set testResults to testResults & "📤 Sent: '" & testSubject & "'" & return
        set testResults to testResults & "📧 From: " & aliceEmail & return  
        set testResults to testResults & "📧 To: " & bobEmail & return & return
        
        set testResults to testResults & "📊 FINAL STATUS:" & return
        set testResults to testResults & finalStatus & return
        
        -- Analyze results
        if bobAccount is not null then
            try
                set bobInbox to mailbox "INBOX" of bobAccount
                set inboxCount to count of messages of bobInbox
                
                if inboxCount > 0 then
                    set latestMessage to item 1 of (messages of bobInbox)
                    set latestSubject to subject of latestMessage
                    
                    if latestSubject contains testSubject then
                        set testResults to testResults & "✅ SUCCESS: Test email found in Bob's INBOX!" & return
                        set testResults to testResults & "🎉 fastn P2P email delivery working!" & return
                    else
                        set testResults to testResults & "⚠️  Bob has mail but not our test email" & return
                        set testResults to testResults & "🔍 Latest: '" & latestSubject & "'" & return
                    end if
                else
                    set testResults to testResults & "❌ No emails in Bob's INBOX" & return
                    set testResults to testResults & "🔍 Check server logs for delivery issues" & return
                end if
            on error
                set testResults to testResults & "❌ Could not check Bob's INBOX" & return
            end try
        else
            set testResults to testResults & "⚠️  Bob's account not configured - cannot verify delivery" & return
        end if
        
        display dialog testResults buttons {"Run Another Test", "View Logs", "Done"} default button 3
        
        if button returned of result = "Run Another Test" then
            -- Run the script again
            run script (load script (path to me))
        else if button returned of result = "View Logs" then
            do shell script "open ~/fastn-email/"
        end if
        
    end tell
end run

-- Helper function to get mail status
on getMailStatus()
    tell application "Mail"
        set statusText to ""
        
        repeat with currentAccount in (every account)
            try
                if name of currentAccount contains ".fastn" then
                    set accountName to name of currentAccount
                    set statusText to statusText & "📮 " & accountName & return
                    
                    set inboxCount to count of messages of mailbox "INBOX" of currentAccount
                    set sentCount to count of messages of mailbox "Sent" of currentAccount
                    set unreadCount to count of (messages of mailbox "INBOX" of currentAccount whose read status is false)
                    
                    set statusText to statusText & "   📥 INBOX: " & inboxCount & " (" & unreadCount & " unread)" & return
                    set statusText to statusText & "   📤 Sent: " & sentCount & return & return
                end if
            end try
        end repeat
        
        return statusText
    end tell
end getMailStatus

-- Helper function to find account by ID
on findAccount(accountId)
    tell application "Mail"
        repeat with currentAccount in (every account)
            try
                if name of currentAccount contains accountId then
                    return currentAccount
                end if
            end try
        end repeat
        return null
    end tell
end findAccount