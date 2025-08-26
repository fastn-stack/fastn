//! # fastn-mail CLI
//!
//! Command-line interface for testing and managing fastn email functionality.

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "fastn-mail")]
#[command(about = "CLI for testing fastn email functionality")]
pub struct Cli {
    /// Path to account directory
    #[arg(short, long, default_value = ".")]
    pub account_path: String,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Send an email message (for testing SMTP functionality)
    SendMail {
        /// Recipient email addresses (comma-separated)
        #[arg(long)]
        to: String,

        /// CC email addresses (comma-separated, optional)
        #[arg(long)]
        cc: Option<String>,

        /// BCC email addresses (comma-separated, optional)  
        #[arg(long)]
        bcc: Option<String>,

        /// Email subject line
        #[arg(long)]
        subject: String,

        /// Email body content
        #[arg(long)]
        body: String,

        /// From address (defaults to first alias in account)
        #[arg(long)]
        from: Option<String>,
    },

    /// List emails in a folder
    ListMails {
        /// Folder name (default: INBOX)
        #[arg(short, long, default_value = "INBOX")]
        folder: String,

        /// Maximum number of emails to show
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },

    /// List available folders
    ListFolders,

    /// Show email content by ID
    ShowMail {
        /// Email ID to display
        email_id: String,
    },

    /// Check pending P2P deliveries
    PendingDeliveries,
}

pub async fn run_command(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    // Load email store for the specified account
    let account_path = std::path::Path::new(&cli.account_path);
    let store = match crate::Store::load(account_path).await {
        Ok(store) => store,
        Err(_) => {
            println!("⚠️  No email store found at path, using test store for CLI demo");
            crate::Store::create_test()
        }
    };

    match cli.command {
        Commands::SendMail {
            to,
            cc,
            bcc,
            subject,
            body,
            from,
        } => {
            send_mail_command(&store, to, cc, bcc, subject, body, from).await?;
        }
        Commands::ListMails { folder, limit } => {
            list_mails_command(&store, &folder, limit).await?;
        }
        Commands::ListFolders => {
            list_folders_command(&store).await?;
        }
        Commands::ShowMail { email_id } => {
            show_mail_command(&store, &email_id).await?;
        }
        Commands::PendingDeliveries => {
            pending_deliveries_command(&store).await?;
        }
    }

    Ok(())
}

async fn send_mail_command(
    store: &crate::Store,
    to: String,
    cc: Option<String>,
    bcc: Option<String>,
    subject: String,
    body: String,
    from: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("📧 Composing email...");

    // Use provided from address or default
    let from_addr = from.unwrap_or_else(|| "test@example.com".to_string());

    // Build RFC 5322 email message
    let message = build_rfc5322_message(
        &from_addr,
        &to,
        cc.as_deref(),
        bcc.as_deref(),
        &subject,
        &body,
    )?;

    println!("📤 Sending via SMTP...");
    println!("From: {from_addr}");
    println!("To: {to}");
    if let Some(cc) = &cc {
        println!("CC: {cc}");
    }
    if let Some(bcc) = &bcc {
        println!("BCC: {bcc}");
    }
    println!("Subject: {subject}");
    println!("Body: {} chars", body.len());

    println!("\n📝 Generated RFC 5322 message:");
    println!("{message}");

    // Call smtp_receive to test the SMTP processing
    match store.smtp_receive(message.into_bytes()).await {
        Ok(email_id) => {
            println!("✅ Email processed with ID: {email_id}");
        }
        Err(e) => {
            println!("⚠️  SMTP processing: {e}");
            println!("   (This is expected - implementation in progress)");
        }
    }

    Ok(())
}

async fn list_mails_command(
    store: &crate::Store,
    folder: &str,
    limit: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("📬 Listing {limit} emails from folder: {folder}");

    // Use folder info to get email count
    let folder_info = store.imap_select_folder(folder).await?;
    println!(
        "📊 Folder stats: {} total, {} recent, {} unseen",
        folder_info.exists,
        folder_info.recent,
        folder_info.unseen.unwrap_or(0)
    );

    // TODO: Implement actual email listing
    println!("⚠️  Email listing not yet implemented");
    Ok(())
}

async fn list_folders_command(store: &crate::Store) -> Result<(), Box<dyn std::error::Error>> {
    println!("📁 Available folders:");

    let folders = store.imap_list_folders().await?;
    for folder in folders {
        println!("  📂 {folder}");
    }

    Ok(())
}

async fn show_mail_command(
    _store: &crate::Store,
    email_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("📧 Showing email: {email_id}");

    // TODO: Implement email content display
    println!("⚠️  Email display not yet implemented");
    Ok(())
}

async fn pending_deliveries_command(
    store: &crate::Store,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("⏳ Checking pending P2P deliveries...");

    let deliveries = store.get_pending_deliveries().await?;

    if deliveries.is_empty() {
        println!("✅ No pending deliveries");
    } else {
        println!("📋 {} pending deliveries:", deliveries.len());
        for delivery in deliveries {
            println!(
                "  📤 → {}: {} emails (oldest: {})",
                delivery.peer_id52,
                delivery.email_count,
                chrono::DateTime::from_timestamp(delivery.oldest_email_date, 0)
                    .unwrap_or_default()
                    .format("%Y-%m-%d %H:%M:%S")
            );
        }
    }

    Ok(())
}

/// Build a simple RFC 5322 email message for testing
fn build_rfc5322_message(
    from: &str,
    to: &str,
    cc: Option<&str>,
    bcc: Option<&str>,
    subject: &str,
    body: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let timestamp = chrono::Utc::now().format("%a, %d %b %Y %H:%M:%S +0000");
    let message_id = format!("<{}.fastn-mail@localhost>", chrono::Utc::now().timestamp());

    let mut message = String::new();
    message.push_str(&format!("From: {from}\r\n"));
    message.push_str(&format!("To: {to}\r\n"));

    if let Some(cc) = cc {
        message.push_str(&format!("CC: {cc}\r\n"));
    }
    if let Some(bcc) = bcc {
        message.push_str(&format!("BCC: {bcc}\r\n"));
    }

    message.push_str(&format!("Subject: {subject}\r\n"));
    message.push_str(&format!("Date: {timestamp}\r\n"));
    message.push_str(&format!("Message-ID: {message_id}\r\n"));
    message.push_str("MIME-Version: 1.0\r\n");
    message.push_str("Content-Type: text/plain; charset=utf-8\r\n");
    message.push_str("\r\n"); // Empty line separates headers from body
    message.push_str(body);
    message.push_str("\r\n");

    Ok(message)
}
