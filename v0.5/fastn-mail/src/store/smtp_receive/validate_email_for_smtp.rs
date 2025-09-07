//! # SMTP Email Validation
//!
//! Validates parsed email messages for SMTP acceptance with P2P-only constraints.

use fastn_mail::errors::SmtpReceiveError;

/// Validate email message for SMTP acceptance (P2P only, no external email)
pub fn validate_email_for_smtp(
    parsed_email: &fastn_mail::ParsedEmail,
) -> Result<(), SmtpReceiveError> {
    // 1. Validate From address format and ownership
    validate_from_address_ownership(&parsed_email.from_addr)?;

    // 2. Validate all recipients are P2P addresses (no external email)
    validate_all_recipients_are_p2p(parsed_email)?;

    // 3. Validate message size limits
    validate_message_size(parsed_email.size_bytes)?;

    // 4. Validate required headers are present
    validate_required_headers(parsed_email)?;

    Ok(())
}

/// Validate From address is one of our account's aliases
fn validate_from_address_ownership(from_addr: &str) -> Result<(), SmtpReceiveError> {
    // Parse From address to extract ID52 component
    let (_username, id52) = parse_email_address(from_addr)?;

    // TODO: Check if this ID52 belongs to our account
    // This requires access to account aliases list
    // For now, just validate the format

    println!("✅ From address format valid: {from_addr} (ID52: {id52})");
    Ok(())
}

/// Validate all recipients are valid P2P addresses (no external email allowed)
fn validate_all_recipients_are_p2p(
    parsed_email: &fastn_mail::ParsedEmail,
) -> Result<(), SmtpReceiveError> {
    // Validate To addresses
    for addr in parsed_email.to_addr.split(',') {
        let addr = addr.trim();
        if !addr.is_empty() {
            let (_username, id52) = parse_email_address(addr)?;
            println!("✅ To address valid: {addr} (ID52: {id52})");
        }
    }

    // Validate CC addresses
    if let Some(cc_addr) = &parsed_email.cc_addr {
        for addr in cc_addr.split(',') {
            let addr = addr.trim();
            if !addr.is_empty() {
                let (_username, id52) = parse_email_address(addr)?;
                println!("✅ CC address valid: {addr} (ID52: {id52})");
            }
        }
    }

    // Validate BCC addresses
    if let Some(bcc_addr) = &parsed_email.bcc_addr {
        for addr in bcc_addr.split(',') {
            let addr = addr.trim();
            if !addr.is_empty() {
                let (_username, id52) = parse_email_address(addr)?;
                println!("✅ BCC address valid: {addr} (ID52: {id52})");
            }
        }
    }

    Ok(())
}

/// Parse email address and validate ID52 format: username@id52.domain
fn parse_email_address(email: &str) -> Result<(String, String), SmtpReceiveError> {
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return Err(SmtpReceiveError::MessageParsingFailed {
            message: format!("Invalid email format: {email}"),
        });
    }

    let username = parts[0].to_string();
    let domain_part = parts[1];

    // Parse domain to extract ID52: id52.domain
    let domain_parts: Vec<&str> = domain_part.split('.').collect();
    if domain_parts.len() < 2 {
        return Err(SmtpReceiveError::MessageParsingFailed {
            message: format!("Invalid domain format: {domain_part}"),
        });
    }

    let id52 = domain_parts[0];

    // Validate ID52 format (52 characters, valid public key)
    if id52.len() != 52 {
        return Err(SmtpReceiveError::MessageParsingFailed {
            message: format!(
                "Invalid ID52 length: {id52} (expected 52 chars, got {})",
                id52.len()
            ),
        });
    }

    // Verify it's a valid fastn_id52::PublicKey
    let _public_key: fastn_id52::PublicKey =
        id52.parse()
            .map_err(|_| SmtpReceiveError::MessageParsingFailed {
                message: format!("Invalid ID52 format: {id52}"),
            })?;

    Ok((username, id52.to_string()))
}

/// Validate message size is within acceptable limits
fn validate_message_size(size_bytes: usize) -> Result<(), SmtpReceiveError> {
    const MAX_MESSAGE_SIZE: usize = 25 * 1024 * 1024; // 25MB limit

    if size_bytes > MAX_MESSAGE_SIZE {
        return Err(SmtpReceiveError::MessageParsingFailed {
            message: format!(
                "Message too large: {size_bytes} bytes (limit: {MAX_MESSAGE_SIZE} bytes)"
            ),
        });
    }

    println!("✅ Message size valid: {size_bytes} bytes");
    Ok(())
}

/// Validate required headers are present
fn validate_required_headers(
    parsed_email: &fastn_mail::ParsedEmail,
) -> Result<(), SmtpReceiveError> {
    if parsed_email.from_addr.is_empty() {
        return Err(SmtpReceiveError::MessageParsingFailed {
            message: "Missing required From header".to_string(),
        });
    }

    if parsed_email.to_addr.is_empty() {
        return Err(SmtpReceiveError::MessageParsingFailed {
            message: "Missing required To header".to_string(),
        });
    }

    if parsed_email.message_id.is_empty() {
        return Err(SmtpReceiveError::MessageParsingFailed {
            message: "Missing required Message-ID header".to_string(),
        });
    }

    println!("✅ Required headers present");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_parsed_email() -> fastn_mail::ParsedEmail {
        // Generate valid ID52s for testing
        let from_key = fastn_id52::SecretKey::generate();
        let to_key = fastn_id52::SecretKey::generate();
        let from_id52 = from_key.public_key().id52();
        let to_id52 = to_key.public_key().id52();

        fastn_mail::ParsedEmail {
            email_id: "test-email-id".to_string(),
            folder: "Sent".to_string(),
            file_path: "test.eml".to_string(),
            message_id: "test-message-id".to_string(),
            from_addr: format!("alice@{from_id52}.fastn"),
            to_addr: format!("bob@{to_id52}.local"),
            cc_addr: None,
            bcc_addr: None,
            subject: "Test Subject".to_string(),
            our_alias_used: Some(to_id52),
            our_username: Some("bob".to_string()),
            their_alias: Some(from_id52),
            their_username: Some("alice".to_string()),
            in_reply_to: None,
            email_references: None,
            date_sent: None,
            date_received: chrono::Utc::now().timestamp(),
            content_type: "text/plain".to_string(),
            content_encoding: None,
            has_attachments: false,
            size_bytes: 100,
            is_seen: false,
            is_flagged: false,
            is_draft: false,
            is_answered: false,
            is_deleted: false,
            custom_flags: None,
        }
    }

    #[test]
    fn test_validate_email_for_smtp_success() {
        let email = create_test_parsed_email();

        // Should succeed with valid P2P addresses
        let result = validate_email_for_smtp(&email);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_email_address_valid_id52() {
        let addr = "alice@i66fo538lfl5ombdf6tcdbrabp4hmp9asv7nrffuc2im13ct4q60.fastn";

        let result = parse_email_address(addr).unwrap();

        assert_eq!(
            result,
            (
                "alice".to_string(),
                "i66fo538lfl5ombdf6tcdbrabp4hmp9asv7nrffuc2im13ct4q60".to_string()
            )
        );
    }

    #[test]
    fn test_parse_email_address_invalid_format() {
        let addr = "invalid-email";

        let result = parse_email_address(addr);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_email_address_invalid_id52() {
        let addr = "alice@invalid-id52.fastn";

        let result = parse_email_address(addr);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_message_size_ok() {
        let result = validate_message_size(1000);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_message_size_too_large() {
        let result = validate_message_size(30 * 1024 * 1024); // 30MB
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_required_headers_success() {
        let email = create_test_parsed_email();

        let result = validate_required_headers(&email);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_required_headers_missing_from() {
        let mut email = create_test_parsed_email();
        email.from_addr = "".to_string();

        let result = validate_required_headers(&email);
        assert!(result.is_err());
    }
}
