pub async fn get_request(
    mut caller: wasmtime::Caller<'_, fastn_ds::wasm::Store>,
) -> wasmtime::Result<i32> {
    let req = caller.data().to_http();
    fastn_wasm::helpers::send_json(req, &mut caller).await
}

impl fastn_ds::wasm::Store {
    pub fn to_http(&self) -> ft_sys_shared::Request {
        self.req.clone()
    }
}
