pub async fn connect(
    mut caller: wasmtime::Caller<'_, fastn_ds::wasm::Store>,
    ptr: i32,
    len: i32,
) -> wasmtime::Result<i32> {
    let db_url = fastn_ds::wasm::helpers::get_str(ptr, len, &mut caller).await?;
    caller.data_mut().pg_connect(db_url.as_str()).await
}

impl fastn_ds::wasm::Store {
    pub async fn pg_connect(&mut self, db_url: &str) -> wasmtime::Result<i32> {
        let db_url = if db_url == "default" {
            self.db_url.as_str()
        } else {
            db_url
        };

        let pool = match self.wasm_pg_pools.get(db_url) {
            Some(pool) => pool,
            None => {
                let pool = fastn_ds::create_pool(db_url).await?;
                self.wasm_pg_pools.insert(db_url.to_string(), pool);
                self.wasm_pg_pools.get(db_url).unwrap() // expect to be there
            }
        };

        let client = pool.get().await?;

        let mut clients = self.clients.lock().await;
        clients.push(fastn_ds::wasm::Conn { client });
        Ok(clients.len() as i32 - 1)
    }
}
