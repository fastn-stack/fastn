//! fastn-p2p Test Suite and Reference Implementations
//!
//! This crate provides comprehensive testing utilities and reference implementations
//! for the fastn-p2p library. It includes:
//!
//! - **Test Protocols**: Simple protocols for basic P2P testing
//! - **Reference Applications**: Complete, well-documented example applications
//! - **Performance Tests**: Benchmarks and stress tests
//! - **Integration Tests**: End-to-end testing scenarios
//!
//! # Modules
//!
//! - [`remote_shell`]: Complete remote shell implementation demonstrating
//!   request/response patterns, command execution, and error handling
//!
//! # Design Philosophy
//!
//! All code in this crate is written to demonstrate best practices:
//!
//! - **Documentation**: Comprehensive docs with examples and rationale
//! - **Error Handling**: Proper error types and user-friendly messages
//! - **Type Safety**: Strong typing and compile-time verification
//! - **Security**: Security considerations and safe defaults
//! - **Testing**: Unit tests and integration test examples
//! - **Code Organization**: Clear module structure and separation of concerns

use serde::{Deserialize, Serialize};

// Reference implementation modules
pub mod remote_shell;

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
