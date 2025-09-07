//! End-to-end integration test for SMTP to P2P email delivery
//!
//! This test verifies the complete flow exactly as done manually:
//! 1. fastn-rig init for both peers
//! 2. fastn-rig run to start both peers  
//! 3. SMTP client sends email to peer 1
//! 4. Verify email stored in peer 1's Sent folder
//! 5. Wait for P2P delivery to peer 2
//! 6. Verify email delivered to peer 2's INBOX folder

use std::path::PathBuf;


// Re-enabled after CPU spinning bug fix and fastn-cli-test-utils improvements
#[tokio::test]
async fn test_p2p_email_goes_to_inbox() {
    println!("🚀 Starting comprehensive SMTP→P2P→INBOX integration test");

    // Use fastn-cli-test-utils for better test management
    let mut test_env = fastn_cli_test_utils::FastnTestEnv::new("p2p-inbox-delivery")
        .expect("Failed to create test environment");
    
    // Create two peers using the correct API
    let peer1_ref = test_env.create_peer("peer1").await.expect("Failed to create peer1");
    let account1_id = peer1_ref.account_id.clone();
    let peer1_home = peer1_ref.home_path.clone();
    
    let peer2_ref = test_env.create_peer("peer2").await.expect("Failed to create peer2");
    let account2_id = peer2_ref.account_id.clone();
    let peer2_home = peer2_ref.home_path.clone();

    // Start both peers
    test_env.start_peer("peer1").await.expect("Failed to start peer1");
    test_env.start_peer("peer2").await.expect("Failed to start peer2");

    // Give peers time to start
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    println!("🔍 Validated account credentials:");
    println!("✅ Peer 1: {} (length: {})", account1_id, account1_id.len());
    println!("✅ Peer 2: {} (length: {})", account2_id, account2_id.len());

    // Verify ID52 format
    assert_eq!(account1_id.len(), 52, "Account1 ID52 should be 52 characters");
    assert_eq!(account2_id.len(), 52, "Account2 ID52 should be 52 characters");

    println!("✅ Both peers started with valid account IDs");

    // Send email using fastn-cli-test-utils email builder  
    println!("📧 Sending email via SMTP...");
    let _send_result = test_env.email()
        .from("peer1")
        .to("peer2") 
        .subject("Integration Test Email")
        .body("End-to-end SMTP to P2P to INBOX test")
        .send()
        .await
        .expect("SMTP email send should succeed");

    println!("✅ Email sent via SMTP");

    // Wait for P2P delivery using fastn-cli-test-utils timing
    println!("⏳ Waiting for P2P delivery...");
    
    for attempt in 1..=10 {
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        println!("⏳ P2P delivery check #{}/10 ({}s elapsed)", attempt, attempt * 2);

        // Check peer 1's Sent folder  
        let peer1_sent_emails = find_emails_in_folder(&peer1_home, &account1_id, "Sent").await;
        println!("📊 Peer 1 Sent: {} emails", peer1_sent_emails.len());

        // Check peer 2's INBOX folder
        let peer2_inbox_emails = find_emails_in_folder(&peer2_home, &account2_id, "INBOX").await;
        println!("📊 Peer 2 INBOX: {} emails", peer2_inbox_emails.len());

        if !peer2_inbox_emails.is_empty() {
            println!("✅ P2P delivery successful on attempt {} ({}s)", attempt, attempt * 2);
            break;
        }

        if attempt == 5 {
            println!("🔍 10s mark: P2P delivery still in progress...");
        }
        if attempt == 8 {
            println!("🐛 16s mark: P2P delivery should have completed by now");
        }
    }

    // Final verification
    let peer1_sent_emails = find_emails_in_folder(&peer1_home, &account1_id, "Sent").await;
    assert!(!peer1_sent_emails.is_empty(), "Email should be in peer 1's Sent folder");
    println!("✅ Found {} emails in peer 1 Sent folder", peer1_sent_emails.len());

    let peer2_inbox_emails = find_emails_in_folder(&peer2_home, &account2_id, "INBOX").await;
    if peer2_inbox_emails.is_empty() {
        println!("🐛 Debug: No emails found in peer 2 INBOX after 20 seconds");
        println!("🐛 Debug: Peer 1 account: {}", account1_id);  
        println!("🐛 Debug: Peer 2 account: {}", account2_id);
        panic!("Email should be delivered to peer 2's INBOX");
    }

    println!("✅ Found {} emails in peer 2 INBOX folder", peer2_inbox_emails.len());

    // Verify email content matches
    let sent_content = tokio::fs::read_to_string(&peer1_sent_emails[0])
        .await
        .expect("Failed to read sent email");
    let inbox_content = tokio::fs::read_to_string(&peer2_inbox_emails[0])
        .await
        .expect("Failed to read inbox email");

    assert!(sent_content.contains("Integration Test Email"));
    assert!(inbox_content.contains("Integration Test Email"));
    assert!(sent_content.contains("End-to-end SMTP to P2P to INBOX test"));
    assert!(inbox_content.contains("End-to-end SMTP to P2P to INBOX test"));
    println!("✅ Email content verified in both folders");

    // Verify folder placement is correct
    assert!(peer1_sent_emails[0].to_string_lossy().contains("/Sent/"));
    assert!(peer2_inbox_emails[0].to_string_lossy().contains("/INBOX/"));
    println!("✅ Email folder placement verified: Sent -> INBOX");

    println!("🎉 Complete end-to-end SMTP to P2P to INBOX test passed!");
    
    // Note: FastnTestEnv handles automatic cleanup
}


/// Find .eml files in a specific mail folder
async fn find_emails_in_folder(
    peer_home: &std::path::Path,
    account_id: &str,
    folder: &str,
) -> Vec<PathBuf> {
    let folder_path = peer_home
        .join("accounts")
        .join(account_id)
        .join("mails")
        .join("default")
        .join(folder);

    let mut emails = Vec::new();
    for entry in walkdir::WalkDir::new(folder_path) {
        if let Ok(entry) = entry
            && entry.path().extension().and_then(|s| s.to_str()) == Some("eml")
        {
            emails.push(entry.path().to_path_buf());
        }
    }

    // Sort by modification time (most recent first)
    emails.sort_by(|a, b| {
        let a_modified = std::fs::metadata(a)
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        let b_modified = std::fs::metadata(b)
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        b_modified.cmp(&a_modified)
    });

    emails
}

/// Simple test of inbox_receive vs smtp_receive methods
#[tokio::test]
async fn test_p2p_inbox_delivery() {
    // Test the core storage methods work correctly
    let store = fastn_mail::Store::create_test();

    // Generate proper ID52s for testing
    let sender_key = fastn_id52::SecretKey::generate();
    let recipient_key = fastn_id52::SecretKey::generate();
    let sender_id52 = sender_key.public_key().id52();
    let recipient_id52 = recipient_key.public_key().id52();

    let from_email = format!("sender@{}.com", sender_id52);
    let to_email = format!("recipient@{}.com", recipient_id52);

    // Test inbox_receive method stores successfully
    let inbox_email = format!(
        "From: {}\r\nTo: {}\r\nSubject: P2P Test\r\nMessage-ID: <test1@localhost>\r\n\r\nTest inbox_receive method",
        from_email, to_email
    );

    let inbox_email_id = store
        .inbox_receive(
            &from_email,
            std::slice::from_ref(&to_email),
            inbox_email.as_bytes().to_vec(),
        )
        .await
        .expect("inbox_receive should succeed");

    assert!(
        inbox_email_id.starts_with("email-"),
        "Email ID should have proper format"
    );
    println!("✅ inbox_receive method works: {}", inbox_email_id);

    // Test smtp_receive method stores successfully
    let smtp_email = format!(
        "From: {}\r\nTo: {}\r\nSubject: SMTP Test\r\nMessage-ID: <test2@localhost>\r\n\r\nTest smtp_receive method",
        from_email, to_email
    );

    let smtp_email_id = store
        .smtp_receive(&from_email, &[to_email], smtp_email.as_bytes().to_vec())
        .await
        .expect("smtp_receive should succeed");

    assert!(
        smtp_email_id.starts_with("email-"),
        "Email ID should have proper format"
    );
    println!("✅ smtp_receive method works: {}", smtp_email_id);

    println!("🎉 Both storage methods functional!");
}

#[tokio::test]
async fn test_email_delivery_response_format() {
    // Test JSON response format used in P2P delivery
    let response = fastn_account::EmailDeliveryResponse {
        email_id: "test-email-123".to_string(),
        status: fastn_account::DeliveryStatus::Accepted,
    };

    let json = serde_json::to_string(&response).expect("Response should serialize");
    let parsed: fastn_account::EmailDeliveryResponse =
        serde_json::from_str(&json).expect("Response should deserialize");

    assert_eq!(parsed.email_id, "test-email-123");
    assert!(matches!(
        parsed.status,
        fastn_account::DeliveryStatus::Accepted
    ));
    println!("✅ EmailDeliveryResponse JSON format verified");

    // Test rejection response
    let rejection = fastn_account::EmailDeliveryResponse {
        email_id: "failed-email-456".to_string(),
        status: fastn_account::DeliveryStatus::Rejected {
            reason: "Storage failed".to_string(),
        },
    };

    let json = serde_json::to_string(&rejection).expect("Rejection should serialize");
    let parsed: fastn_account::EmailDeliveryResponse =
        serde_json::from_str(&json).expect("Rejection should deserialize");

    assert_eq!(parsed.email_id, "failed-email-456");
    if let fastn_account::DeliveryStatus::Rejected { reason } = parsed.status {
        assert_eq!(reason, "Storage failed");
    } else {
        panic!("Expected Rejected status");
    }
    println!("✅ Rejection response format verified");
}

/// Integration test that calls the working bash script
#[test]
fn bash_integration_test() {
    println!("🧪 SMTP to P2P to INBOX integration test via bash script");

    // Find the script in the tests directory (relative to fastn-rig root)
    let script_path = "tests/test_complete_integration.sh";
    if !std::path::Path::new(script_path).exists() {
        panic!(
            "Integration test script not found at: {}\nCurrent dir: {:?}",
            script_path,
            std::env::current_dir().unwrap()
        );
    }

    let output = std::process::Command::new("bash")
        .arg(script_path)
        .output()
        .expect("Failed to execute bash script");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !stderr.trim().is_empty() {
        println!("Script stderr: {}", stderr.trim());
    }

    if output.status.success() {
        println!("✅ Integration test PASSED");
        if stdout.contains("COMPLETE SUCCESS") {
            println!("✅ SMTP→P2P→INBOX delivery working");
        }
    } else {
        println!("❌ Integration test FAILED");
        println!("Last 10 lines of output:");
        for line in stdout
            .lines()
            .rev()
            .take(10)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
        {
            println!("  {}", line);
        }
        panic!("Integration test failed - check ./test_complete_integration.sh");
    }
}
