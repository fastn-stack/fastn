//! fastn-mail specific command builders and helpers

use crate::CommandOutput;
use std::path::{Path, PathBuf};
use tokio::process::Command;

/// Fluent builder for fastn-mail commands
pub struct FastnMailCommand {
    fastn_home: Option<PathBuf>,
    args: Vec<String>,
}

impl FastnMailCommand {
    pub fn new() -> Self {
        Self {
            fastn_home: None,
            args: Vec::new(),
        }
    }
    
    /// Set fastn home directory
    pub fn home(mut self, path: &Path) -> Self {
        self.fastn_home = Some(path.to_path_buf());
        self
    }
    
    /// fastn-mail send-mail command with fluent parameter building
    pub fn send_mail(self) -> FastnMailSendBuilder {
        FastnMailSendBuilder {
            base: self.args(&["send-mail"]),
            from: None,
            to: None,
            subject: "Test Email".to_string(),
            body: "Test Body".to_string(),
            smtp_port: 2525,
            password: None,
        }
    }
    
    /// fastn-mail list-mails command
    pub fn list_mails(self, folder: &str) -> Self {
        self.args(&["list-mails", "--folder", folder])
    }
    
    /// Add raw arguments
    pub fn args(mut self, args: &[&str]) -> Self {
        self.args.extend(args.iter().map(|s| s.to_string()));
        self
    }
    
    /// Execute command and return output
    pub async fn execute(self) -> Result<CommandOutput, Box<dyn std::error::Error>> {
        let binary_path = crate::get_fastn_mail_binary();
        
        let mut cmd = Command::new(binary_path);
        cmd.args(&self.args);
        
        if let Some(home) = &self.fastn_home {
            cmd.env("FASTN_HOME", home);
        }
        
        let output = cmd.output().await?;
        
        Ok(CommandOutput {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            success: output.status.success(),
            exit_code: output.status.code(),
        })
    }
}

/// Fluent builder for fastn-mail send-mail command
pub struct FastnMailSendBuilder {
    base: FastnMailCommand,
    from: Option<String>,
    to: Option<String>,
    subject: String,
    body: String,
    smtp_port: u16,
    password: Option<String>,
}

impl FastnMailSendBuilder {
    /// Set sender email address
    pub fn from(mut self, email: &str) -> Self {
        self.from = Some(email.to_string());
        self
    }
    
    /// Set recipient email address
    pub fn to(mut self, email: &str) -> Self {
        self.to = Some(email.to_string());
        self
    }
    
    /// Set email subject
    pub fn subject(mut self, subject: &str) -> Self {
        self.subject = subject.to_string();
        self
    }
    
    /// Set email body
    pub fn body(mut self, body: &str) -> Self {
        self.body = body.to_string();
        self
    }
    
    /// Set SMTP port
    pub fn smtp_port(mut self, port: u16) -> Self {
        self.smtp_port = port;
        self
    }
    
    /// Set SMTP password
    pub fn password(mut self, password: &str) -> Self {
        self.password = Some(password.to_string());
        self
    }
    
    /// Send using peer credentials (extracts email addresses from peer)
    pub fn from_peer_to_peer(mut self, from_peer: &crate::PeerHandle, to_peer: &crate::PeerHandle) -> Self {
        self.from = Some(from_peer.email_address());
        self.to = Some(to_peer.inbox_address());
        self.smtp_port = from_peer.smtp_port;
        self.password = Some(from_peer.password.clone());
        self.base = self.base.home(&from_peer.home_path);
        self
    }
    
    /// Execute the send-mail command
    pub async fn send(mut self) -> Result<CommandOutput, Box<dyn std::error::Error>> {
        // Add all the send-mail arguments
        let from = self.from.ok_or("From address not specified")?;
        let to = self.to.ok_or("To address not specified")?;
        let password = self.password.ok_or("Password not specified")?;
        
        self.base.args.extend([
            "--smtp".to_string(), self.smtp_port.to_string(),
            "--password".to_string(), password,
            "--from".to_string(), from,
            "--to".to_string(), to,
            "--subject".to_string(), self.subject,
            "--body".to_string(), self.body,
        ]);
        
        self.base.execute().await
    }
}