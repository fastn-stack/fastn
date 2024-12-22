pub async fn print<T: Send>(
    mut caller: wasmtime::Caller<'_, T>,
    ptr: i32,
    len: i32,
) -> wasmtime::Result<()> {
    println!(
        "wasm: {}",
        fastn_wasm::helpers::get_str(ptr, len, &mut caller)?
    );

    Ok(())
}

pub async fn var<S: Send>(
    mut caller: wasmtime::Caller<'_, S>,
    ptr: i32,
    len: i32,
) -> wasmtime::Result<i32> {
    let key = fastn_wasm::helpers::get_str(ptr, len, &mut caller)?;
    let value = std::env::var(key).ok();

    fastn_wasm::helpers::send_json(value, &mut caller).await
}

pub async fn now<S: Send>(mut caller: wasmtime::Caller<'_, S>) -> wasmtime::Result<i32> {
    fastn_wasm::helpers::send_json(chrono::Utc::now(), &mut caller).await
}

pub async fn random<S: Send>(mut caller: wasmtime::Caller<'_, S>) -> wasmtime::Result<i32> {
    fastn_wasm::helpers::send_json(rand::random::<f64>(), &mut caller).await
}
