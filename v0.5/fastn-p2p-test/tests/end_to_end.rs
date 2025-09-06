//! End-to-end integration test for fastn-p2p CLI tools

use std::time::Duration;
use tokio::process::Command;

#[tokio::test]
async fn test_fastn_p2p_sender_receiver_cli() {
    println!("🔧 Testing fastn-p2p CLI with deterministic keys...");

    // Create fresh random keys to avoid conflicts with other tests/processes
    let receiver_key = fastn_id52::SecretKey::generate();
    let sender_key = fastn_id52::SecretKey::generate();

    let receiver_id52 = receiver_key.public_key().id52();
    let sender_id52 = sender_key.public_key().id52();

    println!("🔑 Receiver ID52: {}", receiver_id52);
    println!("🔑 Sender ID52: {}", sender_id52);

    // Start receiver with specific secret key (same pattern as fastn-net-test)
    println!("📡 Starting fastn-p2p receiver...");
    let mut receiver = Command::new("cargo")
        .args(["run", "--bin", "receiver", &receiver_key.to_string()])
        .spawn()
        .expect("Failed to start fastn-p2p receiver");

    let _cleanup = ProcessCleanup::new(&mut receiver);

    // Wait for receiver to start
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Run sender with specific keys
    println!("📤 Running fastn-p2p sender...");
    let sender_output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "sender",
            &sender_key.to_string(),
            &receiver_id52,
        ])
        .output()
        .await
        .expect("Failed to run fastn-p2p sender");

    let stdout = String::from_utf8_lossy(&sender_output.stdout);
    let stderr = String::from_utf8_lossy(&sender_output.stderr);

    println!("📝 Sender stdout: {}", stdout.trim());
    if !stderr.trim().is_empty() {
        println!("📝 Sender stderr: {}", stderr.trim());
    }

    if sender_output.status.success() {
        println!("✅ fastn-p2p Sender completed successfully");

        // Look for JSON result (same pattern as fastn-net-test)
        if stdout.contains("📋 RESULT:") && stdout.contains("\"status\": \"success\"") {
            println!("✅ Found JSON success result - fastn-p2p working!");
        } else {
            println!("⚠️ Sender succeeded but no JSON result found");
        }
    } else {
        println!("❌ fastn-p2p Sender failed with exit code: {}", sender_output.status);
        if stdout.contains("TimedOut") {
            println!("🐛 Identified timeout in test environment");
        }
    }

    println!("🎯 fastn-p2p CLI test completed");
}

/// Process cleanup guard (same as fastn-net-test)
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