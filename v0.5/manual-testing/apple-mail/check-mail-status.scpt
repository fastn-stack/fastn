#!/usr/bin/osascript
-- Apple Script to check mail status across accounts and folders

on run
    tell application "Mail"
        activate
        
        -- Get all accounts
        set accountsList to every account
        set accountCount to count of accountsList
        
        -- Initialize report
        set statusReport to "📧 FASTN MAIL STATUS REPORT" & return & return
        set statusReport to statusReport & "Generated: " & (current date) & return & return
        
        if accountCount = 0 then
            set statusReport to statusReport & "❌ No email accounts configured in Apple Mail" & return
            display dialog statusReport buttons {"OK"} default button 1
            return
        end if
        
        set statusReport to statusReport & "📊 Found " & accountCount & " email account(s)" & return & return
        
        -- Check each account
        repeat with currentAccount in accountsList
            try
                set accountName to name of currentAccount
                set accountType to account type of currentAccount  
                
                set statusReport to statusReport & "📮 Account: " & accountName & " (" & accountType & ")" & return
                
                -- Check if this looks like a fastn account  
                if accountName contains ".fastn" then
                    set statusReport to statusReport & "   ✅ fastn account detected" & return
                else
                    set statusReport to statusReport & "   ℹ️  Non-fastn account" & return
                end if
                
                -- Get mailboxes for this account
                set mailboxList to every mailbox of currentAccount
                
                repeat with currentMailbox in mailboxList
                    try
                        set mailboxName to name of currentMailbox
                        set messageCount to count of messages of currentMailbox
                        set unreadCount to count of (messages of currentMailbox whose read status is false)
                        
                        set statusReport to statusReport & "   📁 " & mailboxName & ": " & messageCount & " total, " & unreadCount & " unread" & return
                        
                        -- Show recent message info for INBOX and Sent
                        if mailboxName is "INBOX" or mailboxName is "Sent" then
                            if messageCount > 0 then
                                try
                                    set recentMessage to item 1 of (messages of currentMailbox)
                                    set messageSubject to subject of recentMessage
                                    set messageDate to date received of recentMessage
                                    set messageSender to sender of recentMessage
                                    
                                    set statusReport to statusReport & "      📩 Latest: '" & messageSubject & "' from " & messageSender & return
                                    set statusReport to statusReport & "      📅 Date: " & messageDate & return
                                on error
                                    set statusReport to statusReport & "      ⚠️  Could not read latest message details" & return
                                end try
                            end if
                        end if
                        
                    on error folderError
                        set statusReport to statusReport & "   ❌ Error reading folder " & mailboxName & ": " & folderError & return
                    end try
                end repeat
                
                set statusReport to statusReport & return
                
            on error accountError
                set statusReport to statusReport & "❌ Error reading account: " & accountError & return & return
            end try
        end repeat
        
        -- Add summary
        set statusReport to statusReport & "🔍 Summary:" & return
        set statusReport to statusReport & "- Total accounts: " & accountCount & return
        
        -- Count fastn accounts
        set fastnAccountCount to 0
        repeat with currentAccount in accountsList
            try
                if name of currentAccount contains ".fastn" then
                    set fastnAccountCount to fastnAccountCount + 1
                end if
            end try
        end repeat
        
        set statusReport to statusReport & "- fastn accounts: " & fastnAccountCount & return & return
        
        if fastnAccountCount > 0 then
            set statusReport to statusReport & "🧪 Test Commands:" & return
            set statusReport to statusReport & "- Send test email: osascript ~/fastn-email/send-test-email.scpt" & return
            set statusReport to statusReport & "- Check for new mail: osascript ~/fastn-email/check-mail-status.scpt" & return
        end if
        
        -- Display report
        display dialog statusReport buttons {"Copy to Clipboard", "OK"} default button 2
        
        if button returned of result = "Copy to Clipboard" then
            set the clipboard to statusReport
            display notification "Mail status copied to clipboard" with title "fastn Mail"
        end if
        
    end tell
end run