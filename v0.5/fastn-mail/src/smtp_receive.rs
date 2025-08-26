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
//! ### Address Parsing
//! - Extract recipients with format: `<username>@<id52>.<domain>`
//! - Separate fastn peers (valid ID52) from external addresses
//! - Queue fastn peer emails in fastn_email_delivery table
//! - Handle external email routing (future feature)
//!
//! ### Storage Strategy
//! - Store in Sent folder (user sent this email via SMTP)
//! - Generate unique email_id and timestamp-based file paths
//! - Save complete .eml files for IMAP compatibility
//! - Index metadata in fastn_emails table

use crate::errors::SmtpReceiveError;

impl crate::Store {
    /// SMTP server receives an email and handles delivery (local storage or P2P queuing)
    ///
    /// Flow: External SMTP → Store in Sent → Queue for P2P delivery to peers
    pub async fn smtp_receive(&self, _raw_message: Vec<u8>) -> Result<String, SmtpReceiveError> {
        // TODO: Parse message headers to extract:
        // - message_id, from_addr, to_addr, cc_addr, bcc_addr, subject
        // - date_sent, content_type, has_attachments
        // - our_alias_used, our_username, their_alias, their_username

        // TODO: Store in Sent/Outbox folder (outbound email)
        // TODO: Generate unique email_id
        // TODO: Generate file_path: mails/default/Sent/{date}/{email_id}.eml
        // TODO: Store raw_message as .eml file
        // TODO: Insert parsed data into fastn_emails table
        // TODO: For fastn P2P recipients, insert into fastn_email_delivery table for queuing
        // TODO: Handle external recipients (forward to external SMTP)

        // For now, return a proper error indicating this is not yet implemented
        Err(SmtpReceiveError::MessageParsingFailed {
            message: "SMTP message parsing and storage not yet implemented".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_email_address_validation() {
        // TODO: Test valid formats
        // alice@i66fo538lfl5ombdf6tcdbrabp4hmp9asv7nrffuc2im13ct4q60.fastn
        // bob@j77gp649mgm6pnce...@local
    }

    #[test]
    fn test_p2p_queuing() {
        // TODO: Test fastn peer emails get queued for delivery
    }

    #[test]
    fn test_sent_folder_storage() {
        // TODO: Test storage in Sent folder for outbound emails
    }
}
