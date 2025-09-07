//! Test environment setup and management

use crate::{BinaryManager, ProcessManager, CliConfig, CommandOutput};
use std::path::{Path, PathBuf};

/// Complete test environment for fastn CLI testing
#[derive(Debug)]
pub struct TestEnvironment {
    pub temp_dir: tempfile::TempDir,
    pub process_manager: ProcessManager,
    pub config: CliConfig,
}

impl TestEnvironment {
    /// Create new test environment with default configuration
    pub async fn new(test_name: &str) -> Result<Self, eyre::Error> {
        Self::with_config(test_name, CliConfig::default()).await
    }
    
    /// Create test environment with custom configuration
    pub async fn with_config(test_name: &str, config: CliConfig) -> Result<Self, eyre::Error> {
        let temp_dir = tempfile::Builder::new()
            .prefix(&format!("fastn-test-{test_name}-"))
            .tempdir()?;
        
        tracing::info!("Created test environment at: {}", temp_dir.path().display());
        
        let mut binaries = BinaryManager::new()?;
        
        if config.pre_build {
            // Pre-build commonly used binaries
            binaries.ensure_built(&[
                crate::binary::BinarySpec::binary("fastn-rig"),
                crate::binary::BinarySpec::binary("test_utils"), 
                crate::binary::BinarySpec::package_with_features("fastn-mail", "net"),
            ]).await?;
        }
        
        let process_manager = ProcessManager::new(binaries, config.clone());
        
        Ok(Self {
            temp_dir,
            process_manager,
            config,
        })
    }
    
    /// Create a new peer with its own fastn_home directory
    pub async fn create_peer(&mut self, name: &str) -> Result<PeerHandle, eyre::Error> {
        let peer_home = self.temp_dir.path().join(name);
        std::fs::create_dir_all(&peer_home)?;
        
        tracing::info!("Initializing peer {name} at {}", peer_home.display());
        
        let output = self.process_manager.spawn_fastn_rig_init(&peer_home).await?;
        let init_output = output.expect_success()?;
        
        let account_id = init_output.extract_account_id()?;
        let password = init_output.extract_password()?;
        
        tracing::info!("Created peer {name}: {account_id}");
        
        Ok(PeerHandle {
            name: name.to_string(),
            home_path: peer_home,
            account_id,
            password,
            process_id: None,
        })
    }
    
    /// Start a peer's fastn-rig run process
    pub async fn start_peer(&mut self, peer: &mut PeerHandle, smtp_port: u16) -> Result<(), eyre::Error> {
        let process_id = self.process_manager.spawn_fastn_rig_run(&peer.home_path, smtp_port).await?;
        peer.process_id = Some(process_id);
        tracing::info!("Started peer {} on SMTP port {smtp_port}", peer.name);
        Ok(())
    }
    
    /// Send email from one peer to another
    pub async fn send_email(&mut self, from_peer: &PeerHandle, to_peer: &PeerHandle, subject: &str, body: &str) -> Result<CommandOutput, eyre::Error> {
        let params = crate::SendMailParams {
            from: format!("test@{}.com", from_peer.account_id),
            to: format!("inbox@{}.com", to_peer.account_id),
            subject: subject.to_string(),
            body: body.to_string(),
            smtp_port: 2525, // Default SMTP port for sender
            password: from_peer.password.clone(),
            fastn_home: from_peer.home_path.clone(),
        };
        
        self.process_manager.fastn_mail_send(params).await
    }
    
    /// Wait for startup with default timeout
    pub async fn wait_for_startup(&self) -> Result<(), eyre::Error> {
        self.process_manager.wait_for_startup(self.config.default_timeout).await
    }
}

/// Handle to a test peer
#[derive(Debug, Clone)]
pub struct PeerHandle {
    pub name: String,
    pub home_path: PathBuf,
    pub account_id: String,
    pub password: String,
    pub process_id: Option<crate::process::ProcessId>,
}

impl PeerHandle {
    /// Get peer's email address
    pub fn email_address(&self) -> String {
        format!("test@{}.com", self.account_id)
    }
    
    /// Get peer's inbox address
    pub fn inbox_address(&self) -> String {
        format!("inbox@{}.com", self.account_id)
    }
    
    /// Check if peer process is running
    pub fn is_running(&self, manager: &mut ProcessManager) -> bool {
        if let Some(process_id) = self.process_id {
            manager.is_running(process_id)
        } else {
            false
        }
    }
}