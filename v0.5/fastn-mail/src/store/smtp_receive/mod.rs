//! # SMTP Receive Module
//!
//! Handles incoming email messages from external SMTP connections.
//!
//! ## Requirements
//!
//! ### Email Processing Flow
//! - External SMTP server sends email to fastn user
//! - Parse and validate email headers (From, To, CC, BCC)
//! - Store in Sent/Outbox folder (this is outbound from user's perspective)
//! - Queue emails to fastn peers for P2P delivery
//!
//! ## Modules
//! - `parse_email_headers`: RFC 5322 parsing and header extraction
//! - `validate_email_for_smtp`: P2P-only address validation and security checks
//! - `store_email`: File system and database storage operations

mod validate_email_for_smtp;

pub use validate_email_for_smtp::validate_email_for_smtp;

use fastn_mail::errors::SmtpReceiveError;

impl fastn_mail::Store {
    /// SMTP server receives an email and handles delivery (local storage or P2P queuing)
    ///
    /// Flow: External SMTP → Store in Sent → Queue for P2P delivery to peers
    ///
    /// # Validation Requirements
    ///
    /// ## Address Validation (STRICT - P2P ONLY)
    /// - **From Address**: Must be one of our account's aliases (SMTP authentication required)
    /// - **All Recipients**: Must use valid ID52 format: `<username>@<id52>.<domain>`
    /// - **No External Email**: Mixed fastn/external recipients NOT supported
    /// - **ID52 Verification**: All ID52 components must be valid PublicKeys
    ///
    /// ## Content Validation
    /// - **Required Headers**: From, To, Subject, Message-ID must be present
    /// - **Size Limits**: Message size must be within reasonable bounds
    /// - **Character Encoding**: Must be valid UTF-8
    ///
    /// ## Authentication
    /// - **SMTP Auth**: Sender must authenticate as From address owner
    /// - **Alias Ownership**: From address must belong to our account
    /// - **DefaultMail Access**: Required for authentication and routing decisions
    ///
    /// SMTP server receives an email with envelope data from SMTP protocol
    pub async fn smtp_receive(
        &self,
        smtp_from: &str,
        smtp_recipients: &[String],
        raw_message: Vec<u8>,
    ) -> Result<String, SmtpReceiveError> {
        // Step 1: Create ParsedEmail using SMTP envelope data (no header parsing needed)
        let parsed_email = create_parsed_email_from_smtp(smtp_from, smtp_recipients, &raw_message)?;

        // Step 2: Validate email for SMTP acceptance
        validate_email_for_smtp(&parsed_email)?;

        // Step 3: Store email file to disk
        self.store_email_file(&parsed_email.file_path, &raw_message)
            .await?;

        // Step 4: Insert email metadata into database
        self.store_email_metadata(&parsed_email).await?;

        // Step 5: Queue P2P deliveries for fastn recipients
        self.queue_p2p_deliveries(&parsed_email).await?;

        println!("✅ Email stored and queued for delivery");
        println!("📂 DEBUG: Email file path: {}", parsed_email.file_path);
        println!("📂 DEBUG: Account path: {}", self.account_path().display());
        Ok(parsed_email.email_id)
    }

    /// Store email file to disk
    async fn store_email_file(
        &self,
        file_path: &str,
        raw_message: &[u8],
    ) -> Result<(), SmtpReceiveError> {
        let full_path = self.account_path().join(file_path);

        // Create directory structure if needed
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                SmtpReceiveError::MessageParsingFailed {
                    message: format!("Failed to create directory {}: {e}", parent.display()),
                }
            })?;
        }

        // Check if file already exists
        if full_path.exists() {
            return Err(SmtpReceiveError::MessageParsingFailed {
                message: format!("Email file already exists: {}", full_path.display()),
            });
        }

        // Write email file
        std::fs::write(&full_path, raw_message).map_err(|e| {
            SmtpReceiveError::MessageParsingFailed {
                message: format!("Failed to write email file {}: {e}", full_path.display()),
            }
        })?;

        println!("💾 Stored email file: {}", full_path.display());
        Ok(())
    }

    /// Store email metadata in database
    async fn store_email_metadata(
        &self,
        parsed_email: &fastn_mail::ParsedEmail,
    ) -> Result<(), SmtpReceiveError> {
        let conn = self.connection().lock().await;

        conn.execute(
            "INSERT INTO fastn_emails (
                email_id, folder, file_path, message_id, from_addr, to_addr, cc_addr, bcc_addr, subject,
                our_alias_used, our_username, their_alias, their_username,
                in_reply_to, email_references, date_sent, date_received,
                content_type, content_encoding, has_attachments, size_bytes,
                is_seen, is_flagged, is_draft, is_answered, is_deleted, custom_flags
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9,
                ?10, ?11, ?12, ?13,
                ?14, ?15, ?16, ?17,
                ?18, ?19, ?20, ?21,
                ?22, ?23, ?24, ?25, ?26, ?27
            )",
            rusqlite::params![
                parsed_email.email_id,
                parsed_email.folder,
                parsed_email.file_path,
                parsed_email.message_id,
                parsed_email.from_addr,
                parsed_email.to_addr,
                parsed_email.cc_addr,
                parsed_email.bcc_addr,
                parsed_email.subject,
                parsed_email.our_alias_used,
                parsed_email.our_username,
                parsed_email.their_alias,
                parsed_email.their_username,
                parsed_email.in_reply_to,
                parsed_email.email_references,
                parsed_email.date_sent,
                parsed_email.date_received,
                parsed_email.content_type,
                parsed_email.content_encoding,
                parsed_email.has_attachments,
                parsed_email.size_bytes,
                parsed_email.is_seen,
                parsed_email.is_flagged,
                parsed_email.is_draft,
                parsed_email.is_answered,
                parsed_email.is_deleted,
                parsed_email.custom_flags,
            ],
        ).map_err(|e| {
            SmtpReceiveError::MessageParsingFailed {
                message: format!("Failed to insert email metadata: {e}"),
            }
        })?;

        println!("🗄️  Stored email metadata: {}", parsed_email.email_id);
        Ok(())
    }

    /// Queue P2P deliveries for fastn recipients
    async fn queue_p2p_deliveries(
        &self,
        parsed_email: &fastn_mail::ParsedEmail,
    ) -> Result<(), SmtpReceiveError> {
        let conn = self.connection().lock().await;
        let mut queued_count = 0;

        // Collect all recipients from To, CC, and BCC
        let mut all_recipients = Vec::new();

        // Add To recipients
        all_recipients.extend(parsed_email.to_addr.split(',').map(|addr| addr.trim()));

        // Add CC recipients
        if let Some(cc_addr) = &parsed_email.cc_addr {
            all_recipients.extend(cc_addr.split(',').map(|addr| addr.trim()));
        }

        // Add BCC recipients
        if let Some(bcc_addr) = &parsed_email.bcc_addr {
            all_recipients.extend(bcc_addr.split(',').map(|addr| addr.trim()));
        }

        // Process all recipients in one loop
        for addr in all_recipients {
            if !addr.is_empty()
                && let Ok((Some(_username), Some(id52))) = parse_id52_address(addr)
            {
                // This is a fastn peer - queue for P2P delivery
                conn.execute(
                    "INSERT INTO fastn_email_delivery (
                        email_id, recipient_id52, delivery_status, attempts, last_attempt, next_retry
                    ) VALUES (?1, ?2, 'queued', 0, NULL, ?3)",
                    rusqlite::params![
                        parsed_email.email_id,
                        id52,
                        chrono::Utc::now().timestamp(), // Schedule for immediate delivery
                    ],
                ).map_err(|e| {
                    SmtpReceiveError::MessageParsingFailed {
                        message: format!("Failed to queue P2P delivery: {e}"),
                    }
                })?;
                queued_count += 1;
                println!("📤 Queued P2P delivery to: {id52}");
            }
        }

        println!("📤 Total P2P deliveries queued: {queued_count}");
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    /// Helper macro for creating RFC 5322 test emails with proper CRLF endings
    /// Supports both static text and variable interpolation
    macro_rules! test_email {
        ($($tt:tt)*) => {
            indoc::formatdoc! { $($tt)* }.replace('\n', "\r\n")
        };
    }

    #[tokio::test]
    async fn test_smtp_receive_basic() {
        let store = fastn_mail::Store::create_test();

        // Generate valid ID52s for testing
        let from_key = fastn_id52::SecretKey::generate();
        let to_key = fastn_id52::SecretKey::generate();
        let from_id52 = from_key.public_key().id52();
        let to_id52 = to_key.public_key().id52();

        let email = test_email! {"
            From: alice@{from_id52}.fastn
            To: bob@{to_id52}.local
            Subject: Test
            Message-ID: <test@localhost>
            
            Hello World!
        "};

        let result = store
            .smtp_receive(
                &format!("alice@{from_id52}.fastn"),
                &[format!("bob@{to_id52}.local")],
                email.into_bytes(),
            )
            .await;

        // Should succeed with valid P2P addresses
        assert!(result.is_ok());
        let email_id = result.unwrap();
        assert!(!email_id.is_empty());
        assert!(email_id.starts_with("email-"));
    }

    #[tokio::test]
    async fn test_smtp_receive_validation_failure() {
        let store = fastn_mail::Store::create_test();

        let email = test_email! {"
            From: external@gmail.com
            To: bob@example.com
            Subject: Test
            
            Body content
        "};

        let result = store
            .smtp_receive(
                "external@gmail.com",
                &["bob@example.com".to_string()],
                email.into_bytes(),
            )
            .await;

        // Should fail validation for external From address
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_smtp_receive_missing_headers() {
        let store = fastn_mail::Store::create_test();

        let email = test_email! {"
            Subject: No From Header
            
            Body content
        "};

        let result = store
            .smtp_receive(
                "", // Empty from - should fail validation
                &["test@example.com".to_string()],
                email.into_bytes(),
            )
            .await;

        // Should fail with missing From header
        assert!(result.is_err());
    }
}

/// Create ParsedEmail from SMTP envelope data with minimal parsing
pub fn create_parsed_email_from_smtp(
    smtp_from: &str,
    smtp_recipients: &[String],
    raw_message: &[u8],
) -> Result<fastn_mail::ParsedEmail, SmtpReceiveError> {
    // Reject non-UTF-8 emails early
    let message_text =
        std::str::from_utf8(raw_message).map_err(|_| SmtpReceiveError::InvalidUtf8Encoding)?;

    // Extract only essential headers we can't get from SMTP envelope
    let essential_headers = extract_essential_headers(message_text)?;

    // Generate storage information
    let email_id = format!("email-{}", uuid::Uuid::new_v4());
    let folder = "Sent".to_string(); // SMTP emails are outgoing from authenticated user
    let timestamp = chrono::Utc::now().format("%Y/%m/%d");
    let file_path = format!("mails/default/Sent/{timestamp}/{email_id}.eml");
    let date_received = chrono::Utc::now().timestamp();
    let size_bytes = raw_message.len();

    // Use SMTP envelope data directly - no header parsing for addresses!
    let to_addr = smtp_recipients.join(", ");

    // Extract P2P routing information from SMTP envelope
    let (our_username, our_alias_used) = parse_id52_address(smtp_from).unwrap_or((None, None));

    Ok(fastn_mail::ParsedEmail {
        email_id,
        folder,
        file_path,
        message_id: essential_headers.message_id,
        from_addr: smtp_from.to_string(), // Use SMTP envelope FROM
        to_addr,                          // Use SMTP envelope recipients
        cc_addr: None,                    // SMTP doesn't distinguish CC from TO
        bcc_addr: None,                   // SMTP doesn't expose BCC to us
        subject: essential_headers.subject,
        our_alias_used,
        our_username,
        their_alias: None,    // Multiple recipients possible
        their_username: None, // Multiple recipients possible
        in_reply_to: essential_headers.in_reply_to,
        email_references: essential_headers.references,
        date_sent: essential_headers.date_sent,
        date_received,
        content_type: essential_headers.content_type.clone(),
        has_attachments: essential_headers.content_type.contains("multipart"),
        content_encoding: essential_headers.content_encoding,
        size_bytes,
        is_seen: false,
        is_flagged: false,
        is_draft: false,
        is_answered: false,
        is_deleted: false,
        custom_flags: None,
    })
}

/// Essential headers we need from email body (not available in SMTP envelope)
#[derive(Debug)]
struct EssentialHeaders {
    message_id: String,
    subject: String,
    date_sent: Option<i64>,
    in_reply_to: Option<String>,
    references: Option<String>,
    content_type: String,
    content_encoding: Option<String>,
}

/// Extract only essential headers we can't get from SMTP envelope
fn extract_essential_headers(message_text: &str) -> Result<EssentialHeaders, SmtpReceiveError> {
    // Find header/body separator
    let header_section = match message_text.split_once("\r\n\r\n") {
        Some((headers, _body)) => headers,
        None => {
            // No header/body separator found - malformed email
            // Check if it uses \n\n instead of \r\n\r\n
            if message_text.contains("\n\n") {
                return Err(SmtpReceiveError::InvalidLineEndings);
            } else {
                return Err(SmtpReceiveError::MissingHeaderBodySeparator);
            }
        }
    };

    let mut message_id = None;
    let mut subject = None;
    let date_sent = None;
    let mut in_reply_to = None;
    let mut references = None;
    let mut content_type = None;
    let mut content_encoding = None;

    // Parse only the headers we actually need
    for line in header_section.lines() {
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            let value = value.trim();

            match key.to_ascii_lowercase().as_str() {
                "message-id" => message_id = Some(value.to_string()),
                "subject" => subject = Some(value.to_string()),
                "date" => {
                    // TODO: Parse RFC 5322 date format to Unix timestamp
                    // For now, leave as None
                }
                "in-reply-to" => in_reply_to = Some(value.to_string()),
                "references" => references = Some(value.to_string()),
                "content-type" => content_type = Some(value.to_string()),
                "content-transfer-encoding" => content_encoding = Some(value.to_string()),
                _ => {} // Ignore all other headers
            }
        }
    }

    Ok(EssentialHeaders {
        message_id: message_id.unwrap_or_else(|| format!("generated-{}", uuid::Uuid::new_v4())),
        subject: subject.unwrap_or_else(|| "(no subject)".to_string()),
        date_sent,
        in_reply_to,
        references,
        content_type: content_type.unwrap_or_else(|| "text/plain".to_string()),
        content_encoding,
    })
}

/// Parse email address to extract username and ID52 components for P2P routing
/// Returns: (Some(username), Some(id52)) if valid fastn format, (None, None) if external email
pub fn parse_id52_address(
    email: &str,
) -> Result<(Option<String>, Option<String>), SmtpReceiveError> {
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return Ok((None, None)); // Invalid format - treat as external email
    }

    let username = parts[0];
    let domain_part = parts[1];

    // Parse domain to extract potential ID52: id52.domain
    let domain_parts: Vec<&str> = domain_part.split('.').collect();
    if domain_parts.is_empty() {
        return Ok((None, None)); // No domain parts
    }

    let potential_id52 = domain_parts[0];

    // Check if it's a valid 52-character ID52
    if potential_id52.len() != 52 {
        return Ok((None, None)); // Not ID52 format - external email
    }

    // Verify it's a valid fastn_id52::PublicKey
    match potential_id52.parse::<fastn_id52::PublicKey>() {
        Ok(_) => Ok((Some(username.to_string()), Some(potential_id52.to_string()))),
        Err(_) => Ok((None, None)), // Invalid ID52 - external email
    }
}

impl fastn_mail::Store {
    /// INBOX receives an email from P2P delivery (incoming from peer)  
    ///
    /// Flow: P2P message → Store in INBOX → No further queuing needed
    pub async fn inbox_receive(
        &self,
        envelope_from: &str,
        smtp_recipients: &[String],
        raw_message: Vec<u8>,
    ) -> Result<String, SmtpReceiveError> {
        // Reuse SMTP parsing but override folder to INBOX
        let mut parsed_email =
            create_parsed_email_from_smtp(envelope_from, smtp_recipients, &raw_message)?;

        // Override folder for INBOX storage
        parsed_email.folder = "INBOX".to_string();

        // Update file path for INBOX
        let timestamp = chrono::Utc::now().format("%Y/%m/%d");
        parsed_email.file_path = format!(
            "mails/default/INBOX/{timestamp}/{}.eml",
            parsed_email.email_id
        );

        // Store email file in INBOX
        let full_path = self.account_path.join(&parsed_email.file_path);
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| SmtpReceiveError::FileStoreFailed {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }

        std::fs::write(&full_path, &raw_message).map_err(|e| {
            SmtpReceiveError::FileStoreFailed {
                path: full_path,
                source: e,
            }
        })?;

        // Store metadata in database
        self.store_email_metadata(&parsed_email).await?;

        // No P2P delivery queuing needed for received emails
        tracing::info!(
            "📥 P2P email from {} stored in INBOX with ID: {}",
            envelope_from,
            parsed_email.email_id
        );

        Ok(parsed_email.email_id)
    }
}
