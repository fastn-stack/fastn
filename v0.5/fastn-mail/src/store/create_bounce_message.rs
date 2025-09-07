//! # Create Bounce Message

use fastn_mail::errors::*;

impl fastn_mail::Store {
    /// Create bounce message for rejected email delivery
    pub async fn create_bounce_message(
        &self,
        original_email_id: &str,
        rejection_reason: &str,
    ) -> Result<String, SmtpReceiveError> {
        tracing::info!(
            "📝 Creating bounce message for rejected email {}: {}",
            original_email_id,
            rejection_reason
        );

        // Create RFC 3464-style bounce message
        let bounce_subject = format!("Mail Delivery Failure: {original_email_id}");
        let bounce_body = format!(
            "Your email could not be delivered to the recipient.\n\n\
            Original Email ID: {original_email_id}\n\
            Failure Reason: {rejection_reason}\n\n\
            This is an automated message from the fastn mail delivery system.\n\
            The original message has been removed from the delivery queue."
        );

        // Build bounce email in RFC 5322 format
        let timestamp = chrono::Utc::now().format("%a, %d %b %Y %H:%M:%S +0000");
        let bounce_message_id = format!("bounce-{}", uuid::Uuid::new_v4());

        let bounce_email = format!(
            "From: Mail Delivery System <mailer-daemon@system.local>\r\n\
            To: Original Sender\r\n\
            Subject: {bounce_subject}\r\n\
            Date: {timestamp}\r\n\
            Message-ID: <{bounce_message_id}>\r\n\
            MIME-Version: 1.0\r\n\
            Content-Type: text/plain; charset=utf-8\r\n\
            \r\n\
            {bounce_body}"
        );

        // Store bounce message in sender's INBOX using p2p_receive_email
        // This puts the bounce in INBOX where the sender will see it
        let system_sender = fastn_id52::SecretKey::generate().public_key(); // System identity
        let bounce_email_id = self
            .p2p_receive_email(
                &format!("mailer-daemon@{}.system", system_sender.id52()),
                "original-sender@ourhost.local", // Placeholder
                bounce_email.into_bytes(),
            )
            .await?;

        tracing::info!(
            "📝 Bounce message created with ID {} in INBOX",
            bounce_email_id
        );
        Ok(bounce_email_id)
    }
}
