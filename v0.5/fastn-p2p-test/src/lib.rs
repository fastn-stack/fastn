//! Shared test protocol definitions for fastn-p2p testing

use serde::{Deserialize, Serialize};

/// Test protocol with meaningful names
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub enum TestProtocol {
    Echo,
}

impl std::fmt::Display for TestProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

/// Echo request message
#[derive(Serialize, Deserialize, Debug)]
pub struct EchoRequest {
    pub from: String,
    pub to: String,
    pub message: String,
    pub timestamp: i64,
}

/// Echo response message
#[derive(Serialize, Deserialize, Debug)]
pub struct EchoResponse {
    pub response: String,
}

/// Echo error message
#[derive(Serialize, Deserialize, Debug)]  
pub struct EchoError {
    pub error: String,
}