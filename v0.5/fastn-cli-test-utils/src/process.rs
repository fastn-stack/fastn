//! Process lifecycle management with RAII cleanup

use crate::{BinaryManager, CommandOutput};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;

/// Manages multiple CLI processes with automatic cleanup
#[derive(Debug)]
pub struct ProcessManager {
    processes: Vec<ProcessHandle>,
    binaries: BinaryManager,
    config: crate::CliConfig,
}

impl ProcessManager {
    pub fn new(binaries: BinaryManager, config: crate::CliConfig) -> Self {
        Self {
            processes: Vec::new(),
            binaries,
            config,
        }
    }
    
    /// Spawn fastn-rig init command
    pub async fn spawn_fastn_rig_init(&mut self, home: &Path) -> Result<CommandOutput, eyre::Error> {
        let binary_path = self.binaries.get_binary("fastn-rig")?;
        
        let output = Command::new(binary_path)
            .arg("init")
            .env("SKIP_KEYRING", if self.config.skip_keyring { "true" } else { "false" })
            .env("FASTN_HOME", home)
            .output()
            .await?;
        
        Ok(CommandOutput {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            success: output.status.success(),
            exit_code: output.status.code(),
        })
    }
    
    /// Spawn fastn-rig run process in background
    pub async fn spawn_fastn_rig_run(&mut self, home: &Path, smtp_port: u16) -> Result<ProcessId, eyre::Error> {
        let binary_path = self.binaries.get_binary("fastn-rig")?;
        
        let child = Command::new(binary_path)
            .arg("run")
            .env("SKIP_KEYRING", if self.config.skip_keyring { "true" } else { "false" })
            .env("FASTN_HOME", home)
            .env("FASTN_SMTP_PORT", smtp_port.to_string())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        
        let process_id = ProcessId(self.processes.len());
        let handle = ProcessHandle {
            child,
            name: format!("fastn-rig-run-{smtp_port}"),
            home_dir: Some(home.to_path_buf()),
            process_id,
        };
        
        self.processes.push(handle);
        Ok(process_id)
    }
    
    /// Execute fastn-mail send command
    pub async fn fastn_mail_send(&mut self, params: crate::SendMailParams) -> Result<CommandOutput, eyre::Error> {
        let binary_path = self.binaries.get_binary("fastn-mail")?;
        
        let output = Command::new(binary_path)
            .args([
                "send-mail",
                "--smtp", &params.smtp_port.to_string(),
                "--password", &params.password,
                "--from", &params.from,
                "--to", &params.to,
                "--subject", &params.subject,
                "--body", &params.body,
            ])
            .env("FASTN_HOME", &params.fastn_home)
            .output()
            .await?;
        
        Ok(CommandOutput {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            success: output.status.success(),
            exit_code: output.status.code(),
        })
    }
    
    /// Wait for processes to start up
    pub async fn wait_for_startup(&self, timeout: Duration) -> Result<(), eyre::Error> {
        tokio::time::sleep(timeout).await;
        Ok(())
    }
    
    /// Check if a process is still running
    pub fn is_running(&mut self, id: ProcessId) -> bool {
        if let Some(handle) = self.processes.get_mut(id.0) {
            handle.child.try_wait().map_or(true, |status| status.is_none())
        } else {
            false
        }
    }
    
    /// Kill all managed processes
    pub fn kill_all(&mut self) {
        tracing::info!("🧹 Cleaning up {} processes", self.processes.len());
        
        for handle in &mut self.processes {
            let _ = handle.child.start_kill();
        }
        
        // Give processes time to shut down gracefully
        std::thread::sleep(Duration::from_millis(100));
        
        // Force kill any remaining processes
        for handle in &mut self.processes {
            let _ = handle.child.kill();
        }
    }
}

impl Drop for ProcessManager {
    fn drop(&mut self) {
        if self.config.cleanup_on_drop {
            self.kill_all();
        }
    }
}

/// Handle to a managed process
#[derive(Debug)]
pub struct ProcessHandle {
    pub child: tokio::process::Child,
    pub name: String,
    pub home_dir: Option<PathBuf>,
    pub process_id: ProcessId,
}

/// Unique identifier for a managed process
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProcessId(usize);