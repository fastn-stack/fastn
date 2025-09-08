//! # fastn-rig
//!
//! Central coordination layer for the FASTN P2P network.
//!
//! The Rig is the fundamental node in the FASTN network that manages:
//! - **Network Identity**: Each Rig has its own ID52 identity for P2P communication
//! - **Entity Coordination**: Manages accounts and devices (future)
//! - **Endpoint Lifecycle**: Controls which endpoints are online/offline
//! - **Current Entity**: Tracks which entity is currently active
//! - **Database State**: Maintains persistent state in rig.sqlite
//!
//! ## Architecture
//!
//! The Rig acts as the coordinator between different entities (accounts, devices)
//! and the network layer. It maintains a single database (`rig.sqlite`) that tracks:
//! - Which endpoints are online (`is_online`)
//! - Which entity is current (`is_current`)
//!
//! ## Initialization
//!
//! ```ignore
//! // First time initialization
//! let rig = fastn_rig::Rig::create(fastn_home, None)?;
//!
//! // Loading existing Rig
//! let rig = fastn_rig::Rig::load(fastn_home)?;
//! ```

extern crate self as fastn_rig;

pub mod automerge;
mod certs;
pub mod email_delivery_p2p;
pub mod email_poller_p2p;
pub mod errors;
mod http_routes;
mod http_server;
mod imap;
pub mod p2p_server;
pub mod protocols;
mod rig;
mod run;
mod smtp;

mod template_context;
#[cfg(test)]
pub mod test_utils;

pub use run::run;

/// Resolve fastn_home path with fallback logic
pub fn resolve_fastn_home(
    home: Option<std::path::PathBuf>,
) -> Result<std::path::PathBuf, RunError> {
    match home {
        Some(path) => Ok(path),
        None => match std::env::var("FASTN_HOME") {
            Ok(env_path) => Ok(std::path::PathBuf::from(env_path)),
            Err(_) => {
                let proj_dirs = directories::ProjectDirs::from("com", "fastn", "fastn")
                    .ok_or(RunError::FastnHomeResolution)?;
                Ok(proj_dirs.data_dir().to_path_buf())
            }
        },
    }
}

// Re-export specific error types
pub use errors::{
    CurrentEntityError, EmailDeliveryError, EntityStatusError, MessageProcessingError,
    RigCreateError, RigHttpError, RigLoadError, RunError, SmtpError,
};

/// Type of owner for an endpoint
#[derive(Clone, Debug)]
pub enum OwnerType {
    Account,
    Device,
    Rig,
}

/// The Rig coordinates all entities and networking
#[derive(Clone)]
pub struct Rig {
    /// Path to fastn_home
    pub(crate) path: std::path::PathBuf,
    /// Rig's identity
    pub(crate) secret_key: fastn_id52::SecretKey,
    /// Owner account public key (first account created)
    pub(crate) owner: fastn_id52::PublicKey,
    /// Automerge database
    pub(crate) automerge: std::sync::Arc<tokio::sync::Mutex<fastn_automerge::Db>>,
}

// Old EndpointManager, P2PMessage, and EndpointHandle removed - fastn-p2p handles everything!
