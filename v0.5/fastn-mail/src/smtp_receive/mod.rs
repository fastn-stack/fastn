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

mod parse_email;
mod validate_email_for_smtp;

pub use parse_email::{parse_email, parse_id52_address};
pub use validate_email_for_smtp::validate_email_for_smtp;

use crate::errors::SmtpReceiveError;

impl crate::Store {
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
    pub async fn smtp_receive(&self, raw_message: Vec<u8>) -> Result<String, SmtpReceiveError> {
        // TODO: Load DefaultMail from automerge database for validation
        // For now, create a placeholder for validation testing
        // Step 1: Parse email message headers
        let parsed_email = parse_email(&raw_message)?;

        // Step 2: Validate email for SMTP acceptance
        validate_email_for_smtp(&parsed_email)?;

        // Display parsed information
        println!("📧 Successfully parsed email!");
        println!("  From: {}", parsed_email.from_addr);
        println!("  To: {}", parsed_email.to_addr);
        println!(
            "  CC: {}",
            parsed_email.cc_addr.as_deref().unwrap_or("(none)")
        );
        println!(
            "  BCC: {}",
            parsed_email.bcc_addr.as_deref().unwrap_or("(none)")
        );
        println!("  Subject: {}", parsed_email.subject);
        println!("  Message-ID: {}", parsed_email.message_id);
        println!("  File: {}", parsed_email.file_path);
        println!("  Size: {} bytes", parsed_email.size_bytes);

        // Display P2P routing information
        println!("🔗 P2P Routing:");
        println!(
            "  Our alias: {}",
            parsed_email
                .our_alias_used
                .as_deref()
                .unwrap_or("(external)")
        );
        println!(
            "  Our username: {}",
            parsed_email.our_username.as_deref().unwrap_or("(external)")
        );
        println!(
            "  Their alias: {}",
            parsed_email.their_alias.as_deref().unwrap_or("(external)")
        );
        println!(
            "  Their username: {}",
            parsed_email
                .their_username
                .as_deref()
                .unwrap_or("(external)")
        );

        // Step 3: Store email file to disk
        self.store_email_file(&parsed_email.file_path, &raw_message)
            .await?;

        // Step 4: Insert email metadata into database
        self.store_email_metadata(&parsed_email).await?;

        // Step 5: Queue P2P deliveries for fastn recipients
        self.queue_p2p_deliveries(&parsed_email).await?;

        println!("✅ Email stored and queued for delivery");
        Ok(parsed_email.email_id)
    }

    /// Store email file to disk
    async fn store_email_file(
        &self,
        file_path: &str,
        raw_message: &[u8],
    ) -> Result<(), SmtpReceiveError> {
        // Skip file operations for test stores (in-memory database)
        if self.account_path() == std::path::Path::new(":memory:") {
            println!("💾 Skipped file storage for test store");
            return Ok(());
        }

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
        parsed_email: &crate::ParsedEmail,
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
        parsed_email: &crate::ParsedEmail,
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
    use super::*;

    /// Helper macro for creating RFC 5322 test emails with proper CRLF endings
    /// Supports both static text and variable interpolation
    macro_rules! test_email {
        ($($tt:tt)*) => {
            indoc::formatdoc! { $($tt)* }.replace('\n', "\r\n")
        };
    }

    #[tokio::test]
    async fn test_smtp_receive_basic() {
        let store = crate::Store::create_test();

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

        let result = store.smtp_receive(email.into_bytes()).await;

        // Should succeed with valid P2P addresses
        assert!(result.is_ok());
        let email_id = result.unwrap();
        assert!(!email_id.is_empty());
        assert!(email_id.starts_with("email-"));
    }

    #[tokio::test]
    async fn test_smtp_receive_validation_failure() {
        let store = crate::Store::create_test();

        let email = test_email! {"
            From: external@gmail.com
            To: bob@example.com
            Subject: Test
            
            Body content
        "};

        let result = store.smtp_receive(email.into_bytes()).await;

        // Should fail validation for external From address
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_smtp_receive_missing_headers() {
        let store = crate::Store::create_test();

        let email = test_email! {"
            Subject: No From Header
            
            Body content
        "};

        let result = store.smtp_receive(email.into_bytes()).await;

        // Should fail with missing From header
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_email_integration() {
        // Generate actual valid ID52s for testing
        let from_key = fastn_id52::SecretKey::generate();
        let to_key = fastn_id52::SecretKey::generate();
        let cc_key = fastn_id52::SecretKey::generate();

        let from_id52 = from_key.public_key().id52();
        let to_id52 = to_key.public_key().id52();
        let cc_id52 = cc_key.public_key().id52();

        let email = test_email! {"
            From: alice@{from_id52}.fastn
            To: bob@{to_id52}.local
            CC: charlie@{cc_id52}.fastn
            Subject: Integration Test
            Message-ID: <integration-test@localhost>
            
            Test body content
        "};

        let result = parse_email(email.as_bytes()).unwrap();

        assert_eq!(result.from_addr, format!("alice@{from_id52}.fastn"));
        assert_eq!(result.to_addr, format!("bob@{to_id52}.local"));
        assert_eq!(result.cc_addr, Some(format!("charlie@{cc_id52}.fastn")));
        assert_eq!(result.subject, "Integration Test");

        // P2P routing should be extracted correctly (we are sender)
        assert_eq!(result.our_username, Some("alice".to_string()));
        assert_eq!(result.our_alias_used, Some(from_id52));
        // Recipients info not stored in single fields (multiple possible)
        assert_eq!(result.their_username, None);
        assert_eq!(result.their_alias, None);
    }
}
