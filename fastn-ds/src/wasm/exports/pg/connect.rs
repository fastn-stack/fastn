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
                let pool = deadpool_postgres::Config {
                    url: Some(db_url.to_string()),
                    ..Default::default()
                }
                .create_pool(
                    Some(deadpool_postgres::Runtime::Tokio1),
                    tokio_postgres::NoTls,
                )?;
                self.wasm_pg_pools.insert(db_url.to_string(), pool);
                self.wasm_pg_pools.get(db_url).unwrap() // expect to be there
            }
        };

        let mut clients = self.clients.lock().await;
        clients.push(fastn_ds::wasm::Conn {
            client: pool.get().await?,
        });
        Ok(clients.len() as i32 - 1)
    }
}
