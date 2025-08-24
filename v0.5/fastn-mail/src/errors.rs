use thiserror::Error;

/// Error type for migrate_mail_database function
#[derive(Error, Debug)]
pub enum MigrateMailDatabaseError {
    #[error("Failed to initialize mail database schema")]
    SchemaInitializationFailed {
        #[source]
        source: rusqlite::Error,
    },
}

/// Error type for mail database connection operations
#[derive(Error, Debug)]
pub enum MailDatabaseConnectionError {
    #[error("Failed to connect to mail database: {path}")]
    ConnectionFailed {
        path: std::path::PathBuf,
        #[source]
        source: rusqlite::Error,
    },

    #[error("Mail database not found: {path}")]
    DatabaseNotFound { path: std::path::PathBuf },
}

/// Error type for Mail::create function
#[derive(Error, Debug)]
pub enum MailCreateError {
    #[error("Failed to create mail directories: {path}")]
    DirectoryCreationFailed {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to connect to mail database")]
    ConnectionFailed {
        #[source]
        source: MailDatabaseConnectionError,
    },

    #[error("Failed to migrate mail database")]
    MigrationFailed {
        #[source]
        source: MigrateMailDatabaseError,
    },
}

/// Error type for Mail::load function
#[derive(Error, Debug)]
pub enum MailLoadError {
    #[error("Mail database not found: {path}")]
    DatabaseNotFound { path: std::path::PathBuf },

    #[error("Failed to connect to mail database: {path}")]
    ConnectionFailed {
        path: std::path::PathBuf,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}
