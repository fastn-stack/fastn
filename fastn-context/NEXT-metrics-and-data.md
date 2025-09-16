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

**Implementation**: After basic Context tree structure is working.