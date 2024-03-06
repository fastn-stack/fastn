#[derive(Debug, Clone)]
pub enum EventKind {
    Auth(AuthEvent),
}

impl EventKind {
    pub fn from_auth_event_str(event: &str) -> Self {
        EventKind::Auth(AuthEvent::from_str(event))
    }

    pub fn to_string(&self) -> String {
        match self {
            EventKind::Auth(event) => match event {
                AuthEvent::Initial => "initial",
                AuthEvent::Login => "login",
                AuthEvent::Logout => "logout",
                AuthEvent::GithubLogin => "github-login",
                AuthEvent::GithubCallback => "github-callback",
                AuthEvent::CreateAccount => "create-account",
                AuthEvent::EmailConfirmation => "email-confirmation",
                AuthEvent::ConfirmEmail => "confirm-email",
                AuthEvent::ResendConfirmationEmail => "resend-confirmation-email",
                AuthEvent::Onboarding => "onboarding",
                AuthEvent::ForgotPassword => "forgot-password",
                AuthEvent::ForgotPasswordSuccess => "forgot-password-success",
                AuthEvent::SetPassword => "set-password",
                AuthEvent::SetPasswordSuccess => "set-password-success",
                AuthEvent::InvalidRoute => "invalid-route",
            }
            .to_string(),
        }
    }
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

impl AuthEvent {
    pub fn from_str(event: &str) -> Self {
        match event {
            "login" => AuthEvent::Login,
            "logout" => AuthEvent::Logout,
            "github-login" => AuthEvent::GithubLogin,
            "github-callback" => AuthEvent::GithubCallback,
            "create-account" => AuthEvent::CreateAccount,
            "email-confirmation" => AuthEvent::EmailConfirmation,
            "confirm-email" => AuthEvent::ConfirmEmail,
            "resend-confirmation-email" => AuthEvent::ResendConfirmationEmail,
            "onboarding" => AuthEvent::Onboarding,
            "forgot-password" => AuthEvent::ForgotPassword,
            "forgot-password-success" => AuthEvent::ForgotPasswordSuccess,
            "set-password" => AuthEvent::SetPassword,
            "set-password-success" => AuthEvent::SetPasswordSuccess,
            "initial" => AuthEvent::Initial,
            "invalid-route" | _ => AuthEvent::InvalidRoute,
        }
    }
}

#[derive(Debug, Clone)]
pub enum EntityKind {
    Myself,
}

impl EntityKind {
    pub fn to_string(&self) -> String {
        match self {
            EntityKind::Myself => "myself",
        }
        .to_string()
    }
}

// todo: convert descriptive outcomes as enums
#[derive(Debug, Clone)]
pub enum OutcomeKind {
    Info,
    Success(Outcome),
    Error(Outcome),
}

impl OutcomeKind {
    pub fn success_default() -> Self {
        OutcomeKind::Success(Outcome::Default)
    }

    pub fn success_descriptive(message: String) -> Self {
        OutcomeKind::Success(Outcome::Descriptive(message))
    }

    pub fn error_default() -> Self {
        OutcomeKind::Error(Outcome::Default)
    }

    pub fn error_descriptive(message: String) -> Self {
        OutcomeKind::Error(Outcome::Descriptive(message))
    }
}

// todo: implement this as enum for different auth operations
#[derive(Debug, Clone)]
pub enum Outcome {
    Default,
    Descriptive(String),
}

impl Outcome {
    fn message(&self) -> String {
        match self {
            Outcome::Default => "Default".to_string(),
            Outcome::Descriptive(s) => s.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum LogLevel {
    Info(InfoLevel),
    Error(ErrorLevel),
    Success(SuccessLevel),
}

impl LogLevel {
    pub fn from(
        ekind: &fastn_core::log::EventKind,
        okind: &fastn_core::log::EntityKind,
        outcome: &fastn_core::log::OutcomeKind,
    ) -> Self {
        match (ekind, okind, outcome) {
            (EventKind::Auth(event), EntityKind::Myself, OutcomeKind::Info) => match event {
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
                    LogLevel::Info(InfoLevel::Auth(AuthInfoLevel::InvalidRoute))
                }
            },
            (EventKind::Auth(event), EntityKind::Myself, OutcomeKind::Error(error)) => {
                match event {
                    AuthEvent::InvalidRoute => {
                        LogLevel::Error(ErrorLevel::Auth(AuthErrorLevel::InvalidRoute))
                    }
                    AuthEvent::Login => {
                        LogLevel::Error(ErrorLevel::Auth(AuthErrorLevel::Login(error.to_owned())))
                    }
                    _ => LogLevel::Error(ErrorLevel::Auth(AuthErrorLevel::Undefined)),
                }
            }
            (EventKind::Auth(event), EntityKind::Myself, OutcomeKind::Success(outcome)) => {
                match event {
                    AuthEvent::Login => LogLevel::Success(SuccessLevel::Auth(
                        AuthSuccessLevel::Login(outcome.to_owned()),
                    )),
                    _ => LogLevel::Success(SuccessLevel::Undefined),
                }
            }
        }
    }

    fn message(&self) -> String {
        match self {
            LogLevel::Info(i) => i.message(),
            LogLevel::Error(e) => e.message(),
            LogLevel::Success(s) => s.message(),
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
    InvalidRoute,
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
            AuthInfoLevel::InvalidRoute => "[INFO]: Accessing Invalid Route",
        }
        .to_string()
    }
}

#[derive(Debug, Clone)]
pub enum AuthErrorLevel {
    Login(Outcome),
    InvalidRoute,
    Undefined,
}

impl AuthErrorLevel {
    fn message(&self) -> String {
        match self {
            AuthErrorLevel::Login(error) => format!("[ERROR]: Login: {}", error.message()),
            AuthErrorLevel::InvalidRoute => "[ERROR]: Invalid Auth Route".to_string(),
            AuthErrorLevel::Undefined => "[ERROR]: Undefined Auth Route".to_string(),
        }
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

// todo: remove undefined later
#[derive(Debug, Clone)]
pub enum SuccessLevel {
    Auth(AuthSuccessLevel),
    Undefined,
}

impl SuccessLevel {
    fn message(&self) -> String {
        match self {
            SuccessLevel::Auth(e) => e.message(),
            SuccessLevel::Undefined => "[SUCCESS]: Undefined".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum AuthSuccessLevel {
    Login(Outcome),
}

impl AuthSuccessLevel {
    fn message(&self) -> String {
        match self {
            AuthSuccessLevel::Login(outcome) => format!("[SUCCESS]: Login: {}", outcome.message()),
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
    pub user_agent: String,
    pub scheme: String,
    pub method: String,
    pub path: String,
    pub query_string: String,
    pub ip: Option<String>,
    pub body: Vec<u8>,
    pub cookies: std::collections::HashMap<String, String>,
    pub headers: reqwest::header::HeaderMap,
    pub query: std::collections::HashMap<String, serde_json::Value>,
}

impl RequestLog {
    pub fn user_agent(&self) -> String {
        self.user_agent.clone()
    }

    pub fn body_as_json<T: serde::de::DeserializeOwned>(&self) -> serde_json::Result<T> {
        serde_json::from_slice(&self.body)
    }

    pub fn event_data(&self) -> serde_json::Value {
        // Todo: Store all headers
        let headers: std::collections::HashMap<String, String> = self
            .headers
            .iter()
            .filter_map(|(k, v)| {
                if let Ok(v) = v.to_str() {
                    Some((k.to_string(), v.to_string()))
                } else {
                    None
                }
            })
            .collect();

        serde_json::json!({
            "json": self.body_as_json::<serde_json::Value>().unwrap_or_default(),
            "cookies": &self.cookies,
            "query": &self.query,
            "headers": headers
        })
    }
}

#[derive(Debug, Clone)]
pub struct Log {
    pub level: fastn_core::log::LogLevel,
    pub ekind: fastn_core::log::EventKind,
    pub okind: fastn_core::log::EntityKind,
    pub outcome: fastn_core::log::OutcomeKind,
    pub message: String,
    pub request: fastn_core::log::RequestLog,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub doc: String,
    pub line_number: u32,
}

impl Log {
    pub fn outcome(&self) -> String {
        match &self.outcome {
            // todo: improve error outcome
            OutcomeKind::Error(_outcome) => "error",
            OutcomeKind::Success(_outcome) => "success",
            OutcomeKind::Info => "info",
        }
        .to_string()
    }

    pub fn outcome_data(&self) -> String {
        format!("{:?}", self)
    }

    pub fn event_data(&self) -> serde_json::Value {
        self.request.event_data()
    }

    pub fn source(&self) -> String {
        format!("{}, Line: {}", self.doc.as_str(), self.line_number)
    }

    pub fn message(&self) -> String {
        self.message.clone()
    }
}

impl fastn_core::http::Request {
    pub fn log(
        &self,
        ekind: &str,
        outcome: fastn_core::log::OutcomeKind,
        doc_name: &str,
        line_number: u32,
    ) {
        // Auth specific ----------------------------------
        let ekind = fastn_core::log::EventKind::Auth(AuthEvent::from_str(ekind));
        let okind = fastn_core::log::EntityKind::Myself;
        // ------------------------------------------------
        let log_level = LogLevel::from(&ekind, &okind, &outcome);
        let mut log = self.log.write().unwrap();
        (*log).push(Log {
            ekind,
            okind,
            outcome,
            request: self.to_request_log(),
            message: log_level.message(),
            level: log_level,
            timestamp: chrono::Utc::now(),
            doc: doc_name.to_string(),
            line_number,
        });
    }
}
