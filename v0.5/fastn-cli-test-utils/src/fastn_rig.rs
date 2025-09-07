//! fastn-rig specific command builders and helpers

use crate::CommandOutput;
use std::path::{Path, PathBuf};
use tokio::process::Command;

/// Fluent builder for fastn-rig commands
pub struct FastnRigCommand {
    home: Option<PathBuf>,
    skip_keyring: bool,
    smtp_port: Option<u16>,
    args: Vec<String>,
}

impl FastnRigCommand {
    pub fn new() -> Self {
        Self {
            home: None,
            skip_keyring: true,
            smtp_port: None,
            args: Vec::new(),
        }
    }
    
    /// Set fastn home directory  
    pub fn home(mut self, path: &Path) -> Self {
        self.home = Some(path.to_path_buf());
        self
    }
    
    /// Skip keyring operations (default: true for tests)
    pub fn skip_keyring(mut self, skip: bool) -> Self {
        self.skip_keyring = skip;
        self
    }
    
    /// Add raw arguments
    pub fn args(mut self, args: &[&str]) -> Self {
        self.args.extend(args.iter().map(|s| s.to_string()));
        self
    }
    
    /// fastn-rig init command
    pub fn init(self) -> Self {
        self.args(&["init"])
    }
    
    /// fastn-rig status command
    pub fn status(self) -> Self {
        self.args(&["status"])
    }
    
    /// fastn-rig entities command
    pub fn entities(self) -> Self {
        self.args(&["entities"])
    }
    
    /// fastn-rig run command with SMTP port
    pub fn run(mut self, smtp_port: u16) -> Self {
        self.args.push("run".to_string());
        self.smtp_port = Some(smtp_port);
        self
    }
    
    /// fastn-rig set-online command
    pub fn set_online(self, entity_id: &str, online: bool) -> Self {
        self.args(&["set-online", entity_id, &online.to_string()])
    }
    
    /// Execute command and return output
    pub async fn execute(self) -> Result<CommandOutput, Box<dyn std::error::Error>> {
        let binary_path = crate::get_fastn_rig_binary();
        
        let mut cmd = Command::new(binary_path);
        cmd.args(&self.args);
        
        if let Some(home) = &self.home {
            cmd.env("FASTN_HOME", home);
        }
        
        if self.skip_keyring {
            cmd.env("SKIP_KEYRING", "true");
        }
        
        if let Some(smtp_port) = self.smtp_port {
            cmd.env("FASTN_SMTP_PORT", smtp_port.to_string());
        }
        
        let output = cmd.output().await?;
        
        Ok(CommandOutput {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            success: output.status.success(),
            exit_code: output.status.code(),
        })
    }
    
    /// Execute command as background process
    pub async fn spawn(self) -> Result<tokio::process::Child, Box<dyn std::error::Error>> {
        let binary_path = crate::get_fastn_rig_binary();
        
        let mut cmd = Command::new(binary_path);
        cmd.args(&self.args);
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());
        
        if let Some(home) = &self.home {
            cmd.env("FASTN_HOME", home);
        }
        
        if self.skip_keyring {
            cmd.env("SKIP_KEYRING", "true");
        }
        
        if let Some(smtp_port) = self.smtp_port {
            cmd.env("FASTN_SMTP_PORT", smtp_port.to_string());
        }
        
        Ok(cmd.spawn()?)
    }
}