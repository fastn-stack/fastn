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

use fastn_mail::errors::SmtpReceiveError;
use tracing::info;

impl fastn_mail::Store {
    /// P2P receives an email from another peer and stores in INBOX
    ///
    /// Flow: Peer P2P message → Use envelope data → Store efficiently
    /// Note: sender_id52 is implicit from the authenticated P2P connection
    pub async fn p2p_receive_email(
        &self,
        envelope_from: &str,
        envelope_to: &str,
        raw_message: Vec<u8>,
    ) -> Result<String, SmtpReceiveError> {
        // P2P receives store in INBOX (incoming email from peer)
        let email_id = self
            .inbox_receive(
                envelope_from,
                &[envelope_to.to_string()], // Single recipient (this peer)
                raw_message,
            )
            .await?;

        // smtp_receive already handled all storage and P2P queuing
        info!(
            "📧 P2P email from {} to {} stored with ID: {}",
            envelope_from, envelope_to, email_id
        );
        Ok(email_id)
    }
}
