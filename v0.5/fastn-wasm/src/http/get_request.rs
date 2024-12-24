pub async fn get_request<T: fastn_wasm::StoreExt>(
    mut caller: wasmtime::Caller<'_, fastn_wasm::Store<T>>,
) -> wasmtime::Result<i32> {
    let req = caller.data().to_http();
    fastn_wasm::helpers::send_json(req, &mut caller).await
}

impl<T: fastn_wasm::StoreExt> fastn_wasm::Store<T> {
    pub fn to_http(&self) -> ft_sys_shared::Request {
        self.req.clone()
    }
}
