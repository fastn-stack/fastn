//! Message types for peer-to-peer communication between accounts
//!
//! This module defines the message format for delivering email between FASTN accounts
//! over the P2P network. The messages contain complete RFC 5322 email content that
//! can be delivered to any SMTP/IMAP email client.

/// Messages sent from one account to another account
///
/// This enum will be extended in the future with DeviceToAccount and other message types
/// as new entity types are added to the FASTN network.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum AccountToAccountMessage {
    /// Peer-to-peer email delivery
    ///
    /// Contains a complete RFC 5322 email message plus envelope data for efficient processing.
    /// The envelope data allows the recipient to process the email without header parsing.
    Email {
        /// Complete RFC 5322 message as bytes
        ///
        /// This contains everything: headers, body, attachments, MIME encoding.
        /// It's the exact message that would be sent over SMTP or stored in
        /// an IMAP mailbox, ensuring full compatibility with email clients.
        raw_message: Vec<u8>,

        /// SMTP envelope FROM (original sender)
        envelope_from: String,

        /// SMTP envelope TO (specific recipient - this peer)
        envelope_to: String,
    },
}

/// Response from peer for individual email delivery
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EmailDeliveryResponse {
    /// Email ID being responded to
    pub email_id: String,
    /// Delivery result
    pub status: DeliveryStatus,
}

/// Status of individual email delivery
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum DeliveryStatus {
    /// Email accepted and stored in recipient's INBOX
    Accepted,
    /// Email rejected with reason (permanent failure)
    Rejected { reason: String },
}

impl AccountToAccountMessage {
    /// Create a new email message with envelope data
    pub fn new_email(raw_message: Vec<u8>, envelope_from: String, envelope_to: String) -> Self {
        Self::Email {
            raw_message,
            envelope_from,
            envelope_to,
        }
    }

    /// Get the size of the message for network planning
    pub fn size(&self) -> usize {
        match self {
            Self::Email { raw_message, .. } => raw_message.len(),
        }
    }

    /// Get the raw message bytes for storage or transmission
    pub fn raw_bytes(&self) -> &[u8] {
        match self {
            Self::Email { raw_message, .. } => raw_message,
        }
    }

    /// Get envelope data for efficient processing
    pub fn envelope_data(&self) -> Option<(&str, &str)> {
        match self {
            Self::Email {
                envelope_from,
                envelope_to,
                ..
            } => Some((envelope_from, envelope_to)),
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_email_message_creation() {
        let raw_email =
            b"From: alice@example.com\r\nTo: bob@example.com\r\nSubject: Test\r\n\r\nHello World!"
                .to_vec();
        let msg = crate::AccountToAccountMessage::new_email(
            raw_email.clone(),
            "alice@test.com".to_string(),
            "bob@test.com".to_string(),
        );

        assert_eq!(msg.size(), raw_email.len());
        assert_eq!(msg.raw_bytes(), raw_email.as_slice());
    }

    #[test]
    fn test_message_serialization() {
        let raw_email =
            b"From: alice@example.com\r\nTo: bob@example.com\r\nSubject: Test\r\n\r\nHello World!"
                .to_vec();
        let msg = crate::AccountToAccountMessage::new_email(
            raw_email,
            "alice@test.com".to_string(),
            "bob@test.com".to_string(),
        );

        // Should be serializable for P2P transmission
        let serialized = serde_json::to_string(&msg).unwrap();
        let deserialized: crate::AccountToAccountMessage =
            serde_json::from_str(&serialized).unwrap();

        assert_eq!(msg, deserialized);
    }

    #[test]
    fn test_multipart_email() {
        let multipart_email = b"From: alice@example.com\r\n\
To: bob@example.com\r\n\
Subject: Test with attachment\r\n\
Content-Type: multipart/mixed; boundary=\"boundary123\"\r\n\
\r\n\
--boundary123\r\n\
Content-Type: text/plain\r\n\
\r\n\
Hello World!\r\n\
\r\n\
--boundary123\r\n\
Content-Type: application/pdf; name=\"document.pdf\"\r\n\
Content-Disposition: attachment; filename=\"document.pdf\"\r\n\
\r\n\
[PDF content would be here]\r\n\
\r\n\
--boundary123--\r\n"
            .to_vec();

        let msg = crate::AccountToAccountMessage::new_email(
            multipart_email.clone(),
            "alice@test.com".to_string(),
            "bob@test.com".to_string(),
        );

        // Should handle any RFC 5322 compliant email
        assert_eq!(msg.size(), multipart_email.len());
        assert_eq!(msg.raw_bytes(), multipart_email.as_slice());
    }
}
