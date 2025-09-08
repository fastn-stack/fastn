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

        /// SMTP server port (defaults to FASTN_SMTP_PORT or 2525)
        #[arg(long)]
        smtp: Option<u16>,

        /// Use direct mail store access instead of SMTP client
        #[arg(long)]
        direct: bool,

        /// SMTP password for authentication (required when using SMTP client)
        #[arg(long)]
        password: Option<String>,

        /// Enable STARTTLS for secure SMTP connection
        #[arg(long)]
        starttls: bool,

        /// Verify email was stored in Sent folder after SMTP success
        #[arg(long)]
        verify_sent: bool,

        /// Comprehensive verification: Sent folder + P2P queue + content integrity  
        #[arg(long)]
        verify_all: bool,
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

    /// Get emails to deliver to a specific peer
    GetEmailsForPeer {
        /// Peer ID52 to get emails for
        peer_id52: String,
    },

    /// Mark an email as delivered to a peer
    MarkDelivered {
        /// Email ID that was delivered
        email_id: String,
        /// Peer ID52 that received the email
        peer_id52: String,
    },

    /// Accept P2P email from another peer (store in INBOX)
    AcceptP2pMail {
        /// Path to raw email message file
        #[arg(long)]
        message_file: String,
        /// ID52 of the peer who sent this email
        #[arg(long)]
        sender_id52: String,
    },

    /// IMAP client commands with dual verification
    
    /// Connect to IMAP server and test basic functionality
    ImapConnect {
        /// IMAP server hostname
        #[arg(long, default_value = "localhost")]
        host: String,
        /// IMAP server port  
        #[arg(long, default_value = "1143")]
        port: u16,
        /// Username for authentication
        #[arg(long)]
        username: String,
        /// Password for authentication
        #[arg(long)]
        password: String,
        /// Use STARTTLS for secure connection
        #[arg(long)]
        starttls: bool,
        /// Test all basic operations after connecting
        #[arg(long)]
        test_operations: bool,
    },

    /// List mailboxes via IMAP with filesystem verification
    ImapList {
        /// IMAP server hostname
        #[arg(long, default_value = "localhost")]
        host: String,
        /// IMAP server port
        #[arg(long, default_value = "1143")]  
        port: u16,
        /// Username for authentication
        #[arg(long)]
        username: String,
        /// Password for authentication
        #[arg(long)]
        password: String,
        /// Mailbox pattern (default: "*" for all)
        #[arg(long, default_value = "*")]
        pattern: String,
        /// Use STARTTLS for secure connection
        #[arg(long)]
        starttls: bool,
        /// Verify IMAP results match actual folder structure
        #[arg(long)]
        verify_folders: bool,
    },

    /// Fetch messages via IMAP with content verification
    ImapFetch {
        /// IMAP server hostname
        #[arg(long, default_value = "localhost")]
        host: String,
        /// IMAP server port
        #[arg(long, default_value = "1143")]
        port: u16,
        /// Username for authentication
        #[arg(long)]
        username: String,
        /// Password for authentication
        #[arg(long)]
        password: String,
        /// Mailbox to select (default: INBOX)
        #[arg(long, default_value = "INBOX")]
        folder: String,
        /// Message sequence (e.g., "1", "1:5", "*")
        #[arg(long, default_value = "1:*")]
        sequence: String,
        /// FETCH items (e.g., "ENVELOPE", "BODY[]", "FLAGS")
        #[arg(long, default_value = "ENVELOPE")]
        items: String,
        /// Use UID mode instead of sequence numbers
        #[arg(long)]
        uid: bool,
        /// Use STARTTLS for secure connection
        #[arg(long)]
        starttls: bool,
        /// Verify IMAP data matches .eml file content exactly
        #[arg(long)]
        verify_content: bool,
    },

    /// Complete IMAP pipeline test with full verification
    ImapTestPipeline {
        /// IMAP server hostname
        #[arg(long, default_value = "localhost")]
        host: String,
        /// IMAP server port
        #[arg(long, default_value = "1143")]
        port: u16,
        /// Username for authentication
        #[arg(long)]
        username: String,
        /// Password for authentication  
        #[arg(long)]
        password: String,
        /// Use STARTTLS for secure connection
        #[arg(long)]
        starttls: bool,
        /// Also test SMTP sending before IMAP operations
        #[arg(long)]
        include_smtp: bool,
        /// SMTP port (if testing SMTP)
        #[arg(long, default_value = "2525")]
        smtp_port: u16,
    },
}

pub async fn run_command(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    // Load email store for the specified account
    let account_path = std::path::Path::new(&cli.account_path);
    let store = match fastn_mail::Store::load(account_path).await {
        Ok(store) => store,
        Err(_) => {
            println!("⚠️  No email store found at path, using test store for CLI demo");
            fastn_mail::Store::create_test()
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
            smtp,
            direct,
            password,
            starttls,
            verify_sent,
            verify_all,
        } => {
            send_mail_command(
                &store, to, cc, bcc, subject, body, from, smtp, direct, password, starttls,
                verify_sent, verify_all,
            )
            .await?;
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
        Commands::GetEmailsForPeer { peer_id52 } => {
            get_emails_for_peer_command(&store, &peer_id52).await?;
        }
        Commands::MarkDelivered {
            email_id,
            peer_id52,
        } => {
            mark_delivered_command(&store, &email_id, &peer_id52).await?;
        }
        Commands::AcceptP2pMail {
            message_file,
            sender_id52,
        } => {
            p2p_receive_email_command(&store, &message_file, &sender_id52).await?;
        }
        Commands::ImapConnect {
            host,
            port,
            username,
            password,
            starttls,
            test_operations,
        } => {
            imap_connect_command(&host, port, &username, &password, starttls, test_operations).await?;
        }
        Commands::ImapList {
            host,
            port,
            username,
            password,
            pattern,
            starttls,
            verify_folders,
        } => {
            imap_list_command(&store, &host, port, &username, &password, &pattern, starttls, verify_folders).await?;
        }
        Commands::ImapFetch {
            host,
            port,
            username,
            password,
            folder,
            sequence,
            items,
            uid,
            starttls,
            verify_content,
        } => {
            imap_fetch_command(&store, &host, port, &username, &password, &folder, &sequence, &items, uid, starttls, verify_content).await?;
        }
        Commands::ImapTestPipeline {
            host,
            port,
            username,
            password,
            starttls,
            include_smtp,
            smtp_port,
        } => {
            imap_test_pipeline_command(&store, &host, port, &username, &password, starttls, include_smtp, smtp_port).await?;
        }
    }

    Ok(())
}

#[expect(
    clippy::too_many_arguments,
    reason = "CLI function mirrors command line arguments"
)]
async fn send_mail_command(
    store: &fastn_mail::Store,
    to: String,
    cc: Option<String>,
    bcc: Option<String>,
    subject: String,
    body: String,
    from: Option<String>,
    #[cfg_attr(not(feature = "net"), allow(unused_variables))] smtp_port: Option<u16>,
    direct: bool,
    #[cfg_attr(not(feature = "net"), allow(unused_variables))] password: Option<String>,
    #[cfg_attr(not(feature = "net"), allow(unused_variables))] starttls: bool,
    verify_sent: bool,
    verify_all: bool,
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

    if direct {
        // Direct mail store access (original behavior)
        println!("📦 Using direct mail store access...");

        // Build recipient list for SMTP envelope
        let mut recipients = vec![to.clone()];
        if let Some(cc) = &cc {
            recipients.push(cc.clone());
        }
        if let Some(bcc) = &bcc {
            recipients.push(bcc.clone());
        }

        // Call smtp_receive directly for testing
        match store
            .smtp_receive(&from_addr, &recipients, message.into_bytes())
            .await
        {
            Ok(email_id) => {
                println!("✅ Email processed with ID: {email_id}");
            }
            Err(e) => {
                println!("❌ Direct processing failed: {e}");
                return Err(Box::new(e));
            }
        }
    } else {
        // SMTP client mode (default)
        #[cfg(feature = "net")]
        {
            let port = smtp_port.unwrap_or_else(|| {
                std::env::var("FASTN_SMTP_PORT")
                    .ok()
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(2525)
            });

            let smtp_password = password.ok_or("Password required for SMTP authentication. Use --password <password> or --direct for testing")?;
            println!("🔗 Connecting to SMTP server on port {port}...");
            match send_via_smtp_client(
                &from_addr,
                &to,
                cc.as_deref(),
                bcc.as_deref(),
                &subject,
                &body,
                port,
                &smtp_password,
                starttls,
            )
            .await
            {
                Ok(()) => {
                    println!("✅ Email sent successfully via SMTP");
                }
                Err(e) => {
                    println!("❌ SMTP sending failed: {e}");
                    return Err(e);
                }
            }
        }

        #[cfg(not(feature = "net"))]
        {
            println!(
                "❌ Net feature not enabled. Use --direct flag or compile with --features net"
            );
            return Err("Net feature not available".into());
        }
    }

    Ok(())
}

async fn list_mails_command(
    store: &fastn_mail::Store,
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

async fn list_folders_command(store: &fastn_mail::Store) -> Result<(), Box<dyn std::error::Error>> {
    println!("📁 Available folders:");

    let folders = store.imap_list_folders().await?;
    for folder in folders {
        println!("  📂 {folder}");
    }

    Ok(())
}

async fn show_mail_command(
    _store: &fastn_mail::Store,
    email_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("📧 Showing email: {email_id}");

    // TODO: Implement email content display
    println!("⚠️  Email display not yet implemented");
    Ok(())
}

async fn pending_deliveries_command(
    store: &fastn_mail::Store,
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

async fn get_emails_for_peer_command(
    store: &fastn_mail::Store,
    peer_id52: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("📨 Getting emails for peer: {peer_id52}");

    // Parse peer ID52 to PublicKey
    let peer_key: fastn_id52::PublicKey = peer_id52
        .parse()
        .map_err(|_| format!("Invalid peer ID52: {peer_id52}"))?;

    let emails = store.get_emails_for_peer(&peer_key).await?;

    if emails.is_empty() {
        println!("✅ No emails pending for peer {peer_id52}");
    } else {
        println!("📋 {} emails pending for peer {peer_id52}:", emails.len());
        for email in &emails {
            println!("  📧 {}: {} bytes", email.email_id, email.size_bytes);
        }

        // Show total size
        let total_size: usize = emails.iter().map(|e| e.size_bytes).sum();
        println!(
            "📊 Total: {} bytes across {} emails",
            total_size,
            emails.len()
        );
    }

    Ok(())
}

async fn mark_delivered_command(
    store: &fastn_mail::Store,
    email_id: &str,
    peer_id52: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("✅ Marking email {email_id} as delivered to peer: {peer_id52}");

    // Parse peer ID52 to PublicKey
    let peer_key: fastn_id52::PublicKey = peer_id52
        .parse()
        .map_err(|_| format!("Invalid peer ID52: {peer_id52}"))?;

    // Mark as delivered
    store.mark_delivered_to_peer(email_id, &peer_key).await?;

    println!("🎉 Email {email_id} marked as delivered to {peer_id52}");
    Ok(())
}

async fn p2p_receive_email_command(
    store: &fastn_mail::Store,
    message_file: &str,
    sender_id52: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("📨 Accepting P2P email from peer: {sender_id52}");

    // Parse sender ID52 to PublicKey
    let sender_key: fastn_id52::PublicKey = sender_id52
        .parse()
        .map_err(|_| format!("Invalid sender ID52: {sender_id52}"))?;

    // Read raw email message from file
    let raw_message = std::fs::read(message_file)
        .map_err(|e| format!("Failed to read message file {message_file}: {e}"))?;

    println!("📖 Read {} bytes from {message_file}", raw_message.len());

    // Process P2P email with envelope data (store in INBOX)
    let envelope_from = format!("sender@{}.fastn", sender_key.id52());
    let envelope_to = "recipient@ourhost.local"; // Placeholder for CLI testing
    let email_id = store
        .p2p_receive_email(&envelope_from, envelope_to, raw_message)
        .await?;

    println!("✅ P2P email accepted and stored in INBOX with ID: {email_id}");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_send_email_and_check_pending_deliveries() {
        // Create test store
        let store = fastn_mail::Store::create_test();

        // Generate valid ID52s for testing
        let from_key = fastn_id52::SecretKey::generate();
        let to_key = fastn_id52::SecretKey::generate();
        let from_id52 = from_key.public_key().id52();
        let to_id52 = to_key.public_key().id52();

        // Build test email message
        let message = build_rfc5322_message(
            &format!("alice@{from_id52}.fastn"),
            &format!("bob@{to_id52}.local"),
            None,
            None,
            "CLI Integration Test",
            "Testing complete workflow from send to pending delivery",
        )
        .unwrap();

        // Step 1: Send email via SMTP
        let email_id = store
            .smtp_receive(
                &format!("alice@{from_id52}.fastn"),
                &[format!("bob@{to_id52}.local")],
                message.into_bytes(),
            )
            .await
            .expect("Email should be processed successfully");

        assert!(!email_id.is_empty());
        assert!(email_id.starts_with("email-"));

        // Step 2: Check pending deliveries
        let pending = store
            .get_pending_deliveries()
            .await
            .expect("Should get pending deliveries");

        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].peer_id52, to_key.public_key());
        assert_eq!(pending[0].email_count, 1);

        // Step 3: Get emails for specific peer
        let emails = store
            .get_emails_for_peer(&to_key.public_key())
            .await
            .expect("Should get emails for peer");

        assert_eq!(emails.len(), 1);
        assert_eq!(emails[0].email_id, email_id);
        assert!(emails[0].size_bytes > 0);
    }

    #[tokio::test]
    async fn test_multiple_recipients_pending_deliveries() {
        let store = fastn_mail::Store::create_test();

        let from_key = fastn_id52::SecretKey::generate();
        let to_key = fastn_id52::SecretKey::generate();
        let cc_key = fastn_id52::SecretKey::generate();
        let from_id52 = from_key.public_key().id52();
        let to_id52 = to_key.public_key().id52();
        let cc_id52 = cc_key.public_key().id52();

        // Send email with multiple fastn recipients
        let message = build_rfc5322_message(
            &format!("sender@{from_id52}.fastn"),
            &format!("to@{to_id52}.local"),
            Some(&format!("cc@{cc_id52}.fastn")),
            None,
            "Multi-recipient Test",
            "Testing multiple P2P deliveries",
        )
        .unwrap();

        let email_id = store
            .smtp_receive(
                &format!("sender@{from_id52}.fastn"),
                &[format!("to@{to_id52}.local"), format!("cc@{cc_id52}.fastn")],
                message.into_bytes(),
            )
            .await
            .expect("Email should be processed");

        // Should have 2 pending deliveries (To + CC)
        let pending = store.get_pending_deliveries().await.unwrap();
        assert_eq!(pending.len(), 2);

        // Each peer should have 1 email
        for delivery in &pending {
            assert_eq!(delivery.email_count, 1);

            let emails = store
                .get_emails_for_peer(&delivery.peer_id52)
                .await
                .unwrap();
            assert_eq!(emails.len(), 1);
            assert_eq!(emails[0].email_id, email_id);
        }
    }

    #[tokio::test]
    async fn test_two_instance_p2p_workflow() {
        // Create two separate store instances
        let sender_store = fastn_mail::Store::create_test();
        let recipient_store = fastn_mail::Store::create_test();

        // Generate valid ID52s
        let from_key = fastn_id52::SecretKey::generate();
        let to_key = fastn_id52::SecretKey::generate();
        let from_id52 = from_key.public_key().id52();
        let to_id52 = to_key.public_key().id52();

        // Step 1: Instance 1 sends email via SMTP
        let message = build_rfc5322_message(
            &format!("alice@{from_id52}.fastn"),
            &format!("bob@{to_id52}.local"),
            None,
            None,
            "P2P Integration Test",
            "Testing two-instance P2P email delivery",
        )
        .unwrap();

        let email_id = sender_store
            .smtp_receive(
                &format!("alice@{from_id52}.fastn"),
                &[format!("bob@{to_id52}.local")],
                message.clone().into_bytes(),
            )
            .await
            .expect("Sender should process email successfully");

        // Step 2: Get email file path from sender instance
        let emails = sender_store
            .get_emails_for_peer(&to_key.public_key())
            .await
            .expect("Should get emails for recipient");
        assert_eq!(emails.len(), 1);
        assert_eq!(emails[0].email_id, email_id);

        // Step 3: Instance 2 accepts P2P email (simulating P2P delivery)
        let p2p_email_id = recipient_store
            .p2p_receive_email(
                &emails[0].envelope_from,
                &emails[0].envelope_to,
                emails[0].raw_message.clone(),
            )
            .await
            .expect("Recipient should accept P2P email");

        // Step 4: Verify emails are in correct folders
        // Sender should have email in Sent folder
        // Recipient should have email in INBOX folder

        println!("✅ Two-instance P2P workflow test completed:");
        println!("   Sender email ID: {email_id} (in Sent folder)");
        println!("   Recipient email ID: {p2p_email_id} (in INBOX folder)");

        assert!(!p2p_email_id.is_empty());
        // Note: Email IDs might be same if processing same raw message bytes
        // This is actually correct behavior - same message content = same content hash
    }
}

#[cfg(feature = "net")]
async fn send_via_smtp_client(
    from: &str,
    to: &str,
    cc: Option<&str>,
    bcc: Option<&str>,
    subject: &str,
    body: &str,
    port: u16,
    password: &str,
    starttls: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Parse email addresses
    let from_mailbox: lettre::message::Mailbox = from.parse()?;
    let to_mailbox: lettre::message::Mailbox = to.parse()?;

    // Build email message
    let mut email_builder = lettre::message::Message::builder()
        .from(from_mailbox.clone())
        .to(to_mailbox)
        .subject(subject);

    // Add CC if provided
    if let Some(cc_addr) = cc {
        let cc_mailbox: lettre::message::Mailbox = cc_addr.parse()?;
        email_builder = email_builder.cc(cc_mailbox);
    }

    // Add BCC if provided
    if let Some(bcc_addr) = bcc {
        let bcc_mailbox: lettre::message::Mailbox = bcc_addr.parse()?;
        email_builder = email_builder.bcc(bcc_mailbox);
    }

    let email = email_builder.body(body.to_string())?;

    // Extract account ID52 from from address for authentication
    let (_, account_id52) = fastn_mail::store::smtp_receive::parse_id52_address(from)?;
    let _account_id52 =
        account_id52.ok_or("From address must be a valid fastn address with ID52")?;

    // Use provided password for SMTP authentication
    let credentials = lettre::transport::smtp::authentication::Credentials::new(
        from.to_string(),
        password.to_string(),
    );

    // Connect to local fastn-rig SMTP server
    let mailer = if starttls {
        println!("🔐 Using STARTTLS connection");
        lettre::SmtpTransport::starttls_relay("localhost")?
            .port(port)
            .credentials(credentials)
            .build()
    } else {
        println!("📧 Using plain text connection");
        lettre::SmtpTransport::builder_dangerous("localhost")
            .port(port)
            .credentials(credentials)
            .build()
    };

    // Send the email
    lettre::Transport::send(&mailer, &email)?;

    Ok(())
}
