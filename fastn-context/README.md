# fastn-context: Hierarchical Application Context for Debugging and Operations

This crate provides a hierarchical context system for fastn applications, enabling tree-based cancellation, metrics collection, and operational visibility. It forms the operational backbone for all fastn services.

## Design Philosophy

- **Hierarchical Structure**: Applications naturally form trees of operations
- **Automatic Inheritance**: Child contexts inherit cancellation and settings from parents
- **Zero Boilerplate**: Context trees build themselves as applications run
- **Production Ready**: Status trees enable debugging of stuck/slow operations
- **Bounded Complexity**: Simple spawn vs detailed child creation as needed

## Core Concepts

### Context Tree Structure

Every fastn application forms a natural hierarchy:

```
Global Context (application level)
├── Service Context (e.g., "remote-access-listener")
│   ├── Session Context (e.g., "alice@bv478gen")
│   │   ├── Task Context (e.g., "stdout-handler") 
│   │   └── Task Context (e.g., "stderr-stream")
│   └── Session Context (e.g., "bob@p2nd7avq")
├── Service Context (e.g., "http-proxy")
└── Service Context (e.g., "chat-service")
```

### Automatic Context Creation

fastn-context integrates seamlessly with fastn ecosystem:

```rust
// 1. Global context created by main macro
#[fastn_context::main]
async fn main() -> eyre::Result<()> {
    // Global context automatically available
}

// 2. Service contexts created by operations  
let listener = fastn_p2p::server::listen(key, protocols).await?;
// Creates child context: "p2p-listener" under global

// 3. Session contexts created per connection
// Each incoming connection gets child context: "session-{peer_id}"

// 4. Task contexts created by spawn operations
session_ctx.child("shell-handler").spawn(handle_shell);
```

## API Reference

### Core Context

```rust
pub struct Context {
    /// Context name for debugging/status
    pub name: String,
    
    /// When this context was created
    pub created_at: std::time::Instant,
    
    // Private: parent, children, cancellation, metrics, data
}

impl Context {
    /// Create new root context (typically only used by main macro)
    pub fn new(name: &str) -> std::sync::Arc<Context>;
    
    /// Create child context with given name
    pub fn child(&self, name: &str) -> ContextBuilder;
    
    /// Simple spawn (inherits current context, no child creation)
    pub fn spawn<F>(&self, task: F) -> tokio::task::JoinHandle<F::Output>
    where F: std::future::Future + Send + 'static;
    
    /// Wait for cancellation signal
    pub async fn wait(&self);
    
    /// Cancel this context and all children recursively
    pub fn cancel(&self);
    
    /// Add metric data for status reporting
    pub fn add_metric(&self, key: &str, value: MetricValue);
    
    /// Store arbitrary data on this context
    pub fn set_data(&self, key: &str, value: serde_json::Value);
    
    /// Get stored data
    pub fn get_data(&self, key: &str) -> Option<serde_json::Value>;
}
```

### Context Builder

```rust
pub struct ContextBuilder {
    // Pre-created child context ready for configuration
}

impl ContextBuilder {
    /// Add initial data to context
    pub fn with_data(self, key: &str, value: serde_json::Value) -> Self;
    
    /// Add initial metric to context
    pub fn with_metric(self, key: &str, value: MetricValue) -> Self;
    
    /// Spawn task with this configured child context
    pub fn spawn<F>(self, task: F) -> tokio::task::JoinHandle<F::Output>
    where F: FnOnce(std::sync::Arc<Context>) -> Fut + Send + 'static;
}
```

### Global Access

```rust
/// Get the global application context
pub fn global() -> std::sync::Arc<Context>;
```

### Metric Types

```rust
#[derive(Debug, Clone)]
pub enum MetricValue {
    Counter(u64),
    Gauge(f64), 
    Duration(std::time::Duration),
    Text(String),
    Bytes(u64),
}
```

## Usage Patterns

### Simple Task Spawning

```rust
// Use explicit context (no child creation)
let ctx = fastn_context::global(); // or passed as parameter
let ctx_clone = ctx.clone();
ctx.spawn(async move {
    // Simple background task with explicit context
    ctx_clone.add_metric("task_completed", fastn_context::MetricValue::Counter(1));
});
```

### Detailed Task Spawning  

```rust
// Create child context with debugging info
ctx.child("remote-shell-handler")
    .with_data("peer", alice_id52)
    .with_data("shell", "bash")
    .with_metric("commands_executed", 0)
    .spawn(|task_ctx| async move {
        // Task can update its own context
        task_ctx.add_metric("commands_executed", cmd_count);
        task_ctx.set_data("last_command", "ls -la");
        
        // Task waits for its own cancellation
        tokio::select! {
            _ = task_ctx.wait() => {
                println!("Shell handler cancelled");
            }
            _ = handle_shell_session() => {
                println!("Shell session completed");
            }
        }
    });
```

## Integration with fastn-p2p

fastn-p2p depends on fastn-context and automatically creates context hierarchies:

```rust
// fastn-p2p sessions provide access to their context
async fn handle_remote_shell(session: fastn_p2p::server::Session<RemoteShellProtocol>) {
    let ctx = session.context(); // Auto-created by fastn-p2p
    
    // Simple spawn (inherits session context)
    ctx.spawn(pipe_stdout(session.send));
    
    // Detailed spawn (creates child for debugging)
    ctx.child("command-executor")
        .with_data("command", session.protocol.command)
        .spawn(|task_ctx| async move {
            let result = execute_command(&session.protocol.command).await;
            task_ctx.set_data("exit_code", result.code);
        });
}
```

## Main Function Integration

The main macro sets up the global context and provides comprehensive configuration:

```rust
#[fastn_context::main]
async fn main() -> eyre::Result<()> {
    // Global context automatically created and available
    
    let ctx = fastn_context::global();
    ctx.child("startup")
        .with_data("version", env!("CARGO_PKG_VERSION"))
        .spawn(|_| async {
            // Application initialization
        });
}
```

### Configuration Options

```rust
#[fastn_context::main(
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
}
```

## Design Benefits

1. **Names Required for Debugging** - Every important operation has a name in status tree
2. **Selective Complexity** - Simple spawn vs detailed child creation as needed  
3. **Automatic Tree Building** - Context hierarchy builds as application runs
4. **Production Debugging** - Status trees show exactly where system is stuck
5. **Clean Separation** - Context concerns separate from networking concerns
6. **Ecosystem Wide** - All fastn crates can use the same context infrastructure

**Key Insight**: Names aren't optional - they're essential for production debugging and operational visibility.

## Future Features

See NEXT-*.md files for planned enhancements:
- **NEXT-monitoring.md**: Status trees, timing, system metrics monitoring
- **NEXT-locks.md**: Named locks and deadlock detection
- **NEXT-counters.md**: Global counter storage with dotted paths
- **NEXT-status-distribution.md**: P2P distributed status access