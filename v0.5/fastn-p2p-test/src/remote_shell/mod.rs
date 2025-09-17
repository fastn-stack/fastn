//! Remote Shell Module
//!
//! This module provides a complete reference implementation of a remote shell
//! system using fastn-p2p. It demonstrates best practices for:
//!
//! - Protocol design and type safety
//! - Command execution and security
//! - Error handling and user experience
//! - P2P communication patterns
//! - Code organization and documentation
//!
//! # Architecture
//!
//! The remote shell system consists of three main components:
//!
//! - **Protocol**: Type-safe definitions for P2P communication
//! - **Executor**: Secure command execution engine
//! - **Binaries**: Client and daemon applications
//!
//! # Security Considerations
//!
//! This implementation is intended for demonstration and testing purposes.
//! Production deployments should include additional security measures:
//!
//! - Authentication and authorization
//! - Command sandboxing and restrictions
//! - Resource limits and monitoring
//! - Audit logging and compliance
//! - Network security and encryption
//!
//! # Usage Patterns
//!
//! This module serves as a reference for implementing other P2P applications:
//!
//! - Request/response protocols
//! - Error handling strategies
//! - User interface design
//! - Testing approaches
//! - Documentation standards

pub mod executor;
pub mod protocol;

// Re-export commonly used types for convenience
pub use executor::execute_command;
pub use protocol::{ExecuteRequest, ExecuteResponse, RemoteShellError, RemoteShellProtocol};