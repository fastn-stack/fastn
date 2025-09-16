# fastn-p2p: High-Level Type-Safe P2P Communication

This crate provides a high-level, type-safe API for P2P communication in the fastn ecosystem. It builds on top of `fastn-net` but exposes only the essential, locked-down APIs that reduce the possibility of bugs through strong typing and compile-time verification.

## Design Philosophy

- **Type Safety First**: All communication uses strongly-typed protocol contracts
- **Minimal Surface Area**: Only essential APIs are exposed to reduce complexity  
- **Bug Prevention**: API design makes common mistakes impossible or unlikely
- **Ergonomic**: High-level APIs handle boilerplate automatically
- **Zero Boilerplate**: Main functions are clean with automatic setup/teardown

## Core API Patterns

fastn-p2p provides three main communication patterns:

### 1. **Simple Request/Response** - `client::call()`

For simple RPC-style communication with typed request/response.

### 2. **Streaming Sessions** - `client::connect()`

For complex interactions requiring bidirectional streaming with optional side channels.

### 3. **Server Listening** - `server::listen()`

For accepting incoming connections and handling both patterns.

## Quick Start

### Application Entry Point

Use `#[fastn_p2p::main]` to eliminate all boilerplate around graceful shutdown, logging, and signal handling:

```rust
#[fastn_p2p::main]
async fn main() -> eyre::Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Command::Serve { port } => {
            fastn_p2p::spawn(start_server());
        }
        Command::Client { target } => {
            fastn_p2p::spawn(run_client(target));
        }
    }
    
    // Automatic graceful shutdown handling
    // No manual signal handlers needed
}
```

### Configuration Options

```rust
#[fastn_p2p::main(
    // Logging configuration
    logging = true,                    // Default: true - simple logging setup
    
    // Shutdown behavior  
    shutdown_mode = "single_ctrl_c",   // Default: "single_ctrl_c"
    shutdown_timeout = "30s",          // Default: "30s" - graceful shutdown timeout
    
    // Double Ctrl+C specific (only when shutdown_mode = "double_ctrl_c")
    double_ctrl_c_window = "2s",       // Default: "2s" - time window for second Ctrl+C
    status_fn = my_status_printer,     // Required for double_ctrl_c mode
)]
async fn main() -> eyre::Result<()> {
    // Your application code
}

// Status function (required for double_ctrl_c mode)
async fn my_status_printer() {
    println!("=== Application Status ===");
    // Custom status logic - access global registries, counters, etc.
    println!("Active services: {}", get_active_service_count());
    println!("P2P listeners: {}", fastn_p2p::active_listener_count());
}
```

#### Logging Configuration

The `logging` parameter supports multiple formats:

```rust
// Simple on/off (most common)
#[fastn_p2p::main(logging = true)]   // tracing_subscriber::fmt::init()
#[fastn_p2p::main(logging = false)]  // No logging setup

// Global log level
#[fastn_p2p::main(logging = "debug")]  // All logs at debug level
#[fastn_p2p::main(logging = "trace")]  // All logs at trace level

// Module-specific filtering (for debugging)
#[fastn_p2p::main(logging = "fastn_p2p=trace,fastn_net=debug,warn")]
#[fastn_p2p::main(logging = "fastn_p2p=trace,info")]
```

**Logging Examples:**

- **Production**: `logging = true` or `logging = "info"` 
- **Development**: `logging = "debug"`
- **Deep debugging**: `logging = "fastn_p2p=trace,fastn_net=trace"`
- **Specific modules**: `logging = "my_app=debug,fastn_p2p=info,warn"`
- **No logging**: `logging = false`

#### Environment Variable Override

The `RUST_LOG` environment variable always takes precedence over macro configuration, allowing runtime debugging without code changes:

```bash
# Environment variable overrides macro setting
RUST_LOG=trace cargo run                    # Uses trace level, ignores macro
RUST_LOG=fastn_p2p=debug cargo run         # Module-specific override
RUST_LOG=fastn_p2p=trace,warn cargo run    # Complex filtering override

# No RUST_LOG - uses macro configuration
cargo run                                   # Uses macro parameter as fallback

# No RUST_LOG, no macro parameter - uses default
#[fastn_p2p::main]                         # Uses "info" as default
```

**Priority Order:**

1. **`RUST_LOG` environment variable** (highest priority)
2. **Macro `logging` parameter** (fallback)  
3. **Default `"info"`** (lowest priority)

**Special Cases:**

```rust
// Even logging = false can be overridden for debugging
#[fastn_p2p::main(logging = false)]
```

```bash
RUST_LOG=debug cargo run  # Still enables logging despite logging = false
```

This design allows developers to debug any application by setting `RUST_LOG` without modifying source code.

## API Reference

### 1. Simple Request/Response - `client::call()`

For RPC-style communication with typed request/response patterns:

```rust
#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct EchoRequest {
    message: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct EchoResponse {
    reply: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct EchoError {
    reason: String,
}

// Client side - must specify protocol (same as server)
let response: Result<EchoResponse, EchoError> = fastn_p2p::client::call(
    my_key,
    target_id52,
    EchoProtocol,  // Must match server's protocol
    EchoRequest { message: "Hello!".to_string() }
).await?;

match response {
    Ok(resp) => println!("Got reply: {}", resp.reply),
    Err(err) => println!("Server error: {}", err.reason),
}
```

### 2. Streaming Sessions - `client::connect()`

For complex interactions requiring persistent bidirectional communication with optional side channels:

```rust
#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct RemoteShellProtocol {
    shell: String,
    args: Vec<String>,
}

// Client side - establish streaming session
let mut session = fastn_p2p::client::connect(
    my_key,
    target_id52, 
    RemoteShellProtocol {
        shell: "bash".to_string(),
        args: vec![],
    }
).await?;

// Direct access to main streams
fastn_p2p::spawn(pipe_stdin_to_remote(session.stdin));
fastn_p2p::spawn(pipe_stdout_from_remote(session.stdout));

// Optional: accept side channels back from server (iroh terminology)
if let Ok(stderr_stream) = session.accept_uni().await {
    fastn_p2p::spawn(pipe_stderr_from_remote(stderr_stream));
}
```

### 3. Server Listening - `server::listen()`

For accepting both call() and connect() patterns with unified Session handling:

```rust
// Server side - listen for incoming connections
let mut listener = fastn_p2p::server::listen!(my_key, &[
    EchoProtocol,
    RemoteShellProtocol::default(),
]).await?;

while let Some(session) = listener.next().await {
    match session.protocol {
        EchoProtocol => {
            // RPC pattern - convert Session to Request
            let request = session.into_request();
            request.handle(|input: EchoRequest| async move {
                Ok::<EchoResponse, EchoError>(EchoResponse { 
                    reply: format!("Echo: {}", input.message) 
                })
            }).await?;
        }
        RemoteShellProtocol { .. } => {
            // Streaming pattern - use Session directly
            let mut shell_session = session;
            
            // Direct access to streams (public fields)
            fastn_p2p::spawn(handle_stdin(shell_session.recv));
            fastn_p2p::spawn(handle_stdout(shell_session.send));
            
            // Optional: open stderr channel back to client
            if let Ok(stderr) = shell_session.open_uni().await {
                fastn_p2p::spawn(handle_stderr(stderr));
            }
        }
    }
}
```

#### Shutdown Modes

**Single Ctrl+C Mode (Default):**

```rust
#[fastn_p2p::main(shutdown_mode = "single_ctrl_c")]
async fn main() -> eyre::Result<()> { ... }
```

- Ctrl+C immediately triggers graceful shutdown
- Wait `shutdown_timeout` for tasks to complete  
- Force exit if timeout exceeded
- Simple and predictable for most applications

**Double Ctrl+C Mode:**

```rust
#[fastn_p2p::main(
    shutdown_mode = "double_ctrl_c",
    status_fn = print_status,
    double_ctrl_c_window = "2s"
)]
async fn main() -> eyre::Result<()> { ... }

async fn print_status() {
    // Show application status
    println!("Services: {} active", get_service_count());
}
```

- First Ctrl+C calls `status_fn` and waits for second Ctrl+C
- Second Ctrl+C within `double_ctrl_c_window` triggers shutdown
- If no second Ctrl+C, continues running normally
- Useful for long-running services where you want to check status

#### Environment Variable Overrides

Several configuration options can be overridden via environment variables for debugging:

```bash
# Logging (always takes precedence)
RUST_LOG=trace cargo run

# Shutdown behavior (for debugging)
FASTN_SHUTDOWN_TIMEOUT=60s cargo run
FASTN_SHUTDOWN_MODE=single_ctrl_c cargo run
```

### What `#[fastn_p2p::main]` Provides

1. **Automatic Runtime Setup**: Replaces `#[tokio::main]` with additional functionality
2. **Logging Initialization**: Configurable `tracing_subscriber` setup with `RUST_LOG` override
3. **Global Graceful Singleton**: No need to create/pass `Graceful` instances
4. **Signal Handling**: Automatic Ctrl+C handling with configurable behavior
5. **Status Reporting**: Optional status display on first Ctrl+C (double mode)
6. **Automatic Shutdown**: Calls graceful shutdown with configurable timeout

## Server API

### Starting a Server

```rust
async fn start_echo_server(port: u16) -> eyre::Result<()> {
    let listener = fastn_p2p::listen(Protocol::Http, ("0.0.0.0", port), None).await?;
    
    loop {
        tokio::select! {
            // Automatic cancellation support
            _ = fastn_p2p::cancelled() => break,
            
            request = listener.accept() => {
                let request = request?;
                fastn_p2p::spawn(handle_request(request));
            }
        }
    }
    
    Ok(())
}
```

### Handling Requests

```rust
async fn handle_request(request: Request) -> eyre::Result<()> {
    let input: EchoRequest = request.get_input().await?;
    
    let response = EchoResponse {
        message: format!("Echo: {}", input.message),
    };
    
    request.respond(response).await?;
    Ok(())
}
```

## Client API

### Making Type-Safe Calls

```rust
async fn run_client(target: String) -> eyre::Result<()> {
    let request = EchoRequest {
        message: "Hello, World!".to_string(),
    };
    
    // Type-safe P2P call with shared error types
    let response: Result<EchoResponse, EchoError> = 
        fastn_p2p::call(&target, request).await?;
        
    match response {
        Ok(echo) => println!("Received: {}", echo.message),
        Err(e) => eprintln!("Error: {e:?}"),
    }
    
    Ok(())
}
```

## Task Management

### Spawning Background Tasks

```rust
async fn start_background_services() {
    // All spawned tasks are automatically tracked for graceful shutdown
    fastn_p2p::spawn(periodic_cleanup());
    fastn_p2p::spawn(health_check_service());
    fastn_p2p::spawn(metrics_collector());
}

async fn periodic_cleanup() {
    loop {
        tokio::select! {
            _ = fastn_p2p::cancelled() => {
                println!("Cleanup service shutting down gracefully");
                break;
            }
            _ = tokio::time::sleep(Duration::from_secs(60)) => {
                // Perform cleanup
            }
        }
    }
}
```

### Cancellation Support

```rust
async fn long_running_task() {
    loop {
        tokio::select! {
            // Clean cancellation when shutdown is requested
            _ = fastn_p2p::cancelled() => {
                println!("Task cancelled gracefully");
                break;
            }
            
            // Your work here
            result = do_work() => {
                match result {
                    Ok(data) => process_data(data).await,
                    Err(e) => eprintln!("Work failed: {e}"),
                }
            }
        }
    }
}
```

## Migration from Manual Graceful Management

### Before (Tedious)

```rust
#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();
    
    let graceful = fastn_net::Graceful::new();
    
    match cli.command {
        Command::Serve { port } => {
            let graceful_clone = graceful.clone();
            graceful.spawn(async move {
                start_server(port, graceful_clone).await
            });
        }
    }
    
    graceful.shutdown().await
}

async fn start_server(port: u16, graceful: fastn_net::Graceful) -> eyre::Result<()> {
    // Must thread graceful through all functions
    let listener = fastn_p2p::listen(Protocol::Http, ("0.0.0.0", port), None).await?;
    
    loop {
        tokio::select! {
            _ = graceful.cancelled() => break,
            request = listener.accept() => {
                let graceful_clone = graceful.clone();
                graceful.spawn(async move {
                    handle_request(request, graceful_clone).await
                });
            }
        }
    }
    Ok(())
}
```

### After (Clean)

```rust
#[fastn_p2p::main]
async fn main() -> eyre::Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Command::Serve { port } => {
            fastn_p2p::spawn(start_server(port));
        }
    }
}

async fn start_server(port: u16) -> eyre::Result<()> {
    // No graceful parameters needed
    let listener = fastn_p2p::listen(Protocol::Http, ("0.0.0.0", port), None).await?;
    
    loop {
        tokio::select! {
            _ = fastn_p2p::cancelled() => break,
            request = listener.accept() => {
                fastn_p2p::spawn(handle_request(request));
            }
        }
    }
    Ok(())
}
```

## Error Handling

All fastn-p2p operations return `eyre::Result` for consistent error handling:

```rust
#[fastn_p2p::main]
async fn main() -> eyre::Result<()> {
    let result = risky_operation().await?;
    
    // Any error automatically propagates and shuts down gracefully
    Ok(())
}
```

## Examples

See the `/tests` directory for complete working examples:

- `multi_protocol_server.rs`: Multiple protocol listeners with graceful shutdown
- `echo_client_server.rs`: Type-safe request/response communication
- `background_tasks.rs`: Task spawning and cancellation patterns

## Advanced Usage

For advanced use cases that need direct access to `fastn_net::Graceful`, you can still access it through `fastn_p2p::globals`, but this is discouraged for most applications.

The `#[fastn_p2p::main]` approach handles 99% of use cases while providing excellent ergonomics and maintainability.

## Complete API Reference

### Client API

```rust
pub mod client {
    /// Simple request/response communication
    pub async fn call<PROTOCOL, REQUEST, RESPONSE, ERROR>(
        our_key: fastn_id52::SecretKey,
        target: fastn_id52::PublicKey,
        protocol: PROTOCOL,
        request: REQUEST
    ) -> Result<Result<RESPONSE, ERROR>, CallError>;
    
    /// Establish streaming session 
    pub async fn connect<PROTOCOL>(
        our_key: fastn_id52::SecretKey,
        target: fastn_id52::PublicKey,
        protocol: PROTOCOL
    ) -> Result<Session, ConnectionError>;
    
    /// Client-side streaming session
    pub struct Session {
        pub stdin: iroh::endpoint::SendStream,    // To server
        pub stdout: iroh::endpoint::RecvStream,   // From server
    }
    
    impl Session {
        /// Accept unidirectional stream back from server (e.g., stderr)
        pub async fn accept_uni(&mut self) -> Result<iroh::endpoint::RecvStream, ConnectionError>;
        
        /// Accept bidirectional stream back from server
        pub async fn accept_bi(&mut self) -> Result<(iroh::endpoint::RecvStream, iroh::endpoint::SendStream), ConnectionError>;
    }
}
```

### Server API

```rust
pub mod server {
    /// Listen for incoming connections
    pub async fn listen<PROTOCOL>(
        our_key: fastn_id52::SecretKey,
        protocols: &[PROTOCOL]
    ) -> impl futures_core::stream::Stream<Item = Session<PROTOCOL>>;
    
    /// Server-side session (handles both RPC and streaming)
    pub struct Session<PROTOCOL> {
        pub protocol: PROTOCOL,                   // Protocol negotiated
        pub send: iroh::endpoint::SendStream,     // To client (stdout)
        pub recv: iroh::endpoint::RecvStream,     // From client (stdin)
        // private: peer
    }
    
    impl<PROTOCOL> Session<PROTOCOL> {
        /// Get the peer's public key
        pub fn peer(&self) -> &fastn_id52::PublicKey;
        
        /// Convert to Request for RPC handling (consumes Session)
        pub fn into_request(self) -> Request<PROTOCOL>;
        
        /// Open unidirectional stream back to the client (e.g., stderr)
        pub async fn open_uni(&mut self) -> Result<iroh::endpoint::SendStream, ConnectionError>;
        
        /// Open bidirectional stream to back to the client
        pub async fn open_bi(&mut self) -> Result<(iroh::endpoint::SendStream, iroh::endpoint::RecvStream), ConnectionError>;
    }
    
    /// Request handle for RPC-style communication
    pub struct Request<PROTOCOL> {
        pub protocol: PROTOCOL,
        // private: peer, send, recv
    }
    
    impl<PROTOCOL> Request<PROTOCOL> {
        /// Get the peer's public key
        pub fn peer(&self) -> &fastn_id52::PublicKey;
        
        /// Read and deserialize JSON request, get response handle
        pub async fn get_input<INPUT>(self) -> Result<(INPUT, ResponseHandle), GetInputError>;
        
        /// Handle request with async closure (automatic serialization)
        pub async fn handle<INPUT, OUTPUT, ERROR, F, Fut>(
            self, 
            handler: F
        ) -> Result<(), HandleRequestError>;
    }
}
```

### Global Coordination

```rust
/// Spawn task with graceful shutdown tracking
pub fn spawn<F>(task: F) -> tokio::task::JoinHandle<F::Output>;

/// Check for graceful shutdown signal  
pub async fn cancelled();

/// Trigger graceful shutdown
pub async fn shutdown() -> eyre::Result<()>;
```

### Macros

```rust
/// Main function macro with automatic setup
#[fastn_p2p::main]
#[fastn_p2p::main(logging = "debug", shutdown_mode = "double_ctrl_c")]
```

## Use Cases Analysis

This section analyzes various future use cases to validate the API design and identify areas needing additional consideration.

### ✅ **Remote Shell/Command Execution**
**Pattern**: Streaming sessions with optional stderr  
**Implementation**: 
- `client::connect(target, RemoteShellProtocol)` establishes session
- Use `stdin`/`stdout` for main interaction
- Server calls `session.open_uni()` for stderr when needed
- **Status**: Fully supported by current design

### ✅ **Audio/Video/Text Chat**  
**Pattern**: Single connection with multiple stream types  
**Implementation**:
- `client::connect(target, ChatProtocol)` establishes base session
- Use `session.stdin`/`stdout` for text messages
- Server/client use `open_bi()` for audio streams, `open_uni()` for video
- Protocol negotiates capabilities (audio/video support)
- **Status**: Well supported - unlimited side channels enable this

### ✅ **TCP Proxy (Port Forwarding)**
**Pattern**: Bridge local TCP socket to P2P session  
**Implementation**:
- `client::connect(target, TcpProxyProtocol { port: 8080 })`
- Bridge local TCP socket ↔ `session.stdin`/`stdout`
- Server bridges session streams ↔ local TCP socket to target port
- **Status**: Trivial - direct stream bridging

### 🤔 **HTTP Proxy (Multiple Requests per Connection)**
**Pattern**: Reuse connection for multiple HTTP requests  
**Implementation Options**:
1. **New session per request** - High overhead but simple
2. **Session reuse** - `open_bi()` for each HTTP request within same session
3. **Request multiplexing** - Send multiple HTTP requests over same streams

**Considerations Needed**:
- Session reuse patterns in API design
- Request correlation/multiplexing within sessions
- Connection pooling for efficiency

### ✅ **KVM (Remote Desktop/Input)**
**Pattern**: Multiple unidirectional event streams  
**Implementation**:
- `client::connect(target, KvmProtocol)` establishes session
- Server uses `open_uni()` for screen updates (high bandwidth)
- Client uses `open_uni()` for mouse/keyboard events (low latency)
- Multiple streams allow prioritization (input vs display)
- **Status**: Well supported - side channels handle different data types

### 🤔 **File Transfer (SCP-like)**
**Pattern**: Large file transfers with progress  
**Implementation**:
- `client::connect(target, FileTransferProtocol)` 
- Use `stdin`/`stdout` for file data
- Use `open_uni()` for progress updates, metadata
- **Considerations**: Large stream handling, resume capability

## Design Gaps Identified

### 1. **Hierarchical Cancellation** ⚠️
**Current**: Global `fastn_p2p::cancelled()` affects entire application  
**Need**: Tree-based cancellation to shut down specific features
```rust
// Proposed
let cancellation_token = fastn_p2p::CancellationToken::new();
let listener = fastn_p2p::server::listen(key, protocols)
    .with_cancellation(cancellation_token.clone()).await?;

// Later: cancel just this listener
cancellation_token.cancel();
```

### 2. **Hierarchical Status/Metrics** ⚠️
**Current**: Basic connection counting  
**Need**: Tree-based status with timing and nested metrics
```rust
// Proposed
fastn_p2p::status() -> StatusTree
├── Application (45.2s uptime)
├── Remote Access (32.1s active)
│   ├── alice connection (12.3s, 2 streams)
│   └── bob connection (5.7s, 1 stream)
├── HTTP Proxy (45.2s active, 234 requests handled)
```

### 3. **Session Reuse Patterns** ⚠️
**Current**: Each `connect()` creates new session  
**Need**: Patterns for reusing sessions for multiple operations (HTTP-like)
```rust
// May need
session.create_request_channel().await?  // For HTTP-like patterns
```

## Context: Hierarchical Cancellation and Metrics

To address the hierarchical gaps, fastn-p2p will include a `Context` system that provides tree-based cancellation, metrics, and status tracking.

### Context API

```rust
pub struct Context {
    // Identity and timing
    name: String,
    created_at: std::time::Instant,
    
    // Tree structure
    children: std::sync::Arc<std::sync::Mutex<Vec<std::sync::Arc<Context>>>>,
    
    // Functionality
    cancellation: tokio_util::sync::CancellationToken,
    metrics: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, MetricValue>>>,
    data: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, serde_json::Value>>>,
}

impl Context {
    /// Create new child context with given name
    pub fn create_child(&self, name: &str) -> std::sync::Arc<Context>;
    
    /// Wait for cancellation signal
    pub async fn wait(&self);
    
    /// Cancel this context and all children recursively
    pub fn cancel(&self);
    
    /// Spawn task that inherits this context's cancellation
    pub fn spawn<F>(&self, task: F) -> tokio::task::JoinHandle<F::Output>;
    
    /// Add metric data for status reporting
    pub fn add_metric(&self, key: &str, value: MetricValue);
    
    /// Store arbitrary data on this context
    pub fn set_data(&self, key: &str, value: serde_json::Value);
}

/// Global status function - prints context tree
pub fn status() -> StatusTree;
```

### Automatic Context Creation

```rust
// Contexts are auto-created and passed to handlers:

// 1. Global context (created by #[fastn_p2p::main])
// 2. Listener context (created by server::listen())  
// 3. Session context (created per connection)

async fn handle_remote_shell(session: server::Session<RemoteShellProtocol>) {
    let ctx = session.context(); // Auto-created session context
    
    // Spawned tasks inherit session's cancellation automatically
    ctx.spawn(handle_stdout(session.send));
    ctx.spawn(handle_stdin(session.recv));
    
    // When session ends or is cancelled, all spawned tasks stop
}
```

### Status Tree Output

```
$ fastn status
Application Context (45.2s uptime)
├── Remote Access Listener (32.1s active, 2 connections)
│   ├── alice@bv478gen... (12.3s, 2 streams active)
│   │   ├── stdout handler (12.3s)
│   │   └── stderr stream (8.1s)
│   └── bob@p2nd7avq... (5.7s, 1 stream active)
│       └── shell session (5.7s)
├── HTTP Proxy Listener (45.2s active, 234 requests)
│   └── connection pool (15 active connections)
└── Chat Service (12.1s active, 3 users)
    ├── alice chat (8.2s, text + audio)
    └── bob chat (3.1s, text only)
```

**Recommendation**: Start with current Session/Request API, add Context as enhancement layer that threads through the existing design.
