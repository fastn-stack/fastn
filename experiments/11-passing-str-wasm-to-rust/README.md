# Passing String WASM to Rust

In this experiment, I am going to pass a string buffer from wasm(guest) to 
rust(host). To accomplish this, In this directory we have two folders `guest`
and `host` both are the Rust crates.

## Guest

In guest, we are using one function `from_host` which is exported from host, and
function in guest itself.

I am using another function which is called from host `call_guest`, It is 
creating a `data` as string, converting it `into_bytes` array and getting the
pointer of it, because we cannot pass direct data from guest to host, we are
passing pointer and length of this data to host, host will read this from wasm
memory.

```rust
// Functions from host
extern "C" {
    fn from_host(ptr: u32, len: u32) -> u32;
}

#[no_mangle]
pub fn call_guest(_ptr: u32) -> u32 {
    unsafe {
        let data = "Call From Guest".to_string();
        let buf = data.into_bytes();
        let buf_pointer = buf.as_ptr() as u32;
        let buf_len = buf.len();
        // std::mem::forget(buf);
        let from_host = from_host(buf_pointer, buf_len as u32);
        return from_host;
    }
}
```

## Host

In host, we are exporting one function `from_host` to guest, which guest is calling.
In this function, we are receiving one pointer(of wasm memory and len of it).

Below line is doing magic, this is way to access the wasm memory in the host 
provided function to guest. This way I got the reference to wasm memory

```rust
let mem = match caller.get_export("memory") {
                Some(wasmtime::Extern::Memory(mem)) => mem,
                _ => anyhow::bail!("failed to find host memory"),
            };
```

Below line is reading the data from wasm memory, by passing the pointer to it
and len, which are passed as arguments from wasm while calling `from_host(ptr, len)`.

```rust
let data = mem
    .data(&caller)
    .get(ptr as u32 as usize..)
    .and_then(|arr| arr.get(..len as u32 as usize));
```

Whole function body

```rust
async fn main() -> anyhow::Result<()> {
    let engine = wasmtime::Engine::default();
    let module = wasmtime::Module::from_file(
        &engine,
        "../guest/target/wasm32-unknown-unknown/debug/guest.wasm",
    )?;
    let mut store = wasmtime::Store::new(&engine, ());

    // How to access the memory in host function
    let from_host = wasmtime::Func::wrap(
        &mut store,
        |mut caller: wasmtime::Caller<'_, ()>, ptr: u32, len: u32| {
            let mem = match caller.get_export("memory") {
                Some(wasmtime::Extern::Memory(mem)) => mem,
                _ => anyhow::bail!("failed to find host memory"),
            };

            let data = mem
                .data(&caller)
                .get(ptr as u32 as usize..)
                .and_then(|arr| arr.get(..len as u32 as usize));
            let string = match data {
                Some(data) => match std::str::from_utf8(data) {
                    Ok(s) => s,
                    Err(_) => anyhow::bail!("invalid utf-8"),
                },
                None => anyhow::bail!("pointer/length out of bounds"),
            };
            // assert_eq!(string, "Hello, world!");
            println!("Guest Passed: {}", string);
            Ok(1)
        },
    );

    let instance = wasmtime::Instance::new(&mut store, &module, &[from_host.into()])?;
    let call_guest = instance.get_typed_func::<(u32,), u32, _>(&mut store, "call_guest")?;
    // And finally we can call the wasm!
    println!("wasm said: {}", call_guest.call(&mut store, (1,))?);

    Ok(())
}

```