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
        _raw_message: Vec<u8>,
        _sender_id52: &fastn_id52::PublicKey,
    ) -> Result<String, SmtpReceiveError> {
        // TODO: Check AliasNotes.allow_mail for sender_id52
        // TODO: If blocked, return error and log
        // TODO: Parse message headers (validate sender matches From header)
        // TODO: Store in INBOX folder (inbound email)
        // TODO: Generate unique email_id
        // TODO: Store raw_message as .eml file in INBOX/{date}/{email_id}.eml
        // TODO: Insert parsed data into fastn_emails table

        // For now, return a proper error indicating this is not yet implemented
        Err(SmtpReceiveError::MessageParsingFailed {
            message: "P2P email receive not yet implemented".to_string(),
        })
    }
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
