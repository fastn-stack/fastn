pub async fn connect<STORE: fastn_wasm::StoreExt>(
    mut caller: wasmtime::Caller<'_, fastn_wasm::Store<STORE>>,
    ptr: i32,
    len: i32,
) -> wasmtime::Result<i32> {
    let db_url = fastn_wasm::helpers::get_str(ptr, len, &mut caller)?;
    println!("sqlite_connect: {db_url}");
    caller.data_mut().sqlite_connect(db_url.as_str()).await
}

impl<STORE: fastn_wasm::StoreExt> fastn_wasm::Store<STORE> {
    pub async fn sqlite_connect(&mut self, db_url: &str) -> wasmtime::Result<i32> {
        let db = self.inner.connection_open(self.db_url.as_str(), db_url)?;
        self.sqlite = Some(std::sync::Arc::new(async_lock::Mutex::new(db)));
        Ok(0)
    }
}
