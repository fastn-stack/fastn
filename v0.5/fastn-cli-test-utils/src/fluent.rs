//! Fluent builder API for pleasant test writing

use crate::{TestEnvironment, CliConfig, PeerHandle, CommandOutput};
use std::time::Duration;

/// Fluent builder for test scenarios
#[derive(Debug)]
pub struct TestScenario {
    name: String,
    config: CliConfig,
    peers: Vec<String>,
    smtp_ports: Vec<u16>,
}

impl TestScenario {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            config: CliConfig::default(),
            peers: Vec::new(),
            smtp_ports: vec![2525, 2526, 2527, 2528], // Default ports
        }
    }
    
    /// Configure the test environment
    pub fn with_config(mut self, config: CliConfig) -> Self {
        self.config = config;
        self
    }
    
    /// Add peers to the test
    pub fn with_peers(mut self, peer_names: &[&str]) -> Self {
        self.peers = peer_names.iter().map(|s| s.to_string()).collect();
        self
    }
    
    /// Configure SMTP ports for peers
    pub fn with_smtp_ports(mut self, ports: &[u16]) -> Self {
        self.smtp_ports = ports.to_vec();
        self
    }
    
    /// Disable keyring (default: disabled)
    pub fn without_keyring(mut self) -> Self {
        self.config.skip_keyring = true;
        self
    }
    
    /// Enable pre-building (default: enabled)
    pub fn with_pre_build(mut self) -> Self {
        self.config.pre_build = true;
        self
    }
    
    /// Set startup timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.config.default_timeout = timeout;
        self
    }
    
    /// Execute the test scenario
    pub async fn run<F, Fut>(self, test_fn: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnOnce(TestRunner) -> Fut,
        Fut: std::future::Future<Output = Result<(), Box<dyn std::error::Error>>>,
    {
        let mut env = TestEnvironment::with_config(&self.name, self.config).await?;
        
        // Create all requested peers
        let mut peers = Vec::new();
        for (i, peer_name) in self.peers.iter().enumerate() {
            let peer = env.create_peer(peer_name).await?;
            peers.push(peer);
        }
        
        // Create test runner
        let runner = TestRunner {
            env,
            peers,
            smtp_ports: self.smtp_ports,
        };
        
        // Execute user test function
        test_fn(runner).await
    }
}

/// Fluent test execution context
#[derive(Debug)]
pub struct TestRunner {
    env: TestEnvironment,
    peers: Vec<PeerHandle>,
    smtp_ports: Vec<u16>,
}

impl TestRunner {
    /// Get peer by name
    pub fn peer(&self, name: &str) -> Result<&PeerHandle, Box<dyn std::error::Error>> {
        self.peers.iter()
            .find(|p| p.name == name)
            .ok_or_else(|| format!("Peer '{name}' not found").into())
    }
    
    /// Get mutable peer by name
    pub fn peer_mut(&mut self, name: &str) -> Result<&mut PeerHandle, Box<dyn std::error::Error>> {
        self.peers.iter_mut()
            .find(|p| p.name == name)
            .ok_or_else(|| format!("Peer '{name}' not found").into())
    }
    
    /// Start all peers
    pub async fn start_all_peers(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        for (i, peer) in self.peers.iter_mut().enumerate() {
            let port = self.smtp_ports.get(i).copied().unwrap_or(2525 + i as u16);
            self.env.start_peer(peer, port).await?;
        }
        Ok(())
    }
    
    /// Start specific peer
    pub async fn start_peer(&mut self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let peer_index = self.peers.iter().position(|p| p.name == name)
            .ok_or_else(|| format!("Peer '{name}' not found"))?;
        let port = self.smtp_ports.get(peer_index).copied().unwrap_or(2525 + peer_index as u16);
        let peer = &mut self.peers[peer_index];
        self.env.start_peer(peer, port).await?;
        Ok(())
    }
    
    /// Wait for system startup
    pub async fn wait_for_startup(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.env.wait_for_startup().await?;
        Ok(())
    }
    
    /// Send email between peers
    pub async fn send_email(&mut self, from: &str, to: &str, subject: &str, body: &str) -> Result<EmailResult, Box<dyn std::error::Error>> {
        let from_peer = self.peer(from)?.clone();
        let to_peer = self.peer(to)?.clone();
        
        let output = self.env.send_email(&from_peer, &to_peer, subject, body).await?;
        
        Ok(EmailResult {
            output,
            from: from_peer,
            to: to_peer,
        })
    }
    
    /// Create fluent email builder
    pub fn email(&mut self) -> EmailBuilder {
        EmailBuilder::new(self)
    }
    
    /// Monitor for email delivery with timeout
    pub async fn wait_for_delivery(&self, from: &str, to: &str, timeout: Duration) -> Result<bool, Box<dyn std::error::Error>> {
        // Implementation would check using test_utils binary for email counts
        // This is a placeholder for the monitoring logic
        tokio::time::sleep(timeout).await;
        Ok(true) // Simplified for now
    }
}

/// Fluent email sending builder
pub struct EmailBuilder<'a> {
    runner: &'a mut TestRunner,
    from: Option<String>,
    to: Option<String>,
    subject: String,
    body: String,
}

impl<'a> EmailBuilder<'a> {
    fn new(runner: &'a mut TestRunner) -> Self {
        Self {
            runner,
            from: None,
            to: None,
            subject: "Test Email".to_string(),
            body: "Test Body".to_string(),
        }
    }
    
    pub fn from(mut self, peer_name: &str) -> Self {
        self.from = Some(peer_name.to_string());
        self
    }
    
    pub fn to(mut self, peer_name: &str) -> Self {
        self.to = Some(peer_name.to_string());
        self
    }
    
    pub fn subject(mut self, subject: &str) -> Self {
        self.subject = subject.to_string();
        self
    }
    
    pub fn body(mut self, body: &str) -> Self {
        self.body = body.to_string();
        self
    }
    
    pub async fn send(self) -> Result<EmailResult, Box<dyn std::error::Error>> {
        let from = self.from.ok_or("From peer not specified")?;
        let to = self.to.ok_or("To peer not specified")?;
        
        self.runner.send_email(&from, &to, &self.subject, &self.body).await
    }
}

/// Result of an email sending operation
#[derive(Debug)]
pub struct EmailResult {
    pub output: CommandOutput,
    pub from: PeerHandle,
    pub to: PeerHandle,
}

impl EmailResult {
    /// Expect the email sending to succeed
    pub fn expect_success(mut self) -> Result<Self, Box<dyn std::error::Error>> {
        self.output = self.output.expect_success()?;
        Ok(self)
    }
    
    /// Wait for P2P delivery to complete
    pub async fn wait_for_delivery(self, timeout: Duration) -> Result<Self, Box<dyn std::error::Error>> {
        tokio::time::sleep(timeout).await; // Simplified - real implementation would monitor
        Ok(self)
    }
    
    /// Check if email contains expected content
    pub fn contains_output(&self, text: &str) -> bool {
        self.output.contains_output(text)
    }
}