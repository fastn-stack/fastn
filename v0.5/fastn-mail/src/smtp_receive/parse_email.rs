//! # Email Header Parsing
//!
//! RFC 5322 compliant email parsing and header extraction.

use crate::errors::SmtpReceiveError;

/// Parse email message and extract headers for processing
pub fn parse_email(raw_message: &[u8]) -> Result<crate::ParsedEmail, SmtpReceiveError> {
    // Temporary: Simple text-based parsing for testing
    // TODO: Replace with proper mail-parser once format issues are resolved
    let message_text = String::from_utf8_lossy(raw_message);

    // Extract headers using simple text parsing
    let headers = extract_headers_simple(&message_text)?;

    // Extract message ID (generate unique one if missing)
    let message_id = headers.get("Message-ID").cloned().unwrap_or_else(|| {
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
    let from_addr = headers
        .get("From")
        .ok_or_else(|| SmtpReceiveError::MessageParsingFailed {
            message: "Missing required From header".to_string(),
        })?
        .clone();

    // Extract To addresses (required)
    let to_addr_str = headers
        .get("To")
        .ok_or_else(|| SmtpReceiveError::MessageParsingFailed {
            message: "Missing required To header".to_string(),
        })?
        .clone();

    // Extract CC addresses (optional)
    let cc_addr_str = headers.get("CC").cloned();

    // Extract BCC addresses (optional)
    let bcc_addr_str = headers.get("BCC").cloned();

    // Extract subject
    let subject = headers
        .get("Subject")
        .unwrap_or(&"(no subject)".to_string())
        .clone();

    // Extract date - simplified for now
    let date_sent = None; // TODO: Parse Date header

    // Extract threading headers - simplified for now
    let in_reply_to = headers.get("In-Reply-To").cloned();
    let email_references = headers.get("References").cloned();

    // Extract MIME information - simplified for now
    let content_type = headers
        .get("Content-Type")
        .unwrap_or(&"text/plain".to_string())
        .clone();
    let content_encoding = headers.get("Content-Transfer-Encoding").cloned();
    let has_attachments = content_type.contains("multipart");

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

    // Extract P2P routing information from addresses
    let (their_username, their_alias) = parse_id52_address(&from_addr).unwrap_or((None, None));

    let (our_username, our_alias_used) = parse_id52_address(&to_addr_str).unwrap_or((None, None));

    Ok(crate::ParsedEmail {
        email_id,
        folder,
        file_path,
        message_id,
        from_addr,
        to_addr: to_addr_str,
        cc_addr: cc_addr_str,
        bcc_addr: bcc_addr_str,
        subject,
        our_alias_used,
        our_username,
        their_alias,
        their_username,
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

/// Simple text-based header extraction for testing (temporary)
fn extract_headers_simple(
    message_text: &str,
) -> Result<std::collections::HashMap<String, String>, SmtpReceiveError> {
    let mut headers = std::collections::HashMap::new();

    // Split message into header and body parts
    let parts: Vec<&str> = message_text.splitn(2, "\r\n\r\n").collect();
    if parts.is_empty() {
        return Err(SmtpReceiveError::MessageParsingFailed {
            message: "No headers found in message".to_string(),
        });
    }

    let header_section = parts[0];

    // Parse each header line
    for line in header_section.lines() {
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim().to_string();
            let value = value.trim().to_string();
            headers.insert(key, value);
        }
    }

    Ok(headers)
}

/// Parse email address to extract username and ID52 components for P2P routing
///
/// Parses format: username@id52.domain
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
    if domain_parts.len() < 2 {
        return Ok((None, None)); // Not a fastn domain format
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

    #[test]
    fn test_parse_email_basic() {
        let email = test_email! {"
            From: alice@test.com
            To: bob@example.com
            Subject: Test
            
            Hello World!
        "};

        let result = parse_email(email.as_bytes()).unwrap();

        assert_eq!(result.from_addr, "alice@test.com");
        assert_eq!(result.to_addr, "bob@example.com");
        assert_eq!(result.subject, "Test");
        assert_eq!(result.folder, "Sent");
        assert!(!result.email_id.is_empty());
    }

    #[test]
    fn test_parse_email_with_cc_bcc() {
        let email = test_email! {"
            From: sender@test.com
            To: to@example.com
            CC: cc@example.com
            BCC: bcc@example.com
            Subject: Multi-recipient
            
            Body content here
        "};

        let result = parse_email(email.as_bytes()).unwrap();

        assert_eq!(result.from_addr, "sender@test.com");
        assert_eq!(result.to_addr, "to@example.com");
        assert_eq!(result.cc_addr, Some("cc@example.com".to_string()));
        assert_eq!(result.bcc_addr, Some("bcc@example.com".to_string()));
        assert_eq!(result.subject, "Multi-recipient");
    }

    #[test]
    fn test_generated_message_id() {
        let email = test_email! {"
            From: alice@test.com
            To: bob@example.com
            Subject: No Message ID
            
            Body content
        "};

        let result = parse_email(email.as_bytes()).unwrap();

        // Should generate unique message ID when missing
        assert!(result.message_id.starts_with("generated-"));
        assert!(result.message_id.len() > 20);
    }

    #[test]
    fn test_parse_id52_address_fastn() {
        let addr = "alice@i66fo538lfl5ombdf6tcdbrabp4hmp9asv7nrffuc2im13ct4q60.fastn";

        let (username, id52) = parse_id52_address(addr).unwrap();

        assert_eq!(username, Some("alice".to_string()));
        assert_eq!(
            id52,
            Some("i66fo538lfl5ombdf6tcdbrabp4hmp9asv7nrffuc2im13ct4q60".to_string())
        );
    }

    #[test]
    fn test_parse_id52_address_external() {
        let addr = "alice@external.com";

        let (username, id52) = parse_id52_address(addr).unwrap();

        assert_eq!(username, None);
        assert_eq!(id52, None);
    }

    #[test]
    fn test_parse_id52_address_invalid() {
        let addr = "invalid-email";

        let (username, id52) = parse_id52_address(addr).unwrap();

        assert_eq!(username, None);
        assert_eq!(id52, None);
    }

    #[test]
    fn test_extract_headers_simple() {
        let message =
            "From: alice@test.com\r\nTo: bob@example.com\r\nSubject: Test\r\n\r\nBody content";

        let headers = extract_headers_simple(message).unwrap();

        assert_eq!(headers.get("From"), Some(&"alice@test.com".to_string()));
        assert_eq!(headers.get("To"), Some(&"bob@example.com".to_string()));
        assert_eq!(headers.get("Subject"), Some(&"Test".to_string()));
    }

    #[test]
    fn test_p2p_routing_extraction() {
        // Use valid ID52 for testing
        let to_key = fastn_id52::SecretKey::generate();
        let to_id52 = to_key.public_key().id52();

        let email = test_email! {"
            From: alice@external.com
            To: bob@{to_id52}.fastn
            Subject: P2P Test
            
            Body content
        "};

        let result = parse_email(email.as_bytes()).unwrap();

        // From address is external
        assert_eq!(result.their_username, None);
        assert_eq!(result.their_alias, None);

        // To address is fastn peer
        assert_eq!(result.our_username, Some("bob".to_string()));
        assert_eq!(result.our_alias_used, Some(to_id52));
    }
}
