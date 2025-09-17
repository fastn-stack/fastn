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
    
    // Private: parent, children, cancellation_token
}

impl Context {
    /// Create new root context (typically only used by main macro)
    pub fn new(name: &str) -> std::sync::Arc<Context>;
    
    /// Create child context with given name
    pub fn child(&self, name: &str) -> ContextBuilder;
    
    /// Simple spawn (inherits current context, no child creation)
    pub fn spawn<F>(&self, task: F) -> tokio::task::JoinHandle<F::Output>
    where F: std::future::Future + Send + 'static;
    
    /// Spawn task with named child context (common case shortcut)
    pub fn spawn_child<F, Fut>(&self, name: &str, task: F) -> tokio::task::JoinHandle<Fut::Output>
    where 
        F: FnOnce(std::sync::Arc<Context>) -> Fut + Send + 'static,
        Fut: std::future::Future + Send + 'static;
    
    /// Wait for cancellation signal
    pub async fn wait(&self);
    
    /// Cancel this context and all children recursively
    pub fn cancel(&self);
    
    /// Mark this context for persistence (distributed tracing)
    pub fn persist(&self);
    
    /// Set completion status and persist (common pattern)
    pub fn complete_with_status(&self, success: bool, message: &str);
}
```

### Context Builder

```rust
pub struct ContextBuilder {
    // Pre-created child context ready for spawning
}

impl ContextBuilder {
    /// Spawn task with this child context
    pub fn spawn<F>(self, task: F) -> tokio::task::JoinHandle<F::Output>
    where F: FnOnce(std::sync::Arc<Context>) -> Fut + Send + 'static;
}
```

#### Global Access

```rust
/// Get the global application context
pub fn global() -> std::sync::Arc<Context>;
```

### Status Display

```rust
/// Get current status snapshot of entire context tree
pub fn status() -> Status;

/// Get status including recent completed contexts (distributed tracing)
pub fn status_with_latest() -> Status;

#[derive(Debug, Clone)]
pub struct Status {
    pub global_context: ContextStatus,
    pub persisted_contexts: Option<Vec<PersistedContext>>, // Recent completed contexts
    pub timestamp: std::time::SystemTime,
}

#[derive(Debug, Clone)]
pub struct ContextStatus {
    pub name: String,
    pub is_cancelled: bool,
    pub duration: std::time::Duration,
    pub children: Vec<ContextStatus>,
}

#[derive(Debug, Clone)]
pub struct PersistedContext {
    pub name: String,
    pub context_path: String,
    pub duration: std::time::Duration,
    pub completion_time: std::time::SystemTime,
    pub success: bool,
    pub message: String,
}

impl std::fmt::Display for Status {
    // ANSI-formatted display with tree structure and status icons
}
```


## Usage Patterns

### Simple Task Spawning

```rust
// Simple named task spawning (common case)
let ctx = fastn_context::global(); // or passed as parameter
ctx.spawn_child("background-task", |task_ctx| async move {
    // Simple background task with explicit context
    println!("Running in context: {}", task_ctx.name);
});

// Alternative: builder pattern for simple case
ctx.child("background-task")
    .spawn(|task_ctx| async move {
        // Same result, different syntax
        println!("Running in context: {}", task_ctx.name);
    });
```

### Status Monitoring

```rust
// Get current context tree status
let status = fastn_context::status();
println!("{}", status);

// Example output:
// fastn Context Status
// ✅ global (2h 15m, active)
//   ✅ remote-access-listener (1h 45m, active)
//     ✅ alice@bv478gen (23m, active)
//       ✅ stdout-handler (23m, active)
//   ✅ startup-task (2h 15m, active)

// With recent completed contexts:
let status = fastn_context::status_with_latest();
// Shows live contexts + recent completed ones
```

### Context Persistence (Distributed Tracing)

```rust
// P2P stream handler example
async fn handle_p2p_stream(session: Session<Protocol>) {
    let ctx = session.context(); // "global.p2p.alice@bv478gen.stream-456"
    
    let result = process_stream().await;
    
    // Persist completed context for tracing
    ctx.complete_with_status(result.is_ok(), &format!("Processed {} bytes", result.bytes));
    // -> Logs trace, adds to circular buffer, sends to external systems
}

// HTTP request handler example  
async fn handle_http_request(request: HttpRequest) {
    let ctx = request.context(); // "global.http.request-789"
    
    let result = process_request().await;
    
    // Persist with completion info
    ctx.complete_with_status(result.status.is_success(), &result.summary);
}
```

### Enhanced Status Display

```
$ fastn status --include-latest
✅ global (2h 15m, active)
  ✅ p2p-listener (1h 45m, active)
    ✅ alice@bv478gen (23m, active, 3 live streams)

Recent completed contexts (last 10):
- global.p2p.alice@bv478gen.stream-455 (2.3s, success: "Processed 1.2MB")
- global.p2p.bob@p2nd7avq.stream-454 (1.1s, success: "Processed 512KB") 
- global.http.request-123 (0.8s, failed: "Database timeout")
- global.p2p.alice@bv478gen.stream-453 (4.1s, success: "Processed 2.1MB")
```

### Cancellation Handling

```rust
// Task waits for context cancellation
ctx.spawn_child("shell-handler", |task_ctx| async move {
    println!("Shell handler starting: {}", task_ctx.name);
    
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
    
    // Named child spawn for debugging
    ctx.spawn_child("command-executor", |task_ctx| async move {
        println!("Executing command in context: {}", task_ctx.name);
        let result = execute_command(&session.protocol.command).await;
        println!("Command completed with: {:?}", result);
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
    ctx.spawn_child("startup", |startup_ctx| async move {
        println!("Application starting: {}", startup_ctx.name);
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

- **NEXT-metrics-and-data.md**: Metric storage and arbitrary data on contexts
- **NEXT-monitoring.md**: Status trees, timing, system metrics monitoring
- **NEXT-locks.md**: Named locks and deadlock detection
- **NEXT-counters.md**: Global counter storage with dotted paths
- **NEXT-status-distribution.md**: P2P distributed status access
