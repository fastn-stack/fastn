//! Debug test environment differences
//! 
//! Tests why manual commands work but test commands fail

use std::time::Duration;
use tokio::process::Command;

#[tokio::test]
async fn debug_receiver_startup_in_test() {
    println!("🔧 Debug: Testing receiver startup in test environment");
    
    // Test 1: Can we even start the receiver?
    println!("📡 Starting receiver...");
    let output = Command::new("cargo")
        .args(["run", "--bin", "receiver"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .current_dir("/Users/amitu/Projects/fastn-me/v0.5/fastn-net-test")
        .kill_on_drop(true)  // This should kill the process when dropped
        .spawn();
    
    match output {
        Ok(mut process) => {
            println!("✅ Receiver process spawned successfully");
            
            // Wait a moment then kill
            tokio::time::sleep(Duration::from_secs(2)).await;
            
            match process.kill().await {
                Ok(_) => println!("✅ Receiver process killed successfully"),
                Err(e) => println!("❌ Failed to kill receiver: {}", e),
            }
        }
        Err(e) => {
            panic!("❌ Failed to spawn receiver process: {}", e);
        }
    }
    
    println!("🎉 Basic process spawning works in test environment");
}

#[tokio::test]
async fn debug_environment_differences() {
    println!("🔧 Debug: Checking environment differences");
    
    // Check current working directory
    let cwd = std::env::current_dir().unwrap();
    println!("📁 Test CWD: {:?}", cwd);
    
    // Check environment variables
    for (key, value) in std::env::vars() {
        if key.contains("FASTN") || key.contains("RUST") || key.contains("CARGO") {
            println!("🔧 Env: {}={}", key, value);
        }
    }
    
    // Test basic cargo command
    println!("📦 Testing basic cargo command...");
    let output = Command::new("cargo")
        .args(["--version"])
        .output()
        .await
        .expect("Failed to run cargo --version");
    
    println!("✅ Cargo version: {}", String::from_utf8_lossy(&output.stdout).trim());
    
    // Test if we can see the fastn-net-test binary
    let check_bins = Command::new("cargo")
        .args(["build", "--bin", "receiver"])
        .current_dir("/Users/amitu/Projects/fastn-me/v0.5/fastn-net-test")
        .output()
        .await
        .expect("Failed to check receiver binary");
    
    if check_bins.status.success() {
        println!("✅ Receiver binary builds successfully in test");
    } else {
        println!("❌ Receiver binary build failed: {}", String::from_utf8_lossy(&check_bins.stderr));
    }
}

#[tokio::test]
async fn debug_networking_in_test() {
    println!("🔧 Debug: Testing basic networking in test environment");
    
    // Create a simple TCP listener to test networking
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await
        .expect("Failed to create TCP listener");
    
    let addr = listener.local_addr().expect("Failed to get listener address");
    println!("🔗 Test TCP listener on: {}", addr);
    
    // Test connection to localhost
    match tokio::net::TcpStream::connect(addr).await {
        Ok(_stream) => {
            println!("✅ Basic TCP networking works in test environment");
        }
        Err(e) => {
            println!("❌ Basic TCP networking failed: {}", e);
        }
    }
    
    drop(listener);
    println!("🎉 Networking test completed");
}