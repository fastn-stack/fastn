pub async fn execute(
    mut caller: wasmtime::Caller<'_, fastn_ds::wasm::Store>,
    conn: i32,
    ptr: i32,
    len: i32,
) -> wasmtime::Result<i32> {
    let q: fastn_ds::wasm::exports::pg::Query =
        fastn_ds::wasm::helpers::get_json(ptr, len, &mut caller).await?;
    let res = caller.data_mut().pg_execute(conn, q).await?;
    fastn_ds::wasm::helpers::send_json(res, &mut caller).await
}

impl fastn_ds::wasm::Store {
    pub async fn pg_execute(
        &mut self,
        conn: i32,
        q: fastn_ds::wasm::exports::pg::Query,
    ) -> wasmtime::Result<Result<usize, ft_sys_shared::DbError>> {
        let mut clients = self.clients.lock().await;
        let client = match clients.get_mut(conn as usize) {
            Some(c) => c,
            None => panic!(
                "unknown connection asked: {conn}, have {} connections",
                clients.len()
            ),
        };

        Ok(
            match client.client.execute_raw(q.sql.as_str(), q.binds).await {
                Ok(count) => Ok(count as usize),
                Err(e) => Err(fastn_ds::wasm::exports::pg::pg_to_shared(e)),
            },
        )
    }
}
