pub async fn connect<STORE: fastn_wasm::StoreExt>(
    mut caller: wasmtime::Caller<'_, fastn_wasm::Store<STORE>>,
    ptr: i32,
    len: i32,
) -> wasmtime::Result<i32> {
    let db_url = fastn_wasm::helpers::get_str(ptr, len, &mut caller)?;
    caller.data_mut().pg_connect(db_url.as_str()).await
}

impl<STORE: fastn_wasm::StoreExt> fastn_wasm::Store<STORE> {
    pub async fn pg_connect(&mut self, db_url: &str) -> wasmtime::Result<i32> {
        let db_url = self.inner.get_db_url(self.db_url.as_str(), db_url);

        let mut clients = self.clients.lock().await;

        return match self.pg_pools.get(db_url.as_str()) {
            Some(pool) => get_client(pool.get(), &mut clients).await,
            None => {
                let pool = fastn_wasm::pg::create_pool(db_url.as_str()).await?;
                fastn_wasm::insert_or_update(&self.pg_pools, db_url.to_string(), pool);
                get_client(
                    self.pg_pools.get(db_url.as_str()).unwrap().get(),
                    &mut clients,
                )
                .await
            }
        };

        async fn get_client(
            pool: &deadpool_postgres::Pool,
            clients: &mut Vec<fastn_wasm::Conn>,
        ) -> wasmtime::Result<i32> {
            let client = pool.get().await?;
            clients.push(fastn_wasm::Conn { client });
            Ok(clients.len() as i32 - 1)
        }
    }
}
