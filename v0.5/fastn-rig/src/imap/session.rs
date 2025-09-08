//! IMAP session management

use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use fastn_account::AccountManager;

/// IMAP session state
#[derive(Debug, Clone)]
pub enum SessionState {
    NotAuthenticated,
    Authenticated { account_id: String },
    Selected { account_id: String, mailbox: String },
    Logout,
}

/// IMAP session handler
pub struct ImapSession {
    stream: TcpStream,
    client_addr: std::net::SocketAddr,
    state: SessionState,
    account_manager: Arc<AccountManager>,
}

impl ImapSession {
    pub fn new(
        stream: TcpStream,
        client_addr: std::net::SocketAddr,
        account_manager: Arc<AccountManager>,
    ) -> Self {
        Self {
            stream,
            client_addr,
            state: SessionState::NotAuthenticated,
            account_manager,
        }
    }

    /// Handle IMAP session from start to finish
    pub async fn handle(mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📨 IMAP session started for {}", self.client_addr);
        
        // Send greeting
        self.send_response("* OK fastn IMAP server ready").await?;
        
        // Split stream for reading and writing
        let (reader, mut writer) = self.stream.split();
        let reader = BufReader::new(reader);
        let mut lines = reader.lines();
        
        // Main command loop
        while let Some(line) = lines.next_line().await? {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            
            println!("📨 IMAP command from {}: {}", self.client_addr, line);
            
            // Parse command: tag command args
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 2 {
                Self::send_response_static(&mut writer, "* BAD Invalid command format").await?;
                continue;
            }
            
            let tag = parts[0];
            let command = parts[1].to_uppercase();
            
            match command.as_str() {
                "CAPABILITY" => {
                    Self::handle_capability_static(&mut writer, tag).await?;
                }
                "LOGIN" => {
                    if parts.len() >= 4 {
                        let username = parts[2].trim_matches('"');  // Remove quotes
                        let password = parts[3].trim_matches('"');  // Remove quotes
                        Self::handle_login_static(&mut writer, tag, username, password).await?;
                    } else {
                        Self::send_response_static(&mut writer, &format!("{} BAD LOGIN command requires username and password", tag)).await?;
                    }
                }
                "LOGOUT" => {
                    Self::handle_logout_static(&mut writer, tag).await?;
                    break;
                }
                _ => {
                    Self::send_response_static(&mut writer, &format!("{} BAD Command not implemented", tag)).await?;
                }
            }
        }
        
        println!("📨 IMAP session ended for {}", self.client_addr);
        Ok(())
    }
    
    async fn send_response(&mut self, response: &str) -> Result<(), Box<dyn std::error::Error>> {
        let response_line = format!("{}\r\n", response);
        self.stream.write_all(response_line.as_bytes()).await?;
        self.stream.flush().await?;
        println!("📤 IMAP response to {}: {}", self.client_addr, response);
        Ok(())
    }
    
    async fn send_response_static(
        writer: &mut tokio::net::tcp::WriteHalf<'_>, 
        response: &str
    ) -> Result<(), Box<dyn std::error::Error>> {
        let response_line = format!("{}\r\n", response);
        writer.write_all(response_line.as_bytes()).await?;
        writer.flush().await?;
        println!("📤 IMAP response: {}", response);
        Ok(())
    }
    
    async fn handle_capability_static(
        writer: &mut tokio::net::tcp::WriteHalf<'_>, 
        tag: &str
    ) -> Result<(), Box<dyn std::error::Error>> {
        Self::send_response_static(writer, "* CAPABILITY IMAP4rev1").await?;
        Self::send_response_static(writer, &format!("{} OK CAPABILITY completed", tag)).await?;
        Ok(())
    }
    
    async fn handle_login_static(
        writer: &mut tokio::net::tcp::WriteHalf<'_>, 
        tag: &str,
        username: &str,
        password: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔑 IMAP LOGIN attempt: user={}", username);
        
        // For now, accept any login (TODO: implement real authentication)
        // This allows us to test the complete protocol flow
        Self::send_response_static(writer, &format!("{} OK LOGIN completed", tag)).await?;
        println!("✅ IMAP LOGIN successful for user: {}", username);
        Ok(())
    }
    
    async fn handle_logout_static(
        writer: &mut tokio::net::tcp::WriteHalf<'_>, 
        tag: &str
    ) -> Result<(), Box<dyn std::error::Error>> {
        Self::send_response_static(writer, "* BYE Logging out").await?;
        Self::send_response_static(writer, &format!("{} OK LOGOUT completed", tag)).await?;
        Ok(())
    }
}