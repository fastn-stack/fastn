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
    fastn_home: std::path::PathBuf,
    authenticated_account: Option<String>,  // Account ID after LOGIN
}

impl ImapSession {
    pub fn new(
        stream: TcpStream,
        client_addr: std::net::SocketAddr,
        account_manager: Arc<AccountManager>,
        fastn_home: std::path::PathBuf,
    ) -> Self {
        Self {
            stream,
            client_addr,
            state: SessionState::NotAuthenticated,
            account_manager,
            fastn_home,
            authenticated_account: None,
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
                "LIST" => {
                    if parts.len() >= 4 {
                        let _reference = parts[2].trim_matches('"');  // Reference name (usually "")
                        let pattern = parts[3].trim_matches('"');     // Mailbox pattern
                        Self::handle_list_static(&mut writer, tag, pattern).await?;
                    } else {
                        Self::send_response_static(&mut writer, &format!("{} BAD LIST command requires reference and pattern", tag)).await?;
                    }
                }
                "SELECT" => {
                    if parts.len() >= 3 {
                        let folder = parts[2].trim_matches('"');  // Folder name
                        Self::handle_select_static(&mut writer, tag, folder).await?;
                    } else {
                        Self::send_response_static(&mut writer, &format!("{} BAD SELECT command requires folder name", tag)).await?;
                    }
                }
                "FETCH" => {
                    if parts.len() >= 4 {
                        let sequence = parts[2];  // Message sequence (e.g., "1", "1:5", "*")
                        let items = parts[3..].join(" ");  // FETCH items (e.g., "BODY[]", "ENVELOPE")
                        Self::handle_fetch_static(&mut writer, tag, sequence, &items).await?;
                    } else {
                        Self::send_response_static(&mut writer, &format!("{} BAD FETCH command requires sequence and items", tag)).await?;
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
        
        // Extract account ID from username format: user@{account_id52}.com
        let account_id = if username.contains('@') {
            let parts: Vec<&str> = username.split('@').collect();
            if parts.len() >= 2 {
                let domain_part = parts[1];
                // Extract ID52 from domain (before .com or .local)
                domain_part.split('.').next().unwrap_or(domain_part)
            } else {
                username
            }
        } else {
            username
        };
        
        println!("🔍 Extracted account ID: {}", account_id);
        
        // For now, accept any login (TODO: implement real authentication)
        Self::send_response_static(writer, &format!("{} OK LOGIN completed", tag)).await?;
        println!("✅ IMAP LOGIN successful for account: {}", account_id);
        Ok(())
    }
    
    async fn handle_list_static(
        writer: &mut tokio::net::tcp::WriteHalf<'_>, 
        tag: &str,
        _pattern: &str,  // TODO: Use pattern for filtering
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("📁 IMAP LIST command");
        
        // Return standard email folders
        // For now, return hardcoded list (TODO: read from filesystem)
        let folders = vec![
            ("INBOX", "\\HasNoChildren"),
            ("Sent", "\\HasNoChildren"),
            ("Drafts", "\\HasNoChildren"),
            ("Trash", "\\HasNoChildren"),
        ];
        
        for (folder_name, flags) in folders {
            Self::send_response_static(
                writer, 
                &format!("* LIST ({}) \"/\" {}", flags, folder_name)
            ).await?;
        }
        
        Self::send_response_static(writer, &format!("{} OK LIST completed", tag)).await?;
        println!("✅ IMAP LIST completed - returned {} folders", 4);
        Ok(())
    }
    
    async fn handle_select_static(
        writer: &mut tokio::net::tcp::WriteHalf<'_>, 
        tag: &str,
        folder: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("📁 IMAP SELECT folder: {}", folder);
        
        // For now, return basic folder stats (TODO: read actual .eml files)
        match folder {
            "INBOX" | "Sent" | "Drafts" | "Trash" => {
                // Return required SELECT response data
                Self::send_response_static(writer, "* FLAGS (\\Answered \\Flagged \\Deleted \\Seen \\Draft)").await?;
                Self::send_response_static(writer, "* OK [PERMANENTFLAGS (\\Answered \\Flagged \\Deleted \\Seen \\Draft \\*)] Flags permitted").await?;
                Self::send_response_static(writer, "* 0 EXISTS").await?;  // No messages yet
                Self::send_response_static(writer, "* 0 RECENT").await?;  // No recent messages
                Self::send_response_static(writer, "* OK [UNSEEN 0] No unseen messages").await?;
                Self::send_response_static(writer, "* OK [UIDVALIDITY 1] UIDs valid").await?;
                Self::send_response_static(writer, "* OK [UIDNEXT 1] Next UID").await?;
                Self::send_response_static(writer, &format!("{} OK [READ-WRITE] SELECT completed", tag)).await?;
                
                println!("✅ IMAP SELECT completed for folder: {}", folder);
                Ok(())
            }
            _ => {
                Self::send_response_static(writer, &format!("{} NO Mailbox does not exist", tag)).await?;
                println!("❌ IMAP SELECT failed - folder '{}' does not exist", folder);
                Ok(())
            }
        }
    }
    
    async fn handle_fetch_static(
        writer: &mut tokio::net::tcp::WriteHalf<'_>, 
        tag: &str,
        sequence: &str,
        items: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("📨 IMAP FETCH sequence: '{}', items: '{}'", sequence, items);
        
        // For now, return that no messages exist (since folder is empty)
        // TODO: Read actual .eml files and return real message data
        
        // Parse sequence - for now, handle simple cases
        if sequence == "*" || sequence.starts_with("1:") {
            // No messages exist, so return empty result
            Self::send_response_static(writer, &format!("{} OK FETCH completed (no messages)", tag)).await?;
            println!("✅ IMAP FETCH completed - no messages to fetch");
        } else if let Ok(seq_num) = sequence.parse::<u32>() {
            // Specific message number requested
            if seq_num == 1 {
                // First message requested but none exist
                Self::send_response_static(writer, &format!("{} OK FETCH completed (no messages)", tag)).await?;
                println!("✅ IMAP FETCH completed - message {} not found (folder empty)", seq_num);
            } else {
                Self::send_response_static(writer, &format!("{} NO Message {} does not exist", tag, seq_num)).await?;
                println!("❌ IMAP FETCH failed - message {} not found", seq_num);
            }
        } else {
            Self::send_response_static(writer, &format!("{} BAD Invalid sequence format", tag)).await?;
            println!("❌ IMAP FETCH failed - invalid sequence: {}", sequence);
        }
        
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