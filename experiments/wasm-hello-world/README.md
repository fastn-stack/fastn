# `wasm-hello-world`

In this folder you will find a `guest` and `host` crates. 

Note: we do not use wasi in this, we are planning to do manual memory management.

## `guest`

Install wasm32 target: `rustup target add wasm32-unknown-unknown`.

Build `guest` using: `cargo build --target wasm32-unknown-unknown`. This creates a wasm file
`target/wasm32-unknown-unknown/debug/guest.wasm`, which exports a function `sum`.

```rust
#[no_mangle]
pub extern "C" fn sum(x: i32, y: i32) -> i32 {
    x + y
}
```

## `host`

Run `cargo run` to execute the host, which runs the `guest` wasm file.
