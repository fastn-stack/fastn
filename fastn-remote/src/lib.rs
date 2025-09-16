#![warn(unused_extern_crates)]
#![deny(unused_crate_dependencies)]

extern crate self as fastn_remote;

use clap as _; // used by main for CLI
use fastn_p2p as _; // used by main for macro
use tokio as _; // used by main for macro
use tracing_subscriber as _; // used by main macro for logging

mod cli;
mod init;
mod listen;
mod run;

pub use cli::{Cli, handle_cli};
pub use init::init;
pub use listen::{listen, listen_cli};
pub use run::run;

/// Execute single command on remote SSH server
pub async fn exec(secret_key: fastn_id52::SecretKey, target: fastn_id52::PublicKey, command: &str) {
    todo!(
        "Execute command '{command}' on {target} using {}",
        secret_key.id52()
    );
}

/// Start interactive TTY session on remote SSH server  
pub async fn tty(secret_key: fastn_id52::SecretKey, target: fastn_id52::PublicKey) {
    todo!("Start TTY session to {target} using {}", secret_key.id52());
}

/// Start stream-based session (like scp -t, separate stdout/stderr)
pub async fn spawn(
    secret_key: fastn_id52::SecretKey,
    target: fastn_id52::PublicKey,
    command: &str,
) {
    todo!(
        "Spawn streamed command '{command}' on {target} using {}",
        secret_key.id52()
    );
}

/// CLI wrapper for exec command
pub async fn exec_cli(private_key: &str, target: &str, command: &str) {
    use std::str::FromStr;

    let secret_key = match fastn_id52::SecretKey::from_str(private_key.trim()) {
        Ok(key) => key,
        Err(e) => {
            eprintln!("Error: Invalid private key format: {}", e);
            std::process::exit(1);
        }
    };

    let target_key = match fastn_id52::PublicKey::from_str(target.trim()) {
        Ok(key) => key,
        Err(e) => {
            eprintln!("Error: Invalid target ID52 '{}': {}", target, e);
            std::process::exit(1);
        }
    };

    exec(secret_key, target_key, command).await;
}

/// CLI wrapper for tty command  
pub async fn tty_cli(private_key: &str, target: &str) {
    use std::str::FromStr;

    let secret_key = match fastn_id52::SecretKey::from_str(private_key.trim()) {
        Ok(key) => key,
        Err(e) => {
            eprintln!("Error: Invalid private key format: {}", e);
            std::process::exit(1);
        }
    };

    let target_key = match fastn_id52::PublicKey::from_str(target.trim()) {
        Ok(key) => key,
        Err(e) => {
            eprintln!("Error: Invalid target ID52 '{}': {}", target, e);
            std::process::exit(1);
        }
    };

    tty(secret_key, target_key).await;
}

/// CLI wrapper for spawn command
pub async fn spawn_cli(private_key: &str, target: &str, command: &str) {
    use std::str::FromStr;

    let secret_key = match fastn_id52::SecretKey::from_str(private_key.trim()) {
        Ok(key) => key,
        Err(e) => {
            eprintln!("Error: Invalid private key format: {}", e);
            std::process::exit(1);
        }
    };

    let target_key = match fastn_id52::PublicKey::from_str(target.trim()) {
        Ok(key) => key,
        Err(e) => {
            eprintln!("Error: Invalid target ID52 '{}': {}", target, e);
            std::process::exit(1);
        }
    };

    spawn(secret_key, target_key, command).await;
}
