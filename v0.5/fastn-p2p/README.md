# fastn-p2p: High-Level Type-Safe P2P Communication

This crate provides a high-level, type-safe API for P2P communication in the fastn ecosystem. It builds on top of `fastn-net` but exposes only the essential, locked-down APIs that reduce the possibility of bugs through strong typing and compile-time verification.

## Design Philosophy

- **Type Safety First**: All communication uses strongly-typed REQUEST/RESPONSE/ERROR contracts
- **Minimal Surface Area**: Only essential APIs are exposed to reduce complexity  
- **Bug Prevention**: API design makes common mistakes impossible or unlikely
- **Ergonomic**: High-level APIs handle boilerplate automatically
- **Zero Boilerplate**: Main functions are clean with automatic setup/teardown

## Quick Start

### Application Entry Point

Use `#[fastn_p2p::main]` to eliminate all boilerplate around graceful shutdown, logging, and signal handling:

```rust
#[fastn_p2p::main]
async fn main() -> eyre::Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Command::Serve { port } => {
            fastn_p2p::spawn(start_server(port));
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
