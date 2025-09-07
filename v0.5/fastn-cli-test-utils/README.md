# fastn-cli-test-utils

Unified CLI testing utilities for the fastn workspace, eliminating duplication of binary discovery, process management, and test environment setup.

## Problem Solved

Before this crate, CLI testing was scattered across multiple files with duplicated patterns:

- **fastn-rig/tests/cli_tests.rs**: Custom binary path detection
- **fastn-rig/tests/p2p_inbox_delivery.rs**: Complex process management  
- **fastn-rig/tests/test_complete_integration.sh**: Bash script with manual cleanup
- Multiple files implementing similar Drop-based process cleanup

## Key Features

### 🎯 **Automatic Binary Discovery**
- Supports both workspace (`v0.5/target/debug/`) and home (`~/target/debug/`) target directories
- Respects `CARGO_TARGET_DIR` environment variable
- Falls back gracefully across multiple locations

### 🧹 **RAII Process Management** 
- Automatic process cleanup via Drop trait
- No more manual `pkill` or forgotten process cleanup
- Proper async process lifecycle management

### ⚡ **Pre-compilation Strategy**
- Eliminates compilation delays during test execution
- Configurable: can disable for faster test iteration

### 🔗 **Fluent API**
- Chain operations for readable test code
- Type-safe builder patterns for complex scenarios

## Usage Examples

### Simple Two-Peer Email Test
```rust
use fastn_cli_test_utils::TestScenario;
use std::time::Duration;

#[tokio::test]
async fn test_email_delivery() -> Result<(), Box<dyn std::error::Error>> {
    TestScenario::new("email-test")
        .with_peers(&["sender", "receiver"])
        .with_smtp_ports(&[2525, 2526])
        .without_keyring()
        .run(|mut test| async move {
            // Start peers and wait for ready
            test.start_all_peers().await?;
            test.wait_for_startup().await?;
            
            // Send email with fluent API
            test.email()
                .from("sender")
                .to("receiver")
                .subject("Test Email")
                .body("Testing P2P delivery")
                .send()
                .await?
                .expect_success()?
                .wait_for_delivery(Duration::from_secs(30))
                .await?;
            
            // Automatic cleanup on scope exit
            Ok(())
        })
        .await
}
```

### Advanced Custom Configuration
```rust
let custom_config = CliConfig {
    pre_build: false,           // Build on demand
    cleanup_on_drop: true,      // Auto cleanup
    skip_keyring: true,         // Skip keyring in tests
    default_timeout: Duration::from_secs(60),
    smtp_port_range: 3000..3100,
};

TestScenario::new("advanced-test")
    .with_config(custom_config)
    .with_peers(&["hub", "client1", "client2"])
    .run(|mut test| async move {
        // Selective peer startup
        test.start_peer("hub").await?;
        test.start_peer("client1").await?;
        
        // Multiple email sequence
        for i in 1..=10 {
            test.email()
                .from("client1")
                .to("hub")
                .subject(&format!("Message {i}"))
                .send()
                .await?
                .expect_success()?;
        }
        
        Ok(())
    })
    .await
```

## Benefits Over Previous Approach

### Before (Duplicated Code)
```rust
// Every test file had this complexity:
fn detect_target_dir() -> PathBuf {
    if PathBuf::from("$HOME/target/debug/fastn-rig").exists() {
        PathBuf::from("$HOME/target/debug")  
    } else if PathBuf::from("./target/debug/fastn-rig").exists() {
        // ... 5 more fallback locations
    } else {
        panic!("Binary not found");
    }
}

struct ProcessCleanup { /* manual Drop impl */ }

// Manual process spawning, environment setup, cleanup...
```

### After (Unified & Fluent)
```rust
// Simple, readable, reliable:
TestScenario::new("my-test")
    .with_peers(&["peer1", "peer2"])
    .run(|test| async { /* test logic */ })
    .await
```

## Target Directory Flexibility

The crate automatically handles both configurations:
- **Workspace builds**: `v0.5/target/debug/` (current setup)
- **Home builds**: `~/target/debug/` (if you switch back)
- **Custom builds**: Via `CARGO_TARGET_DIR` environment variable

Tests work seamlessly regardless of your cargo configuration.

## Migration Path

Replace existing CLI test patterns:
1. Add `fastn-cli-test-utils = "0.1.0"` to test dependencies
2. Replace manual binary detection with `TestScenario` 
3. Replace manual process management with fluent API
4. Remove custom cleanup code (handled automatically)

This creates more reliable, readable, and maintainable CLI tests.