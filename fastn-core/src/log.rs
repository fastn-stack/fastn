#[derive(Debug, Clone)]
pub enum EventKind {
    Auth(AuthEvent),
}

#[derive(Debug, Clone)]
pub enum AuthEvent {
    Initial,
    Login,
    Logout,
    GithubLogin,
    GithubCallback,
    CreateAccount,
    EmailConfirmation,
    ConfirmEmail,
    ResendConfirmationEmail,
    Onboarding,
    ForgotPassword,
    ForgotPasswordSuccess,
    SetPassword,
    SetPasswordSuccess,
    InvalidRoute,
}

#[derive(Debug, Clone)]
pub enum EntityKind {
    Myself,
}

#[derive(Debug, Clone)]
pub enum LogLevel {
    Info(InfoLevel),
    Error(ErrorLevel),
    // todo: implement this
    // Warning(WarningLevel),
}

impl LogLevel {
    pub fn from(ekind: &fastn_core::log::EventKind, okind: &fastn_core::log::EntityKind) -> Self {
        match (ekind, okind) {
            (EventKind::Auth(event), EntityKind::Myself) => match event {
                AuthEvent::Initial => LogLevel::Info(InfoLevel::Auth(AuthInfoLevel::Initial)),
                AuthEvent::Login => LogLevel::Info(InfoLevel::Auth(AuthInfoLevel::LoginRoute)),
                AuthEvent::Logout => LogLevel::Info(InfoLevel::Auth(AuthInfoLevel::LogoutRoute)),
                AuthEvent::GithubLogin => {
                    LogLevel::Info(InfoLevel::Auth(AuthInfoLevel::GithubLoginRoute))
                }
                AuthEvent::GithubCallback => {
                    LogLevel::Info(InfoLevel::Auth(AuthInfoLevel::GithubCallbackRoute))
                }
                AuthEvent::CreateAccount => {
                    LogLevel::Info(InfoLevel::Auth(AuthInfoLevel::CreateAccountRoute))
                }
                AuthEvent::EmailConfirmation => {
                    LogLevel::Info(InfoLevel::Auth(AuthInfoLevel::EmailConfirmationSentRoute))
                }
                AuthEvent::ConfirmEmail => {
                    LogLevel::Info(InfoLevel::Auth(AuthInfoLevel::ConfirmEmailRoute))
                }
                AuthEvent::ResendConfirmationEmail => {
                    LogLevel::Info(InfoLevel::Auth(AuthInfoLevel::ResendConfirmationEmailRoute))
                }
                AuthEvent::Onboarding => {
                    LogLevel::Info(InfoLevel::Auth(AuthInfoLevel::OnboardingRoute))
                }
                AuthEvent::ForgotPassword => {
                    LogLevel::Info(InfoLevel::Auth(AuthInfoLevel::ForgotPasswordRoute))
                }
                AuthEvent::ForgotPasswordSuccess => {
                    LogLevel::Info(InfoLevel::Auth(AuthInfoLevel::ForgotPasswordSuccessRoute))
                }
                AuthEvent::SetPassword => {
                    LogLevel::Info(InfoLevel::Auth(AuthInfoLevel::SetPasswordRoute))
                }
                AuthEvent::SetPasswordSuccess => {
                    LogLevel::Info(InfoLevel::Auth(AuthInfoLevel::SetPasswordSuccessRoute))
                }
                AuthEvent::InvalidRoute => {
                    LogLevel::Error(ErrorLevel::Auth(AuthErrorLevel::InvalidRoute))
                }
            },
        }
    }

    fn message(&self) -> String {
        match self {
            LogLevel::Info(i) => i.message(),
            LogLevel::Error(e) => e.message(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum AuthInfoLevel {
    Initial,
    LoginRoute,
    GithubLoginRoute,
    GithubCallbackRoute,
    LogoutRoute,
    CreateAccountRoute,
    EmailConfirmationSentRoute,
    ConfirmEmailRoute,
    ResendConfirmationEmailRoute,
    OnboardingRoute,
    ForgotPasswordRoute,
    ForgotPasswordSuccessRoute,
    SetPasswordRoute,
    SetPasswordSuccessRoute,
}

impl AuthInfoLevel {
    fn message(&self) -> String {
        match self {
            AuthInfoLevel::Initial => "[INFO]: Attempting Auth",
            AuthInfoLevel::LoginRoute => "[INFO]: Login Route",
            AuthInfoLevel::GithubLoginRoute => "[INFO]: Github Login Route",
            AuthInfoLevel::GithubCallbackRoute => "[INFO]: Github CallBack Route",
            AuthInfoLevel::LogoutRoute => "[INFO]: Logout Route",
            AuthInfoLevel::CreateAccountRoute => "[INFO]: Create Account Route",
            AuthInfoLevel::EmailConfirmationSentRoute => "[INFO]: Email Confirmation Route",
            AuthInfoLevel::ConfirmEmailRoute => "[INFO]: Confirm Email Route",
            AuthInfoLevel::ResendConfirmationEmailRoute => {
                "[INFO]: Resend Confirmation Email Route"
            }
            AuthInfoLevel::OnboardingRoute => "[INFO]: Onboarding Route",
            AuthInfoLevel::ForgotPasswordRoute => "[INFO]: Forgot Password Route",
            AuthInfoLevel::ForgotPasswordSuccessRoute => "[INFO]: Forgot Password Success Route",
            AuthInfoLevel::SetPasswordRoute => "[INFO]: Set Password Route",
            AuthInfoLevel::SetPasswordSuccessRoute => "[INFO]: Set Password Success Route",
        }
        .to_string()
    }
}

#[derive(Debug, Clone)]
pub enum AuthErrorLevel {
    InvalidRoute,
}

impl AuthErrorLevel {
    fn message(&self) -> String {
        match self {
            AuthErrorLevel::InvalidRoute => "[ERROR]: Invalid Auth Route",
        }
        .to_string()
    }
}

#[derive(Debug, Clone)]
pub enum InfoLevel {
    Auth(AuthInfoLevel),
}

impl InfoLevel {
    fn message(&self) -> String {
        match self {
            InfoLevel::Auth(i) => i.message(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ErrorLevel {
    Auth(AuthErrorLevel),
}

impl ErrorLevel {
    fn message(&self) -> String {
        match self {
            ErrorLevel::Auth(e) => e.message(),
        }
    }
}

// todo: more relevant fields will be added in future
#[derive(Debug, Clone)]
pub struct SiteLog {
    pub site_id: Option<i64>,
    pub org_id: Option<i64>,
    pub someone: Option<i64>,
    pub myself: Option<i64>,
}

// todo: more relevant fields will be added in future
#[derive(Debug, Clone)]
pub struct RequestLog {
    pub host: String,
    pub scheme: String,
    pub method: String,
    pub path: String,
    pub query: String,
    pub ip: Option<String>,
    pub body: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Log {
    pub level: fastn_core::log::LogLevel,
    pub ekind: fastn_core::log::EventKind,
    pub okind: fastn_core::log::EntityKind,
    pub message: String,
    pub site: Option<fastn_core::log::SiteLog>,
    pub request: fastn_core::log::RequestLog,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub line_number: u32,
}

impl fastn_core::http::Request {
    pub fn log(
        &self,
        site: Option<fastn_core::log::SiteLog>,
        ekind: fastn_core::log::EventKind,
        okind: fastn_core::log::EntityKind,
        line_number: u32,
    ) {
        let log_level = LogLevel::from(&ekind, &okind);
        let mut log = self.log.write().unwrap();
        (*log).push(Log {
            ekind,
            okind,
            request: self.to_request_log(),
            message: log_level.message(),
            level: log_level,
            site,
            timestamp: chrono::Utc::now(),
            line_number,
        });
    }

    pub fn log_with_no_message(
        &self,
        site: Option<fastn_core::log::SiteLog>,
        ekind: fastn_core::log::EventKind,
        okind: fastn_core::log::EntityKind,
        line_number: u32,
    ) {
        let log_level = LogLevel::from(&ekind, &okind);
        let mut log = self.log.write().unwrap();
        (*log).push(Log {
            ekind,
            okind,
            request: self.to_request_log(),
            message: log_level.message(),
            level: log_level,
            site,
            timestamp: chrono::Utc::now(),
            line_number,
        });
    }

    pub fn log_with_no_site(
        &self,
        ekind: fastn_core::log::EventKind,
        okind: fastn_core::log::EntityKind,
        line_number: u32,
    ) {
        let log_level = LogLevel::from(&ekind, &okind);
        let mut log = self.log.write().unwrap();
        (*log).push(Log {
            ekind,
            okind,
            request: self.to_request_log(),
            message: log_level.message(),
            level: log_level,
            site: None,
            timestamp: chrono::Utc::now(),
            line_number,
        });
    }
}
