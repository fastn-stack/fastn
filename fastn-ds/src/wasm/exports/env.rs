pub async fn print(
    mut caller: wasmtime::Caller<'_, fastn_ds::wasm::Store>,
    ptr: i32,
    len: i32,
) -> wasmtime::Result<()> {
    println!(
        "wasm: {}",
        fastn_ds::wasm::helpers::get_str(ptr, len, &mut caller)?
    );

    Ok(())
}

pub async fn var(
    mut caller: wasmtime::Caller<'_, fastn_ds::wasm::Store>,
    ptr: i32,
    len: i32,
) -> wasmtime::Result<i32> {
    let key = fastn_ds::wasm::helpers::get_str(ptr, len, &mut caller)?;
    let value = std::env::var(key).ok();

    fastn_ds::wasm::helpers::send_json(value, &mut caller).await
}

pub async fn now(mut caller: wasmtime::Caller<'_, fastn_ds::wasm::Store>) -> wasmtime::Result<i32> {
    fastn_ds::wasm::helpers::send_json(chrono::Utc::now(), &mut caller).await
}

pub async fn random(
    mut caller: wasmtime::Caller<'_, fastn_ds::wasm::Store>,
) -> wasmtime::Result<i32> {
    fastn_ds::wasm::helpers::send_json(rand::random::<f64>(), &mut caller).await
}
