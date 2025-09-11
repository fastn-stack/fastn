#!/usr/bin/osascript
-- Apple Script to refresh mailboxes and check for new messages

on run
    tell application "Mail"
        activate
        delay 1
        
        -- Force mail check for all accounts
        check for new mail
        delay 3
        
        -- Wait for refresh to complete
        display dialog "📬 Refreshing all mailboxes...

This will force Apple Mail to:
1. Check for new emails via IMAP
2. Sync folder contents
3. Update message counts

Wait 5-10 seconds then check results..." buttons {"Check Status", "Wait Longer"} default button 1
        
        if button returned of result = "Wait Longer" then
            delay 10
        end if
        
        -- Get updated status
        set statusReport to "📧 MAIL REFRESH RESULTS" & return & return
        
        repeat with currentAccount in (every account)
            try
                if name of currentAccount contains ".fastn" then
                    set accountName to name of currentAccount
                    set statusReport to statusReport & "📮 " & accountName & ":" & return
                    
                    -- Check key folders
                    repeat with folderName in {"INBOX", "Sent", "Drafts", "Trash"}
                        try
                            set targetMailbox to mailbox folderName of currentAccount
                            set messageCount to count of messages of targetMailbox
                            set unreadCount to count of (messages of targetMailbox whose read status is false)
                            
                            set statusReport to statusReport & "   📁 " & folderName & ": " & messageCount & " messages (" & unreadCount & " unread)" & return
                            
                            -- Show newest message in INBOX
                            if folderName = "INBOX" and messageCount > 0 then
                                try
                                    set newestMessage to item 1 of (messages of targetMailbox)
                                    set messageSubject to subject of newestMessage
                                    set messageFrom to sender of newestMessage
                                    set messageDate to date received of newestMessage
                                    
                                    set statusReport to statusReport & "      🆕 Newest: '" & messageSubject & "'" & return
                                    set statusReport to statusReport & "      👤 From: " & messageFrom & return
                                    set statusReport to statusReport & "      📅 " & messageDate & return
                                end try
                            end if
                            
                        on error
                            set statusReport to statusReport & "   ❌ Could not access " & folderName & " folder" & return
                        end try
                    end repeat
                    
                    set statusReport to statusReport & return
                end if
            end try
        end repeat
        
        -- Add testing instructions
        set statusReport to statusReport & "🧪 TESTING:" & return
        set statusReport to statusReport & "1. Send test: osascript ~/fastn-email/send-test-email.scpt" & return
        set statusReport to statusReport & "2. Check delivery: Wait 5-10 seconds, then run this script again" & return
        set statusReport to statusReport & "3. Monitor logs: tail -f ~/fastn-email/*_server.log" & return
        
        display dialog statusReport buttons {"Copy Report", "Send Test Email", "OK"} default button 3
        
        if button returned of result = "Copy Report" then
            set the clipboard to statusReport
            display notification "Mail status copied to clipboard" with title "fastn Mail"
        else if button returned of result = "Send Test Email" then
            do shell script "osascript ~/fastn-email/send-test-email.scpt > /dev/null 2>&1 &"
        end if
        
    end tell
end run