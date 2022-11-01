#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let engine = wasmtime::Engine::new(wasmtime::Config::new().async_support(true))?;
    let module = wasmtime::Module::from_file(
        &engine,
        "../guest/target/wasm32-unknown-unknown/debug/guest.wasm",
    )?;
    let mut store = wasmtime::Store::new(&engine, ());

    let from_host = wasmtime::Func::new_async(
        &mut store,
        wasmtime::FuncType::new(
            vec![wasmtime::ValType::I32, wasmtime::ValType::I32],
            Some(wasmtime::ValType::I32),
        ),
        |_caller, params, results| {
            Box::new(async move {
                println!("called from wasm: {:?}", params);
                results[0] = wasmtime::Val::I32(
                    tokio::fs::read_to_string("Cargo.toml").await.unwrap().len() as i32,
                );
                Ok(())
            })
        },
    );

    let instance = wasmtime::Instance::new_async(&mut store, &module, &[from_host.into()]).await?;
    let sum = instance.get_typed_func::<(i32,), i32, _>(&mut store, "sum")?;

    println!("wasm said: {}", sum.call_async(&mut store, (223,)).await?);

    Ok(())
}
