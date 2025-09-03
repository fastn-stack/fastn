# fastn-net-test: Minimal P2P Examples

Minimal working examples demonstrating correct usage of fastn-net for peer-to-peer communication.

## Purpose

These examples serve as:
- **Reference implementations** for correct fastn-net usage
- **Testing tools** to verify P2P networking works
- **Debugging utilities** to isolate networking issues from business logic
- **Documentation** of working patterns

## Examples

### sender.rs - Basic Message Sending

Demonstrates:
- ✅ Proper endpoint creation with `fastn_net::get_endpoint`
- ✅ Correct `get_stream` usage with all required parameters
- ✅ Message serialization and sending
- ✅ Response handling and confirmation

### receiver.rs - Connection and Stream Handling  

Demonstrates:
- ✅ **Non-blocking accept loop** (critical pattern!)
- ✅ Immediate task spawning without I/O in main loop
- ✅ Multiple concurrent connection handling
- ✅ Proper stream acceptance and processing
- ✅ Response sending and stream cleanup

## Running the Examples

### Terminal 1 - Start Receiver
```bash
cd fastn-net-test
cargo run --bin receiver
```

Note the receiver's ID52 from the output.

### Terminal 2 - Send Message
```bash
cargo run --bin sender <receiver_id52>
```

### Multiple Concurrent Messages
```bash
# Send 3 concurrent messages to test parallel handling
cargo run --bin sender <receiver_id52> &
cargo run --bin sender <receiver_id52> &
cargo run --bin sender <receiver_id52> &
```

## Key Findings from Testing

### 1. Accept Loop Must Not Block

**The critical bug** we discovered: 
- ❌ **Blocking I/O in accept loop** prevents concurrent connections
- ✅ **Immediate spawning** allows unlimited concurrent connections

### 2. fastn-net Works Perfectly When Used Correctly

The testing proved that:
- ✅ `get_stream` coordination works correctly
- ✅ Multiple concurrent streams are supported  
- ✅ ACK mechanisms work automatically
- ✅ Request-response patterns work reliably

### 3. Connection Architecture

**Correct pattern**:
- Each sender creates its own connection manager
- Receivers handle multiple connections concurrently
- Each connection can have multiple concurrent streams
- Streams are processed in separate tasks

## Tracing and Debugging

Enable detailed tracing to see the complete flow:
```bash
RUST_LOG="fastn_net=trace,fastn_net_test=info" cargo run --bin receiver
```

This shows:
- Connection establishment details
- Protocol negotiation steps
- Stream lifecycle events  
- ACK mechanisms in action
- Error handling and recovery

## Integration Notes

This minimal implementation can be used as a **reference** for:
- **fastn-rig endpoint handlers** - Apply non-blocking accept pattern
- **fastn-mail P2P delivery** - Use proper get_stream coordination  
- **Any fastn-net integration** - Follow established patterns

The examples prove that **fastn-net is reliable and efficient** when used with the correct architectural patterns.