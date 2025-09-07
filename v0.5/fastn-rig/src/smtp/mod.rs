//! # SMTP Server Module
//!
//! Provides SMTP server functionality for fastn-rig with multi-account support.
//!
//! ## Features
//! - Single SMTP server handling multiple accounts
//! - Username format: anything@<id52>.com for account routing  
//! - Authentication via fastn-account stored passwords
//! - Message routing to account mail stores
//! - P2P integration for cross-network delivery

mod parser;

pub struct SmtpServer {
    /// Account manager for authentication and storage
    account_manager: std::sync::Arc<fastn_account::AccountManager>,
    /// Server bind address
    bind_addr: std::net::SocketAddr,
    // No graceful parameter - use fastn_p2p::spawn() and fastn_p2p::cancelled() directly
}

#[derive(Debug)]
pub struct SmtpSession {
    /// Client connection
    stream: tokio::net::TcpStream,
    /// Current session state
    state: SessionState,
    /// Authenticated account ID52 (if any)
    authenticated_account: Option<fastn_id52::PublicKey>,
    /// Email being composed
    current_email: Option<EmailInProgress>,
    /// Client IP address
    client_addr: std::net::SocketAddr,
}

#[derive(Debug, PartialEq)]
enum SessionState {
    /// Initial state, waiting for EHLO/HELO
    Initial,
    /// Connected, ready for commands
    Ready,
    /// Expecting email content after DATA command
    Data,
    /// Session terminated
    Quit,
}

#[derive(Debug)]
struct EmailInProgress {
    /// Sender address from MAIL FROM
    from: String,
    /// Recipient addresses from RCPT TO
    recipients: Vec<String>,
    /// Email content
    data: Vec<u8>,
}

impl SmtpServer {
    pub fn new(
        account_manager: std::sync::Arc<fastn_account::AccountManager>,
        bind_addr: std::net::SocketAddr,
    ) -> Self {
        Self {
            account_manager,
            bind_addr,
        }
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let listener = tokio::net::TcpListener::bind(self.bind_addr).await?;
        tracing::info!("📧 SMTP server listening on {}", self.bind_addr);
        println!("📧 SMTP server listening on {}", self.bind_addr);

        loop {
            tokio::select! {
                _ = fastn_p2p::cancelled() => {
                    tracing::info!("📧 SMTP server shutting down");
                    println!("📧 SMTP server shutting down");
                    break;
                }

                result = listener.accept() => {
                    match result {
                        Ok((stream, addr)) => {
                            tracing::debug!("📧 New SMTP connection from {}", addr);
                            let session = SmtpSession::new(stream, addr);
                            let account_manager = self.account_manager.clone();
                            // Spawn session handler using global coordination
                            fastn_p2p::spawn(async move {
                                if let Err(e) = session.handle(account_manager).await {
                                    tracing::error!("📧 SMTP session error from {addr}: {e}");
                                }
                            });
                        }
                        Err(e) => {
                            tracing::error!("📧 Failed to accept SMTP connection: {e}");
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

impl SmtpSession {
    fn new(stream: tokio::net::TcpStream, client_addr: std::net::SocketAddr) -> Self {
        Self {
            stream,
            state: SessionState::Initial,
            authenticated_account: None,
            current_email: None,
            client_addr,
        }
    }

    async fn handle(
        mut self,
        account_manager: std::sync::Arc<fastn_account::AccountManager>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use tokio::io::AsyncReadExt;

        tracing::debug!("📧 Starting SMTP session with {}", self.client_addr);

        // Send greeting
        self.write_response("220 fastn SMTP Server").await?;

        // Use a simple line-by-line reading approach to avoid borrowing conflicts
        let mut buffer = Vec::new();
        let mut temp_buf = [0u8; 1024];

        loop {
            // Read data from stream
            let bytes_read = self.stream.read(&mut temp_buf).await?;
            if bytes_read == 0 {
                break; // EOF
            }

            buffer.extend_from_slice(&temp_buf[..bytes_read]);

            // Process complete lines
            while let Some(line_end) = buffer.windows(2).position(|w| w == b"\r\n") {
                let line_bytes = buffer.drain(..line_end + 2).collect::<Vec<u8>>();
                let line = String::from_utf8_lossy(&line_bytes[..line_bytes.len() - 2]);
                let line = line.trim();

                tracing::debug!("📧 Received: {}", line);

                // Don't skip empty lines during DATA state - they're part of email content
                if line.is_empty() && self.state != SessionState::Data {
                    continue;
                }

                // Handle DATA state specially - collect email content
                if self.state == SessionState::Data {
                    if line == "." {
                        // End of data
                        let response = match self.process_email_data(&account_manager).await {
                            Ok(response) => response,
                            Err(e) => {
                                tracing::error!("📧 Email processing error: {}", e);
                                "450 Temporary failure - try again later".to_string()
                            }
                        };
                        self.write_response(&response).await?;
                        self.state = SessionState::Ready;
                        continue;
                    } else {
                        // Accumulate email data
                        if let Some(ref mut email) = self.current_email {
                            // Remove dot-stuffing (lines starting with .. become .)
                            let data_line = if line.starts_with("..") {
                                &line[1..]
                            } else {
                                line
                            };
                            email.data.extend_from_slice(data_line.as_bytes());
                            email.data.extend_from_slice(b"\r\n");
                        }
                        continue;
                    }
                }

                let response = match self.process_command(line, &account_manager).await {
                    Ok(response) => response,
                    Err(fastn_rig::SmtpError::InvalidCommandSyntax { command }) => {
                        tracing::debug!("📧 Invalid command syntax: {}", command);
                        format!("500 Syntax error: {command}")
                    }
                    Err(fastn_rig::SmtpError::AuthenticationFailed) => {
                        "535 Authentication failed".to_string()
                    }
                    Err(e) => {
                        tracing::error!("📧 Command processing error: {}", e);
                        "421 Service not available - try again later".to_string()
                    }
                };

                self.write_response(&response).await?;

                // Break on QUIT
                if self.state == SessionState::Quit {
                    break;
                }
            }
        }

        tracing::debug!("📧 SMTP session ended with {}", self.client_addr);
        Ok(())
    }

    async fn process_command(
        &mut self,
        line: &str,
        account_manager: &fastn_account::AccountManager,
    ) -> Result<String, fastn_rig::SmtpError> {
        let parts: Vec<&str> = line.splitn(2, ' ').collect();
        let command = parts[0].to_uppercase();
        let args = parts.get(1).unwrap_or(&"");

        match command.as_str() {
            "EHLO" | "HELO" => self.handle_helo(args).await,
            "AUTH" => self.handle_auth(args, account_manager).await,
            "MAIL" => self.handle_mail_from(args).await,
            "RCPT" => self.handle_rcpt_to(args).await,
            "DATA" => self.handle_data().await,
            "RSET" => self.handle_reset().await,
            "QUIT" => self.handle_quit().await,
            "NOOP" => Ok("250 OK".to_string()),
            _ => Ok(format!("500 Command '{command}' not recognized")),
        }
    }

    async fn handle_helo(&mut self, _args: &str) -> Result<String, fastn_rig::SmtpError> {
        self.state = SessionState::Ready;
        Ok("250-fastn SMTP Server\r\n250-AUTH PLAIN LOGIN\r\n250 HELP".to_string())
    }

    async fn handle_auth(
        &mut self,
        args: &str,
        account_manager: &fastn_account::AccountManager,
    ) -> Result<String, fastn_rig::SmtpError> {
        let parts: Vec<&str> = args.split_whitespace().collect();
        if parts.len() < 2 {
            return Ok("500 AUTH requires mechanism and credentials".to_string());
        }

        let mechanism = parts[0].to_uppercase();
        match mechanism.as_str() {
            "PLAIN" => self.handle_auth_plain(parts[1], account_manager).await,
            "LOGIN" => Ok("500 AUTH LOGIN not yet implemented".to_string()),
            _ => Ok(format!("500 AUTH mechanism '{mechanism}' not supported")),
        }
    }

    async fn handle_auth_plain(
        &mut self,
        credentials: &str,
        account_manager: &fastn_account::AccountManager,
    ) -> Result<String, fastn_rig::SmtpError> {
        // Parse credentials using parser module
        let creds = match parser::AuthCredentials::parse_plain(credentials) {
            Ok(creds) => creds,
            Err(e) => {
                tracing::debug!("📧 Auth parsing error: {}", e);
                return Ok("535 Authentication failed: invalid format".to_string());
            }
        };

        // Extract account ID52 from username with debug logging
        let account_id52 = match creds.extract_account_id52() {
            Some(id52) => {
                tracing::info!("📧 SMTP: Successfully extracted account ID52: {} from username: {}", id52.id52(), creds.username);
                id52
            }
            None => {
                tracing::warn!("📧 SMTP: Failed to extract account ID52 from username: {}", creds.username);
                return Ok("535 Authentication failed: invalid username format".to_string());
            }
        };

        // Authenticate with fastn-account
        match self
            .authenticate_account(&account_id52, &creds.password, account_manager)
            .await
        {
            Ok(true) => {
                self.authenticated_account = Some(account_id52);
                Ok("235 Authentication successful".to_string())
            }
            Ok(false) => Ok("535 Authentication failed".to_string()),
            Err(e) => {
                tracing::warn!("📧 Authentication error for {}: {}", creds.username, e);
                Ok("535 Authentication failed".to_string())
            }
        }
    }

    async fn authenticate_account(
        &self,
        account_id52: &fastn_id52::PublicKey,
        password: &str,
        account_manager: &fastn_account::AccountManager,
    ) -> Result<bool, fastn_rig::SmtpError> {
        tracing::debug!(
            "📧 Authenticating account {} with SMTP password",
            account_id52.id52()
        );

        // Find the account by alias
        let account = account_manager
            .find_account_by_alias(account_id52)
            .await
            .map_err(|e| fastn_rig::SmtpError::AccountLookupFailed { source: e })?;

        // Verify SMTP password using the account's stored hash
        match account.verify_smtp_password(password).await {
            Ok(is_valid) => {
                if is_valid {
                    tracing::info!(
                        "📧 SMTP authentication successful for {}",
                        account_id52.id52()
                    );
                } else {
                    tracing::warn!(
                        "📧 SMTP authentication failed for {} - invalid password or SMTP disabled",
                        account_id52.id52()
                    );
                }
                Ok(is_valid)
            }
            Err(fastn_account::MailConfigError::ConfigNotFound) => {
                tracing::warn!(
                    "📧 SMTP authentication failed for {} - no mail configuration found",
                    account_id52.id52()
                );
                Ok(false)
            }
            Err(e) => {
                tracing::error!(
                    "📧 SMTP authentication error for {}: {}",
                    account_id52.id52(),
                    e
                );
                Err(fastn_rig::SmtpError::MailConfigError { source: e })
            }
        }
    }

    async fn handle_mail_from(&mut self, args: &str) -> Result<String, fastn_rig::SmtpError> {
        if self.authenticated_account.is_none() {
            return Ok("530 Authentication required".to_string());
        }

        // Parse MAIL FROM using parser module
        let from_addr = parser::parse_mail_from(args).map_err(|e| {
            fastn_rig::SmtpError::InvalidCommandSyntax {
                command: format!("MAIL FROM: {e}"),
            }
        })?;

        self.current_email = Some(EmailInProgress {
            from: from_addr,
            recipients: Vec::new(),
            data: Vec::new(),
        });

        Ok("250 Sender OK".to_string())
    }

    async fn handle_rcpt_to(&mut self, args: &str) -> Result<String, fastn_rig::SmtpError> {
        if self.current_email.is_none() {
            return Ok("503 Need MAIL FROM first".to_string());
        }

        // Parse RCPT TO using parser module
        let to_addr = parser::parse_rcpt_to(args).map_err(|e| {
            fastn_rig::SmtpError::InvalidCommandSyntax {
                command: format!("RCPT TO: {e}"),
            }
        })?;

        if let Some(ref mut email) = self.current_email {
            email.recipients.push(to_addr);
        }

        Ok("250 Recipient OK".to_string())
    }

    async fn handle_data(&mut self) -> Result<String, fastn_rig::SmtpError> {
        if self.current_email.is_none() {
            return Ok("503 Need MAIL FROM and RCPT TO first".to_string());
        }

        self.state = SessionState::Data;
        Ok("354 Start mail input; end with <CRLF>.<CRLF>".to_string())
    }

    async fn handle_reset(&mut self) -> Result<String, fastn_rig::SmtpError> {
        self.current_email = None;
        self.state = SessionState::Ready;
        Ok("250 Reset OK".to_string())
    }

    async fn handle_quit(&mut self) -> Result<String, fastn_rig::SmtpError> {
        self.state = SessionState::Quit;
        Ok("221 Goodbye".to_string())
    }

    async fn write_response(&mut self, response: &str) -> Result<(), std::io::Error> {
        use tokio::io::AsyncWriteExt;

        tracing::debug!("📧 Sending: {}", response);
        self.stream.write_all(response.as_bytes()).await?;
        self.stream.write_all(b"\r\n").await?;
        self.stream.flush().await?;
        Ok(())
    }

    async fn process_email_data(
        &mut self,
        account_manager: &fastn_account::AccountManager,
    ) -> Result<String, fastn_rig::SmtpError> {
        let email = match self.current_email.take() {
            Some(email) => email,
            None => return Ok("503 No email in progress".to_string()),
        };

        let authenticated_account = match &self.authenticated_account {
            Some(account) => account,
            None => return Ok("530 Authentication required".to_string()),
        };

        tracing::debug!(
            "📧 Processing email from {} to {} recipients ({} bytes)",
            email.from,
            email.recipients.len(),
            email.data.len()
        );

        // Store the email using fastn-account
        match self
            .store_received_email(&email, authenticated_account, account_manager)
            .await
        {
            Ok(()) => {
                tracing::info!(
                    "📧 Email stored successfully for account {}",
                    authenticated_account.id52()
                );
                Ok("250 Message accepted for delivery".to_string())
            }
            Err(e) => {
                tracing::error!(
                    "📧 Failed to store email from {} to {:?}: {}",
                    email.from,
                    email.recipients,
                    e
                );
                println!("🐛 DEBUG: Email storage error details: {e}");
                if let Some(source) = std::error::Error::source(&e) {
                    println!("🐛 DEBUG: Root cause: {source:?}");
                } else {
                    println!("🐛 DEBUG: No additional error details");
                }
                Ok("450 Temporary failure - try again later".to_string())
            }
        }
    }

    async fn store_received_email(
        &self,
        email: &EmailInProgress,
        account_id52: &fastn_id52::PublicKey,
        account_manager: &fastn_account::AccountManager,
    ) -> Result<(), fastn_rig::SmtpError> {
        // Find the account that should receive this email
        let account = account_manager
            .find_account_by_alias(account_id52)
            .await
            .map_err(|e| fastn_rig::SmtpError::AccountLookupFailed { source: e })?;

        // Get the account's mail store
        let account_path = account.path().await;
        let mail_store = fastn_mail::Store::load(&account_path)
            .await
            .map_err(|e| fastn_rig::SmtpError::MailStoreLoadFailed { source: e })?;

        // For now, use smtp_receive which stores in INBOX and queues for P2P delivery
        // This actually works because smtp_receive will queue the email for P2P delivery
        // The email lands in INBOX first, then gets delivered via P2P
        let email_id = mail_store
            .smtp_receive(&email.from, &email.recipients, email.data.clone())
            .await
            .map_err(|e| fastn_rig::SmtpError::EmailStorageFailed { source: e })?;

        tracing::info!(
            "📧 Stored email {} from {} in account {} (queued for P2P delivery)",
            email_id,
            email.from,
            account_id52.id52()
        );
        Ok(())
    }
}
