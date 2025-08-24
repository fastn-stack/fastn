use thiserror::Error;

/// Error type for hash_password function
#[derive(Error, Debug)]
pub enum HashPasswordError {
    #[error("Failed to hash password: {message}")]
    HashingFailed { message: String },
}

/// Error type for verify_password function
#[derive(Error, Debug)]
pub enum VerifyPasswordError {
    #[error("Failed to parse password hash: {message}")]
    HashParsingFailed { message: String },
}

/// Error type for Account::create function
#[derive(Error, Debug)]
pub enum AccountCreateError {
    #[error("Account already exists at path: {path}")]
    AccountAlreadyExists { path: std::path::PathBuf },

    #[error("Failed to create directory: {path}")]
    DirectoryCreationFailed {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to write file: {path}")]
    FileWriteFailed {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to store key in keyring for ID52: {id52}")]
    KeyringStorageFailed { id52: String },

    #[error("Failed to connect to database: {path}")]
    DatabaseConnectionFailed { path: std::path::PathBuf },

    #[error("Failed to initialize automerge database")]
    AutomergeInitFailed {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Mail creation failed")]
    MailCreationFailed {
        #[source]
        source: fastn_mail::MailCreateError,
    },

    #[error("User database migration failed")]
    UserMigrationFailed {
        #[source]
        source: MigrateUserDatabaseError,
    },

    #[error("Failed to create initial automerge documents")]
    InitialDocumentsCreationFailed {
        #[source]
        source: CreateInitialDocumentsError,
    },
}

impl From<CreateInitialDocumentsError> for AccountCreateError {
    fn from(error: CreateInitialDocumentsError) -> Self {
        Self::InitialDocumentsCreationFailed { source: error }
    }
}

/// Error type for Account::load function
#[derive(Error, Debug)]
pub enum AccountLoadError {
    #[error("Account directory not found: {path}")]
    AccountDirectoryNotFound { path: std::path::PathBuf },

    #[error("Account directory not readable: {path}")]
    AccountDirectoryNotReadable { path: std::path::PathBuf },

    #[error("Account directory invalid: {path}")]
    AccountDirectoryInvalid { path: std::path::PathBuf },

    #[error("Mail database not found: {path}")]
    MailDatabaseNotFound { path: std::path::PathBuf },

    #[error("User database not found: {path}")]
    UserDatabaseNotFound { path: std::path::PathBuf },

    #[error("Automerge database not found: {path}")]
    AutomergeDatabaseNotFound { path: std::path::PathBuf },

    #[error("Failed to open database: {path}")]
    DatabaseOpenFailed {
        path: std::path::PathBuf,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("No aliases found in account")]
    NoAliasesFound,

    #[error("Failed to load alias: {id52}")]
    AliasLoadingFailed {
        id52: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

/// Error type for AccountManager::create function
#[derive(Error, Debug)]
pub enum AccountManagerCreateError {
    #[error("Failed to create account")]
    AccountCreationFailed {
        #[source]
        source: AccountCreateError,
    },

    #[error("Primary account ID not found")]
    PrimaryAccountIdNotFound,
}

/// Error type for AccountManager::load function
#[derive(Error, Debug)]
pub enum AccountManagerLoadError {
    #[error("No accounts directory found: {path}")]
    AccountsDirectoryNotFound { path: std::path::PathBuf },

    #[error("Failed to scan accounts directory: {path}")]
    AccountsScanFailed {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("No accounts found in fastn_home")]
    NoAccountsFound,
}

/// Error type for AccountManager::get_all_endpoints function
#[derive(Error, Debug)]
pub enum GetAllEndpointsError {
    #[error("Failed to scan accounts directory: {path}")]
    AccountsScanFailed {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to load account: {path}")]
    AccountLoadFailed {
        path: std::path::PathBuf,
        #[source]
        source: AccountLoadError,
    },
}

// Re-export mail error types from fastn-mail
pub use fastn_mail::MigrateMailDatabaseError;

/// Error type for migrate_user_database function
#[derive(Error, Debug)]
pub enum MigrateUserDatabaseError {
    #[error("Failed to initialize user database schema")]
    SchemaInitializationFailed {
        #[source]
        source: rusqlite::Error,
    },
}

/// Error type for create_initial_documents function
#[derive(Error, Debug)]
pub enum CreateInitialDocumentsError {
    #[error("Failed to create account config document")]
    AccountConfigCreationFailed {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Failed to create alias document")]
    AliasDocumentCreationFailed {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}
