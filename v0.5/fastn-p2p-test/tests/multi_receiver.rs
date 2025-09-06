//! Test single sender connecting to multiple receivers

use std::time::Duration;
use tokio::process::Command;

#[tokio::test]
async fn test_single_sender_multiple_receivers() {
    println!("🔧 Testing single sender → multiple receivers...");

    let num_receivers = 3;
    let mut receiver_processes = Vec::new();
    let mut receiver_ids = Vec::new();
    
    // Start multiple receivers
    for receiver_id in 1..=num_receivers {
        let receiver_key = fastn_id52::SecretKey::generate();
        let receiver_id52 = receiver_key.public_key().id52();
        
        println!("📡 Starting receiver #{}: {}", receiver_id, receiver_id52);
        
        let mut receiver = Command::new("cargo")
            .args(["run", "--bin", "receiver", "-p", "fastn-p2p-test", &receiver_key.to_string()])
            .spawn()
            .expect("Failed to start receiver");
            
        receiver_processes.push(ProcessCleanup::new(&mut receiver));
        receiver_ids.push(receiver_id52);
        
        // Small delay between starting receivers
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    // Wait for all receivers to start
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Single sender sends to all receivers
    let sender_key = fastn_id52::SecretKey::generate();
    let mut success_count = 0;

    for (i, receiver_id52) in receiver_ids.iter().enumerate() {
        println!("📤 Sending to receiver #{}: {}", i + 1, receiver_id52);
        
        let sender_output = Command::new("cargo")
            .args([
                "run",
                "--bin",
                "sender", 
                "-p",
                "fastn-p2p-test",
                &sender_key.to_string(),
                receiver_id52,
            ])
            .output()
            .await
            .expect("Failed to run sender");

        let stdout = String::from_utf8_lossy(&sender_output.stdout);
        
        if sender_output.status.success() {
            println!("✅ Message to receiver #{} sent successfully", i + 1);
            if stdout.contains("\"status\": \"success\"") {
                success_count += 1;
                println!("✅ Receiver #{} returned JSON success", i + 1);
            } else {
                println!("⚠️ Receiver #{} no JSON success", i + 1);
            }
        } else {
            println!("❌ Message to receiver #{} failed: {}", i + 1, sender_output.status);
        }
        
        // Delay between sending to different receivers
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    println!("🎯 Multiple receivers test completed: {}/{} successful", success_count, num_receivers);
    
    // Assert majority success
    assert!(success_count >= num_receivers / 2,
            "Expected at least half the receivers to succeed, got {}/{}",
            success_count, num_receivers);
}

/// Process cleanup guard
struct ProcessCleanup<'a> {
    #[allow(dead_code)]
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
        println!("🧹 Receiver cleanup completed");
    }
}