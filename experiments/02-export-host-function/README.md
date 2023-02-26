# `export-host-function`

We have started with `01-wasm-hello-world` and now we are going to export a function from the host to the guest.

Build guest using `cargo build --target wasm32-unknown-unknown` and run `host` using `cargo run` from respective 
folders. The output is:

```txt
$ cargo run
    Finished dev [unoptimized + debuginfo] target(s) in 0.22s
     Running `target/debug/host`
called from wasm
wasm said: 11
```

## Guest Changes

We have added:

```rust
extern "C" {
    fn from_host();
}
```

And then calling it:

```rust
#[no_mangle]
pub extern "C" fn sum(x: i32) -> i32 {
    x + unsafe {
        from_host();
        10
    }
}
```

Seems we have to use `unsafe` to call the host functions.

## Host Changes

We have added:

```rust
fn main() {
    // ...

    let from_host = wasmtime::Func::wrap(&mut store, |_caller: wasmtime::Caller<'_, ()>| {
        println!("called from wasm");
    });
    
    // ...
}
```

As you see we are not yet using any arguments, nor returning anything.
