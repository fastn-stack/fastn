//! # Email Header Parsing
//!
//! RFC 5322 compliant email parsing and header extraction.

use crate::errors::SmtpReceiveError;

/// Parse email message and extract headers for processing
pub fn parse_email(raw_message: &[u8]) -> Result<crate::ParsedEmail, SmtpReceiveError> {
    // Parse email message
    let parsed_email = mail_parser::MessageParser::default()
        .parse(raw_message)
        .ok_or_else(|| SmtpReceiveError::MessageParsingFailed {
            message: "Failed to parse RFC 5322 email message".to_string(),
        })?;

    // Extract message ID (generate unique one if missing)
    let message_id = parsed_email
        .message_id()
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            // Generate unique message ID if missing
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};

            let mut hasher = DefaultHasher::new();
            raw_message.hash(&mut hasher);
            chrono::Utc::now().timestamp_millis().hash(&mut hasher);

            format!(
                "generated-{}-{:x}",
                chrono::Utc::now().timestamp_millis(),
                hasher.finish()
            )
        });

    // Extract From address (required)
    let from_addr = parsed_email
        .from()
        .and_then(|from| from.first())
        .and_then(|addr| addr.address())
        .ok_or_else(|| SmtpReceiveError::MessageParsingFailed {
            message: "Missing required From header".to_string(),
        })?
        .to_string();

    // Extract To addresses (required) - simplified for now
    let to_addr_str = "to-addresses-todo".to_string();
    
    // Extract CC addresses (optional) - simplified for now  
    let cc_addr_str = None;
    
    // Extract BCC addresses (optional) - simplified for now
    let bcc_addr_str = None;

    // Extract subject
    let subject = parsed_email.subject().unwrap_or("(no subject)").to_string();

    // Extract date (if present)
    let date_sent = parsed_email.date().map(|dt| dt.to_timestamp());

    // Extract threading headers - simplified for now
    let in_reply_to = None; // TODO: Extract In-Reply-To header properly
    let email_references = None; // TODO: Extract References header properly

    // Extract MIME information
    let content_type = "text/plain".to_string(); // TODO: Implement proper content type detection
    let content_encoding = None; // TODO: Extract Content-Transfer-Encoding
    let has_attachments = parsed_email.attachments().count() > 0;

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

    // Convert from address to string for storage
    let from_addr_str = from_addr;

    Ok(crate::ParsedEmail {
        email_id,
        folder,
        file_path,
        message_id,
        from_addr: from_addr_str,
        to_addr: to_addr_str,
        cc_addr: cc_addr_str,
        bcc_addr: bcc_addr_str,
        subject,
        our_alias_used: None, // TODO: Extract from To addresses
        our_username: None,   // TODO: Extract from To addresses
        their_alias: None,    // TODO: Extract from From address
        their_username: None, // TODO: Extract from From address
        in_reply_to,
        email_references,
        date_sent,
        date_received,
        content_type,
        content_encoding,
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

// TODO: Implement proper helper functions once mail-parser API is better understood

#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_email() {
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
