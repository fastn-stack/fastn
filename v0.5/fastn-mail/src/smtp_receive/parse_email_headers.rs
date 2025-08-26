//! # Email Header Parsing
//!
//! RFC 5322 compliant email parsing and header extraction.

use crate::errors::SmtpReceiveError;

/// Parse email message and extract headers for processing
pub fn parse_email_headers(raw_message: &[u8]) -> Result<crate::ParsedEmail, SmtpReceiveError> {
    // Parse email message
    let parsed_email = mail_parser::MessageParser::default()
        .parse(raw_message)
        .ok_or_else(|| SmtpReceiveError::MessageParsingFailed {
            message: "Failed to parse RFC 5322 email message".to_string(),
        })?;

    // Extract message ID
    let message_id = parsed_email
        .message_id()
        .unwrap_or("no-message-id")
        .to_string();

    // Extract From address (required)
    let from_addr = parsed_email
        .from()
        .and_then(|from| from.first())
        .and_then(|addr| addr.address())
        .ok_or_else(|| SmtpReceiveError::MessageParsingFailed {
            message: "Missing required From header".to_string(),
        })?
        .to_string();

    // Extract recipient addresses - simplified for now
    // TODO: Implement proper To/CC/BCC parsing

    // Extract subject
    let subject = parsed_email.subject().unwrap_or("(no subject)").to_string();

    // Extract date (if present) - simplified for now
    let date_sent = None;

    // Basic content type detection - use default for now
    let content_type = "text/plain".to_string();

    // Basic attachment detection - simplified for now
    let has_attachments = false;

    // Generate storage information
    let email_id = format!(
        "email-{}-{}",
        chrono::Utc::now().timestamp_millis(),
        message_id.len()
    );
    let folder = "Sent".to_string(); // SMTP emails go to Sent folder
    let timestamp = chrono::Utc::now().format("%Y/%m/%d");
    let file_path = format!("mails/default/Sent/{timestamp}/{email_id}.eml");
    let date_received = chrono::Utc::now().timestamp();
    let size_bytes = raw_message.len();

    // Convert addresses to comma-separated strings for storage
    let from_addr_str = from_addr.to_string();
    let to_addr_str = "to-addresses-todo".to_string();

    Ok(crate::ParsedEmail {
        email_id,
        folder,
        file_path,
        message_id,
        from_addr: from_addr_str,
        to_addr: to_addr_str,
        cc_addr: None,
        bcc_addr: None,
        subject,
        our_alias_used: None,   // TODO: Extract from To addresses
        our_username: None,     // TODO: Extract from To addresses
        their_alias: None,      // TODO: Extract from From address
        their_username: None,   // TODO: Extract from From address
        in_reply_to: None,      // TODO: Extract In-Reply-To header
        email_references: None, // TODO: Extract References header
        date_sent,
        date_received,
        content_type,
        content_encoding: None, // TODO: Extract Content-Transfer-Encoding
        has_attachments,
        size_bytes,
        is_seen: false, // Default IMAP flags
        is_flagged: false,
        is_draft: false,
        is_answered: false,
        is_deleted: false,
        custom_flags: None,
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_email_headers() {
        // TODO: Test RFC 5322 parsing
    }

    #[test]
    fn test_message_id_extraction() {
        // TODO: Test message ID parsing
    }

    #[test]
    fn test_address_extraction() {
        // TODO: Test From/To/CC/BCC extraction
    }
}
