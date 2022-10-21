# `arguments-and-return`

We have started with `03-arguments-and-return` and now we are trying to export an async function to the guest. 

From the point of view of wasm the host function would be sync, the wasm would call it, and then it would block until the 
host function returns. But host function would be async and would be able to do some async work. Read more about [async
in wasmtime docs](https://docs.wasmtime.dev/api/wasmtime/struct.Config.html#asynchronous-wasm).

Build guest using `cargo build --target wasm32-unknown-unknown` and run `host` using `cargo run` from respective 
folders. The output is:

```txt
$ cargo run
  Compiling host v0.1.0 (/Users/amitu/Projects/fpm/experiments/04-async/host)
    Finished dev [unoptimized + debuginfo] target(s) in 2.16s
     Running `target/debug/host`
called from wasm: [I32(10), I32(1)]
wasm said: 532
```

While working on this experiment, I found [wasmtime's async_functions tests](https://github.com/bytecodealliance/wasmtime/blob/ff0e84ecf449cf89d970457fd6206e1c06429980/tests/all/async_functions.rs)
to be quite helpful.

## Guest Changes

There is no significant change in the guest code. WASM guest code has no idea if a host function is sync or async. 
It just blocks until the host function returns.

## Host Changes

### Activate `async` feature

In `Cargo.toml` we have added `async` feature to `wasmtime` dependency:

```toml
[dependencies]
wasmtime = { version = "1", features = ["async"] }
anyhow = "1"
tokio = { version ="1", features = ["rt-multi-thread", "macros", "fs"] }
```

We have also added `tokio` dependency, which is used to run async code in the host.

### Use `tokio::main` macro

```rust
#[tokio::main]
fn main() {
    // ...
}
```

### Engine supports async

Now we are using:

```rust
async fn main() {
    let engine = wasmtime::Engine::new(&wasmtime::Config::new().async_support(true))?;
    // ...
}
```  

Earlier we used `wasmtime::Engine::default()` which does not support async.

## Async host function

We are using `wasmtime::Func::new_async` to create an async host function. 

```rust
async fn main() {
    // ...
    let from_host = wasmtime::Func::new_async(
        &mut store,
        wasmtime::FuncType::new(
            vec![wasmtime::ValType::I32, wasmtime::ValType::I32],
            Some(wasmtime::ValType::I32),
        ),
        move |_caller, params, results| {
            Box::new(async move {
                println!("called from wasm: {:?}", params);
                results[0] = wasmtime::Val::I32(
                    tokio::fs::read_to_string("Cargo.toml").await.unwrap().len() as i32,
                );
                Ok(())
            })
        },
    );
    // ...
}
```

The second argument is the signature of the host function. It takes two `i32` arguments and returns an `i32`.

The third argument is the body of the host function. It is an async function. It reads the `Cargo.toml` file and
returns its length as `i32`. Getting this to compile was the trickiest part of this experiment.

## `async` instance

We had to change the way we create an instance and call the exported function:

```rust
async fn main() {
    // ...

    let instance = wasmtime::Instance::new_async(&mut store, &module, &[from_host.into()]).await?;

    // ...
}
```

Notice the call to `wasmtime::Instance::new_async` instead of `wasmtime::Instance::new`. Further, notice the `.await`.

## `async` call

We had to change the way we call the exported function as well:

```rust
async fn main() {
    // ...

    println!("wasm said: {}", sum.call_async(&mut store, (223,)).await?);
    // ...
}
```

Note the `.call_async()` and the `.await`.