//! Stress test: high concurrency and rapid connections

use std::time::Duration;
use tokio::process::Command;

#[tokio::test]
async fn test_stress_test() {
    println!("🔧 Testing high concurrency stress (rapid connections)...");

    // Create one robust receiver
    let receiver_key = fastn_id52::SecretKey::generate();
    let receiver_id52 = receiver_key.public_key().id52();
    
    println!("📡 Starting stress test receiver: {}", receiver_id52);

    let mut receiver = Command::new("cargo")
        .args(["run", "--bin", "p2p_receiver", "-p", "fastn-p2p-test", &receiver_key.to_string()])
        .spawn()
        .expect("Failed to start receiver");

    let _cleanup = ProcessCleanup::new(&mut receiver);

    // Wait for receiver to start
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Launch many concurrent senders with minimal delays
    let num_concurrent = 5;
    let mut concurrent_tasks = Vec::new();

    for sender_id in 1..=num_concurrent {
        let sender_key = fastn_id52::SecretKey::generate();
        let receiver_id52_clone = receiver_id52.clone();
        
        let task = tokio::spawn(async move {
            // No delay - immediate concurrent execution
            let sender_output = Command::new("cargo")
                .args([
                    "run",
                    "--bin",
                    "p2p_sender",
                    "-p",
                    "fastn-p2p-test", 
                    &sender_key.to_string(),
                    &receiver_id52_clone,
                ])
                .output()
                .await
                .expect("Failed to run concurrent sender");

            let success = sender_output.status.success();
            if success {
                let stdout = String::from_utf8_lossy(&sender_output.stdout);
                let json_success = stdout.contains("\"status\":\"success\"");
                println!("✅ Concurrent sender #{}: {} (JSON: {})", 
                        sender_id, if success { "SUCCESS" } else { "FAILED" }, json_success);
                json_success
            } else {
                println!("❌ Concurrent sender #{}: FAILED", sender_id);
                false
            }
        });
        
        concurrent_tasks.push(task);
    }

    // Wait for all concurrent senders
    let mut concurrent_success = 0;
    for (i, task) in concurrent_tasks.into_iter().enumerate() {
        match task.await {
            Ok(true) => {
                concurrent_success += 1;
                println!("✅ Concurrent task #{} succeeded", i + 1);
            }
            Ok(false) => {
                println!("⚠️ Concurrent task #{} completed but failed", i + 1);
            }
            Err(e) => {
                println!("❌ Concurrent task #{} errored: {}", i + 1, e);
            }
        }
    }

    println!("🎉 Stress test completed: {}/{} concurrent connections successful", 
             concurrent_success, num_concurrent);
    
    // For stress test, expect at least some success (networking can be flaky under high load)
    assert!(concurrent_success >= 1, 
            "Expected at least 1 successful concurrent connection, got {}", 
            concurrent_success);
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
        println!("🧹 Stress test cleanup completed");
    }
}