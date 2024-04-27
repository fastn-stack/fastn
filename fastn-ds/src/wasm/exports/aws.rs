pub async fn pre_signed_request(
    _caller: wasmtime::Caller<'_, fastn_ds::wasm::Store>,
    _ptr: i32,
    _len: i32,
) -> wasmtime::Result<i32> {
    unimplemented!()
}
