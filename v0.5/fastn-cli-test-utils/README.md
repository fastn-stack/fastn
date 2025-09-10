# fastn-cli-test-utils

**Comprehensive fastn CLI testing utilities** that make testing fastn commands pleasant by handling all the drudgery of binary discovery, process management, argument passing, and cleanup.

## 🎯 **fastn-Centric Philosophy** 

This is **not** a generic CLI testing framework - it's specifically designed for **fastn** and knows about:
- All fastn commands (`fastn-rig`, `fastn-mail`, `fastn-automerge`, etc.)
- fastn concepts (peers, SMTP ports, keyring, accounts, passwords)
- fastn environment variables (`FASTN_HOME`, `SKIP_KEYRING`, `FASTN_SMTP_PORT`)
- fastn test patterns (two-peer email tests, P2P delivery, etc.)

This makes writing fastn tests **extremely pleasant** because the test utilities handle all repetitive patterns.

## ✨ **Pleasant API Examples**

### Simple CLI Command Testing
```rust
use fastn_cli_test_utils::FastnRigCommand;

#[tokio::test]
async fn test_rig_status() -> Result<(), Box<dyn std::error::Error>> {
    let mut env = FastnTestEnv::new("status-test")?;
    let peer = env.create_peer("test-peer").await?;
    
    // Test status with one beautiful line
    FastnRigCommand::new()
        .home(&peer.home_path)
        .status()
        .execute().await?
        .expect_success()?;
        
    Ok(())
}
```

### Email Testing (The Pain Point Eliminated)
```rust
#[tokio::test]
async fn test_email_delivery() -> Result<(), Box<dyn std::error::Error>> {
    let mut env = FastnTestEnv::new("email-test")?;
    
    // Create peers (automatic init, account extraction, port assignment)
    env.create_peer("sender").await?;
    env.create_peer("receiver").await?;
    
    // Start processes (automatic SMTP port configuration)
    env.start_peer("sender").await?;
    env.start_peer("receiver").await?;
    env.wait_for_startup().await?;
    
    // Send email (automatic credential management, address formatting)
    env.email()
        .from("sender")
        .to("receiver")
        .subject("Pleasant Test")
        .body("No more argument drudgery!")
        .send().await?
        .expect_success()?;
    
    // Automatic process cleanup on drop
    Ok(())
}
```

### Custom Mail Operations
```rust
use fastn_cli_test_utils::FastnMailCommand;

// Direct fastn-mail command with all the argument handling done for you
FastnMailCommand::new()
    .home(&peer_home)
    .send_mail()
    .from("test@sender.fastn")  
    .to("inbox@receiver.fastn") 
    .subject("Custom Email")
    .smtp_port(2525)
    .password("secret123")
    .send().await?
    .expect_success()?;
```

## 🚀 **What Gets Handled For You**

### Before (Manual Drudgery)
```rust
// Every test had to handle:
fn detect_target_dir() -> PathBuf { /* 30+ lines of fallback logic */ }

let output = Command::new(&binary_path)
    .arg("send-mail")
    .arg("--smtp").arg("2525")
    .arg("--password").arg(&password)
    .arg("--from").arg(&format!("test@{}.fastn", sender_id))
    .arg("--to").arg(&format!("inbox@{}.fastn", receiver_id))
    .arg("--subject").arg("Test")
    .arg("--body").arg("Body")
    .env("FASTN_HOME", &sender_home)
    .output().await?;

// Manual process cleanup, error handling, output parsing...
```

### After (Pleasant API)
```rust
// All that becomes:
env.email().from("sender").to("receiver").send().await?.expect_success()?;
```

## 🔧 **Handles All fastn Complexity**

- ✅ **Binary Discovery**: Workspace/home target flexibility, `CARGO_TARGET_DIR` support
- ✅ **Argument Patterns**: All fastn command argument structures 
- ✅ **Environment Variables**: `FASTN_HOME`, `SKIP_KEYRING`, `FASTN_SMTP_PORT`, etc.
- ✅ **Process Lifecycle**: RAII cleanup, background processes, startup waiting
- ✅ **Peer Management**: Account IDs, passwords, SMTP port allocation
- ✅ **Error Handling**: Expect success/failure, output validation, account extraction
- ✅ **Email Patterns**: Peer-to-peer addressing, credential management

## 📊 **Migration Results**

| File | Before | After | Reduction |
|------|--------|-------|-----------|
| `cli_tests.rs` | 40+ lines of binary detection | 1 function call | 95%+ |
| `p2p_inbox_delivery.rs` | 25+ lines of helper class | 2 function calls | 90%+ |
| `test_complete_integration.sh` | Manual bash detection | References same logic | Consistent |

## 💡 **Target Directory Flexibility**

Works seamlessly with all configurations:
- **Workspace**: `v0.5/target/debug/` (current)
- **Home**: `~/target/debug/` (if you switch back)
- **Custom**: Via `CARGO_TARGET_DIR` environment variable

Tests just work regardless of your cargo target configuration!

This crate transforms fastn CLI testing from tedious to pleasant. 🎉