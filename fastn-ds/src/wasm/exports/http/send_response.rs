pub async fn send_response(
    mut caller: wasmtime::Caller<'_, fastn_wasm::Store>,
    ptr: i32,
    len: i32,
) -> wasmtime::Result<()> {
    let r = fastn_wasm::helpers::get_json(ptr, len, &mut caller)?;
    caller.data_mut().store_response(r);
    Ok(())
}
