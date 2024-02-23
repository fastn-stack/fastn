mod confirm_email;
mod create_account;
mod create_and_send_confirmation_email;
mod email_confirmation_sent;
mod login;
mod onboarding;
mod resend_confirmation_email;
mod set_password;
mod urls;

pub(crate) use {
    confirm_email::confirm_email,
    create_account::create_account,
    create_and_send_confirmation_email::create_and_send_confirmation_email,
    email_confirmation_sent::email_confirmation_sent,
    login::login,
    onboarding::onboarding,
    resend_confirmation_email::resend_confirmation_email,
    set_password::*,
    urls::{confirmation_link, redirect_url_from_next},
};

/// check if it has been 3 days since the verification code was sent
/// can be configured using EMAIL_CONFIRMATION_EXPIRE_DAYS
async fn key_expired(ds: &fastn_ds::DocumentStore, sent_at: chrono::DateTime<chrono::Utc>) -> bool {
    let expiry_limit_in_days: u64 = ds
        .env("EMAIL_CONFIRMATION_EXPIRE_DAYS")
        .await
        .unwrap_or("3".to_string())
        .parse()
        .expect("EMAIL_CONFIRMATION_EXPIRE_DAYS should be a number");

    sent_at
        .checked_add_days(chrono::Days::new(expiry_limit_in_days))
        .unwrap()
        <= chrono::offset::Utc::now()
}

fn confirmation_mail_body(content: String, link: &str) -> String {
    // content will have a placeholder for the link
    let content = content.replace("{{link}}", link);

    content.to_string()
}

fn generate_key(length: usize) -> String {
    let mut rng = rand::thread_rng();
    rand::distributions::DistString::sample_string(
        &rand::distributions::Alphanumeric,
        &mut rng,
        length,
    )
}

fn email_confirmation_sent_ftd() -> &'static str {
    r#"
    -- auth.email-confirmation-request-sent-page:
    "#
}

fn forgot_password_request_success_ftd() -> &'static str {
    r#"
    -- auth.forgot-password-request-email-sent-page:
    "#
}

fn set_password_form_ftd() -> &'static str {
    r#"
    -- auth.set-password-form-page:
    "#
}

fn set_password_success_ftd() -> &'static str {
    r#"
    -- auth.set-password-success-page:
    "#
}

fn forgot_password_form_ftd() -> &'static str {
    r#"
    -- auth.forgot-password-form-page:
    "#
}

fn create_account_ftd() -> &'static str {
    r#"
    -- auth.create-account-page:
    "#
}

fn login_ftd() -> &'static str {
    r#"
    -- auth.login-page:
    "#
}

fn onboarding_ftd() -> &'static str {
    r#"
    -- auth.onboarding-page:
    "#
}
