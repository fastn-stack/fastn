pub async fn tejar_write(
    _caller: wasmtime::Caller<'_, fastn_ds::wasm::Store>,
    _ptr: i32,
    _len: i32,
) -> wasmtime::Result<i32> {
    unimplemented!()
}

pub async fn tejar_read(
    _caller: wasmtime::Caller<'_, fastn_ds::wasm::Store>,
    _ptr: i32,
    _len: i32,
) -> wasmtime::Result<i32> {
    unimplemented!()
}
