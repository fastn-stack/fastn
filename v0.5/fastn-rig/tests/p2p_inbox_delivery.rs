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
use std::time::Duration;
use tempfile::TempDir;
use tokio::process::Command;

/// RAII guard to ensure background processes are killed even on test failure/panic
struct ProcessCleanup<'a> {
    processes: Vec<&'a mut tokio::process::Child>,
}

impl<'a> ProcessCleanup<'a> {
    fn new(peer1: &'a mut tokio::process::Child, peer2: &'a mut tokio::process::Child) -> Self {
        Self {
            processes: vec![peer1, peer2],
        }
    }
}

impl<'a> Drop for ProcessCleanup<'a> {
    fn drop(&mut self) {
        for process in &mut self.processes {
            let _ = process.start_kill();
        }
        println!("🧹 Process cleanup completed");
    }
}

/// Helper for running fastn-rig commands
struct FastnRigHelper {
    skip_keyring: String,
}

impl FastnRigHelper {
    fn new() -> Self {
        Self {
            skip_keyring: std::env::var("SKIP_KEYRING").unwrap_or_else(|_| "true".to_string()),
        }
    }

    /// Run fastn-rig init
    async fn init(
        &self,
        fastn_home: &PathBuf,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let output = Command::new("cargo")
            .args(["run", "--bin", "fastn-rig", "--", "init"])
            .env("SKIP_KEYRING", &self.skip_keyring)
            .env("FASTN_HOME", fastn_home)
            .output()
            .await?;

        if !output.status.success() {
            return Err(format!(
                "fastn-rig init failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Start fastn-rig run process
    async fn start_run(
        &self,
        fastn_home: &PathBuf,
        smtp_port: u16,
    ) -> Result<tokio::process::Child, Box<dyn std::error::Error + Send + Sync>> {
        let process = Command::new("cargo")
            .args(["run", "--bin", "fastn-rig", "--", "run"])
            .env("SKIP_KEYRING", &self.skip_keyring)
            .env("FASTN_HOME", fastn_home)
            .env("FASTN_SMTP_PORT", smtp_port.to_string())
            .spawn()?;

        Ok(process)
    }

    /// Send email via SMTP (uses hardcoded default@ username for now)
    #[expect(clippy::too_many_arguments)]
    async fn send_email_smtp(
        &self,
        fastn_home: &PathBuf,
        smtp_port: u16,
        password: &str,
        from: &str,
        to: &str,
        subject: &str,
        body: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let output = Command::new("cargo")
            .args([
                "run",
                "--package",
                "fastn-mail",
                "--features",
                "net",
                "--bin",
                "fastn-mail",
                "--",
                "send-mail",
                "--smtp",
                &smtp_port.to_string(),
                "--password",
                password,
                "--from",
                from,
                "--to",
                to,
                "--subject",
                subject,
                "--body",
                body,
            ])
            .env("FASTN_HOME", fastn_home)
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("SMTP send failed: {}", stderr).into());
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

#[tokio::test]
async fn test_p2p_email_goes_to_inbox() {
    let helper = FastnRigHelper::new();
    println!("🔧 Using SKIP_KEYRING={}", helper.skip_keyring);

    // Create temporary directories for both peers
    let test_dir = TempDir::new().expect("Failed to create temp dir");
    let peer1_path = test_dir.path().join("peer1");
    let peer2_path = test_dir.path().join("peer2");

    // Initialize both peers
    println!("🔧 Initializing peers...");
    let init1_output = helper
        .init(&peer1_path)
        .await
        .expect("Peer1 init should succeed");
    let init2_output = helper
        .init(&peer2_path)
        .await
        .expect("Peer2 init should succeed");

    // Extract account credentials with debugging
    let account1_id =
        extract_account_id(&init1_output).expect("Failed to extract peer1 account ID");
    let account1_password =
        extract_password(&init1_output).expect("Failed to extract peer1 password");
    let account2_id =
        extract_account_id(&init2_output).expect("Failed to extract peer2 account ID");
    let _account2_password =
        extract_password(&init2_output).expect("Failed to extract peer2 password");

    println!("✅ Peer 1: {} (length: {})", account1_id, account1_id.len());
    println!("✅ Peer 2: {} (length: {})", account2_id, account2_id.len());
    
    // Verify ID52 lengths are correct
    assert_eq!(account1_id.len(), 52, "Account1 ID52 should be 52 characters");
    assert_eq!(account2_id.len(), 52, "Account2 ID52 should be 52 characters");

    // Start both peers with cleanup guard
    println!("🚀 Starting peers...");
    let mut peer1_process = helper
        .start_run(&peer1_path, 2525)
        .await
        .expect("Failed to start peer1");
    let mut peer2_process = helper
        .start_run(&peer2_path, 2526)
        .await
        .expect("Failed to start peer2");

    // Ensure cleanup happens even on panic/failure
    let _cleanup = ProcessCleanup::new(&mut peer1_process, &mut peer2_process);

    // Wait for peers to start
    tokio::time::sleep(Duration::from_secs(5)).await;
    println!("✅ Both peers started");

    // Send email via SMTP (use format that worked in manual testing)  
    let from_email = format!("test@{}.com", account1_id);
    let to_email = format!("inbox@{}.com", account2_id);

    println!("📧 Sending email via SMTP...");
    let _send_result = helper
        .send_email_smtp(
            &peer1_path,
            2525,
            &account1_password,
            &from_email,
            &to_email,
            "Integration Test Email",
            "End-to-end SMTP to P2P to INBOX test",
        )
        .await
        .expect("SMTP email send should succeed");

    println!("✅ Email sent via SMTP");

    // Debug: Check if email was queued for P2P delivery first
    println!("🔍 Debug: Checking if email was queued for P2P delivery...");
    
    // Wait and check delivery status incrementally  
    for attempt in 1..=6 {
        tokio::time::sleep(Duration::from_secs(5)).await;
        println!("⏳ P2P delivery check #{}/6", attempt);
        
        // Check peer 1's Sent folder
        let peer1_sent_emails = find_emails_in_folder(&peer1_path, &account1_id, "Sent").await;
        println!("📊 Peer 1 Sent: {} emails", peer1_sent_emails.len());
        
        // Check peer 2's INBOX folder
        let peer2_inbox_emails = find_emails_in_folder(&peer2_path, &account2_id, "INBOX").await;
        println!("📊 Peer 2 INBOX: {} emails", peer2_inbox_emails.len());
        
        // Check peer 2's Sent folder (the bug we fixed)
        let peer2_sent_emails = find_emails_in_folder(&peer2_path, &account2_id, "Sent").await;
        println!("📊 Peer 2 Sent: {} emails (should be 0 for received emails)", peer2_sent_emails.len());
        
        if !peer2_inbox_emails.is_empty() {
            println!("✅ P2P delivery successful on attempt {}", attempt);
            break;
        }
        
        if attempt == 6 {
            // Final attempt - gather detailed debug info
            println!("🐛 Debug: P2P delivery failed after {} attempts", attempt);
            println!("🐛 Debug: Checking database for pending deliveries...");
            
            // Check if there are pending deliveries in the database
            if let Ok(db_path) = std::fs::read_dir(peer1_path.join("accounts").join(&account1_id)) {
                for entry in db_path.flatten() {
                    if entry.path().extension().and_then(|s| s.to_str()) == Some("sqlite") {
                        println!("🐛 Debug: Found database: {:?}", entry.path());
                    }
                }
            }
        }
    }

    // Final verification
    let peer1_sent_emails = find_emails_in_folder(&peer1_path, &account1_id, "Sent").await;
    assert!(
        !peer1_sent_emails.is_empty(),
        "Email should be in peer 1's Sent folder"
    );
    println!("✅ Found {} emails in peer 1 Sent folder", peer1_sent_emails.len());

    let peer2_inbox_emails = find_emails_in_folder(&peer2_path, &account2_id, "INBOX").await;
    if peer2_inbox_emails.is_empty() {
        // Print debug info before failing
        println!("🐛 Debug: No emails found in peer 2 INBOX");
        println!("🐛 Debug: Peer 1 account: {}", account1_id);
        println!("🐛 Debug: Peer 2 account: {}", account2_id); 
        println!("🐛 Debug: From email: {}", from_email);
        println!("🐛 Debug: To email: {}", to_email);
        
        // Check if peer 2 accounts directory exists
        let peer2_account_dir = peer2_path.join("accounts").join(&account2_id);
        println!("🐛 Debug: Peer 2 account dir exists: {}", peer2_account_dir.exists());
        
        panic!("Email should be delivered to peer 2's INBOX after 30 seconds");
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
    // Note: ProcessCleanup guard will handle process termination automatically
}

/// Extract account ID from fastn-rig init output
fn extract_account_id(output: &str) -> Option<String> {
    // Look for "Primary account:" line which has the actual account ID
    for line in output.lines() {
        if line.contains("Primary account:") {
            if let Some(id_part) = line.split("Primary account:").nth(1) {
                return Some(id_part.trim().to_string());
            }
        }
    }
    
    // Fallback: look for first ID52 that's not a Rig ID52
    for line in output.lines() {
        if line.contains("ID52:")
            && !line.contains("Rig ID52:")
            && let Some(id_part) = line.split("ID52:").nth(1)
        {
            return Some(id_part.trim().to_string());
        }
    }
    None
}

/// Extract password from fastn-rig init output  
fn extract_password(output: &str) -> Option<String> {
    for line in output.lines() {
        if line.contains("Password:")
            && let Some(pwd_part) = line.split("Password:").nth(1)
        {
            return Some(pwd_part.trim().to_string());
        }
    }
    None
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
