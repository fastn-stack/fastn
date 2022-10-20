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

## Wasm File Size

Experimented with https://github.com/WebAssembly/binaryen/releases/tag/version_110. The binary size is 1.7M, and
with the `wasm-opt` it became 1.4M. Not a great reduction.

`~/Downloads/binaryen-version_110/bin/wasm-opt -Oz -o output.wasm target/wasm32-unknown-unknown/debug/guest.wasm`

## Wee Allocator

Wasm Book says use https://github.com/rustwasm/wee_alloc, but using it increased the size by 100K. Not sure if I did
everything correctly or not tho.
