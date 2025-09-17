# fastn-p2p-test

High-quality reference implementations and testing utilities for the fastn-p2p library.

## Overview

This crate provides comprehensive examples of how to build P2P applications using fastn-p2p. All code is written to demonstrate best practices in:

- **API Design**: Type-safe protocols with strong error handling
- **Documentation**: Comprehensive docs with examples and rationale  
- **Code Organization**: Clear module structure and separation of concerns
- **Security**: Security considerations and safe defaults
- **Testing**: Unit tests and integration examples
- **User Experience**: Helpful error messages and CLI design

## Reference Implementations

### Remote Shell System

A complete P2P remote shell implementation demonstrating request/response patterns:

- **`remote-shell-daemon`**: Server that accepts P2P connections and executes commands
- **`remote-shell-client`**: Client that connects to remote daemons to execute commands

#### Quick Start

1. **Start the daemon** on the target machine:
   ```bash
   cargo run --bin remote-shell-daemon
   ```
   
   This will generate a new private key and display the ID52 for clients to connect to:
   ```
   🔑 Generated new private key
   Your ID52: 12D3KooWABC123...xyz
   ```

2. **Execute commands** from the client machine:
   ```bash
   # Replace 12D3K...xyz with the daemon's ID52
   cargo run --bin remote-shell-client 12D3KooWABC123...xyz whoami
   cargo run --bin remote-shell-client 12D3KooWABC123...xyz ls -la
   cargo run --bin remote-shell-client 12D3KooWABC123...xyz "echo 'Hello World'"
   ```

#### Security Warning

⚠️ **SECURITY WARNING** ⚠️

The remote shell implementation allows **FULL COMMAND EXECUTION** from remote machines. Only use this with completely trusted systems. In production, implement additional security measures:

- Command whitelisting/blacklisting
- User privilege separation  
- Resource limits and sandboxing
- Authentication and authorization
- Audit logging and monitoring

#### Architecture

The remote shell system demonstrates several fastn-p2p patterns:

- **Protocol Definition**: Type-safe enum protocols with Display implementation
- **Request/Response Types**: Structured data with comprehensive error types
- **Server Implementation**: Using `fastn_p2p::listen()` and `Request::handle()`
- **Client Implementation**: Using `fastn_p2p::client::call()` for RPC-style calls
- **Error Handling**: Comprehensive error types and user-friendly messages
- **Concurrent Processing**: Using `fastn_p2p::spawn()` for handling multiple requests

## API Design Patterns

### Protocol Definition

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RemoteShellProtocol {
    Execute,
}

impl std::fmt::Display for RemoteShellProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RemoteShellProtocol::Execute => write!(f, "remote-shell-execute"),
        }
    }
}
```

### Request/Response Types

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct ExecuteRequest {
    pub command: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecuteResponse {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RemoteShellError {
    CommandFailed { message: String, exit_code: Option<i32> },
    ExecutionError { message: String },
}
```

### Server Implementation Pattern

```rust
// Start listener
let protocols = [RemoteShellProtocol::Execute];
let stream = fastn_p2p::listen(private_key, &protocols)?;
let mut stream = std::pin::pin!(stream);

// Handle requests
while let Some(request_result) = stream.next().await {
    let request = request_result?;
    
    // Spawn concurrent handler
    fastn_p2p::spawn(async move {
        request.handle(|execute_request: ExecuteRequest| async move {
            // Your business logic here
            execute_command(execute_request).await
        }).await
    });
}
```

### Client Implementation Pattern

```rust
let request = ExecuteRequest { command };

let result: Result<Result<ExecuteResponse, RemoteShellError>, CallError> = 
    fastn_p2p::client::call(
        private_key,
        target,
        RemoteShellProtocol::Execute,
        request
    ).await;

match result {
    Ok(Ok(response)) => {
        // Handle successful response
        println!("{}", response.stdout);
        std::process::exit(response.exit_code);
    }
    Ok(Err(shell_error)) => {
        // Handle application-level errors
        eprintln!("Command failed: {}", shell_error);
    }
    Err(p2p_error) => {
        // Handle network/P2P errors
        eprintln!("Connection failed: {}", p2p_error);
    }
}
```

## Testing

The crate includes comprehensive tests demonstrating various scenarios:

```bash
# Run all tests
cargo test

# Run specific test module
cargo test remote_shell

# Run integration tests
cargo test --test '*'
```

## Building

```bash
# Build all binaries
cargo build --release

# Build specific binary
cargo build --release --bin remote-shell-daemon
cargo build --release --bin remote-shell-client

# Run with arguments
cargo run --bin remote-shell-daemon -- --help
cargo run --bin remote-shell-client -- --help
```

## Development Guidelines

When extending or modifying this crate, follow these guidelines:

### Documentation
- Every public item should have comprehensive documentation
- Include examples in doc comments where appropriate
- Explain design decisions and trade-offs
- Document security considerations

### Error Handling
- Use specific error types rather than generic errors
- Provide helpful error messages with context
- Include troubleshooting information where appropriate
- Log errors appropriately for debugging

### Code Organization
- Separate protocol definitions from implementation logic
- Use modules to organize related functionality
- Keep binary entry points simple and focused
- Extract reusable logic into library functions

### Testing
- Write unit tests for individual functions
- Include integration tests for end-to-end scenarios
- Test error conditions and edge cases
- Use property-based testing where appropriate

## Future Enhancements

This crate provides a foundation for more advanced P2P applications:

### Streaming Applications
- **Interactive Shell**: Real-time terminal interaction using streaming APIs
- **File Transfer**: Large file transfers with progress tracking
- **Audio/Video**: Real-time media streaming over P2P

### Advanced Protocols
- **Multi-Protocol Support**: Negotiating different protocol versions
- **Authentication**: Secure authentication and authorization
- **Discovery**: Automatic peer discovery and service advertisement

### Performance Applications
- **Load Testing**: Tools for testing P2P network performance
- **Benchmarking**: Measuring throughput and latency
- **Monitoring**: Real-time network and application metrics

## Contributing

When contributing to fastn-p2p-test:

1. Follow the established code organization patterns
2. Add comprehensive documentation and examples
3. Include appropriate tests
4. Consider security implications
5. Update this README for significant changes

## License

This crate follows the same license as the main fastn project.