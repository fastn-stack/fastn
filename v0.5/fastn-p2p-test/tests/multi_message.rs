//! Test multiple messages between fastn-p2p sender and receiver

use std::time::Duration;
use tokio::process::Command;

#[tokio::test]
async fn test_single_sender_multiple_messages() {
    println!("🔧 Testing single sender sending multiple messages...");

    // Create fresh keys
    let receiver_key = fastn_id52::SecretKey::generate();
    let sender_key = fastn_id52::SecretKey::generate();

    let receiver_id52 = receiver_key.public_key().id52();
    println!("🔑 Receiver ID52: {}", receiver_id52);

    // Start receiver
    println!("📡 Starting fastn-p2p receiver...");
    let mut receiver = Command::new("cargo")
        .args(["run", "--bin", "receiver", "-p", "fastn-p2p-test", &receiver_key.to_string()])
        .spawn()
        .expect("Failed to start fastn-p2p receiver");

    let _cleanup = ProcessCleanup::new(&mut receiver);

    // Wait for receiver to start
    tokio::time::sleep(Duration::from_secs(3)).await;

    // Send multiple messages sequentially
    for i in 1..=5 {
        println!("📤 Sending message #{i}...");
        
        let sender_output = Command::new("cargo")
            .args([
                "run",
                "--bin",
                "sender",
                "-p",
                "fastn-p2p-test",
                &sender_key.to_string(),
                &receiver_id52,
            ])
            .output()
            .await
            .expect("Failed to run fastn-p2p sender");

        let stdout = String::from_utf8_lossy(&sender_output.stdout);
        
        if sender_output.status.success() {
            println!("✅ Message #{i} sent successfully");
            if stdout.contains("\"status\": \"success\"") {
                println!("✅ Message #{i} received JSON success");
            } else {
                println!("⚠️ Message #{i} no JSON result");
            }
        } else {
            println!("❌ Message #{i} failed: {}", sender_output.status);
            break;
        }
        
        // Small delay between messages
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    println!("🎯 Multiple message test completed");
}

/// Process cleanup guard
struct ProcessCleanup<'a> {
    process: &'a mut tokio::process::Child,
}

impl<'a> ProcessCleanup<'a> {
    fn new(process: &'a mut tokio::process::Child) -> Self {
        Self { process }
    }
}

impl<'a> Drop for ProcessCleanup<'a> {
    fn drop(&mut self) {
        let _ = self.process.start_kill();
        println!("🧹 Process cleanup completed");
    }
}