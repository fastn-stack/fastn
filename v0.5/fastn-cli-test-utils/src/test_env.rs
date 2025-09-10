//! Complete fastn test environment with peer management

use crate::{CommandOutput, FastnCliConfig, FastnMailCommand, FastnRigCommand};
use std::path::PathBuf;
use std::time::Duration;

/// Complete test environment for fastn testing with peer management
pub struct FastnTestEnv {
    temp_dir: tempfile::TempDir,
    peers: Vec<PeerHandle>,
    config: FastnCliConfig,
    next_smtp_port: u16,
}

impl FastnTestEnv {
    /// Create new test environment
    pub fn new(test_name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let temp_dir = tempfile::Builder::new()
            .prefix(&format!("fastn-test-{test_name}-"))
            .tempdir()?;

        Ok(Self {
            temp_dir,
            peers: Vec::new(),
            config: FastnCliConfig::default(),
            next_smtp_port: 2525,
        })
    }

    /// Create with custom configuration
    pub fn with_config(
        test_name: &str,
        config: FastnCliConfig,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut env = Self::new(test_name)?;
        env.next_smtp_port = config.smtp_port_range.start;
        env.config = config;
        Ok(env)
    }

    /// Create a new peer using fastn-rig init
    pub async fn create_peer(
        &mut self,
        name: &str,
    ) -> Result<&PeerHandle, Box<dyn std::error::Error>> {
        let peer_home = self.temp_dir.path().join(name);

        let output = FastnRigCommand::new()
            .home(&peer_home)
            .skip_keyring(self.config.skip_keyring)
            .init()
            .execute()
            .await?
            .expect_success()?;

        let account_id = output.extract_account_id()?;
        let password = output.extract_password()?;

        let peer = PeerHandle {
            name: name.to_string(),
            home_path: peer_home,
            account_id,
            password,
            smtp_port: self.next_smtp_port,
            process: None,
        };

        self.next_smtp_port += 1;
        self.peers.push(peer);

        Ok(self.peers.last().unwrap())
    }

    /// Start peer's fastn-rig run process
    pub async fn start_peer(&mut self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let peer_index = self
            .peers
            .iter()
            .position(|p| p.name == name)
            .ok_or(format!("Peer {name} not found"))?;

        let peer = &self.peers[peer_index];
        let process = FastnRigCommand::new()
            .home(&peer.home_path)
            .skip_keyring(self.config.skip_keyring)
            .run(peer.smtp_port)
            .spawn()
            .await?;

        // Update peer with process
        let peer = &mut self.peers[peer_index];
        peer.process = Some(process);

        Ok(())
    }

    /// Send email between peers
    pub async fn send_email(
        &self,
        from_peer: &str,
        to_peer: &str,
        subject: &str,
        body: &str,
    ) -> Result<CommandOutput, Box<dyn std::error::Error>> {
        let from = self
            .peer(from_peer)
            .ok_or(format!("Peer {from_peer} not found"))?;
        let to = self
            .peer(to_peer)
            .ok_or(format!("Peer {to_peer} not found"))?;

        FastnMailCommand::new()
            .send_mail()
            .peer_to_peer(from, to)
            .subject(subject)
            .body(body)
            .send()
            .await
    }

    /// Get peer by name
    pub fn peer(&self, name: &str) -> Option<&PeerHandle> {
        self.peers.iter().find(|p| p.name == name)
    }

    /// Wait for startup
    pub async fn wait_for_startup(&self) -> Result<(), Box<dyn std::error::Error>> {
        tokio::time::sleep(self.config.default_timeout).await;
        Ok(())
    }

    /// Create fluent email builder
    pub fn email(&self) -> EmailBuilder<'_> {
        EmailBuilder {
            env: self,
            from: None,
            to: None,
            subject: "Test Email".to_string(),
            body: "Test Body".to_string(),
            starttls: false, // Default to plain text
        }
    }
}

impl Drop for FastnTestEnv {
    fn drop(&mut self) {
        if self.config.cleanup_on_drop {
            // Kill all running processes
            for peer in &mut self.peers {
                if let Some(ref mut process) = peer.process {
                    let _ = process.start_kill();
                }
            }

            // Give processes time to shut down
            std::thread::sleep(Duration::from_millis(200));

            // Force kill any remaining
            for peer in &mut self.peers {
                if let Some(ref mut process) = peer.process {
                    std::mem::drop(process.kill());
                }
            }
        }
    }
}

/// Handle to a fastn peer (rig + account)
#[derive(Debug)]
pub struct PeerHandle {
    pub name: String,
    pub home_path: PathBuf,
    pub account_id: String,
    pub password: String,
    pub smtp_port: u16,
    pub process: Option<tokio::process::Child>,
}

impl Clone for PeerHandle {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            home_path: self.home_path.clone(),
            account_id: self.account_id.clone(),
            password: self.password.clone(),
            smtp_port: self.smtp_port,
            process: None, // Process handles cannot be cloned
        }
    }
}

impl PeerHandle {
    /// Get peer's email address for sending
    pub fn email_address(&self) -> String {
        format!("test@{}.fastn", self.account_id)
    }

    /// Get peer's inbox address for receiving
    pub fn inbox_address(&self) -> String {
        format!("inbox@{}.fastn", self.account_id)
    }

    /// Check if peer process is running
    pub fn is_running(&mut self) -> bool {
        if let Some(ref mut process) = self.process {
            process.try_wait().map_or(true, |status| status.is_none())
        } else {
            false
        }
    }
}

/// Fluent email builder for sending between peers
pub struct EmailBuilder<'a> {
    env: &'a FastnTestEnv,
    from: Option<String>,
    to: Option<String>,
    subject: String,
    body: String,
    starttls: bool,
}

impl<'a> EmailBuilder<'a> {
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

    pub fn starttls(mut self, enable: bool) -> Self {
        self.starttls = enable;
        self
    }

    pub async fn send(self) -> Result<EmailResult, Box<dyn std::error::Error>> {
        let from_name = self.from.ok_or("From peer not specified")?;
        let to_name = self.to.ok_or("To peer not specified")?;

        let from_peer = self
            .env
            .peer(&from_name)
            .ok_or(format!("Peer {from_name} not found"))?;
        let to_peer = self
            .env
            .peer(&to_name)
            .ok_or(format!("Peer {to_name} not found"))?;

        let output = FastnMailCommand::new()
            .send_mail()
            .peer_to_peer(from_peer, to_peer)
            .subject(&self.subject)
            .body(&self.body)
            .starttls(self.starttls)
            .send()
            .await?;

        Ok(EmailResult {
            output,
            from: from_peer.clone(),
            to: to_peer.clone(),
        })
    }
}

/// Result of email sending operation
pub struct EmailResult {
    pub output: CommandOutput,
    pub from: PeerHandle,
    pub to: PeerHandle,
}

impl EmailResult {
    pub fn expect_success(mut self) -> Result<Self, Box<dyn std::error::Error>> {
        self.output = self.output.expect_success()?;
        Ok(self)
    }

    pub async fn wait_for_delivery(
        self,
        timeout: Duration,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // In a real implementation, this would monitor email delivery status
        tokio::time::sleep(timeout).await;
        Ok(self)
    }

    pub fn contains_output(&self, text: &str) -> bool {
        self.output.contains_output(text)
    }
}
