//! Message types for peer-to-peer communication between accounts
//!
//! This module defines the message format for delivering email between FASTN accounts
//! over the P2P network. The messages must be compatible with full SMTP/IMAP email
//! clients and support the complete RFC 5322 email specification.

use serde::{Deserialize, Serialize};

/// Messages sent from one account to another account
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AccountToAccountMessage {
    /// Peer-to-peer email delivery containing a complete RFC 5322 email message
    Email(EmailMessage),
}

/// Complete email message for P2P delivery
///
/// This represents a full RFC 5322 compliant email message that can be delivered
/// to any SMTP/IMAP client. The raw_message contains the complete email as it
/// would appear on the wire, including all headers and MIME encoding.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmailMessage {
    /// Complete raw email message as bytes (RFC 5322 format)
    /// This is the actual email message that would be sent over SMTP
    /// and stored in an IMAP mailbox, including all headers and body
    pub raw_message: Vec<u8>,

    /// Parsed envelope information for routing and indexing
    pub envelope: EmailEnvelope,

    /// Size of the complete message in bytes
    pub size_bytes: usize,

    /// Unix timestamp when email was originally sent (from Date header)
    pub date_sent: Option<i64>,

    /// Unix timestamp when we received this email
    pub date_received: i64,
}

/// Email envelope information extracted from the raw message for routing and indexing
///
/// This contains the essential information needed for P2P routing, database storage,
/// and IMAP server functionality without having to parse the raw message every time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmailEnvelope {
    /// Message-ID header (unique identifier)
    pub message_id: String,

    /// From header (original sender)
    pub from: String,

    /// To header (can be multiple recipients, comma-separated)
    pub to: String,

    /// CC header (optional, can be multiple recipients)
    pub cc: Option<String>,

    /// BCC header (optional, can be multiple recipients)  
    pub bcc: Option<String>,

    /// Subject header
    pub subject: Option<String>,

    /// In-Reply-To header (for threading)
    pub in_reply_to: Option<String>,

    /// References header (for threading, space-separated Message-IDs)
    pub references: Option<String>,

    /// Content-Type header (for MIME type detection)
    pub content_type: Option<String>,

    /// Whether the email has attachments (multipart/mixed or similar)
    pub has_attachments: bool,

    /// IMAP flags (\\Seen, \\Flagged, \\Draft, \\Answered, \\Deleted, etc.)
    pub flags: Vec<String>,

    /// Target folder for IMAP delivery (INBOX, Sent, Drafts, Trash, etc.)
    pub target_folder: String,
}

impl AccountToAccountMessage {
    /// Create a new email message
    pub fn new_email(
        message_id: String,
        from_alias: String,
        to_alias: String,
        subject: String,
        body: EmailBody,
    ) -> Self {
        let mut headers = std::collections::HashMap::new();
        headers.insert("From".to_string(), from_alias.clone());
        headers.insert("To".to_string(), to_alias.clone());
        headers.insert("Subject".to_string(), subject);
        headers.insert("Date".to_string(), chrono::Utc::now().to_rfc2822());
        headers.insert("Message-ID".to_string(), message_id.clone());

        Self::Email(EmailMessage {
            message_id,
            from_alias,
            to_alias,
            headers,
            body,
            attachments: Vec::new(),
            sent_at: chrono::Utc::now().timestamp(),
        })
    }

    /// Add an attachment to an email message
    pub fn add_attachment(&mut self, attachment: EmailAttachment) -> Result<(), &'static str> {
        match self {
            Self::Email(email) => {
                email.attachments.push(attachment);
                Ok(())
            }
        }
    }

    /// Get the recipient alias ID52 for routing
    pub fn recipient(&self) -> &str {
        match self {
            Self::Email(email) => &email.to_alias,
        }
    }

    /// Get the sender alias ID52
    pub fn sender(&self) -> &str {
        match self {
            Self::Email(email) => &email.from_alias,
        }
    }

    /// Get the message size for network planning
    pub fn size(&self) -> usize {
        match self {
            Self::Email(email) => {
                // Rough estimate: headers + body + attachments
                let headers_size: usize = email
                    .headers
                    .iter()
                    .map(|(k, v)| k.len() + v.len() + 4) // +4 for ": " and "\r\n"
                    .sum();

                let body_size = match &email.body {
                    EmailBody::Text { content } => content.len(),
                    EmailBody::Html { content } => content.len(),
                    EmailBody::Multipart { text, html } => text.len() + html.len(),
                };

                let attachments_size: usize = email.attachments.iter().map(|a| a.size).sum();

                headers_size + body_size + attachments_size
            }
        }
    }
}

impl EmailBody {
    /// Create a simple text email body
    pub fn text(content: String) -> Self {
        Self::Text { content }
    }

    /// Create an HTML email body
    pub fn html(content: String) -> Self {
        Self::Html { content }
    }

    /// Create a multipart email body (text + HTML)
    pub fn multipart(text: String, html: String) -> Self {
        Self::Multipart { text, html }
    }

    /// Get the plain text version of the body
    pub fn as_text(&self) -> &str {
        match self {
            Self::Text { content } => content,
            Self::Html { content } => content, // TODO: Strip HTML tags
            Self::Multipart { text, .. } => text,
        }
    }
}

impl EmailAttachment {
    /// Create a new email attachment
    pub fn new(filename: String, content_type: String, content: Vec<u8>) -> Self {
        let size = content.len();
        Self {
            filename,
            content_type,
            content,
            content_id: None,
            size,
        }
    }

    /// Create an inline attachment with Content-ID
    pub fn new_inline(
        filename: String,
        content_type: String,
        content: Vec<u8>,
        content_id: String,
    ) -> Self {
        let size = content.len();
        Self {
            filename,
            content_type,
            content,
            content_id: Some(content_id),
            size,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_message_creation() {
        let msg = AccountToAccountMessage::new_email(
            "test@example.com".to_string(),
            "alice123".to_string(),
            "bob456".to_string(),
            "Hello World".to_string(),
            EmailBody::text("Hello from Alice!".to_string()),
        );

        assert_eq!(msg.sender(), "alice123");
        assert_eq!(msg.recipient(), "bob456");
        assert!(msg.size() > 0);
    }

    #[test]
    fn test_email_with_attachments() {
        let mut msg = AccountToAccountMessage::new_email(
            "test2@example.com".to_string(),
            "alice123".to_string(),
            "bob456".to_string(),
            "With Attachment".to_string(),
            EmailBody::html("<h1>Hello!</h1>".to_string()),
        );

        let attachment = EmailAttachment::new(
            "document.pdf".to_string(),
            "application/pdf".to_string(),
            vec![1, 2, 3, 4, 5], // Mock PDF content
        );

        msg.add_attachment(attachment).unwrap();

        // Size should include attachment
        assert!(msg.size() > 5); // At least the attachment size
    }

    #[test]
    fn test_multipart_body() {
        let body = EmailBody::multipart(
            "Plain text version".to_string(),
            "<p>HTML version</p>".to_string(),
        );

        assert_eq!(body.as_text(), "Plain text version");
    }
}
