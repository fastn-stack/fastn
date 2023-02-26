#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let engine = wasmtime::Engine::new(wasmtime::Config::new().async_support(true))?;
    let module = wasmtime::Module::from_file(
        &engine,
        "../guest/target/wasm32-unknown-unknown/debug/guest.wasm",
    )?;
    let mut store = wasmtime::Store::new(&engine, ());
    let instance = wasmtime::Instance::new_async(&mut store, &module, &[]).await?;
    let memory: wasmtime::Memory = instance.get_memory(&mut store, "memory").unwrap();

    let alloc = instance.get_typed_func::<u32, u32, _>(&mut store, "alloc")?;
    let size = 10;
    let memory_address = alloc.call_async(&mut store, size as u32).await?;
    println!("Wasm Memory address: {}", memory_address);
    let input = vec![1 as u8, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    println!("Coping the data into the wasm memory");
    memory
        .write(&mut store, memory_address as usize, input.as_ref())
        .unwrap();
    println!("data copied successfully");

    let array_sum = instance.get_typed_func::<(u32, u32), u32, _>(&mut store, "array_sum")?;
    let sum_of_array = array_sum
        .call_async(&mut store, (memory_address, size))
        .await?;
    println!("Array Sum: {}", sum_of_array);

    // println!("Deallocating from the wasm memory");
    // println!("Coping the data into the wasm memory");

    // let sum_of_array = array_sum.call_async(&mut store, (memory_address, size)).await?;
    // println!("Array Sum: {}", sum_of_array);

    let dealloc = instance.get_typed_func::<(u32, u32), (), _>(&mut store, "dealloc")?;
    dealloc
        .call_async(&mut store, (memory_address, size))
        .await?;
    println!("Memory deallocated");

    Ok(())
}
