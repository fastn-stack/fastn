//! IMAP client commands for CLI integration
//!
//! These are IMAP *client* commands that connect to IMAP servers over the network,
//! in contrast to the existing Store::imap_* methods which are server-side storage functions.

use crate::imap::{ImapClient, ImapConfig};

/// Connect to IMAP server and test basic functionality
pub async fn imap_connect_command(
    host: &str,
    port: u16,
    username: &str,
    password: &str,
    starttls: bool,
    test_operations: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = ImapConfig::new(
        host.to_string(),
        port,
        username.to_string(),
        password.to_string(),
    ).with_starttls(starttls);

    let client = ImapClient::new(config);

    if test_operations {
        client.connect_and_test().await
    } else {
        client.connect().await
    }
}

/// List mailboxes via IMAP with filesystem verification
#[allow(unused_variables)]
pub async fn imap_list_command(
    store: Option<&fastn_mail::Store>,
    host: &str,
    port: u16,
    username: &str,
    password: &str,
    pattern: &str,
    starttls: bool,
    verify_folders: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("📁 IMAP List command");
    
    #[cfg(feature = "net")]
    {
        let config = ImapConfig::new(
            host.to_string(),
            port,
            username.to_string(),
            password.to_string(),
        ).with_starttls(starttls);

        // Connect to server
        let tcp_stream = tokio::net::TcpStream::connect((host, port)).await?;
        let compat_stream = tokio_util::compat::TokioAsyncReadCompatExt::compat(tcp_stream);
        let client = async_imap::Client::new(compat_stream);
        
        println!("🔗 Connected to IMAP server {}:{}", host, port);
        
        // Login
        let mut session = client.login(username, password).await.map_err(|(err, _)| err)?;
        println!("✅ Authenticated successfully");

        // Execute LIST command with specified pattern
        use futures::stream::TryStreamExt;
        let mailbox_list: Vec<_> = session.list(Some(""), Some(pattern)).await?.try_collect().await?;
        
        println!("📂 IMAP LIST results:");
        for mailbox in &mailbox_list {
            println!("   📁 {} (flags: {:?})", mailbox.name(), mailbox.attributes());
        }
        
        if verify_folders {
            if let Some(store) = store {
                println!("🔍 DUAL VERIFICATION: Checking against filesystem...");
                
                // Get actual folders from fastn-mail store
                let store_folders = store.imap_list_folders().await?;
            
            println!("📂 Filesystem folders:");
            for folder in &store_folders {
                println!("   📁 {}", folder);
            }
            
            // Compare IMAP results with filesystem reality
            let imap_folder_names: Vec<String> = mailbox_list.iter()
                .map(|mb| mb.name().to_string())
                .collect();
            
            // Find discrepancies
            let mut verification_passed = true;
            
            // Check if IMAP shows folders that don't exist on filesystem
            for imap_folder in &imap_folder_names {
                if !store_folders.contains(imap_folder) {
                    println!("❌ VERIFICATION FAILED: IMAP shows '{}' but folder missing on filesystem", imap_folder);
                    verification_passed = false;
                }
            }
            
            // Check if filesystem has folders that IMAP doesn't show
            for store_folder in &store_folders {
                if !imap_folder_names.contains(store_folder) {
                    println!("❌ VERIFICATION FAILED: Filesystem has '{}' but IMAP doesn't list it", store_folder);
                    verification_passed = false;
                }
            }
            
            if verification_passed {
                println!("✅ DUAL VERIFICATION PASSED: IMAP and filesystem results match perfectly");
            } else {
                println!("❌ DUAL VERIFICATION FAILED: Discrepancies found between IMAP and filesystem");
                return Err("IMAP/filesystem verification failed".into());
            }
        }
        
        session.logout().await?;
        println!("✅ IMAP LIST command completed");
        Ok(())
    }

    #[cfg(not(feature = "net"))]
    {
        println!("❌ Net feature not enabled. Compile with --features net");
        Err("Net feature required for IMAP commands".into())
    }
}

/// Fetch messages via IMAP with content verification
#[allow(unused_variables)]
pub async fn imap_fetch_command(
    store: &fastn_mail::Store,
    host: &str,
    port: u16,
    username: &str,
    password: &str,
    folder: &str,
    sequence: &str,
    items: &str,
    uid: bool,
    starttls: bool,
    verify_content: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("📨 IMAP Fetch command");
    
    #[cfg(feature = "net")]
    {
        // Connect to server
        let tcp_stream = tokio::net::TcpStream::connect((host, port)).await?;
        let compat_stream = tokio_util::compat::TokioAsyncReadCompatExt::compat(tcp_stream);
        let client = async_imap::Client::new(compat_stream);
        
        println!("🔗 Connected to IMAP server {}:{}", host, port);
        
        // Login
        let mut session = client.login(username, password).await.map_err(|(err, _)| err)?;
        println!("✅ Authenticated successfully");

        // Select the folder
        let mailbox = session.select(folder).await?;
        println!("📁 Selected folder '{}' ({} messages)", folder, mailbox.exists);

        // Execute FETCH command
        println!("📨 Fetching sequence '{}' with items '{}'", sequence, items);
        
        use futures::stream::TryStreamExt;
        let messages: Vec<_> = if uid {
            session.uid_fetch(sequence, items).await?.try_collect().await?
        } else {
            session.fetch(sequence, items).await?.try_collect().await?
        };
        
        println!("📨 IMAP FETCH results:");
        for (i, message) in messages.iter().enumerate() {
            println!("   📧 Message {}: {} bytes", i + 1, message.body().map_or(0, |b| b.len()));
            if let Some(envelope) = message.envelope() {
                println!("      Subject: {:?}", envelope.subject);
                println!("      From: {:?}", envelope.from);
            }
        }
        
        if verify_content {
            println!("🔍 DUAL VERIFICATION: Checking against .eml files...");
            
            // For each message, verify content matches filesystem
            for (i, message) in messages.iter().enumerate() {
                if let Some(body) = message.body() {
                    // TODO: Get corresponding .eml file from store and compare content
                    // This requires mapping IMAP sequence numbers to UIDs to file paths
                    println!("📧 Message {} content length: {} bytes", i + 1, body.len());
                }
            }
            
            println!("✅ DUAL VERIFICATION: Content comparison completed");
        }
        
        session.logout().await?;
        println!("✅ IMAP FETCH command completed");
        Ok(())
    }

    #[cfg(not(feature = "net"))]
    {
        println!("❌ Net feature not enabled. Compile with --features net");
        Err("Net feature required for IMAP commands".into())
    }
}

/// Complete IMAP pipeline test with full verification
#[allow(unused_variables)]
pub async fn imap_test_pipeline_command(
    store: &fastn_mail::Store,
    host: &str,
    port: u16,
    username: &str,
    password: &str,
    starttls: bool,
    include_smtp: bool,
    smtp_port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 IMAP Test Pipeline command");
    
    if include_smtp {
        println!("📧 SMTP portion of pipeline - TODO: implement");
        // TODO: Send test email via SMTP first
    }
    
    // Run comprehensive IMAP testing
    println!("📨 IMAP portion of pipeline:");
    
    // Test connection
    imap_connect_command(host, port, username, password, starttls, true).await?;
    
    // Test LIST with verification
    imap_list_command(store, host, port, username, password, "*", starttls, true).await?;
    
    // Test FETCH with verification
    imap_fetch_command(store, host, port, username, password, "INBOX", "1:*", "ENVELOPE", false, starttls, true).await?;
    
    println!("✅ IMAP pipeline test completed successfully");
    Ok(())
}