#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let engine = wasmtime::Engine::new(wasmtime::Config::new().async_support(true))?;
    let module = wasmtime::Module::from_file(
        &engine,
        "../guest/target/wasm32-unknown-unknown/debug/guest.wasm",
    )?;
    let mut store = wasmtime::Store::new(&engine, ());
    let instance = wasmtime::Instance::new_async(&mut store, &module, &[]).await?;
    let sum = instance.get_typed_func::<(), i32, _>(&mut store, "usize_size")?;

    println!("wasm usize: {}", sum.call_async(&mut store, ()).await?);
    println!("host usize: {}", 1usize.to_ne_bytes().len() as i32);

    Ok(())
}
