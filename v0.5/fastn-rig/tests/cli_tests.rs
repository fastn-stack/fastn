use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

fn get_binary_path() -> PathBuf {
    // Use cargo test to build and get the correct binary path
    let output = Command::new("cargo")
        .args(["build", "-p", "fastn-rig", "--bin", "fastn-rig"])
        .output()
        .expect("Failed to build binary");

    if !output.status.success() {
        panic!(
            "Failed to build binary: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let target_dir = std::env::var("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            // Default location in home directory
            PathBuf::from(std::env::var("HOME").unwrap()).join("target")
        });
    target_dir.join("debug").join("fastn-rig")
}

#[test]
fn test_cli_help() {
    let output = Command::new(get_binary_path())
        .arg("--help")
        .output()
        .expect("Failed to execute fastn-rig");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("A CLI for testing and managing fastn-rig"));
    assert!(stdout.contains("init"));
    assert!(stdout.contains("status"));
    assert!(stdout.contains("entities"));
}

#[test]
fn test_cli_init_and_status() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let home_path = temp_dir.path().to_str().unwrap();

    // Test init command
    let output = Command::new(get_binary_path())
        .arg("--home")
        .arg(home_path)
        .arg("init")
        .env("SKIP_KEYRING", "true")
        .output()
        .expect("Failed to execute init");

    assert!(
        output.status.success(),
        "Init failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Rig initialized successfully!"));
    assert!(stdout.contains("Rig ID52:"));
    assert!(stdout.contains("Owner:"));

    // Test status command
    let output = Command::new(get_binary_path())
        .arg("--home")
        .arg(home_path)
        .arg("status")
        .env("SKIP_KEYRING", "true")
        .output()
        .expect("Failed to execute status");

    assert!(
        output.status.success(),
        "Status failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Rig Status"));
    assert!(stdout.contains("Rig ID52:"));
    assert!(stdout.contains("Current entity:"));
}

#[test]
fn test_cli_entities() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let home_path = temp_dir.path().to_str().unwrap();

    // Initialize first
    let output = Command::new(get_binary_path())
        .arg("--home")
        .arg(home_path)
        .arg("init")
        .env("SKIP_KEYRING", "true")
        .output()
        .expect("Failed to execute init");
    assert!(output.status.success());

    // Test entities command
    let output = Command::new(get_binary_path())
        .arg("--home")
        .arg(home_path)
        .arg("entities")
        .env("SKIP_KEYRING", "true")
        .output()
        .expect("Failed to execute entities");

    assert!(
        output.status.success(),
        "Entities failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Entities"));
    assert!(stdout.contains("(rig)"));
    assert!(stdout.contains("(account)"));
}

#[test]
fn test_cli_set_online() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let home_path = temp_dir.path().to_str().unwrap();

    // Initialize first
    let output = Command::new(get_binary_path())
        .arg("--home")
        .arg(home_path)
        .arg("init")
        .env("SKIP_KEYRING", "true")
        .output()
        .expect("Failed to execute init");
    assert!(output.status.success());

    // Get the rig ID52 from status
    let output = Command::new(get_binary_path())
        .arg("--home")
        .arg(home_path)
        .arg("status")
        .env("SKIP_KEYRING", "true")
        .output()
        .expect("Failed to execute status");
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    let rig_id52 = stdout
        .lines()
        .find(|line| line.contains("Rig ID52:"))
        .and_then(|line| line.split("Rig ID52: ").nth(1))
        .expect("Could not find rig ID52");

    // Test set-online command
    let output = Command::new(get_binary_path())
        .arg("--home")
        .arg(home_path)
        .arg("set-online")
        .arg(rig_id52)
        .arg("false")
        .env("SKIP_KEYRING", "true")
        .output()
        .expect("Failed to execute set-online");

    assert!(
        output.status.success(),
        "Set-online failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Set") && stdout.contains("to OFFLINE"));
}

#[test]
fn test_status_without_init() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let home_path = temp_dir.path().to_str().unwrap();

    // Test status on uninitialized home should fail gracefully
    let output = Command::new(get_binary_path())
        .arg("--home")
        .arg(home_path)
        .arg("status")
        .env("SKIP_KEYRING", "true")
        .output()
        .expect("Failed to execute status");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("KeyLoadingFailed")
            || stderr.contains("Failed to load rig")
            || stderr.contains("Run 'init' first")
    );
}
