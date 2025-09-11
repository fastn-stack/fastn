//! IMAP client implementation
//!
//! Provides IMAP client functionality with dual verification testing support.

use crate::imap::ImapConfig;

/// IMAP client for connecting to and testing IMAP servers
pub struct ImapClient {
    config: ImapConfig,
}

impl ImapClient {
    pub fn new(config: ImapConfig) -> Self {
        Self { config }
    }

    /// Connect to IMAP server and perform basic authentication test
    pub async fn connect(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.connect_with_test_operations(false).await
    }

    /// Connect to IMAP server and perform comprehensive operation testing
    pub async fn connect_and_test(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.connect_with_test_operations(true).await
    }

    async fn connect_with_test_operations(
        &self,
        test_operations: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(feature = "net")]
        {
            println!(
                "🔗 Connecting to IMAP server {}:{}",
                self.config.host, self.config.port
            );
            println!("👤 Username: {}", self.config.username);
            println!(
                "🔐 STARTTLS: {}",
                if self.config.starttls {
                    "enabled"
                } else {
                    "disabled"
                }
            );

            // Connect to IMAP server
            let tcp_stream =
                tokio::net::TcpStream::connect((&*self.config.host, self.config.port)).await?;
            println!("✅ TCP connection established");

            // Wrap tokio stream to be compatible with futures-io traits
            let compat_stream = tokio_util::compat::TokioAsyncReadCompatExt::compat(tcp_stream);

            // Create IMAP client
            let client = async_imap::Client::new(compat_stream);
            println!("✅ IMAP client created");

            // Handle STARTTLS if requested
            let mut imap_session = if self.config.starttls {
                println!("🔐 STARTTLS requested but not yet implemented - using plain text");
                println!("📧 Using plain text connection");

                // Login with credentials (plain text)
                println!("🔑 Authenticating...");
                client
                    .login(&self.config.username, &self.config.password)
                    .await
                    .map_err(|(err, _)| err)?
            } else {
                println!("📧 Using plain text connection");

                // Login with credentials (plain text)
                println!("🔑 Authenticating...");
                client
                    .login(&self.config.username, &self.config.password)
                    .await
                    .map_err(|(err, _)| err)?
            };

            println!("✅ Authentication successful");

            if test_operations {
                self.run_test_operations(&mut imap_session).await?;
            }

            // Logout
            println!("👋 Logging out...");
            imap_session.logout().await?;
            println!("✅ IMAP connection test completed successfully");

            Ok(())
        }

        #[cfg(not(feature = "net"))]
        {
            println!("❌ Net feature not enabled. Compile with --features net");
            Err("Net feature required for IMAP commands".into())
        }
    }

    #[cfg(feature = "net")]
    async fn run_test_operations(
        &self,
        imap_session: &mut async_imap::Session<tokio_util::compat::Compat<tokio::net::TcpStream>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("🧪 Running basic operation tests...");

        // Test CAPABILITY
        println!("📋 Testing CAPABILITY...");
        let capabilities = imap_session.capabilities().await?;
        println!("✅ Server capabilities: {} items", capabilities.len());
        for cap in capabilities.iter().take(5) {
            println!("   - {:?}", cap); // Use debug formatting
        }
        if capabilities.len() > 5 {
            println!("   ... and {} more", capabilities.len() - 5);
        }

        // Test LIST (simplified - collect stream first)
        println!("📁 Testing LIST command...");
        use futures::stream::TryStreamExt; // Import TryStreamExt for try_collect
        let mailbox_list: Vec<_> = imap_session
            .list(Some(""), Some("*"))
            .await?
            .try_collect()
            .await?;
        println!("✅ Found {} mailboxes:", mailbox_list.len());
        for mailbox in mailbox_list.iter().take(5) {
            println!("   📂 {}", mailbox.name());
        }

        // Test SELECT command
        println!("📁 Testing SELECT INBOX command...");
        let mailbox = imap_session.select("INBOX").await?;
        println!(
            "✅ Selected INBOX: {} messages, {} recent, {} unseen",
            mailbox.exists,
            mailbox.recent,
            mailbox.unseen.unwrap_or(0)
        );

        println!("✅ All basic operations completed");
        Ok(())
    }
}

// TODO: Implement STARTTLS support with proper certificate verification
