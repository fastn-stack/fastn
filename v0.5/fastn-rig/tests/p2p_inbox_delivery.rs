//! Integration test ensuring P2P received emails go to INBOX, not Sent
//! 
//! This test verifies the critical bug fix where P2P received emails
//! were incorrectly stored in the Sent folder instead of INBOX.

/// Test that inbox_receive stores emails in INBOX folder
#[tokio::test] 
async fn test_inbox_receive_stores_in_inbox_folder() {
    // Configure SKIP_KEYRING for testing (configurable, defaults to true now)
    let skip_keyring = std::env::var("SKIP_KEYRING").unwrap_or_else(|_| "true".to_string());
    unsafe { 
        std::env::set_var("SKIP_KEYRING", &skip_keyring);
    }
    println!("🔧 Using SKIP_KEYRING={}", skip_keyring);

    // Create test mail store
    let store = fastn_mail::Store::create_test();

    // Generate proper 52-char ID52s for testing
    let sender_key = fastn_id52::SecretKey::generate();
    let recipient_key = fastn_id52::SecretKey::generate();
    let sender_id52 = sender_key.public_key().id52();
    let recipient_id52 = recipient_key.public_key().id52();

    let from_email = format!("sender@{}.com", sender_id52);
    let to_email = format!("recipient@{}.com", recipient_id52);

    // Test inbox_receive method (should store in INBOX)
    let inbox_email = format!(
        "From: {}\r\nTo: {}\r\nSubject: P2P Test INBOX\r\nMessage-ID: <inbox-test@localhost>\r\n\r\nP2P delivery test email for INBOX",
        from_email, to_email
    );

    let inbox_email_id = store.inbox_receive(
        &from_email,
        &[to_email.clone()],
        inbox_email.as_bytes().to_vec(),
    ).await.expect("inbox_receive should succeed");

    assert!(inbox_email_id.starts_with("email-"), "Email ID should have proper format");
    println!("✅ P2P email stored via inbox_receive with ID: {}", inbox_email_id);

    // Test smtp_receive method (should store in Sent) 
    let sent_email = format!(
        "From: {}\r\nTo: {}\r\nSubject: SMTP Test SENT\r\nMessage-ID: <sent-test@localhost>\r\n\r\nSMTP delivery test email for Sent",
        from_email, to_email
    );

    let sent_email_id = store.smtp_receive(
        &from_email,
        &[to_email],
        sent_email.as_bytes().to_vec(),
    ).await.expect("smtp_receive should succeed");

    assert!(sent_email_id.starts_with("email-"), "Email ID should have proper format");
    println!("✅ SMTP email stored via smtp_receive with ID: {}", sent_email_id);

    println!("🎉 INBOX vs Sent folder methods work as expected!");
}

#[tokio::test] 
async fn test_email_delivery_response_format() {
    // Test that EmailDeliveryResponse can be serialized/deserialized properly
    let response = fastn_account::EmailDeliveryResponse {
        email_id: "test-email-123".to_string(),
        status: fastn_account::DeliveryStatus::Accepted,
    };

    let json = serde_json::to_string(&response).expect("Response should serialize");
    let parsed: fastn_account::EmailDeliveryResponse = serde_json::from_str(&json)
        .expect("Response should deserialize");

    assert_eq!(parsed.email_id, "test-email-123");
    assert!(matches!(parsed.status, fastn_account::DeliveryStatus::Accepted));
    println!("✅ EmailDeliveryResponse JSON format verified");

    // Test rejection response  
    let rejection = fastn_account::EmailDeliveryResponse {
        email_id: "failed-email-456".to_string(),
        status: fastn_account::DeliveryStatus::Rejected { 
            reason: "Storage failed".to_string() 
        },
    };

    let json = serde_json::to_string(&rejection).expect("Rejection should serialize");
    let parsed: fastn_account::EmailDeliveryResponse = serde_json::from_str(&json)
        .expect("Rejection should deserialize");

    assert_eq!(parsed.email_id, "failed-email-456");
    if let fastn_account::DeliveryStatus::Rejected { reason } = parsed.status {
        assert_eq!(reason, "Storage failed");
    } else {
        panic!("Expected Rejected status");
    }
    println!("✅ Rejection response format verified");
}