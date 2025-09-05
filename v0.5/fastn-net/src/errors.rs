//! Specific error types for fastn-net functions
//! 
//! Replaces generic eyre::Result with proper error types for better error handling

use thiserror::Error;

/// Error types for get_stream function
#[derive(Debug, Error)]
pub enum GetStreamError {
    #[error("Failed to create endpoint")]
    EndpointCreationFailed {
        #[source]
        source: std::io::Error,
    },
    
    #[error("Connection to peer timed out")]
    ConnectionTimedOut,
    
    #[error("Connection to peer failed")]
    ConnectionFailed {
        #[source] 
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    
    #[error("Protocol negotiation failed")]
    ProtocolNegotiationFailed {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    
    #[error("Stream request channel closed")]
    ChannelClosed,
    
    #[error("Graceful shutdown requested")]
    GracefulShutdown,
}

/// Error types for accept_bi function  
#[derive(Debug, Error)]
pub enum AcceptBiError {
    #[error("Connection closed by peer")]
    ConnectionClosed,
    
    #[error("Stream closed by peer")]
    StreamClosed,
    
    #[error("Protocol mismatch: expected {expected:?}, got {actual:?}")]
    ProtocolMismatch {
        expected: Vec<crate::Protocol>,
        actual: crate::Protocol,
    },
    
    #[error("Failed to read protocol from stream")]
    ProtocolReadFailed {
        #[source]
        source: std::io::Error,
    },
    
    #[error("Failed to send ACK response")]
    AckSendFailed {
        #[source]
        source: std::io::Error,
    },
    
    #[error("Connection lost during protocol negotiation")]
    ConnectionLost {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

/// Error types for get_endpoint function
#[derive(Debug, Error)]
pub enum GetEndpointError {
    #[error("Invalid secret key format")]
    InvalidSecretKey,
    
    #[error("Failed to create iroh endpoint")]
    IrohEndpointFailed {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    
    #[error("Network binding failed")]
    NetworkBindFailed {
        #[source]
        source: std::io::Error,
    },
}

/// Error types for stream operations
#[derive(Debug, Error)]
pub enum StreamError {
    #[error("Failed to read from stream")]
    ReadFailed {
        #[source]
        source: std::io::Error,
    },
    
    #[error("Failed to write to stream")]
    WriteFailed {
        #[source]
        source: std::io::Error,
    },
    
    #[error("Invalid UTF-8 data received")]
    InvalidUtf8 {
        #[source]
        source: std::str::Utf8Error,
    },
    
    #[error("JSON deserialization failed")]
    JsonDeserialization {
        #[source]
        source: serde_json::Error,
    },
    
    #[error("Stream unexpectedly closed")]
    StreamClosed,
}