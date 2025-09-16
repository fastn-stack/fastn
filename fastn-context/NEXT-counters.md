# NEXT: Global Counter Storage System

Features for persistent counter tracking with dotted path keys.

## Counter Types

- **Total counters** - Historical cumulative counts (persist after context drops)
- **Live counters** - Current active operations (auto-decremented on drop)

## Global Storage

```rust
// Dotted path keys in global HashMap
"global.connections" -> 1,247
"global.remote-access.alice@bv478gen.commands" -> 45
"global.http-proxy.requests" -> 1,013
```

## API

```rust
impl Context {
    pub fn increment_total(&self, counter: &str);
    pub fn increment_live(&self, counter: &str);
    pub fn decrement_live(&self, counter: &str);
    pub fn get_total(&self, counter: &str) -> u64;
    pub fn get_live(&self, counter: &str) -> u64;
}
```

## Integration

- fastn-p2p automatically tracks connection/request counters
- Counters survive context drops via global storage
- Hierarchical aggregation possible via path prefix matching

**Implementation**: After basic Context + monitoring foundation.