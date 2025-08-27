//! # P2P Email Receive Module
//!
//! Handles incoming email messages from other peers via P2P connections.
//!
//! ## Requirements
//!
//! ### Permission Control
//! - Check AliasNotes.allow_mail boolean for sender permission
//! - Block emails from peers where allow_mail = false
//! - Log blocked attempts for security auditing
//!
//! ### Email Storage
//! - Store in INBOX folder (incoming emails)
//! - Generate unique email_id with timestamp
//! - Save as .eml files with proper directory structure
//! - Update fastn_emails table with metadata
//!
//! ### Address Validation
//! - Validate sender ID52 matches message From header
//! - Parse To/CC/BCC for proper routing validation
//! - Handle mixed fastn/external email addresses

use crate::errors::SmtpReceiveError;

impl crate::Store {
    /// P2P receives an email from another peer and stores in INBOX
    ///
    /// Flow: Peer P2P message → Check permissions → Store in INBOX
    pub async fn p2p_receive_email(
        &self,
        raw_message: Vec<u8>,
        sender_id52: &fastn_id52::PublicKey,
    ) -> Result<String, SmtpReceiveError> {
        // Step 1: Parse email headers
        let mut parsed_email = crate::smtp_receive::parse_email(&raw_message)?;

        // Step 2: Override folder to INBOX (P2P emails are inbound)
        parsed_email.folder = "INBOX".to_string();
        let timestamp = chrono::Utc::now().format("%Y/%m/%d");
        parsed_email.file_path = format!(
            "mails/default/INBOX/{timestamp}/{}.eml",
            parsed_email.email_id
        );

        // Step 3: Check sender permissions (allow_mail in AliasNotes)
        check_sender_permissions(sender_id52).await?;

        // Step 4: Store email file in INBOX
        self.store_email_file_inbox(&parsed_email.file_path, &raw_message)
            .await?;

        // Step 5: Store email metadata in database
        self.store_email_metadata_inbox(&parsed_email).await?;

        println!(
            "✅ P2P email from {sender_id52} stored in INBOX with ID: {}",
            parsed_email.email_id
        );
        Ok(parsed_email.email_id)
    }

    /// Store P2P email file in INBOX
    async fn store_email_file_inbox(
        &self,
        file_path: &str,
        raw_message: &[u8],
    ) -> Result<(), SmtpReceiveError> {
        let full_path = self.account_path().join(file_path);

        // Create directory structure if needed
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                SmtpReceiveError::MessageParsingFailed {
                    message: format!("Failed to create INBOX directory {}: {e}", parent.display()),
                }
            })?;
        }

        // Check if file already exists
        if full_path.exists() {
            return Err(SmtpReceiveError::MessageParsingFailed {
                message: format!("P2P email file already exists: {}", full_path.display()),
            });
        }

        // Write email file
        std::fs::write(&full_path, raw_message).map_err(|e| {
            SmtpReceiveError::MessageParsingFailed {
                message: format!(
                    "Failed to write P2P email file {}: {e}",
                    full_path.display()
                ),
            }
        })?;

        println!("📨 Stored P2P email file: {}", full_path.display());
        Ok(())
    }

    /// Store P2P email metadata in database
    async fn store_email_metadata_inbox(
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
                message: format!("Failed to insert P2P email metadata: {e}"),
            }
        })?;

        println!(
            "📨 Stored P2P email metadata in INBOX: {}",
            parsed_email.email_id
        );
        Ok(())
    }
}

/// Check if sender is allowed to send mail (placeholder)
async fn check_sender_permissions(
    _sender_id52: &fastn_id52::PublicKey,
) -> Result<(), SmtpReceiveError> {
    // TODO: Check AliasNotes.allow_mail for sender
    // For now, allow all senders
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_peer_permission_checking() {
        // TODO: Test allow_mail = false blocks messages
    }

    #[test]
    fn test_sender_validation() {
        // TODO: Test sender ID52 matches From header
    }

    #[test]
    fn test_inbox_storage() {
        // TODO: Test proper INBOX file/db storage
    }
}
