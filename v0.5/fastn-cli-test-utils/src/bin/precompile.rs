//! Pre-compilation binary for CI email tests
//!
//! This binary pre-compiles all necessary binaries for email system testing
//! to isolate compilation time from test execution time in CI.

use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔨 Pre-compiling email system binaries...");
    
    // Build fastn-rig
    println!("📦 Building fastn-rig...");
    let output = Command::new("cargo")
        .args(["build", "--bin", "fastn-rig"])
        .output()?;
    
    if !output.status.success() {
        eprintln!("❌ Failed to build fastn-rig:");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        std::process::exit(1);
    }
    println!("✅ fastn-rig built successfully");
    
    // Build fastn-mail with net features
    println!("📦 Building fastn-mail with net features...");
    let output = Command::new("cargo")
        .args(["build", "--bin", "fastn-mail", "--features", "net"])
        .output()?;
    
    if !output.status.success() {
        eprintln!("❌ Failed to build fastn-mail:");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        std::process::exit(1);
    }
    println!("✅ fastn-mail built successfully with net features");
    
    // Build test_utils
    println!("📦 Building test_utils...");
    let output = Command::new("cargo")
        .args(["build", "--bin", "test_utils"])
        .output()?;
    
    if !output.status.success() {
        eprintln!("❌ Failed to build test_utils:");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        std::process::exit(1);
    }
    println!("✅ test_utils built successfully");
    
    println!("🎉 All email system binaries pre-compiled successfully!");
    println!("⏱️ Compilation time isolated from test execution time");
    
    Ok(())
}