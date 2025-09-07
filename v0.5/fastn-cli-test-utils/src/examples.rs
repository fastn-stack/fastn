//! Usage examples for fastn-cli-test-utils

#[cfg(test)]
mod tests {
    use crate::{TestScenario, CliConfig};
    use std::time::Duration;
    
    #[tokio::test]
    async fn example_fluent_two_peer_email() -> Result<(), Box<dyn std::error::Error>> {
        TestScenario::new("email-delivery")
            .with_peers(&["sender", "receiver"])
            .with_smtp_ports(&[2525, 2526])
            .without_keyring()
            .with_timeout(Duration::from_secs(30))
            .run(|mut test| async move {
                // Start peers
                test.start_all_peers().await?;
                test.wait_for_startup().await?;
                
                // Send email with fluent API
                test.email()
                    .from("sender")
                    .to("receiver") 
                    .subject("P2P Test")
                    .body("Testing SMTP to P2P delivery")
                    .send()
                    .await?
                    .expect_success()?
                    .wait_for_delivery(Duration::from_secs(30))
                    .await?;
                
                println!("✅ Email delivery test completed successfully");
                Ok(())
            })
            .await
    }
    
    #[tokio::test]
    async fn example_simple_init_test() -> Result<(), Box<dyn std::error::Error>> {
        TestScenario::new("simple-init")
            .with_peers(&["test-peer"])
            .without_keyring()
            .run(|test| async move {
                let peer = test.peer("test-peer")?;
                println!("Created peer: {}", peer.account_id);
                
                // Peer is automatically initialized during creation
                assert!(peer.home_path.exists());
                assert!(!peer.account_id.is_empty());
                assert!(!peer.password.is_empty());
                
                Ok(())
            })
            .await
    }
    
    #[tokio::test]
    async fn example_custom_config() -> Result<(), Box<dyn std::error::Error>> {
        let custom_config = CliConfig {
            pre_build: false,  // Build on demand instead of pre-build
            cleanup_on_drop: true,
            skip_keyring: true,
            default_timeout: Duration::from_secs(60),
            smtp_port_range: 3000..3100,  // Custom port range
        };
        
        TestScenario::new("custom-config")
            .with_config(custom_config)
            .with_peers(&["peer1", "peer2", "peer3"])
            .with_smtp_ports(&[3001, 3002, 3003])
            .run(|mut test| async move {
                // Start only some peers
                test.start_peer("peer1").await?;
                test.start_peer("peer3").await?;
                
                test.wait_for_startup().await?;
                
                // Send emails between running peers  
                test.email()
                    .from("peer1")
                    .to("peer3")
                    .subject("Cross-peer test")
                    .body("Testing selective peer startup")
                    .send()
                    .await?
                    .expect_success()?;
                
                Ok(())
            })
            .await
    }
    
    #[tokio::test]
    async fn example_multi_message_test() -> Result<(), Box<dyn std::error::Error>> {
        TestScenario::new("multi-message")
            .with_peers(&["hub", "client1", "client2"])
            .run(|mut test| async move {
                test.start_all_peers().await?;
                test.wait_for_startup().await?;
                
                // Send multiple emails in sequence
                for i in 1..=5 {
                    test.email()
                        .from("client1")
                        .to("hub")
                        .subject(&format!("Message {i}"))
                        .body(&format!("Content of message {i}"))
                        .send()
                        .await?
                        .expect_success()?;
                        
                    test.email()
                        .from("client2") 
                        .to("hub")
                        .subject(&format!("Response {i}"))
                        .body(&format!("Response to message {i}"))
                        .send()
                        .await?
                        .expect_success()?;
                }
                
                println!("✅ Sent 10 emails successfully");
                Ok(())
            })
            .await
    }
}