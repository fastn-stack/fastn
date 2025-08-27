use thiserror::Error;

/// Error type for Rig::create function
#[derive(Error, Debug)]
pub enum RigCreateError {
    #[error("Failed to create fastn_home directory: {path}")]
    FastnHomeCreationFailed {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to generate rig secret key")]
    KeyGenerationFailed,

    #[error("Failed to write rig key file: {path}")]
    KeyFileWriteFailed {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to store rig key in keyring")]
    KeyringStorageFailed,

    #[error("Failed to initialize automerge database")]
    AutomergeInitFailed {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Failed to create account manager")]
    AccountManagerCreateFailed {
        #[source]
        source: fastn_account::AccountManagerCreateError,
    },

    #[error("Failed to parse owner public key")]
    OwnerKeyParsingFailed,

    #[error("Failed to create rig config document")]
    RigConfigCreationFailed {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

/// Error type for Rig::load function
#[derive(Error, Debug)]
pub enum RigLoadError {
    #[error("Failed to load rig secret key from directory: {path}")]
    KeyLoadingFailed {
        path: std::path::PathBuf,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Failed to open automerge database: {path}")]
    AutomergeDatabaseOpenFailed {
        path: std::path::PathBuf,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Failed to load rig config document")]
    RigConfigLoadFailed {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

/// Error type for entity online status functions
#[derive(Error, Debug)]
pub enum EntityStatusError {
    #[error("Failed to parse entity ID52: {id52}")]
    InvalidId52 { id52: String },

    #[error("Failed to access entity status in database")]
    DatabaseAccessFailed {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

/// Error type for current entity functions
#[derive(Error, Debug)]
pub enum CurrentEntityError {
    #[error("Failed to parse entity ID52: {id52}")]
    InvalidId52 { id52: String },

    #[error("Failed to access rig config in database")]
    DatabaseAccessFailed {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

/// Error type for EndpointManager functions
#[derive(Error, Debug)]
pub enum EndpointError {
    #[error("Endpoint {id52} already online")]
    EndpointAlreadyOnline { id52: String },

    #[error("Invalid secret key length: expected 32 bytes")]
    InvalidSecretKeyLength,

    #[error("Failed to create Iroh endpoint")]
    IrohEndpointCreationFailed {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Endpoint {id52} not found")]
    EndpointNotFound { id52: String },

    #[error("Connection handling failed")]
    ConnectionHandlingFailed {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

/// Error type for run function
#[derive(Error, Debug)]
pub enum RunError {
    #[error("Failed to determine fastn_home directory")]
    FastnHomeResolutionFailed,

    #[error("Failed to create fastn_home directory: {path}")]
    FastnHomeCreationFailed {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to open lock file: {path}")]
    LockFileOpenFailed {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to acquire exclusive lock")]
    LockAcquisitionFailed,

    #[error("Failed to create rig")]
    RigCreationFailed {
        #[source]
        source: RigCreateError,
    },

    #[error("Failed to load rig")]
    RigLoadingFailed {
        #[source]
        source: RigLoadError,
    },

    #[error("Failed to load account manager")]
    AccountManagerLoadFailed {
        #[source]
        source: fastn_account::AccountManagerLoadError,
    },

    #[error("Failed to set entity online status")]
    EntityOnlineStatusFailed {
        #[source]
        source: EntityStatusError,
    },

    #[error("Failed to handle current entity operation")]
    CurrentEntityFailed {
        #[source]
        source: CurrentEntityError,
    },

    #[error("Failed to get all endpoints")]
    EndpointEnumerationFailed {
        #[source]
        source: fastn_account::GetAllEndpointsError,
    },

    #[error("Failed to bring endpoint online")]
    EndpointOnlineFailed {
        #[source]
        source: EndpointError,
    },

    #[error("Graceful shutdown failed")]
    ShutdownFailed {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

/// Error type for email delivery operations
#[derive(Error, Debug)]
pub enum EmailDeliveryError {
    #[error("Failed to load account mail store")]
    MailStoreLoadFailed {
        #[source]
        source: fastn_mail::StoreLoadError,
    },

    #[error("Failed to get pending deliveries")]
    PendingDeliveriesQueryFailed {
        #[source]
        source: fastn_mail::GetPendingDeliveriesError,
    },

    #[error("Failed to get emails for peer")]
    EmailsForPeerQueryFailed {
        #[source]
        source: fastn_mail::GetEmailsForPeerError,
    },

    #[error("Failed to mark email as delivered")]
    MarkDeliveredFailed {
        #[source]
        source: fastn_mail::MarkDeliveredError,
    },

    #[error("Invalid alias ID52 format: {alias}")]
    InvalidAliasFormat { alias: String },

    #[error("No sender alias found for peer: {peer_id52}")]
    NoSenderAliasFound { peer_id52: String },

    #[error("Failed to get account endpoints")]
    EndpointEnumerationFailed {
        #[source]
        source: fastn_account::GetAllEndpointsError,
    },
}

/// Error type for message processing functions
#[derive(Error, Debug)]
pub enum MessageProcessingError {
    #[error("Failed to deserialize P2P message")]
    MessageDeserializationFailed {
        #[source]
        source: serde_json::Error,
    },

    #[error("Invalid endpoint ID52: {endpoint_id52}")]
    InvalidEndpointId52 { endpoint_id52: String },

    #[error("Failed to handle account message")]
    AccountMessageHandlingFailed {
        #[source]
        source: fastn_account::HandleAccountMessageError,
    },

    #[error("Message processing not implemented for endpoint: {endpoint_id52}")]
    NotImplemented { endpoint_id52: String },
}
