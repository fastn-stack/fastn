//! fastn-rig specific P2P protocols
//! 
//! Defines meaningful protocol names for fastn-rig P2P communication

use serde::{Deserialize, Serialize};

/// fastn-rig P2P protocols - meaningful names for actual purposes
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum RigProtocol {
    /// Email delivery between accounts
    EmailDelivery,
    /// Account-to-account messaging  
    AccountMessage,
    /// HTTP proxy requests
    HttpProxy,
    /// Rig control and management
    RigControl,
}

impl std::fmt::Display for RigProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RigProtocol::EmailDelivery => write!(f, "EmailDelivery"),
            RigProtocol::AccountMessage => write!(f, "AccountMessage"), 
            RigProtocol::HttpProxy => write!(f, "HttpProxy"),
            RigProtocol::RigControl => write!(f, "RigControl"),
        }
    }
}