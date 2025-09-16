# NEXT: Comprehensive Monitoring System

Features planned for future implementation after basic Context system is working.

## Status Trees and Monitoring

- Hierarchical status display with timing
- ANSI-formatted status output
- Event-driven status updates
- System metrics integration (CPU, RAM, disk, network)
- P2P status distribution (`fastn status <remote>`)

## Counter Management

- Global counter storage with dotted paths
- Total vs live counter tracking  
- Automatic counter integration
- Hierarchical counter aggregation

## Named Locks

- ContextMutex, ContextRwLock, ContextSemaphore
- Deadlock detection and timing
- Lock status in monitoring tree
- Wait time tracking

## Advanced Features

- Builder pattern: `ctx.child("name").with_data().spawn()`
- Metric types and data storage
- HTTP status endpoints  
- Status streaming API

**Implementation**: After basic Context + fastn-p2p integration is complete.