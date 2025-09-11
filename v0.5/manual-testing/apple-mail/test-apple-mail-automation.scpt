#!/usr/bin/osascript
-- Advanced Apple Mail automation for fastn testing with detailed reporting

on run
    display dialog "🍎 ADVANCED FASTN APPLE MAIL AUTOMATION

This script will:
🔍 1. Analyze current Apple Mail setup
📤 2. Send bidirectional test emails  
⏱️ 3. Monitor delivery timing
✅ 4. Verify P2P functionality
📊 5. Generate detailed report

Ready for comprehensive testing?" buttons {"Cancel", "Start Advanced Test"} default button 2
    
    if button returned of result = "Cancel" then
        return
    end if
    
    tell application "Mail"
        activate
        delay 2
        
        -- Phase 1: Environment Analysis
        set analysisReport to my analyzeEnvironment()
        
        display dialog "📊 ENVIRONMENT ANALYSIS:
" & analysisReport & "

Continue with automated testing?" buttons {"Cancel", "Continue"} default button 2
        
        if button returned of result = "Cancel" then
            return
        end if
        
        -- Phase 2: Bidirectional Testing
        set testResults to my runBidirectionalTest()
        
        -- Phase 3: Final Report
        set finalReport to "🧪 COMPREHENSIVE FASTN APPLE MAIL TEST REPORT" & return & return
        set finalReport to finalReport & "📊 Environment:" & return & analysisReport & return
        set finalReport to finalReport & "🧪 Test Results:" & return & testResults & return
        set finalReport to finalReport & "Generated: " & (current date) & return
        
        display dialog finalReport buttons {"Copy Full Report", "Save to File", "Done"} default button 3
        
        if button returned of result = "Copy Full Report" then
            set the clipboard to finalReport
            display notification "Complete test report copied to clipboard" with title "fastn Mail Test"
        else if button returned of result = "Save to File" then
            try
                do shell script "echo " & quoted form of finalReport & " > ~/fastn-email/apple-mail-test-report.txt"
                display notification "Test report saved to ~/fastn-email/apple-mail-test-report.txt" with title "fastn Mail Test"
            end try
        end if
        
    end tell
end run

-- Analyze current environment setup
on analyzeEnvironment()
    tell application "Mail"
        set analysisText to ""
        set fastnAccountCount to 0
        
        repeat with currentAccount in (every account)
            try
                set accountName to name of currentAccount
                if accountName contains ".fastn" then
                    set fastnAccountCount to fastnAccountCount + 1
                    set analysisText to analysisText & "✅ fastn Account " & fastnAccountCount & ": " & return
                    set analysisText to analysisText & "   📧 " & accountName & return
                    
                    -- Check connectivity  
                    try
                        set testMailbox to mailbox "INBOX" of currentAccount
                        set analysisText to analysisText & "   🔗 IMAP: Connected" & return
                    on error
                        set analysisText to analysisText & "   ❌ IMAP: Connection failed" & return
                    end try
                    
                    set analysisText to analysisText & return
                end if
            end try
        end repeat
        
        if fastnAccountCount = 0 then
            set analysisText to "❌ No fastn accounts found" & return
        else
            set analysisText to "✅ Found " & fastnAccountCount & " fastn accounts" & return & analysisText
        end if
        
        return analysisText
    end tell
end analyzeEnvironment

-- Run comprehensive bidirectional email test
on runBidirectionalTest()
    tell application "Mail"
        set testResults to ""
        
        -- Find accounts
        set aliceAccount to my findAccount("71es6evsls5l9l4g3ksjnsp6v8s07fldg8e2ufvu2ohjq5tthus0")
        set bobAccount to my findAccount("2kuos0orl2tu40st5oiasb6dip9ojefv9ob072khncvi7gooahd0")
        
        if aliceAccount is null or bobAccount is null then
            return "❌ Could not find both fastn accounts for testing"
        end if
        
        set testId to (random number from 1000 to 9999)
        set testResults to testResults & "🧪 Bidirectional Test #" & testId & return & return
        
        -- Test 1: Alice → Bob
        try
            set aliceSubject to "🤖 A→B Test #" & testId
            set aliceBody to "Automated Alice to Bob test. ID: " & testId
            
            set aliceMessage to make new outgoing message with properties {sender:aliceAccount, subject:aliceSubject, content:aliceBody}
            tell aliceMessage
                make new to recipient at end of to recipients with properties {address:"bob@2kuos0orl2tu40st5oiasb6dip9ojefv9ob072khncvi7gooahd0.fastn"}
            end tell
            send aliceMessage
            
            set testResults to testResults & "✅ Alice→Bob: Email sent" & return
            
        on error aliceError
            set testResults to testResults & "❌ Alice→Bob: Failed - " & aliceError & return
        end try
        
        -- Wait for delivery
        delay 8
        
        -- Test 2: Bob → Alice  
        try
            set bobSubject to "🤖 B→A Test #" & testId
            set bobBody to "Automated Bob to Alice test. ID: " & testId
            
            set bobMessage to make new outgoing message with properties {sender:bobAccount, subject:bobSubject, content:bobBody}
            tell bobMessage
                make new to recipient at end of to recipients with properties {address:"alice@71es6evsls5l9l4g3ksjnsp6v8s07fldg8e2ufvu2ohjq5tthus0.fastn"}
            end tell
            send bobMessage
            
            set testResults to testResults & "✅ Bob→Alice: Email sent" & return
            
        on error bobError
            set testResults to testResults & "❌ Bob→Alice: Failed - " & bobError & return
        end try
        
        -- Wait and refresh
        delay 10
        check for new mail
        delay 5
        
        -- Check delivery results
        set testResults to testResults & return & "📬 Delivery Check:" & return
        
        -- Check Alice's INBOX for Bob's email
        try
            set aliceInbox to mailbox "INBOX" of aliceAccount
            set aliceMessages to messages of aliceInbox
            set foundBobMessage to false
            
            repeat with currentMessage in aliceMessages
                try
                    if subject of currentMessage contains ("B→A Test #" & testId) then
                        set foundBobMessage to true
                        exit repeat
                    end if
                end try
            end repeat
            
            if foundBobMessage then
                set testResults to testResults & "✅ Bob→Alice: Delivered to Alice's INBOX" & return
            else
                set testResults to testResults & "❌ Bob→Alice: Not found in Alice's INBOX" & return
            end if
        on error
            set testResults to testResults & "❌ Could not check Alice's INBOX" & return
        end try
        
        -- Check Bob's INBOX for Alice's email
        if bobAccount is not null then
            try
                set bobInbox to mailbox "INBOX" of bobAccount
                set bobMessages to messages of bobInbox  
                set foundAliceMessage to false
                
                repeat with currentMessage in bobMessages
                    try
                        if subject of currentMessage contains ("A→B Test #" & testId) then
                            set foundAliceMessage to true
                            exit repeat
                        end if
                    end try
                end repeat
                
                if foundAliceMessage then
                    set testResults to testResults & "✅ Alice→Bob: Delivered to Bob's INBOX" & return
                else
                    set testResults to testResults & "❌ Alice→Bob: Not found in Bob's INBOX" & return
                end if
            on error
                set testResults to testResults & "❌ Could not check Bob's INBOX" & return
            end try
        end if
        
        return testResults
        
    end tell
end runBidirectionalTest

-- Helper functions
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

on getMailStatus()
    tell application "Mail"
        set statusText to ""
        
        repeat with currentAccount in (every account)
            try
                if name of currentAccount contains ".fastn" then
                    set accountName to name of currentAccount
                    set inboxCount to count of messages of mailbox "INBOX" of currentAccount
                    set sentCount to count of messages of mailbox "Sent" of currentAccount
                    set unreadCount to count of (messages of mailbox "INBOX" of currentAccount whose read status is false)
                    
                    set statusText to statusText & "📧 " & accountName & ": INBOX=" & inboxCount & "(" & unreadCount & " unread), Sent=" & sentCount & return
                end if
            end try
        end repeat
        
        return statusText
    end tell
end getMailStatus