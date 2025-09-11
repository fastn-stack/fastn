//! IMAP session management

use fastn_account::AccountManager;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

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
    authenticated_account: Option<String>, // Account ID after LOGIN
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
                        let username = parts[2].trim_matches('"'); // Remove quotes
                        let password = parts[3].trim_matches('"'); // Remove quotes

                        // Extract account ID and store in session
                        let account_id = if username.contains('@') {
                            let parts: Vec<&str> = username.split('@').collect();
                            if parts.len() >= 2 {
                                let domain_part = parts[1];
                                domain_part.split('.').next().unwrap_or(domain_part)
                            } else {
                                username
                            }
                        } else {
                            username
                        };

                        self.authenticated_account = Some(account_id.to_string());
                        self.state = SessionState::Authenticated {
                            account_id: account_id.to_string(),
                        };

                        Self::handle_login_static(&mut writer, tag, username, password).await?;
                    } else {
                        Self::send_response_static(
                            &mut writer,
                            &format!("{} BAD LOGIN command requires username and password", tag),
                        )
                        .await?;
                    }
                }
                "LIST" => {
                    if parts.len() >= 4 {
                        let _reference = parts[2].trim_matches('"'); // Reference name (usually "")
                        let pattern = parts[3].trim_matches('"'); // Mailbox pattern
                        Self::handle_list_static(&mut writer, tag, pattern).await?;
                    } else {
                        Self::send_response_static(
                            &mut writer,
                            &format!("{} BAD LIST command requires reference and pattern", tag),
                        )
                        .await?;
                    }
                }
                "SELECT" => {
                    if parts.len() >= 3 {
                        let folder = parts[2].trim_matches('"'); // Folder name

                        // Use authenticated account (not hardcoded!)
                        if let Some(account_id) = &self.authenticated_account {
                            Self::handle_select_with_account(
                                &mut writer,
                                tag,
                                folder,
                                account_id,
                                &self.fastn_home,
                            )
                            .await?;
                        } else {
                            Self::send_response_static(
                                &mut writer,
                                &format!("{} BAD Please authenticate first", tag),
                            )
                            .await?;
                        }
                    } else {
                        Self::send_response_static(
                            &mut writer,
                            &format!("{} BAD SELECT command requires folder name", tag),
                        )
                        .await?;
                    }
                }
                "FETCH" => {
                    if parts.len() >= 4 {
                        let sequence = parts[2]; // Message sequence (e.g., "1", "1:5", "*")
                        let items = parts[3..].join(" "); // FETCH items (e.g., "BODY[]", "ENVELOPE")

                        // Use authenticated account (not hardcoded!)
                        if let Some(account_id) = &self.authenticated_account {
                            Self::handle_fetch_with_account(
                                &mut writer,
                                tag,
                                sequence,
                                &items,
                                account_id,
                                &self.fastn_home,
                            )
                            .await?;
                        } else {
                            Self::send_response_static(
                                &mut writer,
                                &format!("{} BAD Please authenticate first", tag),
                            )
                            .await?;
                        }
                    } else {
                        Self::send_response_static(
                            &mut writer,
                            &format!("{} BAD FETCH command requires sequence and items", tag),
                        )
                        .await?;
                    }
                }
                "UID" => {
                    if parts.len() >= 4 {
                        let uid_command = parts[2].to_uppercase(); // FETCH, STORE, etc.

                        match uid_command.as_str() {
                            "FETCH" => {
                                let sequence = parts[3]; // UID sequence (e.g., "1:*")
                                let items = parts[4..].join(" "); // FETCH items

                                if let Some(account_id) = &self.authenticated_account {
                                    Self::handle_uid_fetch(
                                        &mut writer,
                                        tag,
                                        sequence,
                                        &items,
                                        account_id,
                                        &self.fastn_home,
                                    )
                                    .await?;
                                } else {
                                    Self::send_response_static(
                                        &mut writer,
                                        &format!("{} BAD Please authenticate first", tag),
                                    )
                                    .await?;
                                }
                            }
                            _ => {
                                Self::send_response_static(
                                    &mut writer,
                                    &format!(
                                        "{} BAD UID command {} not implemented",
                                        tag, uid_command
                                    ),
                                )
                                .await?;
                            }
                        }
                    } else {
                        Self::send_response_static(
                            &mut writer,
                            &format!("{} BAD UID command requires subcommand and arguments", tag),
                        )
                        .await?;
                    }
                }
                "STATUS" => {
                    if parts.len() >= 4 {
                        let folder = parts[2].trim_matches('"'); // Folder name
                        let items = parts[3]; // Status items (UIDNEXT MESSAGES UNSEEN RECENT)

                        if let Some(account_id) = &self.authenticated_account {
                            Self::handle_status(
                                &mut writer,
                                tag,
                                folder,
                                items,
                                account_id,
                                &self.fastn_home,
                            )
                            .await?;
                        } else {
                            Self::send_response_static(
                                &mut writer,
                                &format!("{} BAD Please authenticate first", tag),
                            )
                            .await?;
                        }
                    } else {
                        Self::send_response_static(
                            &mut writer,
                            &format!("{} BAD STATUS command requires folder and items", tag),
                        )
                        .await?;
                    }
                }
                "LSUB" => {
                    // Legacy subscription command - return same as LIST for compatibility
                    if parts.len() >= 4 {
                        Self::handle_list_static(&mut writer, tag, "*").await?;
                    } else {
                        Self::send_response_static(
                            &mut writer,
                            &format!("{} BAD LSUB command requires reference and pattern", tag),
                        )
                        .await?;
                    }
                }
                "NOOP" => {
                    Self::send_response_static(&mut writer, &format!("{} OK NOOP completed", tag))
                        .await?;
                    println!("✅ IMAP NOOP completed - connection kept alive");
                }
                "CLOSE" => {
                    Self::send_response_static(&mut writer, &format!("{} OK CLOSE completed", tag))
                        .await?;
                    println!("✅ IMAP CLOSE completed - mailbox closed");
                }
                "LOGOUT" => {
                    Self::handle_logout_static(&mut writer, tag).await?;
                    break;
                }
                _ => {
                    Self::send_response_static(
                        &mut writer,
                        &format!("{} BAD Command not implemented", tag),
                    )
                    .await?;
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
        response: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let response_line = format!("{}\r\n", response);
        writer.write_all(response_line.as_bytes()).await?;
        writer.flush().await?;
        println!("📤 IMAP response: {}", response);
        Ok(())
    }

    async fn handle_capability_static(
        writer: &mut tokio::net::tcp::WriteHalf<'_>,
        tag: &str,
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
        _pattern: &str, // TODO: Use pattern for filtering
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
                &format!("* LIST ({}) \"/\" {}", flags, folder_name),
            )
            .await?;
        }

        Self::send_response_static(writer, &format!("{} OK LIST completed", tag)).await?;
        println!("✅ IMAP LIST completed - returned {} folders", 4);
        Ok(())
    }

    async fn handle_select_with_account(
        writer: &mut tokio::net::tcp::WriteHalf<'_>,
        tag: &str,
        folder: &str,
        account_id: &str,
        fastn_home: &std::path::Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!(
            "📁 IMAP SELECT folder: {} for account: {}",
            folder, account_id
        );

        // Create account path and try to load Store
        let account_path = fastn_home.join("accounts").join(account_id);

        match folder {
            "INBOX" | "Sent" | "Drafts" | "Trash" => {
                // Use fastn-mail Store for all folder operations (proper separation)
                let message_count = match fastn_mail::Store::load(&account_path).await {
                    Ok(store) => {
                        // Use fastn-mail IMAP helper (now always fresh read)
                        match store.imap_select_folder(folder).await {
                            Ok(folder_info) => {
                                println!(
                                    "📊 Store folder stats: {} exists, {} recent, {} unseen",
                                    folder_info.exists,
                                    folder_info.recent,
                                    folder_info.unseen.unwrap_or(0)
                                );
                                folder_info.exists
                            }
                            Err(e) => {
                                println!("⚠️ Failed to get folder stats from Store: {}", e);
                                0
                            }
                        }
                    }
                    Err(e) => {
                        println!("⚠️ Failed to load Store: {}", e);
                        0
                    }
                };

                // Return required SELECT response data with REAL message count
                Self::send_response_static(
                    writer,
                    "* FLAGS (\\Answered \\Flagged \\Deleted \\Seen \\Draft)",
                )
                .await?;
                Self::send_response_static(writer, "* OK [PERMANENTFLAGS (\\Answered \\Flagged \\Deleted \\Seen \\Draft \\*)] Flags permitted").await?;
                Self::send_response_static(writer, &format!("* {} EXISTS", message_count)).await?; // REAL count!
                Self::send_response_static(writer, "* 0 RECENT").await?; // TODO: Calculate real recent count
                Self::send_response_static(writer, "* OK [UNSEEN 0] No unseen messages").await?; // TODO: Calculate real unseen
                Self::send_response_static(writer, "* OK [UIDVALIDITY 1] UIDs valid").await?;
                Self::send_response_static(writer, "* OK [UIDNEXT 1] Next UID").await?;
                Self::send_response_static(
                    writer,
                    &format!("{} OK [READ-WRITE] SELECT completed", tag),
                )
                .await?;

                println!(
                    "✅ IMAP SELECT completed for folder: {} ({} messages)",
                    folder, message_count
                );
                Ok(())
            }
            _ => {
                Self::send_response_static(writer, &format!("{} NO Mailbox does not exist", tag))
                    .await?;
                println!("❌ IMAP SELECT failed - folder '{}' does not exist", folder);
                Ok(())
            }
        }
    }

    async fn handle_select_static(
        writer: &mut tokio::net::tcp::WriteHalf<'_>,
        tag: &str,
        folder: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Legacy method - keep for compatibility
        Self::send_response_static(writer, &format!("{} BAD Please authenticate first", tag))
            .await?;
        Ok(())
    }

    async fn handle_fetch_with_account(
        writer: &mut tokio::net::tcp::WriteHalf<'_>,
        tag: &str,
        sequence: &str,
        items: &str,
        account_id: &str,
        fastn_home: &std::path::Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!(
            "📨 IMAP FETCH sequence: '{}', items: '{}' for account: {}",
            sequence, items, account_id
        );

        // Create account path and try to load Store
        let account_path = fastn_home.join("accounts").join(account_id);

        // Parse sequence number (simplified for now)
        if let Ok(seq_num) = sequence.parse::<u32>() {
            // Try to load Store and fetch the actual message
            match fastn_mail::Store::load(&account_path).await {
                Ok(store) => {
                    // Get all message UIDs in INBOX ordered by sequence
                    match store.imap_search("INBOX", "ALL").await {
                        Ok(uids) => {
                            // Map sequence number to UID (sequence 1 = first UID, etc.)
                            if seq_num > 0 && (seq_num as usize) <= uids.len() {
                                let uid = uids[seq_num as usize - 1]; // Convert 1-based to 0-based
                                println!("🔍 Mapped sequence {} to UID {}", seq_num, uid);

                                // Now fetch the actual message by UID
                                match store.imap_fetch("INBOX", uid).await {
                                    Ok(message_data) => {
                                        println!(
                                            "📧 Found real message: {} bytes",
                                            message_data.len()
                                        );

                                        // Parse the email to extract headers for ENVELOPE
                                        let message_str = String::from_utf8_lossy(&message_data);
                                        let envelope_data =
                                            Self::parse_envelope_from_eml(&message_str);

                                        // Return proper FETCH response based on requested items
                                        if items.contains("BODY[]") {
                                            // Return full message body with proper IMAP literal format
                                            Self::send_response_static(
                                                writer,
                                                &format!(
                                                    "* {} FETCH (BODY[] {{{}}})",
                                                    seq_num,
                                                    message_data.len()
                                                ),
                                            )
                                            .await?;
                                            Self::send_response_static(writer, &message_str)
                                                .await?;
                                        } else if items.contains("ENVELOPE") {
                                            // Return properly formatted ENVELOPE response
                                            Self::send_response_static(writer, &format!(
                                                "* {} FETCH (ENVELOPE ({} {} {} NIL NIL NIL {} NIL))", 
                                                seq_num,
                                                envelope_data.date,
                                                envelope_data.subject,
                                                envelope_data.from,
                                                envelope_data.message_id
                                            )).await?;
                                        } else {
                                            // Return basic info
                                            Self::send_response_static(
                                                writer,
                                                &format!("* {} FETCH (FLAGS ())", seq_num),
                                            )
                                            .await?;
                                        }

                                        Self::send_response_static(
                                            writer,
                                            &format!("{} OK FETCH completed", tag),
                                        )
                                        .await?;
                                        println!(
                                            "✅ IMAP FETCH completed - returned real message data"
                                        );
                                    }
                                    Err(e) => {
                                        println!(
                                            "❌ IMAP FETCH failed to load message UID {}: {}",
                                            uid, e
                                        );
                                        Self::send_response_static(
                                            writer,
                                            &format!(
                                                "{} NO Message {} does not exist",
                                                tag, seq_num
                                            ),
                                        )
                                        .await?;
                                    }
                                }
                            } else {
                                println!(
                                    "❌ IMAP FETCH sequence {} out of range (have {} messages)",
                                    seq_num,
                                    uids.len()
                                );
                                Self::send_response_static(
                                    writer,
                                    &format!("{} NO Message {} does not exist", tag, seq_num),
                                )
                                .await?;
                            }
                        }
                        Err(e) => {
                            println!("⚠️ Failed to search messages for sequence mapping: {}", e);
                            Self::send_response_static(
                                writer,
                                &format!("{} NO Search failed", tag),
                            )
                            .await?;
                        }
                    }
                }
                Err(e) => {
                    println!("⚠️ Failed to load Store for FETCH: {}", e);
                    Self::send_response_static(writer, &format!("{} NO Store access failed", tag))
                        .await?;
                }
            }
        } else {
            Self::send_response_static(writer, &format!("{} BAD Invalid sequence format", tag))
                .await?;
            println!("❌ IMAP FETCH failed - invalid sequence: {}", sequence);
        }

        Ok(())
    }

    async fn handle_fetch_static(
        writer: &mut tokio::net::tcp::WriteHalf<'_>,
        tag: &str,
        sequence: &str,
        items: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Legacy method - should not be used
        Self::send_response_static(writer, &format!("{} BAD Please authenticate first", tag))
            .await?;
        Ok(())
    }

    async fn handle_uid_fetch(
        writer: &mut tokio::net::tcp::WriteHalf<'_>,
        tag: &str,
        sequence: &str,
        items: &str,
        account_id: &str,
        fastn_home: &std::path::Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!(
            "📨 IMAP UID FETCH sequence: '{}', items: '{}' for account: {}",
            sequence, items, account_id
        );

        // Create account path and try to load Store
        let account_path = fastn_home.join("accounts").join(account_id);

        match fastn_mail::Store::load(&account_path).await {
            Ok(store) => {
                // Get all UIDs in the selected folder (force fresh read from database)
                match store.imap_search("INBOX", "ALL").await {
                    Ok(uids) => {
                        // Handle different UID FETCH requests from Thunderbird
                        for uid in &uids {
                            // Get actual email data for this UID
                            match store.imap_fetch("INBOX", *uid).await {
                                Ok(message_data) => {
                                    let size = message_data.len();
                                    let message_str = String::from_utf8_lossy(&message_data);

                                    if items.contains("BODY[]") {
                                        // Email client wants full message body (double-click)
                                        Self::send_response_static(
                                            writer,
                                            &format!(
                                                "* {} FETCH (UID {} RFC822.SIZE {} BODY[] {{{}}}",
                                                uid, uid, size, size
                                            ),
                                        )
                                        .await?;
                                        Self::send_response_static(writer, &message_str).await?;
                                        Self::send_response_static(writer, ")").await?;
                                    } else if items.contains("BODY.PEEK[HEADER.FIELDS") {
                                        // Email client wants header fields - extract from email
                                        let headers =
                                            Self::extract_headers_for_body_peek(&message_str);
                                        let header_text = format!(
                                            "Subject: {}\r\nFrom: {}\r\nTo: {}\r\nDate: {}\r\n",
                                            headers.subject, headers.from, headers.to, headers.date
                                        );

                                        Self::send_response_static(writer, &format!(
                                            "* {} FETCH (UID {} RFC822.SIZE {} FLAGS () BODY[HEADER.FIELDS (From To Cc Bcc Subject Date)] {{{}}}",
                                            uid, uid, size, header_text.len()
                                        )).await?;
                                        Self::send_response_static(writer, &header_text).await?;
                                        Self::send_response_static(writer, ")").await?;
                                    } else if items.contains("RFC822.SIZE") {
                                        // Return size and basic info
                                        Self::send_response_static(
                                            writer,
                                            &format!(
                                                "* {} FETCH (UID {} RFC822.SIZE {} FLAGS ())",
                                                uid, uid, size
                                            ),
                                        )
                                        .await?;
                                    } else {
                                        // Basic FLAGS only
                                        Self::send_response_static(
                                            writer,
                                            &format!("* {} FETCH (UID {} FLAGS ())", uid, uid),
                                        )
                                        .await?;
                                    }
                                }
                                Err(_) => {
                                    // Fallback for missing messages
                                    Self::send_response_static(
                                        writer,
                                        &format!("* {} FETCH (UID {} FLAGS ())", uid, uid),
                                    )
                                    .await?;
                                }
                            }
                        }

                        Self::send_response_static(
                            writer,
                            &format!("{} OK UID FETCH completed", tag),
                        )
                        .await?;
                        println!("✅ IMAP UID FETCH completed - returned {} UIDs", uids.len());
                    }
                    Err(e) => {
                        println!("❌ IMAP UID FETCH failed to search messages: {}", e);
                        Self::send_response_static(writer, &format!("{} NO Search failed", tag))
                            .await?;
                    }
                }
            }
            Err(e) => {
                println!("⚠️ Failed to load Store for UID FETCH: {}", e);
                Self::send_response_static(writer, &format!("{} NO Store access failed", tag))
                    .await?;
            }
        }

        Ok(())
    }

    async fn handle_status(
        writer: &mut tokio::net::tcp::WriteHalf<'_>,
        tag: &str,
        folder: &str,
        _items: &str, // Items like (UIDNEXT MESSAGES UNSEEN RECENT)
        account_id: &str,
        fastn_home: &std::path::Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!(
            "📨 IMAP STATUS folder: {} for account: {}",
            folder, account_id
        );

        // Create account path and try to load Store
        let account_path = fastn_home.join("accounts").join(account_id);

        match fastn_mail::Store::load(&account_path).await {
            Ok(store) => {
                match store.imap_select_folder(folder).await {
                    Ok(folder_info) => {
                        // Return STATUS response with folder statistics
                        Self::send_response_static(
                            writer,
                            &format!(
                                "* STATUS {} (MESSAGES {} UIDNEXT 2 UNSEEN {} RECENT {})",
                                folder,
                                folder_info.exists,
                                folder_info.unseen.unwrap_or(0),
                                folder_info.recent
                            ),
                        )
                        .await?;
                        Self::send_response_static(writer, &format!("{} OK STATUS completed", tag))
                            .await?;
                        println!("✅ IMAP STATUS completed for folder: {}", folder);
                    }
                    Err(e) => {
                        println!("❌ IMAP STATUS failed for folder {}: {}", folder, e);
                        Self::send_response_static(writer, &format!("{} NO Folder not found", tag))
                            .await?;
                    }
                }
            }
            Err(e) => {
                println!("⚠️ Failed to load Store for STATUS: {}", e);
                Self::send_response_static(writer, &format!("{} NO Store access failed", tag))
                    .await?;
            }
        }

        Ok(())
    }

    async fn handle_logout_static(
        writer: &mut tokio::net::tcp::WriteHalf<'_>,
        tag: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Self::send_response_static(writer, "* BYE Logging out").await?;
        Self::send_response_static(writer, &format!("{} OK LOGOUT completed", tag)).await?;
        Ok(())
    }

    /// Extract headers for IMAP BODY.PEEK requests
    fn extract_headers_for_body_peek(eml_content: &str) -> HeaderFields {
        let mut subject = "".to_string();
        let mut from = "".to_string();
        let mut to = "".to_string();
        let mut date = "".to_string();

        // Parse headers (simple line-by-line parsing)
        for line in eml_content.lines() {
            if line.is_empty() {
                break; // End of headers
            }

            if let Some(value) = line.strip_prefix("Subject: ") {
                subject = value.to_string();
            } else if let Some(value) = line.strip_prefix("From: ") {
                from = value.to_string();
            } else if let Some(value) = line.strip_prefix("To: ") {
                to = value.to_string();
            } else if let Some(value) = line.strip_prefix("Date: ") {
                date = value.to_string();
            }
        }

        HeaderFields {
            subject,
            from,
            to,
            date,
        }
    }

    /// Parse email headers to create IMAP ENVELOPE data
    fn parse_envelope_from_eml(eml_content: &str) -> EnvelopeData {
        let mut date = "NIL".to_string();
        let mut subject = "NIL".to_string();
        let mut from = "NIL".to_string();
        let mut message_id = "NIL".to_string();

        // Parse headers (simple line-by-line parsing)
        for line in eml_content.lines() {
            if line.is_empty() {
                break; // End of headers
            }

            if let Some(value) = line.strip_prefix("Date: ") {
                date = format!("\"{}\"", value);
            } else if let Some(value) = line.strip_prefix("Subject: ") {
                subject = format!("\"{}\"", value);
            } else if let Some(value) = line.strip_prefix("From: ") {
                // Parse From: test@domain.com into proper IMAP format
                from = format!(
                    "((NIL NIL \"{}\" \"{}\" NIL))",
                    value.split('@').next().unwrap_or("unknown"),
                    value.split('@').nth(1).unwrap_or("unknown")
                );
            } else if let Some(value) = line.strip_prefix("Message-ID: ") {
                message_id = format!("\"{}\"", value);
            }
        }

        EnvelopeData {
            date,
            subject,
            from,
            message_id,
        }
    }
}

/// Headers for IMAP BODY.PEEK requests
struct HeaderFields {
    subject: String,
    from: String,
    to: String,
    date: String,
}

/// Simple structure to hold parsed envelope data
struct EnvelopeData {
    date: String,
    subject: String,
    from: String,
    message_id: String,
}
