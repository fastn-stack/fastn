# `arguments-and-return`

We have started with `02-export-host-function` and now we are adding arguments and return values.

Build guest using `cargo build --target wasm32-unknown-unknown` and run `host` using `cargo run` from respective 
folders. The output is:

```txt
$ cargo run
    Finished dev [unoptimized + debuginfo] target(s) in 0.46s
     Running `target/debug/host`
wasm sent: 10
wasm said: 21
```

## Guest Changes

We have updated the signature:

```rust
extern "C" {
    fn from_host(x: i32) -> i32;
}
```

And the call site:

```rust
#[no_mangle]
pub extern "C" fn sum(x: i32) -> i32 {
    x + unsafe { from_host(10) }
}
```

## Host Changes

We have changed:

```rust
fn main() {
    // ...

    let from_host = wasmtime::Func::wrap(&mut store, |_caller: wasmtime::Caller<'_, ()>| {
        println!("called from wasm");
    });
    
    // ...
}
```

to:

```rust
fn main() {
    // ...

    let from_host = wasmtime::Func::wrap(&mut store, |a: i32 | {
        println!("wasm sent: {}", a);
        a + 10
    });
    
    // ...
}
```

God knows what the `_caller` was! It's pretty straight forward.
