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

pub mod endpoint;
mod rig;
mod run;

pub use run::run;

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
    #[expect(unused)]
    pub(crate) path: std::path::PathBuf,
    /// Rig's identity
    pub(crate) secret_key: fastn_id52::SecretKey,
    /// Owner account public key (first account created)
    pub(crate) owner: fastn_id52::PublicKey,
    /// Automerge database connection
    pub(crate) automerge: std::sync::Arc<tokio::sync::Mutex<rusqlite::Connection>>,
}

/// Manages all network endpoints
pub struct EndpointManager {
    /// Active endpoints only
    pub(crate) active: std::collections::HashMap<String, fastn_rig::EndpointHandle>,
    /// Channel to send all incoming messages with owner type
    pub(crate) message_tx: tokio::sync::mpsc::Sender<(String, fastn_rig::OwnerType, Vec<u8>)>,
    /// Graceful handler for spawning tasks
    pub(crate) graceful: fastn_net::Graceful,
}

/// Handle for an active endpoint
pub(crate) struct EndpointHandle {
    /// The secret key for this endpoint
    #[expect(unused)]
    pub(crate) secret_key: Vec<u8>,
    /// Type of owner (Account, Device, Rig)
    #[expect(unused)]
    pub(crate) owner_type: OwnerType,
    /// Path to the owner's storage directory
    #[expect(unused)]
    pub(crate) owner_path: std::path::PathBuf,
    /// The Iroh endpoint
    pub(crate) endpoint: iroh::Endpoint,
    /// Task handle
    pub(crate) handle: tokio::task::JoinHandle<()>,
}
