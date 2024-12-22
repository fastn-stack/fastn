pub async fn tejar_write<S: Send>(
    _caller: wasmtime::Caller<'_, S>,
    _ptr: i32,
    _len: i32,
) -> wasmtime::Result<i32> {
    unimplemented!()
}

pub async fn tejar_read<S: Send>(
    _caller: wasmtime::Caller<'_, S>,
    _ptr: i32,
    _len: i32,
) -> wasmtime::Result<i32> {
    unimplemented!()
}
