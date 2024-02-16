pub(crate) enum Route {
    Login,
    GithubLogin,
    GithubCallback,
    Logout,
    CreateAccount,
    EmailConfirmationSent,
    ConfirmEmail,
    ResendConfirmationEmail,
    Onboarding,
    ForgotPassword,
    ForgotPasswordSuccess,
    ResetPassword,
    SetPassword,
    SetPasswordSuccess,
    Invalid,
}

impl From<&str> for Route {
    fn from(s: &str) -> Self {
        match s {
            "/-/auth/login/" => Self::Login,
            "/-/auth/github/" => Self::GithubLogin,
            "/-/auth/github/callback/" => Self::GithubCallback,
            "/-/auth/logout/" => Self::Logout,
            "/-/auth/create-account/" => Self::CreateAccount,
            "/-/auth/email-confirmation-sent/" => Self::EmailConfirmationSent,
            "/-/auth/confirm-email/" => Self::ConfirmEmail,
            "/-/auth/resend-confirmation-email/" => Self::ResendConfirmationEmail,
            "/-/auth/onboarding/" => Self::Onboarding,
            "/-/auth/forgot-password/" => Self::ForgotPassword,
            "/-/auth/forgot-password-success/" => Self::ForgotPasswordSuccess,
            "/-/auth/reset-password/" => Self::ResetPassword,
            "/-/auth/set-password/" => Self::SetPassword,
            "/-/auth/set-password-success/" => Self::SetPasswordSuccess,
            _ => Self::Invalid,
        }
    }
}

impl std::fmt::Display for Route {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Login => write!(f, "/-/auth/login/"),
            Self::GithubLogin => write!(f, "/-/auth/github/"),
            Self::GithubCallback => write!(f, "/-/auth/github/callback/"),
            Self::Logout => write!(f, "/-/auth/logout/"),
            Self::CreateAccount => write!(f, "/-/auth/create-account/"),
            Self::EmailConfirmationSent => write!(f, "/-/auth/email-confirmation-sent/"),
            Self::ConfirmEmail => write!(f, "/-/auth/confirm-email/"),
            Self::ResendConfirmationEmail => write!(f, "/-/auth/resend-confirmation-email/"),
            Self::Onboarding => write!(f, "/-/auth/onboarding/"),
            Self::ForgotPassword => write!(f, "/-/auth/forgot-password/"),
            Self::ForgotPasswordSuccess => write!(f, "/-/auth/forgot-password-success/"),
            Self::ResetPassword => write!(f, "/-/auth/reset-password/"),
            Self::SetPassword => write!(f, "/-/auth/set-password/"),
            Self::SetPasswordSuccess => write!(f, "/-/auth/set-password-success/"),
            Self::Invalid => write!(f, "invalid route"),
        }
    }
}
