#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let engine = wasmtime::Engine::new(wasmtime::Config::new().async_support(true))?;
    let module = wasmtime::Module::from_file(
        &engine,
        "../guest/target/wasm32-unknown-unknown/debug/guest.wasm",
    )?;
    let mut store = wasmtime::Store::new(&engine, ());
    let instance = wasmtime::Instance::new_async(&mut store, &module, &[]).await?;
    let hello = instance.get_typed_func::<(), u32, _>(&mut store, "hello")?;

    let r = hello.call_async(&mut store, ()).await?;
    println!(
        "wasm => pointer: {}, value: {}",
        r,
        read_string(
            r as usize, // upward cast never panics
            instance.get_memory(&mut store, "memory").unwrap(),
            &mut store
        )
    );

    Ok(())
}

fn read_string(offset: usize, mem: wasmtime::Memory, store: &mut wasmtime::Store<()>) -> String {
    let mem = mem.data(&store);

    // in this function we have used unchecked indexing into mem, so if the offset is out of bounds
    // say because wasm is trying to be funny, we will panic. This is fine for now, but in the future
    // we will want to handle this more gracefully.

    // experiment 06 was to prove that using 4 here is fine
    let size = u32::from_ne_bytes(mem[offset..offset + 4].try_into().unwrap()) as usize;
    let str_offset = u32::from_ne_bytes(mem[offset + 4..offset + 8].try_into().unwrap()) as usize;

    std::str::from_utf8(&mem[str_offset..str_offset + size])
        .unwrap_or("oops")
        .to_string()
}
