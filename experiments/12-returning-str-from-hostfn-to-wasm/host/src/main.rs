/*
Problem statement:
- Call the guest function call_guest
- wasm::call_guest function will call then rust::call_host by passing a string pointer
- rust::call_host function will read this string and print it simply
*/

#[tokio::main]
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
