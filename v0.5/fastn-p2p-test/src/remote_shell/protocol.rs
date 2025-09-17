//! Remote Shell Protocol
//!
//! This module defines the P2P communication protocol for remote shell execution.
//! It demonstrates how to create type-safe, versioned protocols for fastn-p2p applications.

use serde::{Deserialize, Serialize};

/// Protocol identifier for remote shell communication
///
/// This enum defines the different types of operations supported by the remote shell system.
/// Each variant represents a specific protocol that clients and servers can negotiate.
///
/// # Design Notes
///
/// - Uses enum to allow for future protocol extensions (e.g., Interactive, FileTransfer)
/// - Implements all required traits for fastn-p2p protocol negotiation
/// - Display implementation provides human-readable protocol names for debugging
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RemoteShellProtocol {
    /// Execute a single command and return the result
    Execute,
}

impl std::fmt::Display for RemoteShellProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RemoteShellProtocol::Execute => write!(f, "remote-shell-execute"),
        }
    }
}

/// Request to execute a command on a remote machine
///
/// This structure encapsulates all the information needed to execute a command
/// on a remote system through the P2P network.
///
/// # Fields
///
/// - `command`: Vector of strings representing the command and its arguments
///   - First element is the program name
///   - Subsequent elements are command-line arguments
///   - Follows standard Unix convention (e.g., ["ls", "-la", "/tmp"])
///
/// # Examples
///
/// ```rust,ignore
/// // Simple command
/// let request = ExecuteRequest {
///     command: vec!["echo".to_string(), "hello world".to_string()],
/// };
///
/// // Complex command with multiple arguments
/// let request = ExecuteRequest {
///     command: vec![
///         "find".to_string(),
///         "/var/log".to_string(),
///         "-name".to_string(),
///         "*.log".to_string(),
///         "-mtime".to_string(),
///         "-7".to_string(),
///     ],
/// };
/// ```
#[derive(Debug, Serialize, Deserialize)]
pub struct ExecuteRequest {
    pub command: Vec<String>,
}

/// Response from remote command execution
///
/// This structure contains the complete result of a command execution,
/// including exit status and all output streams.
///
/// # Fields
///
/// - `exit_code`: The exit status code returned by the command
///   - 0 typically indicates success
///   - Non-zero values indicate various error conditions
///   - -1 is used when the exit code cannot be determined
/// - `stdout`: All output written to standard output by the command
/// - `stderr`: All output written to standard error by the command
///
/// # Design Notes
///
/// - Captures both stdout and stderr separately for proper handling
/// - Preserves exact command output including whitespace and formatting
/// - Exit code allows clients to determine success/failure programmatically
#[derive(Debug, Serialize, Deserialize)]
pub struct ExecuteResponse {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// Error conditions for remote shell operations
///
/// This enum defines the various error states that can occur during
/// remote command execution, providing detailed error information
/// for proper client-side handling.
///
/// # Variants
///
/// - `CommandFailed`: The command was executed but returned an error
/// - `ExecutionError`: The command could not be executed at all
///
/// # Design Philosophy
///
/// Different error types allow clients to distinguish between:
/// 1. Commands that ran but failed (CommandFailed)
/// 2. System-level execution problems (ExecutionError)
///
/// This enables appropriate error handling and user feedback.
#[derive(Debug, Serialize, Deserialize)]
pub enum RemoteShellError {
    /// Command was executed but returned a non-zero exit code
    CommandFailed {
        /// Human-readable error description
        message: String,
        /// The actual exit code returned by the command
        exit_code: Option<i32>,
    },
    /// Command could not be executed due to system-level errors
    ExecutionError {
        /// Detailed error message explaining what went wrong
        message: String,
    },
}

impl std::fmt::Display for RemoteShellError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RemoteShellError::CommandFailed { message, exit_code } => {
                if let Some(code) = exit_code {
                    write!(f, "Command failed (exit code {}): {}", code, message)
                } else {
                    write!(f, "Command failed: {}", message)
                }
            }
            RemoteShellError::ExecutionError { message } => {
                write!(f, "Execution error: {}", message)
            }
        }
    }
}

impl std::error::Error for RemoteShellError {}