#[derive(thiserror::Error, Debug)]
pub enum MailError {
    #[error("Mail Error: {0}")]
    Mail(#[from] lettre::error::Error),
    #[error("Address Parse Error: {0}")]
    Address(#[from] lettre::address::AddressError),
    #[error("SMTP Error: {0}")]
    Smtp(#[from] lettre::transport::smtp::Error),
}

/// Send SMTP emails
pub struct Mailer {
    smtp_username: String,
    smtp_password: String,
    smtp_host: String,
    sender_email: String,
    sender_name: Option<String>,
    mock: bool,
}

impl Mailer {
    /// Create a new instance of Mail using values from environment variables.
    pub fn from_env() -> fastn_core::Result<Self> {
        let smtp_username = std::env::var("FASTN_SMTP_USERNAME")?;
        let smtp_password = std::env::var("FASTN_SMTP_PASSWORD")?;
        let smtp_host = std::env::var("FASTN_SMTP_HOST")?;
        let sender_email = std::env::var("FASTN_SMTP_SENDER_EMAIL")?;
        let sender_name = std::env::var("FASTN_SMTP_SENDER_NAME").ok();

        Ok(Mailer {
            smtp_username,
            smtp_password,
            sender_email,
            sender_name,
            smtp_host,
            mock: false,
        })
    }

    /// log the email body without actually sending it
    /// useful for testing
    pub fn mock(&mut self) -> &Self {
        self.mock = true;

        self
    }

    // TODO: add support for DKIM
    // https://en.wikipedia.org/wiki/DomainKeys_Identified_Mail
    /// send {body} as html body of the email
    pub async fn send_raw(
        &self,
        to: lettre::message::Mailbox,
        subject: &str,
        body: String,
    ) -> Result<(), MailError> {
        if self.mock {
            println!("to: {}\nsubject: {}\nbody: {}", to, subject, body);
            return Ok(());
        }

        let email = lettre::Message::builder()
            .from(lettre::message::Mailbox::new(
                self.sender_name.clone(),
                self.sender_email.parse::<lettre::Address>()?,
            ))
            .to(to)
            .subject(subject)
            .header(lettre::message::header::ContentType::TEXT_HTML)
            .body(body)?;

        let creds = lettre::transport::smtp::authentication::Credentials::new(
            self.smtp_username.clone(),
            self.smtp_password.clone(),
        );

        let mailer = lettre::AsyncSmtpTransport::<lettre::Tokio1Executor>::relay(&self.smtp_host)?
            .credentials(creds)
            .build();

        let response = lettre::AsyncTransport::send(&mailer, email).await?;

        tracing::info!(
            "sent email: {:?}",
            response.message().collect::<Vec<&str>>()
        );

        Ok(())
    }
}
