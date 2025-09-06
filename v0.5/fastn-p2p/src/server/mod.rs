//! Server-side P2P functionality
//! 
//! This module provides high-level, type-safe APIs for implementing P2P servers.

pub mod handle;
pub mod listener;  
pub mod management;
pub mod request;

// Public API exports - no use statements, direct qualification
pub use handle::{P2PResponseHandle, P2PSendError};
pub use listener::p2p_listen;
pub use management::{
    active_listener_count, active_listeners, is_listening as is_p2p_listening, 
    stop_listening as p2p_stop_listening, ListenerAlreadyActiveError, ListenerNotFoundError,
};
pub use request::{GetInputError, HandleRequestError, PeerRequest};