# fastn-context: Hierarchical Application Context (Minimal Implementation)

This crate provides basic hierarchical context system for fastn applications, enabling tree-based cancellation and task management. This is the minimal implementation needed for fastn-p2p integration.

## Design Philosophy

- **Hierarchical Structure**: Applications form trees of operations
- **Automatic Inheritance**: Child contexts inherit cancellation from parents
- **Zero Boilerplate**: Context trees build themselves as applications run
- **Minimal Surface**: Only essential features for P2P integration

## Current API (Minimal)

### Core Context

```rust
pub struct Context {
    pub name: String,
    // private: parent, children, cancellation_token
}

impl Context {
    /// Create new root context (used by main macro)
    pub fn new(name: &str) -> std::sync::Arc<Context>;
    
    /// Create child context
    pub fn child(&self, name: &str) -> std::sync::Arc<Context>;
    
    /// Spawn task with context inheritance
    pub fn spawn<F>(&self, task: F) -> tokio::task::JoinHandle<F::Output>;
    
    /// Wait for cancellation signal
    pub async fn wait(&self);
    
    /// Cancel this context and all children
    pub fn cancel(&self);
}
```

### Global Access

```rust
/// Get the global application context
pub fn global() -> std::sync::Arc<Context>;

/// Get current task's context
pub fn current() -> std::sync::Arc<Context>;
```

### Main Function Macro

```rust
/// Main function with automatic context setup
#[fastn_context::main]
async fn main() -> eyre::Result<()> {
    // Global context automatically created and available
    let ctx = fastn_context::global();
    
    ctx.spawn(async {
        // Task inherits global context cancellation
    });
}
```

## Usage for fastn-p2p Integration

```rust
// fastn-p2p sessions will provide context access
async fn handle_remote_shell(session: fastn_p2p::server::Session<RemoteShellProtocol>) {
    let ctx = session.context(); // Context provided by fastn-p2p
    
    // Spawn tasks with inherited cancellation
    ctx.spawn(handle_stdout(session.send));
    ctx.spawn(handle_stdin(session.recv));
    
    // Tasks automatically cancelled when session context cancelled
}
```

## Scope

This minimal implementation provides:
- ✅ Basic context tree structure
- ✅ Hierarchical cancellation 
- ✅ Task spawning with inheritance
- ✅ Integration points for fastn-p2p

**Not included** (see NEXT-*.md files):
- Detailed metrics and counters
- Named locks and deadlock detection
- System monitoring
- Status trees and HTTP dashboard
- Advanced timing and monitoring