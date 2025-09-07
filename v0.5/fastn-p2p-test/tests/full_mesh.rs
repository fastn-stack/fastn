//! Test full mesh: multiple senders to multiple receivers

use std::time::Duration;
use tokio::process::Command;

#[tokio::test]
async fn test_full_mesh() {
    println!("🔧 Testing full mesh: multiple senders ↔ multiple receivers...");

    let num_receivers = 2;
    let num_senders = 3;
    let messages_per_sender = 2;

    let mut receiver_processes = Vec::new();
    let mut receiver_ids = Vec::new();

    // Start multiple receivers
    for receiver_id in 1..=num_receivers {
        let receiver_key = fastn_id52::SecretKey::generate();
        let receiver_id52 = receiver_key.public_key().id52();

        println!("📡 Starting receiver #{}: {}", receiver_id, receiver_id52);

        let receiver = Command::new("cargo")
            .args([
                "run",
                "--bin",
                "p2p_receiver",
                "-p",
                "fastn-p2p-test",
                &receiver_key.to_string(),
            ])
            .spawn()
            .expect("Failed to start receiver");

        receiver_processes.push(ProcessCleanup::new(receiver));
        receiver_ids.push(receiver_id52);

        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    // Wait for all receivers to start
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Launch multiple senders concurrently
    let mut sender_tasks = Vec::new();

    for sender_id in 1..=num_senders {
        let sender_key = fastn_id52::SecretKey::generate();
        let receiver_ids_clone = receiver_ids.clone();

        let task = tokio::spawn(async move {
            println!("🚀 Sender #{} starting...", sender_id);
            let mut sender_success_count = 0;

            // Each sender sends to multiple receivers
            for msg_num in 1..=messages_per_sender {
                for (recv_idx, receiver_id52) in receiver_ids_clone.iter().enumerate() {
                    println!(
                        "📤 Sender #{} → Message #{} → Receiver #{}",
                        sender_id,
                        msg_num,
                        recv_idx + 1
                    );

                    let sender_output = Command::new("cargo")
                        .args([
                            "run",
                            "--bin",
                            "p2p_sender",
                            "-p",
                            "fastn-p2p-test",
                            &sender_key.to_string(),
                            receiver_id52,
                        ])
                        .output()
                        .await
                        .expect("Failed to run sender");

                    if sender_output.status.success() {
                        let stdout = String::from_utf8_lossy(&sender_output.stdout);
                        if stdout.contains("\"status\":\"success\"") {
                            sender_success_count += 1;
                            println!(
                                "✅ Sender #{} → Receiver #{} SUCCESS",
                                sender_id,
                                recv_idx + 1
                            );
                        } else {
                            println!(
                                "⚠️ Sender #{} → Receiver #{} no JSON",
                                sender_id,
                                recv_idx + 1
                            );
                        }
                    } else {
                        println!(
                            "❌ Sender #{} → Receiver #{} FAILED",
                            sender_id,
                            recv_idx + 1
                        );
                    }

                    // Small delay between messages
                    tokio::time::sleep(Duration::from_millis(300)).await;
                }
            }

            println!(
                "🎯 Sender #{} completed: {}/{} messages successful",
                sender_id,
                sender_success_count,
                messages_per_sender * receiver_ids_clone.len()
            );
            sender_success_count
        });

        sender_tasks.push(task);
    }

    // Wait for all senders to complete
    let mut total_success = 0;
    let expected_total = num_senders * messages_per_sender * num_receivers;

    for (i, task) in sender_tasks.into_iter().enumerate() {
        match task.await {
            Ok(success_count) => {
                total_success += success_count;
                println!(
                    "✅ Sender #{} finished with {} successes",
                    i + 1,
                    success_count
                );
            }
            Err(e) => {
                println!("❌ Sender #{} task failed: {}", i + 1, e);
            }
        }
    }

    println!(
        "🎉 Full mesh test completed: {}/{} total messages successful",
        total_success, expected_total
    );

    // Assert reasonable success rate for robust mesh testing
    assert!(
        total_success >= expected_total / 2,
        "Expected at least 50% success rate in mesh test, got {}/{}",
        total_success,
        expected_total
    );
}

/// Process cleanup guard
struct ProcessCleanup {
    process: tokio::process::Child,
}

impl ProcessCleanup {
    fn new(process: tokio::process::Child) -> Self {
        Self { process }
    }
}

impl Drop for ProcessCleanup {
    fn drop(&mut self) {
        let _ = self.process.start_kill();
        println!("🧹 Process cleanup completed");
    }
}
