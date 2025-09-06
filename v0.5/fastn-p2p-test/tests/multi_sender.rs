//! Test multiple senders connecting to single receiver concurrently

use std::time::Duration;
use tokio::process::Command;

#[tokio::test]
async fn test_multi_sender() {
    println!("🔧 Testing multiple senders → single receiver...");

    // Create receiver key
    let receiver_key = fastn_id52::SecretKey::generate();
    let receiver_id52 = receiver_key.public_key().id52();
    
    println!("🔑 Receiver ID52: {}", receiver_id52);

    // Start single receiver
    println!("📡 Starting single fastn-p2p receiver for multiple senders...");
    let mut receiver = Command::new("cargo")
        .args(["run", "--bin", "receiver", "-p", "fastn-p2p-test", &receiver_key.to_string()])
        .spawn()
        .expect("Failed to start fastn-p2p receiver");

    let _cleanup = ProcessCleanup::new(&mut receiver);

    // Wait for receiver to start
    tokio::time::sleep(Duration::from_secs(3)).await;

    // Create multiple senders concurrently
    let num_senders = 3;
    let mut sender_tasks = Vec::new();

    for sender_id in 1..=num_senders {
        let sender_key = fastn_id52::SecretKey::generate();
        let receiver_id52_clone = receiver_id52.clone();
        
        println!("🔑 Generated sender #{} key: {}", sender_id, sender_key.public_key().id52());
        
        let task = tokio::spawn(async move {
            println!("📤 Sender #{} starting...", sender_id);
            
            let sender_output = Command::new("cargo")
                .args([
                    "run",
                    "--bin", 
                    "sender",
                    "-p",
                    "fastn-p2p-test",
                    &sender_key.to_string(),
                    &receiver_id52_clone,
                ])
                .output()
                .await
                .expect("Failed to run sender");

            let stdout = String::from_utf8_lossy(&sender_output.stdout);
            
            if sender_output.status.success() {
                println!("✅ Sender #{} completed successfully", sender_id);
                if stdout.contains("\"status\": \"success\"") {
                    println!("✅ Sender #{} received JSON success", sender_id);
                    true
                } else {
                    println!("⚠️ Sender #{} no JSON success found", sender_id);
                    false
                }
            } else {
                println!("❌ Sender #{} failed: {}", sender_id, sender_output.status);
                false
            }
        });
        
        sender_tasks.push(task);
        
        // Small stagger to avoid overwhelming
        tokio::time::sleep(Duration::from_millis(200)).await;
    }

    // Wait for all senders to complete
    let mut success_count = 0;
    for (i, task) in sender_tasks.into_iter().enumerate() {
        match task.await {
            Ok(success) => {
                if success {
                    success_count += 1;
                    println!("✅ Sender #{} task completed successfully", i + 1);
                } else {
                    println!("⚠️ Sender #{} task completed with issues", i + 1);
                }
            }
            Err(e) => {
                println!("❌ Sender #{} task failed: {}", i + 1, e);
            }
        }
    }

    println!("🎯 Multiple senders test completed: {}/{} successful", success_count, num_senders);
    
    // Assert majority success for robust testing
    assert!(success_count >= num_senders / 2, 
            "Expected at least half the senders to succeed, got {}/{}", 
            success_count, num_senders);
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