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
}

impl Mailer {
    /// Create a new instance of Mail using values from environment variables.
    pub async fn from_env(
        ds: &fastn_ds::DocumentStore,
    ) -> Result<Self, fastn_ds::EnvironmentError> {
        let smtp_username = ds.env("FASTN_SMTP_USERNAME").await?;
        let smtp_password = ds.env("FASTN_SMTP_PASSWORD").await?;
        let smtp_host = ds.env("FASTN_SMTP_HOST").await?;
        let sender_email = ds.env("FASTN_SMTP_SENDER_EMAIL").await?;
        let sender_name = ds.env("FASTN_SMTP_SENDER_NAME").await.ok();

        Ok(Mailer {
            smtp_username,
            smtp_password,
            sender_email,
            sender_name,
            smtp_host,
        })
    }

    // TODO: add support for DKIM
    // https://en.wikipedia.org/wiki/DomainKeys_Identified_Mail
    /// send {body} as html body of the email
    pub async fn send_raw(
        &self,
        enable_email: bool,
        to: lettre::message::Mailbox,
        subject: &str,
        body: String,
    ) -> Result<(), MailError> {
        println!("send_raw");
        if !enable_email {
            tracing::info!("enable_mail is not set, not sending mail to: {}", &to);
            println!("enable_mail is not set, not sending mail to: {}", &to);
            return Ok(());
        }

        // read the env vars

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

        let mailer =
            lettre::AsyncSmtpTransport::<lettre::Tokio1Executor>::starttls_relay(&self.smtp_host)?
                .credentials(creds)
                .build();

        println!("mailer created");

        lettre::AsyncTransport::send(&mailer, email).await?;

        println!("mail sent");

        Ok(())
    }
}
