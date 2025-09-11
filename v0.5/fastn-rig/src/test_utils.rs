//! fastn-rig specific test utilities
//!
//! These utilities are built on top of fastn-cli-test-utils but provide
//! fastn-rig specific concepts like peers, SMTP ports, keyring management, etc.

use fastn_cli_test_utils::CommandOutput;
use std::path::PathBuf;
use std::time::Duration;

/// fastn-rig specific test environment
pub struct FastnRigTestEnv {
    temp_dir: tempfile::TempDir,
    peers: Vec<PeerHandle>,
    next_smtp_port: u16,
    skip_keyring: bool,
}

impl FastnRigTestEnv {
    pub fn new(test_name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let temp_dir = tempfile::Builder::new()
            .prefix(&format!("fastn-rig-test-{test_name}-"))
            .tempdir()?;

        Ok(Self {
            temp_dir,
            peers: Vec::new(),
            next_smtp_port: 2525,
            skip_keyring: true,
        })
    }

    /// Create a new peer with fastn-rig init
    pub async fn create_peer(
        &mut self,
        name: &str,
    ) -> Result<&PeerHandle, Box<dyn std::error::Error>> {
        let peer_home = self.temp_dir.path().join(name);
        std::fs::create_dir_all(&peer_home)?;

        let binary_path = fastn_cli_test_utils::get_fastn_rig_binary();

        let output = tokio::process::Command::new(binary_path)
            .arg("init")
            .env(
                "SKIP_KEYRING",
                if self.skip_keyring { "true" } else { "false" },
            )
            .env("FASTN_HOME", &peer_home)
            .output()
            .await?;

        if !output.status.success() {
            return Err(format!(
                "Failed to initialize peer {name}: {}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Extract account ID and password
        let account_id = extract_account_id(&stdout)?;
        let password = extract_password(&stdout)?;

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

    /// Start a peer's fastn-rig run process
    pub async fn start_peer(&mut self, peer_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let peer_index = self
            .peers
            .iter()
            .position(|p| p.name == peer_name)
            .ok_or(format!("Peer {peer_name} not found"))?;

        let peer = &self.peers[peer_index];
        let binary_path = fastn_cli_test_utils::get_fastn_rig_binary();

        let child = tokio::process::Command::new(binary_path)
            .arg("run")
            .env(
                "SKIP_KEYRING",
                if self.skip_keyring { "true" } else { "false" },
            )
            .env("FASTN_HOME", &peer.home_path)
            .env("FASTN_SMTP_PORT", peer.smtp_port.to_string())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        // Update peer with process
        let peer = &mut self.peers[peer_index];
        peer.process = Some(child);

        Ok(())
    }

    /// Send email using fastn-mail
    pub async fn send_email(
        &self,
        from_peer: &str,
        to_peer: &str,
        subject: &str,
        body: &str,
    ) -> Result<CommandOutput, Box<dyn std::error::Error>> {
        let from = self
            .peers
            .iter()
            .find(|p| p.name == from_peer)
            .ok_or(format!("From peer {from_peer} not found"))?;
        let to = self
            .peers
            .iter()
            .find(|p| p.name == to_peer)
            .ok_or(format!("To peer {to_peer} not found"))?;

        let binary_path = fastn_cli_test_utils::get_fastn_mail_binary();

        let output = tokio::process::Command::new(binary_path)
            .args([
                "send-mail",
                "--smtp",
                &from.smtp_port.to_string(),
                "--password",
                &from.password,
                "--from",
                &format!("test@{}.com", from.account_id),
                "--to",
                &format!("inbox@{}.com", to.account_id),
                "--subject",
                subject,
                "--body",
                body,
            ])
            .env("FASTN_HOME", &from.home_path)
            .output()
            .await?;

        Ok(CommandOutput {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            success: output.status.success(),
            exit_code: output.status.code(),
        })
    }

    pub fn peer(&self, name: &str) -> Option<&PeerHandle> {
        self.peers.iter().find(|p| p.name == name)
    }
}

impl Drop for FastnRigTestEnv {
    fn drop(&mut self) {
        // Kill all running processes
        for peer in &mut self.peers {
            if let Some(ref mut process) = peer.process {
                let _ = process.start_kill();
            }
        }

        // Give processes time to shut down
        std::thread::sleep(Duration::from_millis(100));

        // Force kill any remaining
        for peer in &mut self.peers {
            if let Some(ref mut process) = peer.process {
                std::mem::drop(process.kill());
            }
        }
    }
}

/// Handle to a fastn-rig test peer
#[derive(Debug)]
pub struct PeerHandle {
    pub name: String,
    pub home_path: PathBuf,
    pub account_id: String,
    pub password: String,
    pub smtp_port: u16,
    pub process: Option<tokio::process::Child>,
}

impl PeerHandle {
    pub fn email_address(&self) -> String {
        format!("test@{}.com", self.account_id)
    }

    pub fn inbox_address(&self) -> String {
        format!("inbox@{}.com", self.account_id)
    }
}

/// Extract account ID from fastn-rig init output
fn extract_account_id(output: &str) -> Result<String, Box<dyn std::error::Error>> {
    for line in output.lines() {
        if line.contains("Account ID:")
            && let Some(id) = line.split_whitespace().nth(2) {
                return Ok(id.to_string());
            }
    }
    Err("Account ID not found in output".into())
}

/// Extract password from fastn-rig init output
fn extract_password(output: &str) -> Result<String, Box<dyn std::error::Error>> {
    for line in output.lines() {
        if line.contains("Password:")
            && let Some(password) = line.split_whitespace().nth(1) {
                return Ok(password.to_string());
            }
    }
    Err("Password not found in output".into())
}
