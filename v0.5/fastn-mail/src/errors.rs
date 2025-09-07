use thiserror::Error;

/// Error type for Store::create function
#[derive(Error, Debug)]
pub enum StoreCreateError {
    #[error("Failed to create mail directories: {path}")]
    DirectoryCreation {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to create mail database: {path}")]
    DatabaseCreation {
        path: std::path::PathBuf,
        #[source]
        source: rusqlite::Error,
    },

    #[error("Failed to run database migrations")]
    Migration {
        #[source]
        source: rusqlite::Error,
    },
}

/// Error type for Store::load function
#[derive(Error, Debug)]
pub enum StoreLoadError {
    #[error("Mail database not found: {path}")]
    DatabaseNotFound { path: std::path::PathBuf },

    #[error("Failed to open mail database: {path}")]
    DatabaseOpenFailed {
        path: std::path::PathBuf,
        #[source]
        source: rusqlite::Error,
    },

    #[error("Failed to run database migrations")]
    Migration {
        #[source]
        source: rusqlite::Error,
    },
}

/// Error type for smtp_receive function
#[derive(Error, Debug)]
pub enum SmtpReceiveError {
    #[error("Email contains invalid UTF-8 encoding")]
    InvalidUtf8Encoding,

    #[error("Email missing required header/body separator (\\r\\n\\r\\n)")]
    MissingHeaderBodySeparator,

    #[error("Email uses invalid line ending format (found \\n\\n, expected \\r\\n\\r\\n)")]
    InvalidLineEndings,

    #[error("Invalid domain format in address: {address}")]
    InvalidDomainFormat { address: String },

    #[error("Email validation failed: {reason}")]
    ValidationFailed { reason: String },

    #[error("Failed to store message file: {path}")]
    FileStoreFailed {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to insert into database")]
    DatabaseInsertFailed {
        #[source]
        source: rusqlite::Error,
    },

    #[error("Invalid email address format: {address}")]
    InvalidEmailAddress { address: String },

    #[error("Message parsing failed: {message}")]
    MessageParsingFailed { message: String },
}

/// Error type for get_pending_deliveries function
#[derive(Error, Debug)]
pub enum GetPendingDeliveriesError {
    #[error("Database query failed")]
    DatabaseQueryFailed {
        #[source]
        source: rusqlite::Error,
    },
}

/// Error type for get_emails_for_peer function
#[derive(Error, Debug)]
pub enum GetEmailsForPeerError {
    #[error("Database query failed")]
    DatabaseQueryFailed {
        #[source]
        source: rusqlite::Error,
    },

    #[error("Failed to read email file: {path}")]
    FileReadFailed {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
}

/// Error type for mark_delivered_to_peer function
#[derive(Error, Debug)]
pub enum MarkDeliveredError {
    #[error("Database update failed")]
    DatabaseUpdateFailed {
        #[source]
        source: rusqlite::Error,
    },

    #[error("Email not found: {email_id}")]
    EmailNotFound { email_id: String },
}

/// Error type for imap_list_folders function
#[derive(Error, Debug)]
pub enum ImapListFoldersError {
    #[error("Failed to scan mail directories")]
    DirectoryScanFailed {
        #[source]
        source: std::io::Error,
    },
}

/// Error type for imap_select_folder function
#[derive(Error, Debug)]
pub enum ImapSelectFolderError {
    #[error("Folder not found: {folder}")]
    FolderNotFound { folder: String },

    #[error("Database query failed")]
    DatabaseQueryFailed {
        #[source]
        source: rusqlite::Error,
    },
}

/// Error type for imap_fetch function
#[derive(Error, Debug)]
pub enum ImapFetchError {
    #[error("Email not found with UID: {uid}")]
    EmailNotFound { uid: u32 },

    #[error("Failed to read email file: {path}")]
    FileReadFailed {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Database query failed")]
    DatabaseQueryFailed {
        #[source]
        source: rusqlite::Error,
    },
}

/// Error type for imap_search function
#[derive(Error, Debug)]
pub enum ImapSearchError {
    #[error("Invalid search criteria: {criteria}")]
    InvalidSearchCriteria { criteria: String },

    #[error("Database query failed")]
    DatabaseQueryFailed {
        #[source]
        source: rusqlite::Error,
    },
}

/// Error type for imap_store_flags function
#[derive(Error, Debug)]
pub enum ImapStoreFlagsError {
    #[error("Email not found with UID: {uid}")]
    EmailNotFound { uid: u32 },

    #[error("Database update failed")]
    DatabaseUpdateFailed {
        #[source]
        source: rusqlite::Error,
    },
}

/// Error type for imap_expunge function
#[derive(Error, Debug)]
pub enum ImapExpungeError {
    #[error("Database operation failed")]
    DatabaseOperationFailed {
        #[source]
        source: rusqlite::Error,
    },

    #[error("Failed to delete email file: {path}")]
    FileDeleteFailed {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
}

/// Error type for imap_thread function
#[derive(Error, Debug)]
pub enum ImapThreadError {
    #[error("Threading algorithm not supported: {algorithm}")]
    UnsupportedAlgorithm { algorithm: String },

    #[error("Database query failed")]
    DatabaseQueryFailed {
        #[source]
        source: rusqlite::Error,
    },
}
