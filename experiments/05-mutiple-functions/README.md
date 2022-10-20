# `multiple-functions`

We have started with `02-export-host-function` to see how to export multiple functions from host.

If you look at the function registration:

```rust
fn main() -> anyhow::Result<()> {
    // ...
    
    let from_host = wasmtime::Func::wrap(&mut store, |_caller: wasmtime::Caller<'_, ()>| {
        println!("called from wasm");
    });

    let instance = wasmtime::Instance::new(&mut store, &module, &[from_host.into()])?;
    
    // ...
}
```

Not we do not call name the function that we are exporting. The variable name is `from_host`, but how can 
`wasmtime::Instance::new()` find the name of a variable passed? What if we there was no variable and we passed the 
wrapped directly?

```rust
fn main() -> anyhow::Result<()> {
    // ...

    let instance = wasmtime::Instance::new(
        &mut store,
        &module,
        &[
            wasmtime::Func::wrap(&mut store, |_caller: wasmtime::Caller<'_, ()>| {
                println!("called from wasm");
            })
                .into(),
        ],
    )?;
    
    // ...
}
```

Now there is no function name at all.

## Functions Are Referred By Index

If we define more than one function, the function names are ignored, and the first function in the guests' extern list
is the first exported function in the host (wat):

```rust
// guest.rs
extern "C" {
    fn from_host1();
    fn from_host2();
}
```

```rust
// host.rs
fn main() -> anyhow::Result<()> {
    // ...
    let instance =
        wasmtime::Instance::new(&mut store, &module, &[from_host2.into(), from_host1.into()])?;
    // ...
}
```

If we return the order of functions in the list here, or in guest.rs, the wasm called the wrong function.