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
                AuthEvent::Login => "login",
                AuthEvent::Logout => "logout",
                AuthEvent::GithubLogin => "github-login",
                AuthEvent::GithubCallback => "github-callback",
                AuthEvent::CreateAccount => "create-account",
                AuthEvent::EmailConfirmationSent => "email-confirmation-sent",
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
    Login,
    Logout,
    GithubLogin,
    GithubCallback,
    CreateAccount,
    EmailConfirmationSent,
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
            "email-confirmation-sent" => AuthEvent::EmailConfirmationSent,
            "confirm-email" => AuthEvent::ConfirmEmail,
            "resend-confirmation-email" => AuthEvent::ResendConfirmationEmail,
            "onboarding" => AuthEvent::Onboarding,
            "forgot-password" => AuthEvent::ForgotPassword,
            "forgot-password-success" => AuthEvent::ForgotPasswordSuccess,
            "set-password" => AuthEvent::SetPassword,
            "set-password-success" => AuthEvent::SetPasswordSuccess,
            _ => AuthEvent::InvalidRoute,
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
    Success(SuccessOutcome),
    Error(ErrorOutcome),
}

impl OutcomeKind {
    pub fn success_default() -> Self {
        OutcomeKind::Success(SuccessOutcome::Default)
    }

    pub fn success_descriptive(message: String) -> Self {
        OutcomeKind::Success(SuccessOutcome::Descriptive(message))
    }

    pub fn error_default() -> Self {
        OutcomeKind::Error(ErrorOutcome::Default)
    }
}

#[derive(Debug, Clone)]
pub enum SuccessOutcome {
    Default,
    Descriptive(String),
}

impl SuccessOutcome {
    fn message(&self) -> String {
        match self {
            SuccessOutcome::Default => "Processed".to_string(),
            SuccessOutcome::Descriptive(s) => s.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ErrorOutcome {
    Default,
    UnauthorizedError(UnauthorizedErrorOutcome),
    ServerError(ServerErrorOutcome),
    FormError(FormErrorOutcome),
    BadRequest(BadRequestOutcome),
}

impl ErrorOutcome {
    pub fn message(&self) -> String {
        match self {
            ErrorOutcome::Default => "default".to_string(),
            ErrorOutcome::UnauthorizedError(outcome) => outcome.message(),
            ErrorOutcome::ServerError(outcome) => outcome.message(),
            ErrorOutcome::FormError(outcome) => outcome.message(),
            ErrorOutcome::BadRequest(outcome) => outcome.message(),
        }
    }

    pub fn outcome(&self) -> String {
        match self {
            ErrorOutcome::Default => "error".to_string(),
            ErrorOutcome::UnauthorizedError(outcome) => outcome.outcome(),
            ErrorOutcome::ServerError(outcome) => outcome.outcome(),
            ErrorOutcome::FormError(outcome) => outcome.outcome(),
            ErrorOutcome::BadRequest(outcome) => outcome.outcome(),
        }
    }

    pub fn outcome_detail(&self) -> String {
        match self {
            ErrorOutcome::Default => "error".to_string(),
            ErrorOutcome::UnauthorizedError(outcome) => outcome.outcome_detail(),
            ErrorOutcome::ServerError(outcome) => outcome.outcome_detail(),
            ErrorOutcome::FormError(outcome) => outcome.outcome_detail(),
            ErrorOutcome::BadRequest(outcome) => outcome.outcome_detail(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum UnauthorizedErrorOutcome {
    Default,
    UserDoesNotExist,
    UserNotVerified,
}

impl UnauthorizedErrorOutcome {
    pub fn message(&self) -> String {
        match self {
            UnauthorizedErrorOutcome::Default => "default".to_string(),
            UnauthorizedErrorOutcome::UserDoesNotExist => "user: does not exist".to_string(),
            UnauthorizedErrorOutcome::UserNotVerified => "user: not verified".to_string(),
        }
    }

    pub fn outcome(&self) -> String {
        "unauthorized-error".to_string()
    }

    pub fn outcome_detail(&self) -> String {
        match self {
            UnauthorizedErrorOutcome::Default => "unauthorized-error",
            UnauthorizedErrorOutcome::UserDoesNotExist => "user-does-not-exist",
            UnauthorizedErrorOutcome::UserNotVerified => "user-not-verified",
        }
        .to_string()
    }

    pub fn into_kind(self) -> OutcomeKind {
        OutcomeKind::Error(ErrorOutcome::UnauthorizedError(self))
    }
}

#[derive(Debug, Clone)]
pub enum ServerErrorOutcome {
    Default,
    DatabaseQueryError { message: String },
    PoolError { message: String },
    CookieError { message: String },
    ReadFTDError { message: String },
    InterpreterError { message: String },
    HashingError { message: String },
    MailError { message: String },
    EnvironmentError { message: String },
    RequestTokenError { message: String },
    HttpError { message: String },
}

impl ServerErrorOutcome {
    pub fn message(&self) -> String {
        match self {
            ServerErrorOutcome::Default => "default".to_string(),
            ServerErrorOutcome::DatabaseQueryError { message } => {
                format!("database query error: {}", message)
            }
            ServerErrorOutcome::PoolError { message } => format!("pool error: {}", message),
            ServerErrorOutcome::CookieError { message } => {
                format!("session cookie error: {}", message)
            }
            ServerErrorOutcome::ReadFTDError { message } => format!("read_ftd error: {}", message),
            ServerErrorOutcome::HashingError { message } => format!("hashing error: {}", message),
            ServerErrorOutcome::InterpreterError { message } => {
                format!("interpreter error: {}", message)
            }
            ServerErrorOutcome::MailError { message } => format!("mail error: {}", message),
            ServerErrorOutcome::EnvironmentError { message } => format!("env error: {}", message),
            ServerErrorOutcome::RequestTokenError { message } => {
                format!("request token error: {}", message)
            }
            ServerErrorOutcome::HttpError { message } => format!("http error: {}", message),
        }
    }

    pub fn outcome(&self) -> String {
        "server-error".to_string()
    }

    pub fn outcome_detail(&self) -> String {
        match self {
            ServerErrorOutcome::Default => "server-error",
            ServerErrorOutcome::DatabaseQueryError { .. } => "database-query-error",
            ServerErrorOutcome::PoolError { .. } => "pool-error",
            ServerErrorOutcome::CookieError { .. } => "cookie-error",
            ServerErrorOutcome::ReadFTDError { .. } => "read-ftd-error",
            ServerErrorOutcome::InterpreterError { .. } => "interpreter-error",
            ServerErrorOutcome::HashingError { .. } => "hashing-error",
            ServerErrorOutcome::MailError { .. } => "mail-error",
            ServerErrorOutcome::EnvironmentError { .. } => "environment-error",
            ServerErrorOutcome::RequestTokenError { .. } => "request-token-error",
            ServerErrorOutcome::HttpError { .. } => "http-error",
        }
        .to_string()
    }

    pub fn into_kind(self) -> OutcomeKind {
        OutcomeKind::Error(ErrorOutcome::ServerError(self))
    }
}

#[derive(Debug, Clone)]
pub enum FormErrorOutcome {
    Default,
    PayloadError { message: String },
    ValidationError { message: String },
}

impl FormErrorOutcome {
    pub fn message(&self) -> String {
        match self {
            FormErrorOutcome::Default => "default".to_string(),
            FormErrorOutcome::PayloadError { message } => format!("payload error: {}", message),
            FormErrorOutcome::ValidationError { message } => {
                format!("validation error: {}", message)
            }
        }
    }

    pub fn outcome(&self) -> String {
        "form-error".to_string()
    }

    pub fn outcome_detail(&self) -> String {
        match self {
            FormErrorOutcome::Default => "form-error",
            FormErrorOutcome::PayloadError { .. } => "payload-error",
            FormErrorOutcome::ValidationError { .. } => "validation-error",
        }
        .to_string()
    }

    pub fn into_kind(self) -> OutcomeKind {
        OutcomeKind::Error(ErrorOutcome::FormError(self))
    }
}

#[derive(Debug, Clone)]
pub enum BadRequestOutcome {
    Default,
    InvalidCode { code: String },
    QueryNotFoundError { query: String },
    QueryDeserializeError { query: String },
    NotFound { message: String },
    InvalidRoute { message: String },
}

impl BadRequestOutcome {
    pub fn message(&self) -> String {
        match self {
            BadRequestOutcome::Default => "default".to_string(),
            BadRequestOutcome::QueryNotFoundError { query } => {
                format!("query error: {} not found", query)
            }
            BadRequestOutcome::QueryDeserializeError { query } => {
                format!("deserialize error: failed to deserialize query {}", query)
            }
            BadRequestOutcome::InvalidCode { code } => format!(
                "invalid code value. No entry exists for the given code in db. Provided code: {}",
                code
            ),
            BadRequestOutcome::NotFound { message } => format!("not found: {}", message),
            BadRequestOutcome::InvalidRoute { message } => format!("invalid route: {}", message),
        }
    }

    pub fn outcome(&self) -> String {
        "bad-request".to_string()
    }

    pub fn outcome_detail(&self) -> String {
        match self {
            BadRequestOutcome::Default => "bad-request",
            BadRequestOutcome::InvalidCode { .. } => "invalid-code",
            BadRequestOutcome::QueryNotFoundError { .. } => "query-not-found",
            BadRequestOutcome::QueryDeserializeError { .. } => "query-deserialize-error",
            BadRequestOutcome::NotFound { .. } => "not-found",
            BadRequestOutcome::InvalidRoute { .. } => "invalid-route",
        }
        .to_string()
    }

    pub fn into_kind(self) -> OutcomeKind {
        OutcomeKind::Error(ErrorOutcome::BadRequest(self))
    }
}

#[derive(Debug, Clone)]
pub enum LogLevel {
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
            (EventKind::Auth(event), EntityKind::Myself, OutcomeKind::Error(error)) => {
                match event {
                    AuthEvent::Login => {
                        LogLevel::Error(ErrorLevel::Auth(AuthErrorLevel::Login(error.to_owned())))
                    }
                    AuthEvent::Logout => {
                        LogLevel::Error(ErrorLevel::Auth(AuthErrorLevel::Logout(error.to_owned())))
                    }
                    AuthEvent::GithubLogin => LogLevel::Error(ErrorLevel::Auth(
                        AuthErrorLevel::GithubLogin(error.to_owned()),
                    )),
                    AuthEvent::GithubCallback => LogLevel::Error(ErrorLevel::Auth(
                        AuthErrorLevel::GithubCallback(error.to_owned()),
                    )),
                    AuthEvent::CreateAccount => LogLevel::Error(ErrorLevel::Auth(
                        AuthErrorLevel::CreateAccount(error.to_owned()),
                    )),
                    AuthEvent::EmailConfirmationSent => LogLevel::Error(ErrorLevel::Auth(
                        AuthErrorLevel::EmailConfirmationSent(error.to_owned()),
                    )),
                    AuthEvent::ConfirmEmail => LogLevel::Error(ErrorLevel::Auth(
                        AuthErrorLevel::ConfirmEmail(error.to_owned()),
                    )),
                    AuthEvent::ResendConfirmationEmail => LogLevel::Error(ErrorLevel::Auth(
                        AuthErrorLevel::ResendConfirmationEmail(error.to_owned()),
                    )),
                    AuthEvent::Onboarding => LogLevel::Error(ErrorLevel::Auth(
                        AuthErrorLevel::Onboarding(error.to_owned()),
                    )),
                    AuthEvent::ForgotPassword => LogLevel::Error(ErrorLevel::Auth(
                        AuthErrorLevel::ForgotPassword(error.to_owned()),
                    )),
                    AuthEvent::ForgotPasswordSuccess => LogLevel::Error(ErrorLevel::Auth(
                        AuthErrorLevel::ForgotPasswordSuccess(error.to_owned()),
                    )),
                    AuthEvent::SetPassword => LogLevel::Error(ErrorLevel::Auth(
                        AuthErrorLevel::SetPassword(error.to_owned()),
                    )),
                    AuthEvent::SetPasswordSuccess => LogLevel::Error(ErrorLevel::Auth(
                        AuthErrorLevel::SetPasswordSuccess(error.to_owned()),
                    )),
                    AuthEvent::InvalidRoute => {
                        LogLevel::Error(ErrorLevel::Auth(AuthErrorLevel::InvalidRoute))
                    }
                }
            }
            (EventKind::Auth(event), EntityKind::Myself, OutcomeKind::Success(outcome)) => {
                match event {
                    AuthEvent::Login => LogLevel::Success(SuccessLevel::Auth(
                        AuthSuccessLevel::Login(outcome.to_owned()),
                    )),
                    AuthEvent::Logout => LogLevel::Success(SuccessLevel::Auth(
                        AuthSuccessLevel::Logout(outcome.to_owned()),
                    )),
                    AuthEvent::GithubLogin => LogLevel::Success(SuccessLevel::Auth(
                        AuthSuccessLevel::GithubLogin(outcome.to_owned()),
                    )),
                    AuthEvent::GithubCallback => LogLevel::Success(SuccessLevel::Auth(
                        AuthSuccessLevel::GithubCallback(outcome.to_owned()),
                    )),
                    AuthEvent::CreateAccount => LogLevel::Success(SuccessLevel::Auth(
                        AuthSuccessLevel::CreateAccount(outcome.to_owned()),
                    )),
                    AuthEvent::EmailConfirmationSent => LogLevel::Success(SuccessLevel::Auth(
                        AuthSuccessLevel::EmailConfirmationSent(outcome.to_owned()),
                    )),
                    AuthEvent::ConfirmEmail => LogLevel::Success(SuccessLevel::Auth(
                        AuthSuccessLevel::ConfirmEmail(outcome.to_owned()),
                    )),
                    AuthEvent::ResendConfirmationEmail => LogLevel::Success(SuccessLevel::Auth(
                        AuthSuccessLevel::ResendConfirmationEmail(outcome.to_owned()),
                    )),
                    AuthEvent::Onboarding => LogLevel::Success(SuccessLevel::Auth(
                        AuthSuccessLevel::Onboarding(outcome.to_owned()),
                    )),
                    AuthEvent::ForgotPassword => LogLevel::Success(SuccessLevel::Auth(
                        AuthSuccessLevel::ForgotPassword(outcome.to_owned()),
                    )),
                    AuthEvent::ForgotPasswordSuccess => LogLevel::Success(SuccessLevel::Auth(
                        AuthSuccessLevel::ForgotPasswordSuccess(outcome.to_owned()),
                    )),
                    AuthEvent::SetPassword => LogLevel::Success(SuccessLevel::Auth(
                        AuthSuccessLevel::SetPassword(outcome.to_owned()),
                    )),
                    AuthEvent::SetPasswordSuccess => LogLevel::Success(SuccessLevel::Auth(
                        AuthSuccessLevel::SetPasswordSuccess(outcome.to_owned()),
                    )),
                    _ => LogLevel::Success(SuccessLevel::Undefined),
                }
            }
        }
    }

    fn message(&self) -> String {
        match self {
            LogLevel::Error(e) => e.message(),
            LogLevel::Success(s) => s.message(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum AuthErrorLevel {
    Login(ErrorOutcome),
    GithubLogin(ErrorOutcome),
    GithubCallback(ErrorOutcome),
    Logout(ErrorOutcome),
    CreateAccount(ErrorOutcome),
    EmailConfirmationSent(ErrorOutcome),
    ConfirmEmail(ErrorOutcome),
    ResendConfirmationEmail(ErrorOutcome),
    Onboarding(ErrorOutcome),
    ForgotPassword(ErrorOutcome),
    ForgotPasswordSuccess(ErrorOutcome),
    SetPassword(ErrorOutcome),
    SetPasswordSuccess(ErrorOutcome),
    InvalidRoute,
}

impl AuthErrorLevel {
    fn message(&self) -> String {
        match self {
            AuthErrorLevel::Login(error) => format!("[ERROR]: Login: {}", error.message()),
            AuthErrorLevel::InvalidRoute => "[ERROR]: Invalid Auth Route".to_string(),
            AuthErrorLevel::GithubLogin(error) => {
                format!("[ERROR]: Github Login: {}", error.message())
            }
            AuthErrorLevel::GithubCallback(error) => {
                format!("[ERROR]: Github Callback: {}", error.message())
            }
            AuthErrorLevel::Logout(error) => format!("[ERROR]: Logout: {}", error.message()),
            AuthErrorLevel::CreateAccount(error) => {
                format!("[ERROR]: Create Account: {}", error.message())
            }
            AuthErrorLevel::EmailConfirmationSent(error) => {
                format!("[ERROR]: Email Confirmation Sent: {}", error.message())
            }
            AuthErrorLevel::ConfirmEmail(error) => {
                format!("[ERROR]: Confirm Email: {}", error.message())
            }
            AuthErrorLevel::ResendConfirmationEmail(error) => {
                format!("[ERROR]: Resend Confirmation Email: {}", error.message())
            }
            AuthErrorLevel::Onboarding(error) => {
                format!("[ERROR]: Onboarding: {}", error.message())
            }
            AuthErrorLevel::ForgotPassword(error) => {
                format!("[ERROR]: Forgot Password: {}", error.message())
            }
            AuthErrorLevel::ForgotPasswordSuccess(error) => {
                format!("[ERROR]: Forgot Password Success: {}", error.message())
            }
            AuthErrorLevel::SetPassword(error) => {
                format!("[ERROR]: Set Password: {}", error.message())
            }
            AuthErrorLevel::SetPasswordSuccess(error) => {
                format!("[ERROR]: Set Password Success: {}", error.message())
            }
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
    Login(SuccessOutcome),
    GithubLogin(SuccessOutcome),
    GithubCallback(SuccessOutcome),
    Logout(SuccessOutcome),
    CreateAccount(SuccessOutcome),
    EmailConfirmationSent(SuccessOutcome),
    ConfirmEmail(SuccessOutcome),
    ResendConfirmationEmail(SuccessOutcome),
    Onboarding(SuccessOutcome),
    ForgotPassword(SuccessOutcome),
    ForgotPasswordSuccess(SuccessOutcome),
    SetPassword(SuccessOutcome),
    SetPasswordSuccess(SuccessOutcome),
}

impl AuthSuccessLevel {
    fn message(&self) -> String {
        match self {
            AuthSuccessLevel::Login(outcome) => format!("[SUCCESS]: Login: {}", outcome.message()),
            AuthSuccessLevel::GithubLogin(outcome) => {
                format!("[SUCCESS]: Github Login: {}", outcome.message())
            }
            AuthSuccessLevel::GithubCallback(outcome) => {
                format!("[SUCCESS]: Github Callback: {}", outcome.message())
            }
            AuthSuccessLevel::Logout(outcome) => {
                format!("[SUCCESS]: Logout: {}", outcome.message())
            }
            AuthSuccessLevel::CreateAccount(outcome) => {
                format!("[SUCCESS]: Create Account: {}", outcome.message())
            }
            AuthSuccessLevel::EmailConfirmationSent(outcome) => {
                format!("[SUCCESS]: Email Confirmation Sent: {}", outcome.message())
            }
            AuthSuccessLevel::ConfirmEmail(outcome) => {
                format!("[SUCCESS]: Confirm Email: {}", outcome.message())
            }
            AuthSuccessLevel::ResendConfirmationEmail(outcome) => {
                format!(
                    "[SUCCESS]: Resend Confirmation Email: {}",
                    outcome.message()
                )
            }
            AuthSuccessLevel::Onboarding(outcome) => {
                format!("[SUCCESS]: Onboarding: {}", outcome.message())
            }
            AuthSuccessLevel::ForgotPassword(outcome) => {
                format!("[SUCCESS]: Forgot Password: {}", outcome.message())
            }
            AuthSuccessLevel::ForgotPasswordSuccess(outcome) => {
                format!("[SUCCESS]: Forgot Password Success: {}", outcome.message())
            }
            AuthSuccessLevel::SetPassword(outcome) => {
                format!("[SUCCESS]: Set Password: {}", outcome.message())
            }
            AuthSuccessLevel::SetPasswordSuccess(outcome) => {
                format!("[SUCCESS]: Set Password Success: {}", outcome.message())
            }
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

    pub fn body_as_json(&self) -> serde_json::Result<serde_json::Value> {
        serde_json::from_slice(&self.body)
    }

    pub fn filtered_json_body(&self) -> serde_json::Result<serde_json::Value> {
        let json_body = self.body_as_json().unwrap_or_default();
        return match &json_body {
            serde_json::Value::Object(map) => {
                let filtered_map: serde_json::Map<String, serde_json::Value> = map
                    .iter()
                    .filter_map(|(k, v)| {
                        if !k.contains("password") {
                            Some((k.to_string(), v.clone()))
                        } else {
                            None
                        }
                    })
                    .collect();
                Ok(serde_json::Value::from(filtered_map))
            }
            _ => Ok(json_body),
        };
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

        let filtered_body_json = self.filtered_json_body().unwrap_or_default();
        serde_json::json!({
            "body": filtered_body_json,
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
            OutcomeKind::Error(outcome) => outcome.outcome(),
            OutcomeKind::Success(_outcome) => "success".to_string(),
        }
    }

    pub fn outcome_detail(&self) -> String {
        match &self.outcome {
            OutcomeKind::Error(outcome) => outcome.outcome_detail(),
            OutcomeKind::Success(_outcome) => "success".to_string(),
        }
    }

    pub fn outcome_data(&self) -> serde_json::Value {
        let outcome_detail = self.outcome_detail();
        let outcome_message = self.message();
        serde_json::json!({
            "detail": outcome_detail,
            "message": outcome_message
        })
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
