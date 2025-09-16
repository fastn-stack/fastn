# NEXT: Named Locks and Deadlock Detection

Features for named lock monitoring and deadlock detection.

## Named Lock Types

- `ContextMutex<T>` - Named mutex with timing tracking
- `ContextRwLock<T>` - Named read-write lock  
- `ContextSemaphore` - Named semaphore with permit tracking

## Lock Monitoring

- Track who holds what locks and for how long
- Track who's waiting for locks and wait times
- Automatic deadlock risk detection
- Lock status in context tree display

## Usage Pattern

```rust
// Create named locks within context
let user_lock = ctx.mutex("user-data-lock", UserData::new());

// Lock operations automatically tracked
let guard = user_lock.lock().await;
// Shows in status: "HOLDS user-data-lock (2.3s)"

// Waiting operations tracked
// Shows in status: "WAITING user-data-lock (5.1s) ⚠️ STUCK"
```

**Implementation**: After basic Context system + monitoring foundation.