#[derive(Debug, Clone)]
pub enum LogLevel {
    Info(InfoLevel),
    Error(ErrorLevel),
    // todo: implement this
    // Warning(WarningLevel),
}

impl LogLevel {
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

// todo: implement this
#[derive(Debug, Clone)]
pub enum InfoLevel {
    Auth(AuthInfoLevel),
    // Worker(WorkerLevel),
}

impl InfoLevel {
    fn message(&self) -> String {
        match self {
            InfoLevel::Auth(i) => i.message(),
        }
    }
}

// todo: implement this
#[derive(Debug, Clone)]
pub enum ErrorLevel {
    Auth(AuthErrorLevel),
    // Worker(WorkerErrorLevel),
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
    pub ekind: Option<String>,
    pub okind: Option<String>,
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
        log_level: fastn_core::log::LogLevel,
        line_number: u32,
    ) {
        let mut log = self.log.write().unwrap();
        (*log).push(Log {
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
        log_level: fastn_core::log::LogLevel,
        line_number: u32,
    ) {
        let mut log = self.log.write().unwrap();
        (*log).push(Log {
            request: self.to_request_log(),
            message: log_level.message(),
            level: log_level,
            site,
            timestamp: chrono::Utc::now(),
            line_number,
        });
    }

    pub fn log_with_no_site(&self, log_level: fastn_core::log::LogLevel, line_number: u32) {
        let mut log = self.log.write().unwrap();
        (*log).push(Log {
            request: self.to_request_log(),
            message: log_level.message(),
            level: log_level,
            site: None,
            timestamp: chrono::Utc::now(),
            line_number,
        });
    }
}
