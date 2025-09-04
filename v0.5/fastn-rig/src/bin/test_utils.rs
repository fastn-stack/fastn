//! Test utilities for bash script integration testing
//!
//! Provides simple commands for extracting account info and checking email delivery

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "test-utils")]
#[command(about = "Test utilities for fastn-rig integration testing")]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Parser)]
enum Command {
    /// Extract account ID and password from fastn-rig init output
    ExtractAccount {
        /// Path to init output file
        #[arg(short = 'f', long)]
        file: PathBuf,
        /// Output format: json, account-id, password, or all
        #[arg(short = 'o', long, default_value = "json")]
        format: String,
    },
    /// Count emails in a specific folder
    CountEmails {
        /// Account directory path
        #[arg(short = 'a', long)]
        account_dir: PathBuf,
        /// Folder name (Sent, INBOX, etc.)
        #[arg(short = 'f', long)]
        folder: String,
    },
    /// Check if P2P delivery completed by comparing Sent and INBOX counts
    CheckDelivery {
        /// Sender account directory
        #[arg(long)]
        sender_dir: PathBuf,
        /// Receiver account directory  
        #[arg(long)]
        receiver_dir: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    match args.command {
        Command::ExtractAccount { file, format } => {
            let content = std::fs::read_to_string(&file)
                .map_err(|e| format!("Failed to read {}: {}", file.display(), e))?;

            let account_id = extract_account_id(&content)
                .ok_or("Failed to extract account ID from init output")?;
            let password = extract_password(&content)
                .ok_or("Failed to extract password from init output")?;

            match format.as_str() {
                "json" => {
                    let result = serde_json::json!({
                        "account_id": account_id,
                        "password": password,
                        "account_id_length": account_id.len(),
                        "extracted_at": chrono::Utc::now().to_rfc3339()
                    });
                    println!("{}", serde_json::to_string_pretty(&result)?);
                }
                "account-id" => println!("{}", account_id),
                "password" => println!("{}", password),
                "all" => println!("{}:{}", account_id, password),
                _ => return Err(format!("Unknown format: {}", format).into()),
            }
        }
        
        Command::CountEmails { account_dir, folder } => {
            let folder_path = account_dir.join("mails").join("default").join(&folder);
            let count = count_emails_in_folder(&folder_path).await?;
            
            let result = serde_json::json!({
                "folder": folder,
                "count": count,
                "path": folder_path,
                "checked_at": chrono::Utc::now().to_rfc3339()
            });
            println!("{}", serde_json::to_string(&result)?);
        }
        
        Command::CheckDelivery { sender_dir, receiver_dir } => {
            let sender_sent = count_emails_in_folder(&sender_dir.join("mails/default/Sent")).await?;
            let receiver_inbox = count_emails_in_folder(&receiver_dir.join("mails/default/INBOX")).await?;
            let receiver_sent = count_emails_in_folder(&receiver_dir.join("mails/default/Sent")).await?;
            
            let delivery_complete = sender_sent > 0 && receiver_inbox > 0;
            let folder_fix_working = receiver_sent == 0; // Received emails shouldn't be in Sent
            
            let result = serde_json::json!({
                "delivery_complete": delivery_complete,
                "folder_fix_working": folder_fix_working,
                "sender_sent": sender_sent,
                "receiver_inbox": receiver_inbox,
                "receiver_sent": receiver_sent,
                "checked_at": chrono::Utc::now().to_rfc3339()
            });
            println!("{}", serde_json::to_string(&result)?);
        }
    }

    Ok(())
}

/// Extract account ID from fastn-rig init output  
fn extract_account_id(output: &str) -> Option<String> {
    // Look for "Primary account:" line which has the actual account ID
    for line in output.lines() {
        if line.contains("Primary account:") {
            if let Some(id_part) = line.split("Primary account:").nth(1) {
                return Some(id_part.trim().to_string());
            }
        }
    }
    
    // Fallback: look for first ID52 that's not a Rig ID52
    for line in output.lines() {
        if line.contains("ID52:") && !line.contains("Rig ID52:") {
            if let Some(id_part) = line.split("ID52:").nth(1) {
                return Some(id_part.trim().to_string());
            }
        }
    }
    None
}

/// Extract password from fastn-rig init output
fn extract_password(output: &str) -> Option<String> {
    for line in output.lines() {
        if line.contains("Password:") {
            if let Some(pwd_part) = line.split("Password:").nth(1) {
                return Some(pwd_part.trim().to_string());
            }
        }
    }
    None
}

/// Count .eml files in a folder recursively
async fn count_emails_in_folder(folder_path: &std::path::Path) -> Result<usize, Box<dyn std::error::Error>> {
    if !folder_path.exists() {
        return Ok(0);
    }
    
    let mut count = 0;
    let mut walker = walkdir::WalkDir::new(folder_path).into_iter();
    
    while let Some(entry) = walker.next() {
        match entry {
            Ok(entry) => {
                if entry.path().extension().and_then(|s| s.to_str()) == Some("eml") {
                    count += 1;
                }
            }
            Err(_) => continue, // Skip errors (permissions, etc.)
        }
    }
    
    Ok(count)
}