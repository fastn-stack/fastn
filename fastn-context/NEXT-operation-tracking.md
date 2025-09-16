# NEXT: Operation Tracking for Precise Debugging

Features for tracking exactly where tasks are stuck using named await and select operations.

## Named Await Operations

```rust
/// Track what operation a context is waiting for
let result = fastn_context::await!("waiting-for-response", some_operation());
// No .await suffix - the macro handles it

// Examples
let data = fastn_context::await!("reading-file", std::fs::read("config.toml"));
let response = fastn_context::await!("http-request", client.get(url).send());
let connection = fastn_context::await!("database-connect", db.connect());
```

## Simple Select Tracking

```rust
/// Track that context is stuck on select (no branch naming needed)
fastn_context::select! {
    _ = task_ctx.wait() => println!("Cancelled"),
    _ = stream.read() => println!("Data received"),
    _ = database.query() => println!("Query complete"),
}
// Records: "stuck on select"
```

## Status Display with Operation Names

```
Global Context (2h 15m uptime)
├── Remote Access Listener  
│   ├── alice@bv478gen (23m connected)
│   │   ├── stdout-handler (stuck on: "reading-stream" 12.3s) ⚠️ STUCK
│   │   └── stderr-stream (stuck on: "select" 8.1s)
│   └── bob@p2nd7avq (stuck on: "database-query" 0.2s) ✅ ACTIVE
└── HTTP Proxy (stuck on: "select" 0.1s) ✅ ACTIVE
```

## Design Principles

### Single Operation per Context
- **Good design**: One await/select per context encourages proper task breakdown
- **Multiple selects**: Suggests need for child contexts instead

```rust
// ❌ Complex - hard to debug where it's stuck
fastn_context::select! { /* 5 different operations */ }
fastn_context::select! { /* 3 more operations */ }

// ✅ Clear - each operation has its own context
ctx.spawn_child("network-handler", |ctx| async move {
    fastn_context::select! { /* network operations */ }
});
ctx.spawn_child("database-handler", |ctx| async move {
    let result = fastn_context::await!("user-query", db.get_user(id));
});
```

### Automatic Operation Tracking

```rust
// Context automatically tracks current operation
pub struct Context {
    current_operation: std::sync::Arc<std::sync::Mutex<Option<String>>>,
    operation_started: std::sync::Arc<std::sync::Mutex<Option<std::time::Instant>>>,
}

// Status can show:
// - What operation is running
// - How long it's been running  
// - If it's stuck (running too long)
```

## Benefits

1. **Precise debugging** - Know exactly where each task is stuck
2. **Performance insights** - See which operations take too long
3. **Design enforcement** - Encourages proper context decomposition
4. **Production monitoring** - Real-time operation visibility

**Implementation**: After basic Context + monitoring system is complete.