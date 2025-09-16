# fastn-context: Hierarchical Application Context for Debugging and Operations

This crate provides a hierarchical context system for fastn applications, enabling tree-based cancellation, metrics collection, and operational visibility. It forms the operational backbone for all fastn services.

> **Note**: This README documents the complete design iteratively. Some sections may overlap as features build on each other. The design is internally consistent - later sections refine and extend earlier concepts.

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
    
    /// Increment total counter (historical count)
    pub fn increment_total(&self, counter: &str);
    
    /// Increment live counter (current active count)
    pub fn increment_live(&self, counter: &str);
    
    /// Decrement live counter (when operation completes)
    pub fn decrement_live(&self, counter: &str);
    
    /// Get counter values
    pub fn get_total(&self, counter: &str) -> u64;
    pub fn get_live(&self, counter: &str) -> u64;
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

/// Get current task's context (thread-local or task-local)
pub fn current() -> std::sync::Arc<Context>;

/// Print status tree for debugging
pub fn status() -> StatusTree;
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
// Inherit current context (no child creation)
let ctx = fastn_context::current();
ctx.spawn(async {
    // Simple background task
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

### Status Tree Output

```
$ fastn status
Global Context (2h 15m 32s uptime)
├── Remote Access Listener (1h 45m active)
│   ├── alice@bv478gen (23m 12s, bash shell)
│   │   ├── stdout-handler (23m 12s, 15.2MB processed)
│   │   └── stderr-stream (18m 45s, 2.1KB processed)
│   └── bob@p2nd7avq (8m 33s, ls command)
│       └── command-executor (8m 33s, exit pending)
├── HTTP Proxy (2h 15m active)
│   ├── connection-pool (45 active, 1,234 requests)
│   └── request-handler-pool (12 workers active)
└── Chat Service (35m active)
    ├── presence-monitor (35m, 15 users tracked)
    └── message-relay (35m, 4,567 messages)
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

The main macro moves to fastn-context and sets up the global context:

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

## Design Benefits

1. **Names Required for Debugging** - Every important operation has a name in status tree
2. **Selective Complexity** - Simple spawn vs detailed child creation as needed  
3. **Automatic Tree Building** - Context hierarchy builds as application runs
4. **Production Debugging** - `fastn status` shows exactly where system is stuck
5. **Clean Separation** - Context concerns separate from networking concerns
6. **Ecosystem Wide** - All fastn crates can use the same context infrastructure

**Key Insight**: Names aren't optional - they're essential for production debugging and operational visibility.

## Comprehensive Timing and Lock Monitoring

Every context and operation tracks detailed timing for real-time debugging, including named lock monitoring for deadlock detection.

### Timing Integration

```rust
pub struct Context {
    pub name: String,
    pub created_at: std::time::Instant,      // When context started
    pub last_activity: std::sync::Arc<std::sync::Mutex<std::time::Instant>>, // Last activity
    // ... other fields
}

impl Context {
    /// Update last activity timestamp (called automatically by operations)
    pub fn touch(&self);
    
    /// Get how long this context has been alive
    pub fn duration(&self) -> std::time::Duration;
    
    /// Get how long since last activity
    pub fn idle_duration(&self) -> std::time::Duration;
    
    /// Create named mutex within this context
    pub fn mutex<T>(&self, name: &str, data: T) -> ContextMutex<T>;
    
    /// Create named RwLock within this context  
    pub fn rwlock<T>(&self, name: &str, data: T) -> ContextRwLock<T>;
    
    /// Create named semaphore within this context
    pub fn semaphore(&self, name: &str, permits: usize) -> ContextSemaphore;
}
```

### Named Lock Types

```rust
pub struct ContextMutex<T> {
    name: String,
    context: std::sync::Arc<Context>,
    inner: tokio::sync::Mutex<T>,
}

impl<T> ContextMutex<T> {
    /// Lock with automatic status tracking
    pub async fn lock(&self) -> ContextMutexGuard<T>;
}

pub struct ContextMutexGuard<T> {
    acquired_at: std::time::Instant,    // When lock was acquired
    context_name: String,               // Which context holds it
    lock_name: String,                  // Lock identifier
    // Auto-reports to context status system
    // Auto-cleanup on drop
}
```

### Detailed Status Output with Comprehensive Timing

```
$ fastn status
Global Context (2h 15m 32s uptime, active 0.1s ago)
├── Remote Access Listener (1h 45m active, last activity 2.3s ago)
│   ├── alice@bv478gen (23m 12s connected, active 0.5s ago)
│   │   ├── stdout-handler (23m 12s running, CPU active)
│   │   │   └── 🔒 HOLDS "session-output-lock" (12.3s held)
│   │   └── stderr-stream (18m 45s running, idle 8.1s)
│   │       └── ⏳ WAITING "session-output-lock" (8.1s waiting) ⚠️ STUCK
│   └── bob@p2nd7avq (8m 33s connected, active 0.1s ago)
│       └── command-executor (8m 33s running, exit pending)
├── HTTP Proxy (2h 15m active, last request 0.8s ago)
│   ├── connection-pool (2h 15m running, 45 connections, oldest 34m 12s)
│   └── 🔒 HOLDS "pool-resize-lock" (0.2s held)
└── Chat Service (35m active, last message 1.2s ago)
    ├── presence-monitor (35m running, heartbeat 30s ago)
    └── message-relay (35m running, processing queue)

🔒 Active Locks (3):
  - "session-output-lock" held by alice/stdout-handler (12.3s) ⚠️ LONG HELD
  - "user-table-write-lock" held by user-service/db-writer (0.1s)  
  - "pool-resize-lock" held by http-proxy/connection-pool (0.2s)

⏳ Lock Waiters (1):
  - alice/stderr-stream waiting for "session-output-lock" (8.1s) ⚠️ STUCK

⚠️  Potential Issues:
  - Long-held lock "session-output-lock" (12.3s) may indicate deadlock
  - stderr-stream stuck waiting (8.1s) suggests blocked I/O
```

### Automatic Activity Tracking

```rust
// All operations automatically maintain timing
ctx.spawn(async {
    // ctx.touch() called when task starts
    loop {
        do_work().await;
        ctx.touch(); // Update activity timestamp
    }
});

// Lock operations update timing automatically
let guard = ctx.mutex("data-lock", data).lock().await;
// Updates: context last_activity, tracks lock hold time

// Long operations should periodically touch
async fn long_running_task(ctx: std::sync::Arc<Context>) {
    loop {
        process_batch().await;
        ctx.touch(); // Show we're still active, not stuck
        
        tokio::select! {
            _ = ctx.wait() => break, // Cancelled
            _ = tokio::time::sleep(std::time::Duration::from_secs(10)) => {}
        }
    }
}
```

This provides **real-time operational debugging** - administrators can instantly identify stuck operations, deadlocked tasks, and performance bottlenecks with precise timing information.

## Counter Management System

Every context can track both historical totals and live counts for detailed operational metrics.

### Global Counter Storage with Dotted Paths

```rust
pub struct Context {
    pub name: String,
    pub full_path: String,  // "global.remote-access.alice@bv478gen.stdout-handler"
    // ... other fields
}

impl Context {
    /// Get full dotted path for this context
    pub fn path(&self) -> &str;
    
    /// Increment total counter (stored in global hashmap by full path)
    pub fn increment_total(&self, counter: &str);
    
    /// Increment live counter (stored in global hashmap by full path)
    pub fn increment_live(&self, counter: &str);
    
    /// Decrement live counter (stored in global hashmap by full path)
    pub fn decrement_live(&self, counter: &str);
    
    /// Get counter values (retrieved from global storage)
    pub fn get_total(&self, counter: &str) -> u64;
    pub fn get_live(&self, counter: &str) -> u64;
}

// Global counter storage (persists beyond context lifetimes)
static GLOBAL_COUNTERS: LazyLock<RwLock<HashMap<String, u64>>> = ...;

// Counter keys format: "{context_path}.{counter_name}"
// Examples:
// "global.connections" -> 1,247
// "global.remote-access.connections" -> 234  
// "global.remote-access.alice@bv478gen.commands" -> 45
// "global.http-proxy.requests" -> 1,013
```

### Automatic Counter Integration

```rust
// fastn-p2p automatically maintains connection counters
async fn handle_incoming_connection(session: fastn_p2p::server::Session<Protocol>) {
    let ctx = session.context();
    
    // Automatically tracked by fastn-p2p:
    ctx.increment_total("connections");     // Total connections ever
    ctx.increment_live("connections");      // Current active connections
    
    // Your handler code...
    
    // When session ends:
    ctx.decrement_live("connections");      // Automatically called
}

// Custom counters for application logic
async fn handle_remote_command(session: server::Session<RemoteShellProtocol>) {
    let ctx = session.context();
    
    ctx.increment_total("commands");        // Total commands executed
    ctx.increment_live("commands");         // Currently executing commands
    
    let result = execute_command(&session.protocol.command).await;
    
    ctx.decrement_live("commands");         // Command completed
    
    if result.success {
        ctx.increment_total("successful_commands");
    } else {
        ctx.increment_total("failed_commands");
    }
}
```

### Enhanced Status Display with Counters

```
$ fastn status
fastn Status Dashboard
System: CPU 12.3% | RAM 2.1GB/16GB (13%) | Disk 45GB/500GB (9%) | Load 0.8,1.2,1.5
Network: ↓ 125KB/s ↑ 67KB/s | Uptime 5d 12h 45m

Global Context (2h 15m 32s uptime, active 0.1s ago)
├── Total: 1,247 connections, 15,432 requests | Live: 47 connections, 12 active requests
├── Remote Access Listener (1h 45m active, last activity 2.3s ago)
│   ├── Total: 234 connections, 2,156 commands | Live: 2 connections, 3 commands
│   ├── alice@bv478gen (23m 12s connected, active 0.5s ago)
│   │   ├── Total: 45 commands (42 success, 3 failed) | Live: 1 command
│   │   ├── stdout-handler (23m 12s running, CPU active)
│   │   │   └── 🔒 HOLDS "session-output-lock" (12.3s held)
│   │   └── stderr-stream (18m 45s running, idle 8.1s)
│   │       └── ⏳ WAITING "session-output-lock" (8.1s waiting) ⚠️ STUCK
│   └── bob@p2nd7avq (8m 33s connected, active 0.1s ago)
│       ├── Total: 12 commands (12 success) | Live: 1 command
│       └── command-executor (8m 33s running, exit pending)
├── HTTP Proxy (2h 15m active, last request 0.8s ago)
│   ├── Total: 1,013 requests (987 success, 26 failed) | Live: 45 connections, 8 requests
│   ├── connection-pool (2h 15m running, 45 connections, oldest 34m 12s)
│   └── 🔒 HOLDS "pool-resize-lock" (0.2s held)
└── Chat Service (35m active, last message 1.2s ago)
    ├── Total: 4,567 messages, 89 users joined | Live: 15 users, 3 conversations
    ├── presence-monitor (35m running, heartbeat 30s ago)
    └── message-relay (35m running, processing queue)

🔒 Active Locks (3): ...
⏳ Lock Waiters (1): ...
```

### Counter Storage and Paths

```rust
// Counter keys are automatically generated from context paths:

// Global level counters
// "global.connections" -> 1,247 total
// "global.live_connections" -> 47 current

// Service level counters  
// "global.remote-access.connections" -> 234 total
// "global.remote-access.live_connections" -> 2 current

// Session level counters
// "global.remote-access.alice@bv478gen.commands" -> 45 total
// "global.remote-access.alice@bv478gen.live_commands" -> 1 current

// Task level counters
// "global.remote-access.alice@bv478gen.stdout-handler.bytes_processed" -> 1,234,567

// Examples in code:
async fn handle_connection(session: server::Session<Protocol>) {
    let ctx = session.context(); // Path: "global.remote-access.alice@bv478gen"
    
    // These create global entries:
    ctx.increment_total("commands");        // Key: "global.remote-access.alice@bv478gen.commands"
    ctx.increment_live("commands");         // Key: "global.remote-access.alice@bv478gen.live_commands"
    
    // Nested task context
    ctx.child("stdout-handler").spawn(|task_ctx| async move {
        // task_ctx.path() -> "global.remote-access.alice@bv478gen.stdout-handler"
        task_ctx.increment_total("bytes_processed");
    });
}
```

### Persistent Counter Benefits

- **✅ Survives context drops** - Counters stored globally, persist after contexts end
- **✅ Hierarchical aggregation** - Can sum all child counters for parent totals
- **✅ Path-based queries** - Easy to find counters by context path
- **✅ Historical tracking** - Total counters accumulate across all context instances
- **✅ Live tracking** - Live counters automatically decremented when contexts drop

**Live counters** show current activity (auto-decremented on context drop).  
**Total counters** show historical activity (persist forever for trending).  
**Global storage** ensures metrics survive context lifecycles.

## Status Monitoring and HTTP Dashboard

fastn-context automatically provides multiple ways to access real-time status information for debugging and monitoring.

### P2P Status Access

```rust
#[fastn_context::main]
async fn main() -> eyre::Result<()> {
    // Status automatically available over P2P for remote access
    // No HTTP server needed - uses secure P2P connections
    
    // Your application code...
}
```

Status is accessible over the P2P network using the remote access system.

### Status API Functions

```rust
/// Get current status snapshot with ANSI formatting
pub fn status() -> Status;

/// Stream of status updates (max once per second)
pub fn status_stream() -> impl futures_core::stream::Stream<Item = Status>;

/// Get raw status data as structured JSON
pub fn status_json() -> serde_json::Value;
```

### Status Type with ANSI Display

```rust
#[derive(Debug, Clone, serde::Serialize)]
pub struct Status {
    pub global_context: ContextStatus,
    pub active_locks: Vec<LockStatus>,
    pub lock_waiters: Vec<LockWaiter>,
    pub warnings: Vec<StatusWarning>,
    pub timestamp: std::time::SystemTime,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ContextStatus {
    pub name: String,
    pub duration: std::time::Duration,
    pub last_activity: std::time::Duration, // Time since last activity
    pub children: Vec<ContextStatus>,
    pub metrics: std::collections::HashMap<String, MetricValue>,
    pub data: std::collections::HashMap<String, serde_json::Value>,
    pub total_counters: std::collections::HashMap<String, u64>,  // Historical counts
    pub live_counters: std::collections::HashMap<String, u64>,   // Current active counts
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct LockStatus {
    pub name: String,
    pub held_by_context: String,
    pub held_duration: std::time::Duration,
    pub lock_type: LockType, // Mutex, RwLock, Semaphore
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct StatusWarning {
    pub message: String,
    pub context_path: String,
    pub severity: WarningSeverity,
}

#[derive(Debug, Clone, serde::Serialize)]
pub enum WarningSeverity {
    Info,    // FYI information
    Warning, // Potential issue  
    Critical, // Likely problem
}
```

### ANSI-Formatted Display

```rust
impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use colored::*; // For ANSI colors
        
        // Header with timestamp
        writeln!(f, "{}", "fastn Status Dashboard".bold().blue())?;
        writeln!(f, "{}", format!("Snapshot: {}", 
            humantime::format_rfc3339(self.timestamp)).dimmed())?;
        writeln!(f)?;
        
        // Context tree with colors and timing
        self.display_context_tree(f, &self.global_context, 0)?;
        
        // Active locks section
        if !self.active_locks.is_empty() {
            writeln!(f, "\n{} Active Locks ({}):", "🔒".yellow(), self.active_locks.len())?;
            for lock in &self.active_locks {
                let duration_str = humantime::format_duration(lock.held_duration);
                let color = if lock.held_duration.as_secs() > 10 { 
                    "red" 
                } else { 
                    "white" 
                };
                writeln!(f, "  - \"{}\" held by {} ({})", 
                    lock.name.cyan(),
                    lock.held_by_context.white(),
                    duration_str.color(color))?;
            }
        }
        
        // Lock waiters section
        if !self.lock_waiters.is_empty() {
            writeln!(f, "\n{} Lock Waiters ({}):", "⏳".yellow(), self.lock_waiters.len())?;
            for waiter in &self.lock_waiters {
                let duration_str = humantime::format_duration(waiter.waiting_duration);
                writeln!(f, "  - {} waiting for \"{}\" ({})", 
                    waiter.context_name.white(),
                    waiter.lock_name.cyan(),
                    duration_str.red())?;
            }
        }
        
        // Warnings section
        if !self.warnings.is_empty() {
            writeln!(f, "\n{} Warnings:", "⚠️".red())?;
            for warning in &self.warnings {
                let icon = match warning.severity {
                    WarningSeverity::Info => "ℹ️",
                    WarningSeverity::Warning => "⚠️", 
                    WarningSeverity::Critical => "🚨",
                };
                writeln!(f, "  {} {}", icon, warning.message.yellow())?;
            }
        }
        
        Ok(())
    }
}
```

### Status Stream (Event-Driven Updates)

```rust
/// Stream provides updates only when context tree actually changes
/// No polling - efficient for long-running monitoring
let mut status_stream = fastn_context::status_stream();
while let Some(status) = status_stream.next().await {
    // Only prints when something actually changes
    print!("\x1B[2J\x1B[H"); // Clear screen
    println!("{}", status);   // Display with colors
}
```

### CLI Integration with P2P Status Access

fastn-context integrates with the main fastn CLI to provide both local and remote status access:

```bash
# Local machine status
fastn status                 # One-time snapshot with ANSI colors
fastn status -w              # Watch mode (event-driven, no polling)
fastn status --json          # JSON output for programmatic use

# Remote machine status over P2P (requires remote access)
fastn status alice           # Status from machine with alias "alice"  
fastn status bv478gen...     # Status from machine with ID52
fastn status alice -w        # Watch remote machine's status in real-time
fastn status alice --json    # Remote machine status as JSON

# Multiple machines
fastn status alice,bob,prod  # Status from multiple machines
```

**P2P Status Protocol:**
- Uses secure fastn remote access (same as `fastn rshell`)
- Requires target machine in your `remote-access/config.toml` 
- Status data transmitted over encrypted P2P connection
- Real-time streaming for remote watch mode

### Status Protocol Integration

Status access integrates seamlessly with fastn's remote access system:

```rust
// Status is available as a built-in remote command
// When fastn-daemon receives status requests, fastn-context provides the data

// Server side - automatic status command handling
// fastn-daemon automatically handles:
// - StatusRequest -> returns current Status
// - StatusStreamRequest -> returns real-time Status stream

// Client side - transparent remote access
fastn status alice    // Translates to fastn_p2p::client::call(alice, StatusRequest)
fastn status alice -w // Translates to fastn_p2p::client::connect(alice, StatusStreamProtocol)
```

This gives **comprehensive status access** - terminal, HTTP, streaming, and programmatic - all from the same underlying Status structure with rich ANSI formatting for human consumption.

## System Metrics Monitoring

fastn-context automatically monitors system resources and integrates them into the status display.

### Automatic System Monitoring

```rust
#[derive(Debug, Clone, serde::Serialize)]
pub struct SystemMetrics {
    pub cpu_usage_percent: f32,         // Current CPU usage
    pub memory_used_bytes: u64,         // RAM usage
    pub memory_total_bytes: u64,        // Total RAM
    pub disk_used_bytes: u64,           // Disk usage
    pub disk_total_bytes: u64,          // Total disk
    pub network_rx_bytes_per_sec: u64,  // Network receive rate
    pub network_tx_bytes_per_sec: u64,  // Network transmit rate
    pub load_average: [f32; 3],         // 1min, 5min, 15min load
    pub uptime: std::time::Duration,    // System uptime
}

// Added to Status structure
pub struct Status {
    pub system_metrics: SystemMetrics,  // System resource usage
    pub global_context: ContextStatus,
    pub active_locks: Vec<LockStatus>,
    pub lock_waiters: Vec<LockWaiter>,
    pub warnings: Vec<StatusWarning>,
    pub timestamp: std::time::SystemTime,
}
```

### Efficient Metric Collection

```rust
// System metrics cached and updated appropriately:
// - CPU usage: Updated every 1 second (smooth average)
// - Memory/disk: Updated every 5 seconds (less volatile)  
// - Network rates: Updated every 1 second (calculated from deltas)
// - Load average: Updated every 10 seconds (system provides this)

// Metrics only recalculated when status is actually requested
// No background polling unless someone is watching
```

### Enhanced Status Display with System Info

```
$ fastn status
fastn Status Dashboard
System: CPU 12.3% | RAM 2.1GB/16GB (13%) | Disk 45GB/500GB (9%) | Load 0.8,1.2,1.5
Network: ↓ 125KB/s ↑ 67KB/s | Uptime 5d 12h 45m

Global Context (2h 15m 32s uptime, active 0.1s ago)
├── Remote Access Listener (1h 45m active, last activity 2.3s ago)
│   ├── alice@bv478gen (23m 12s connected, active 0.5s ago)
│   │   ├── stdout-handler (23m 12s running, CPU active)
│   │   │   └── 🔒 HOLDS "session-output-lock" (12.3s held)
│   │   └── stderr-stream (18m 45s running, idle 8.1s)
│   │       └── ⏳ WAITING "session-output-lock" (8.1s waiting) ⚠️ STUCK
│   └── bob@p2nd7avq (8m 33s connected, active 0.1s ago)
│       └── command-executor (8m 33s running, exit pending)
├── HTTP Proxy (2h 15m active, last request 0.8s ago)
│   ├── connection-pool (2h 15m running, 45 connections, oldest 34m 12s)
│   └── 🔒 HOLDS "pool-resize-lock" (0.2s held)
└── Chat Service (35m active, last message 1.2s ago)
    ├── presence-monitor (35m running, heartbeat 30s ago)
    └── message-relay (35m running, processing queue)

🔒 Active Locks (3):
  - "session-output-lock" held by alice/stdout-handler (12.3s) ⚠️ LONG HELD
  - "user-table-write-lock" held by user-service/db-writer (0.1s)  
  - "pool-resize-lock" held by http-proxy/connection-pool (0.2s)

⏳ Lock Waiters (1):
  - alice/stderr-stream waiting for "session-output-lock" (8.1s) ⚠️ STUCK

⚠️  Alerts:
  - Long-held lock "session-output-lock" (12.3s) may indicate deadlock
  - stderr-stream stuck waiting (8.1s) suggests blocked I/O
  - CPU usage normal (12.3%), memory usage low (13%)
```

### Watch Mode (`fastn status -w`)

```rust
// Event-driven updates - only when something changes
// No CPU overhead when system is idle
// Immediately shows when new contexts/locks appear or disappear

$ fastn status -w
# Screen updates only when:
# - New context created/destroyed
# - Lock acquired/released  
# - Significant activity changes
# - System metrics cross thresholds
# - No updates for days if system is stable
```

This provides **complete operational visibility** with both application-specific context trees and system resource monitoring, all with efficient event-driven updates instead of wasteful polling.