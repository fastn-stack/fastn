pub async fn pre_signed_request<S: Send>(
    _caller: wasmtime::Caller<'_, S>,
    _ptr: i32,
    _len: i32,
) -> wasmtime::Result<i32> {
    unimplemented!()
}
