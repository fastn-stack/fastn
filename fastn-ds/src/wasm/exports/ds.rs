pub async fn get_content(
    _caller: wasmtime::Caller<'_, fastn_ds::wasm::Store>,
    _ptr: i32,
    _len: i32,
) -> wasmtime::Result<i32> {
    unimplemented!()
}

pub async fn delete_file(
    _caller: wasmtime::Caller<'_, fastn_ds::wasm::Store>,
    _ptr: i32,
    _len: i32,
) -> wasmtime::Result<i32> {
    unimplemented!()
}

pub async fn create_file(
    _caller: wasmtime::Caller<'_, fastn_ds::wasm::Store>,
    _ptr: i32,
    _len: i32,
) -> wasmtime::Result<i32> {
    unimplemented!()
}

pub async fn save_file(
    _caller: wasmtime::Caller<'_, fastn_ds::wasm::Store>,
    _ptr: i32,
    _len: i32,
) -> wasmtime::Result<i32> {
    unimplemented!()
}
