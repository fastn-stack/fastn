pub async fn connect(
    mut caller: wasmtime::Caller<'_, fastn_wasm::Store>,
    ptr: i32,
    len: i32,
) -> wasmtime::Result<i32> {
    let db_url = fastn_wasm::helpers::get_str(ptr, len, &mut caller)?;
    caller.data_mut().pg_connect(db_url.as_str()).await
}

impl fastn_wasm::Store {
    pub async fn pg_connect(&mut self, db_url: &str) -> wasmtime::Result<i32> {
        let db_url = if db_url == "default" {
            self.db_url.as_str()
        } else {
            db_url
        };

        let mut clients = self.clients.lock().await;

        return match self.pg_pools.get(db_url) {
            Some(pool) => get_client(pool.get(), &mut clients).await,
            None => {
                let pool = fastn_ds::create_pool(db_url).await?;
                fastn_ds::insert_or_update(&self.pg_pools, db_url.to_string(), pool);
                get_client(self.pg_pools.get(db_url).unwrap().get(), &mut clients).await
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
