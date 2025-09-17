//! Command Execution Engine
//!
//! This module provides secure and robust command execution capabilities
//! for the remote shell system. It handles process management, output
//! capture, and error handling with appropriate security considerations.

use crate::remote_shell::protocol::{ExecuteRequest, ExecuteResponse, RemoteShellError};
use colored::*;
use tokio::process::Command;

/// Execute a command request and return the response
///
/// This function is the core command execution engine. It takes a validated
/// command request and executes it on the local system, capturing all output
/// and handling errors appropriately.
///
/// # Arguments
///
/// * `request` - The command execution request containing the command and arguments
///
/// # Returns
///
/// * `Ok(ExecuteResponse)` - Successful execution with captured output
/// * `Err(RemoteShellError)` - Various execution failures with detailed error information
///
/// # Security Considerations
///
/// This function executes arbitrary commands on the local system, which poses
/// significant security risks. In production deployments, consider:
///
/// - Command whitelisting/blacklisting
/// - Sandboxing (containers, chroot, etc.)
/// - Resource limits (CPU, memory, time)
/// - User privilege dropping
/// - Audit logging
///
/// # Example
///
/// ```rust,ignore
/// use crate::remote_shell::protocol::ExecuteRequest;
/// use crate::remote_shell::executor::execute_command;
///
/// let request = ExecuteRequest {
///     command: vec!["echo".to_string(), "Hello World".to_string()],
/// };
///
/// match execute_command(request).await {
///     Ok(response) => {
///         println!("Exit code: {}", response.exit_code);
///         println!("Output: {}", response.stdout);
///     }
///     Err(error) => {
///         eprintln!("Error: {}", error);
///     }
/// }
/// ```
pub async fn execute_command(request: ExecuteRequest) -> Result<ExecuteResponse, RemoteShellError> {
    // Validate command is not empty
    if request.command.is_empty() {
        return Err(RemoteShellError::ExecutionError {
            message: "Empty command provided".to_string(),
        });
    }

    let program = &request.command[0];
    let args = &request.command[1..];

    tracing::info!(
        program = %program,
        args = ?args,
        "Executing remote command"
    );

    // Log the command execution for the daemon operator
    println!(
        "{} {} {}",
        "🚀 Executing:".green(),
        program.cyan(),
        args.join(" ").yellow()
    );

    // Execute the command with proper error handling
    let output = Command::new(program)
        .args(args)
        .output()
        .await
        .map_err(|e| {
            let error_msg = format!("Failed to execute '{}': {}", program, e);
            tracing::error!(error = %e, program = %program, "Command execution failed");
            
            RemoteShellError::ExecutionError {
                message: error_msg,
            }
        })?;

    // Convert output to strings, handling invalid UTF-8 gracefully
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code().unwrap_or(-1);

    // Log output for daemon operator (with size limits for readability)
    log_output("stdout", &stdout);
    log_output("stderr", &stderr);

    println!(
        "{} {}",
        "✅ Command completed with exit code:".green(),
        exit_code.to_string().cyan()
    );

    tracing::info!(
        exit_code = exit_code,
        stdout_len = stdout.len(),
        stderr_len = stderr.len(),
        "Command execution completed"
    );

    Ok(ExecuteResponse {
        exit_code,
        stdout,
        stderr,
    })
}

/// Log command output with appropriate formatting and size limits
///
/// This helper function logs command output in a readable format,
/// truncating very large outputs to prevent log spam while still
/// providing useful debugging information.
///
/// # Arguments
///
/// * `stream_name` - Name of the output stream ("stdout" or "stderr")
/// * `content` - The content to log
fn log_output(stream_name: &str, content: &str) {
    if content.is_empty() {
        return;
    }

    const MAX_LOG_LENGTH: usize = 1000;
    let trimmed = content.trim();
    
    if trimmed.len() <= MAX_LOG_LENGTH {
        println!("📤 {}: {}", stream_name.blue(), trimmed);
    } else {
        println!(
            "📤 {} ({}): {}... (truncated, {} total bytes)",
            stream_name.blue(),
            "large output".yellow(),
            &trimmed[..MAX_LOG_LENGTH],
            content.len()
        );
    }
}

/// Validate command for security (placeholder for future implementation)
///
/// This function is a placeholder for implementing command validation
/// and security policies. In production systems, this should include:
///
/// - Whitelist/blacklist checking
/// - Path traversal prevention
/// - Resource limit validation
/// - Privilege checking
///
/// # Arguments
///
/// * `command` - The command vector to validate
///
/// # Returns
///
/// * `Ok(())` - Command is allowed
/// * `Err(RemoteShellError)` - Command is blocked with reason
#[allow(dead_code)]
pub fn validate_command(command: &[String]) -> Result<(), RemoteShellError> {
    // Placeholder for future security implementations
    
    // Example: Block dangerous commands (this is just a demo)
    if !command.is_empty() {
        let program = &command[0];
        
        // This is just an example - real implementations should be more sophisticated
        let blocked_commands = ["rm", "dd", "mkfs", "format"];
        
        if blocked_commands.contains(&program.as_str()) {
            return Err(RemoteShellError::ExecutionError {
                message: format!("Command '{}' is blocked by security policy", program),
            });
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_simple_command() {
        let request = ExecuteRequest {
            command: vec!["echo".to_string(), "test".to_string()],
        };

        let response = execute_command(request).await.unwrap();
        assert_eq!(response.exit_code, 0);
        assert_eq!(response.stdout.trim(), "test");
        assert!(response.stderr.is_empty());
    }

    #[tokio::test]
    async fn test_empty_command() {
        let request = ExecuteRequest {
            command: vec![],
        };

        let result = execute_command(request).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RemoteShellError::ExecutionError { .. }));
    }

    #[tokio::test]
    async fn test_nonexistent_command() {
        let request = ExecuteRequest {
            command: vec!["this_command_does_not_exist_12345".to_string()],
        };

        let result = execute_command(request).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RemoteShellError::ExecutionError { .. }));
    }
}