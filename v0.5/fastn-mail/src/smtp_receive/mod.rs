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

mod parse_email_headers;
mod validate_email_for_smtp;

pub use parse_email_headers::parse_email_headers;
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
        let parsed_email = parse_email_headers(&raw_message)?;

        // Step 2: Validate email for SMTP acceptance
        validate_email_for_smtp(&parsed_email)?;

        // Display parsed information
        println!("📧 Parsed email:");
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

        // TODO: Generate unique email_id
        // TODO: Store in Sent/Outbox folder (outbound email)
        // TODO: Store raw_message as .eml file
        // TODO: Insert parsed data into fastn_emails table
        // TODO: For fastn P2P recipients, insert into fastn_email_delivery table for queuing

        // Return the generated email_id
        Ok(parsed_email.email_id.clone())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_smtp_receive_integration() {
        // TODO: Test complete SMTP receive flow
    }
}
