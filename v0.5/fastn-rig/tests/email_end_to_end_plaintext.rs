//! 🎯 CRITICAL END-TO-END EMAIL TEST (PLAIN TEXT MODE)
//!
//! This is one of the most important tests in the fastn email system.
//! Tests the complete email pipeline using plain text SMTP:
//!
//! 1. ✅ Plain text SMTP server accepts email clients  
//! 2. ✅ Email authentication and routing works
//! 3. ✅ Email storage in Sent folder works
//! 4. ✅ P2P delivery between rigs works via fastn-p2p
//! 5. ✅ Email delivery to INBOX folder works
//! 6. ✅ Complete email pipeline is operational
//!
//! NOTE: This test calls the bash script for independent validation.
//!       Companion test: email_end_to_end_starttls.rs (tests STARTTLS mode)

/// 🎯 CRITICAL TEST: Complete Plain Text Email Pipeline via Bash Script
///
/// This test validates the entire fastn email system using independent bash script execution.
/// Provides redundancy with the STARTTLS Rust test using different validation approach.
#[test]
fn email_end_to_end_plaintext() {
    println!("🎯 CRITICAL END-TO-END EMAIL TEST (Plain Text Mode)");
    println!("📧 Testing: Plain text SMTP → fastn-p2p → INBOX delivery");
    println!("🔗 Method: Independent bash script execution");

    // Find the script in the tests directory (relative to fastn-rig root)
    let script_path = "tests/email_end_to_end_plaintext.sh";
    if !std::path::Path::new(script_path).exists() {
        panic!(
            "CRITICAL: Plain text email test script not found at: {}\nCurrent dir: {:?}",
            script_path,
            std::env::current_dir().unwrap()
        );
    }

    let output = std::process::Command::new("bash")
        .arg(script_path)
        .output()
        .expect("CRITICAL: Failed to execute plain text email test script");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !stderr.trim().is_empty() {
        println!("Script stderr: {}", stderr.trim());
    }

    if output.status.success() {
        println!("✅ CRITICAL: Plain text email test PASSED");
        if stdout.contains("COMPLETE SUCCESS") {
            println!("✅ Plain text SMTP→fastn-p2p→INBOX delivery working");
        }
    } else {
        println!("❌ CRITICAL: Plain text email test FAILED");
        println!("Last 10 lines of output:");
        for line in stdout
            .lines()
            .rev()
            .take(10)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
        {
            println!("  {}", line);
        }
        panic!(
            "CRITICAL: Plain text email pipeline failed - check ./tests/email_end_to_end_plaintext.sh"
        );
    }
}
