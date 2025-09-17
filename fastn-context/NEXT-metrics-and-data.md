# NEXT: Metrics and Data Storage

Features for storing metrics and arbitrary data on contexts for debugging and monitoring.

## Metric Storage

```rust
impl Context {
    /// Add metric data for status reporting
    pub fn add_metric(&self, key: &str, value: MetricValue);
}

#[derive(Debug, Clone)]
pub enum MetricValue {
    Counter(u64),
    Gauge(f64), 
    Duration(std::time::Duration),
    Text(String),
    Bytes(u64),
}
```

## Data Storage

```rust
impl Context {
    /// Store arbitrary data on this context
    pub fn set_data(&self, key: &str, value: serde_json::Value);
    
    /// Get stored data
    pub fn get_data(&self, key: &str) -> Option<serde_json::Value>;
}
```

## Builder Pattern Integration

```rust
impl ContextBuilder {
    /// Add initial data to context
    pub fn with_data(self, key: &str, value: serde_json::Value) -> Self;
    
    /// Add initial metric to context
    pub fn with_metric(self, key: &str, value: MetricValue) -> Self;
}
```

## Usage Examples

```rust
// Add metrics during task execution
task_ctx.add_metric("commands_executed", MetricValue::Counter(cmd_count));
task_ctx.add_metric("response_time", MetricValue::Duration(elapsed));

// Store debugging data
task_ctx.set_data("last_command", serde_json::Value::String("ls -la".to_string()));
task_ctx.set_data("exit_code", serde_json::Value::Number(0.into()));

// Pre-configure context with builder
ctx.child("remote-shell-handler")
    .with_data("peer", serde_json::Value::String(alice_id52))
    .with_data("shell", serde_json::Value::String("bash".to_string()))
    .with_metric("commands_executed", MetricValue::Counter(0))
    .spawn(|task_ctx| async move {
        // Task starts with pre-configured data and metrics
    });
```

## Automatic Request Tracking

For contexts that call `.persist()`, automatic time-windowed counters will be maintained:

```rust
// Automatic counters for persisted contexts (no manual tracking needed)
// Uses full dotted context path as key

// When ctx.persist() is called on "global.p2p.alice@bv478gen.stream-123":
// Auto-increments these counters:
"global.p2p.alice@bv478gen.requests_since_start"     // Total ever
"global.p2p.alice@bv478gen.requests_last_day"       // Last 24 hours  
"global.p2p.alice@bv478gen.requests_last_hour"      // Last 60 minutes
"global.p2p.alice@bv478gen.requests_last_minute"    // Last 60 seconds
"global.p2p.alice@bv478gen.requests_last_second"    // Last 1 second

// Hierarchical aggregation automatically available:
"global.p2p.requests_last_hour"                     // All P2P requests
"global.requests_last_hour"                         // All application requests
```

### Time Window Implementation

```rust
// Sliding window counters with efficient circular buffers
// Updated automatically when any context calls persist()

// Status display shows rates:
✅ global.p2p.alice@bv478gen (23m, active)
    Requests: 1,247 total | 234 last hour | 45 last minute | 2/sec current

// Automatic rate calculation and trending
```

### Usage Pattern

```rust
// P2P stream handler
async fn handle_stream(ctx: Arc<Context>) {
    // Process stream...
    ctx.persist(); // Automatically increments all time window counters
    
    // No manual counter management needed!
    // All metrics tracked automatically by dotted context path
}

// HTTP request handler  
async fn handle_request(ctx: Arc<Context>) {
    // Process request...
    ctx.persist(); // Auto-tracks "global.http.endpoint-xyz.requests_*"
}
```

**Implementation**: After basic Context + counter storage foundation.