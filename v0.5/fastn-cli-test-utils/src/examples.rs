//! Pleasant fastn testing examples

#[cfg(test)]
mod tests {
    use crate::{FastnTestEnv, FastnRigCommand, FastnMailCommand};
    use std::time::Duration;

    #[tokio::test]
    async fn example_pleasant_two_peer_email() -> Result<(), Box<dyn std::error::Error>> {
        let mut env = FastnTestEnv::new("two-peer-test")?;
        
        // Create peers with automatic init
        let _sender = env.create_peer("sender").await?;
        let _receiver = env.create_peer("receiver").await?;
        
        // Start both peers
        env.start_peer("sender").await?;
        env.start_peer("receiver").await?;
        env.wait_for_startup().await?;
        
        // Send email with pleasant fluent API
        env.email()
            .from("sender")
            .to("receiver")
            .subject("P2P Test")
            .body("Testing pleasant API")
            .send()
            .await?
            .expect_success()?
            .wait_for_delivery(Duration::from_secs(30))
            .await?;
        
        println!("✅ Email delivered successfully with pleasant API");
        
        // Automatic cleanup on drop
        Ok(())
    }
    
    #[tokio::test] 
    async fn example_pleasant_cli_commands() -> Result<(), Box<dyn std::error::Error>> {
        let mut env = FastnTestEnv::new("cli-test")?;
        let peer = env.create_peer("test-peer").await?;
        
        // Test all fastn-rig commands with pleasant API
        
        // Status command
        let status_output = FastnRigCommand::new()
            .home(&peer.home_path)
            .skip_keyring(true)
            .status()
            .execute()
            .await?
            .expect_success()?;
        
        assert!(status_output.contains_output("Rig Status"));
        assert!(status_output.contains_output(&peer.account_id));
        
        // Entities command  
        let entities_output = FastnRigCommand::new()
            .home(&peer.home_path)
            .skip_keyring(true)
            .entities()
            .execute()
            .await?
            .expect_success()?;
            
        assert!(entities_output.contains_output("Entities"));
        assert!(entities_output.contains_output("(rig)"));
        
        println!("✅ All CLI commands work with pleasant API");
        Ok(())
    }
    
    #[tokio::test]
    async fn example_pleasant_mail_operations() -> Result<(), Box<dyn std::error::Error>> {
        let mut env = FastnTestEnv::new("mail-test")?;
        
        let sender = env.create_peer("sender").await?;
        let receiver = env.create_peer("receiver").await?;
        
        env.start_peer("sender").await?;
        env.start_peer("receiver").await?;
        env.wait_for_startup().await?;
        
        // Send email with all parameters explicit
        let send_result = FastnMailCommand::new()
            .send_mail()
            .from(&sender.email_address())
            .to(&receiver.inbox_address())
            .subject("Explicit Test")
            .body("Testing explicit parameter setting")
            .smtp_port(sender.smtp_port)
            .password(&sender.password)
            .home(&sender.home_path)
            .send()
            .await?
            .expect_success()?;
        
        assert!(send_result.contains_output("Email sent successfully"));
        
        // Or send with peer-to-peer helper (much more pleasant)
        env.send_email("sender", "receiver", "P2P Helper", "Using peer helper").await?
            .expect_success()?;
        
        println!("✅ Both explicit and helper email sending work");
        Ok(())
    }
    
    #[tokio::test]
    async fn example_pleasant_error_handling() -> Result<(), Box<dyn std::error::Error>> {
        let env = FastnTestEnv::new("error-test")?;
        let temp_path = env.temp_dir.path().join("uninitialized");
        
        // Test error handling with pleasant API
        let result = FastnRigCommand::new()
            .home(&temp_path)
            .skip_keyring(true)
            .status()
            .execute()
            .await?
            .expect_failure()?;  // We expect this to fail
        
        assert!(result.contains_output("Failed to load rig") || result.contains_output("KeyLoading"));
        
        println!("✅ Error handling works pleasantly");
        Ok(())
    }
}