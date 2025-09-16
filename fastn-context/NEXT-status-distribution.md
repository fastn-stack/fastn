# NEXT: P2P Status Distribution

Features for distributed status monitoring over P2P network.

## Remote Status Access

```bash
# Remote machine status over P2P  
fastn status alice           # Status from machine with alias "alice"
fastn status alice -w        # Watch remote machine in real-time
fastn status alice,bob,prod  # Multiple machines
```

## P2P Integration

- Uses secure fastn remote access (same as `fastn rshell`)
- Status transmitted over encrypted P2P connections
- Requires target machine in `remote-access/config.toml`
- Real-time streaming for watch mode

## Protocol

```rust
// Built-in status commands
StatusRequest -> Status          // One-time snapshot
StatusStreamProtocol -> Stream   // Real-time updates
```

## Benefits

- **Distributed monitoring** - Monitor entire fastn network from any machine
- **Secure access** - Uses same permissions as remote shell
- **No HTTP servers** - Uses P2P infrastructure only
- **Real-time** - Event-driven updates across network

**Implementation**: After P2P streaming API + basic status system.