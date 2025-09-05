//! Immediate P2P delivery test tool
//!
//! Creates a test email and immediately attempts P2P delivery
//! Bypasses the email delivery poller for direct P2P testing

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "test-p2p-delivery")]
#[command(about = "Test immediate P2P email delivery")]
struct Args {
    /// Sender account ID52
    #[arg(short, long)]
    from: String,

    /// Target recipient ID52  
    #[arg(short, long)]
    to: String,

    /// FASTN_HOME directory
    #[arg(long)]
    fastn_home: PathBuf,

    /// Test message subject
    #[arg(short, long, default_value = "P2P Test")]
    subject: String,

    /// Test message body
    #[arg(short, long, default_value = "Immediate P2P delivery test")]
    body: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("🧪 Immediate P2P delivery test (bypassing poller)");
    println!("From: {}", args.from);
    println!("To: {}", args.to);

    // Load account manager
    let account_manager = fastn_account::AccountManager::load(args.fastn_home.clone()).await?;

    // Parse account keys
    let from_key: fastn_id52::PublicKey = args.from.parse()?;

    // Create test email directly in mail store
    let from_email = format!("test@{}.com", args.from);
    let to_email = format!("test@{}.com", args.to);

    let test_email_content = format!(
        "From: {}\r\nTo: {}\r\nSubject: {}\r\nMessage-ID: <test-{}@localhost>\r\n\r\n{}",
        from_email,
        to_email,
        args.subject,
        chrono::Utc::now().timestamp(),
        args.body
    );

    println!("📧 Creating test email...");

    // Load the mail store directly
    let account_path = args.fastn_home.join("accounts").join(&args.from);
    let mail_store = fastn_mail::Store::load(&account_path).await?;

    let email_id = mail_store
        .smtp_receive(
            &from_email,
            std::slice::from_ref(&to_email),
            test_email_content.clone().into_bytes(),
        )
        .await?;

    println!("✅ Test email created with ID: {}", email_id);

    // Now trigger IMMEDIATE P2P delivery using fastn-net directly
    println!("🚀 Attempting immediate P2P delivery...");

    let start_time = std::time::Instant::now();

    // Load sender account to get secret key
    let sender_account = account_manager.find_account_by_alias(&from_key).await?;
    let sender_secret_key = sender_account.aliases().await[0].secret_key().clone();

    // Create target key
    let target_key: fastn_id52::PublicKey = args.to.parse()?;

    // Create message for P2P delivery
    let p2p_message = fastn_account::AccountToAccountMessage::Email {
        raw_message: test_email_content.into_bytes(),
        envelope_from: from_email.clone(),
        envelope_to: to_email.clone(),
    };

    // Attempt delivery using the same approach as working fastn-net-test
    match attempt_direct_delivery(sender_secret_key, &target_key, p2p_message).await {
        Ok(_) => {
            let elapsed = start_time.elapsed();
            println!(
                "✅ IMMEDIATE P2P delivery SUCCESS in {:.3}s",
                elapsed.as_secs_f64()
            );

            // Mark as delivered in database
            let conn = mail_store.connection().lock().await;
            conn.execute(
                "UPDATE fastn_email_delivery SET delivery_status = 'delivered' WHERE email_id = ?1",
                [&email_id],
            )?;
            println!("📝 Marked as delivered in database");
        }
        Err(e) => {
            let elapsed = start_time.elapsed();
            println!(
                "❌ IMMEDIATE P2P delivery FAILED in {:.3}s: {}",
                elapsed.as_secs_f64(),
                e
            );
        }
    }

    println!("🎯 Direct P2P test completed");
    Ok(())
}

/// Attempt P2P delivery using the exact same approach as working fastn-net-test
async fn attempt_direct_delivery(
    sender_key: fastn_id52::SecretKey,
    target_key: &fastn_id52::PublicKey,
    message: fastn_account::AccountToAccountMessage,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Create endpoint (like fastn-net-test)
    let endpoint = fastn_net::get_endpoint(sender_key).await?;
    println!("📡 Created endpoint");

    // Create peer stream coordination (like fastn-net-test)
    let peer_stream_senders =
        std::sync::Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new()));
    let graceful = fastn_net::Graceful::new();

    // Use fastn-net get_stream (exact same as working fastn-net-test)
    let (mut send, mut recv) = fastn_net::get_stream(
        endpoint,
        fastn_net::Protocol::AccountToAccount.into(),
        target_key.id52(),
        peer_stream_senders,
        graceful,
    )
    .await?;

    println!("✅ P2P stream established");

    // Send message as JSON
    let message_json = serde_json::to_string(&message)?;
    send.write_all(message_json.as_bytes()).await?;
    send.write_all(b"\n").await?;

    println!("📤 P2P message sent");

    // Wait for delivery response
    let response = fastn_net::next_string(&mut recv).await?;
    println!("📥 P2P response: {}", response);

    // Parse response
    let response_data: fastn_account::EmailDeliveryResponse = serde_json::from_str(&response)?;

    match response_data.status {
        fastn_account::DeliveryStatus::Accepted => {
            println!("✅ P2P delivery confirmed by recipient");
            Ok(())
        }
        fastn_account::DeliveryStatus::Rejected { reason } => {
            Err(format!("P2P delivery rejected: {}", reason).into())
        }
    }
}
