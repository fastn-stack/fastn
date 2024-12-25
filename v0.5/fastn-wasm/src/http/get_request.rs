pub async fn get_request<STORE: fastn_wasm::StoreExt>(
    mut caller: wasmtime::Caller<'_, fastn_wasm::Store<STORE>>,
) -> wasmtime::Result<i32> {
    let req = caller.data().to_http();
    fastn_wasm::helpers::send_json(req, &mut caller).await
}

impl<STORE: fastn_wasm::StoreExt> fastn_wasm::Store<STORE> {
    pub fn to_http(&self) -> ft_sys_shared::Request {
        self.req.clone()
    }
}
