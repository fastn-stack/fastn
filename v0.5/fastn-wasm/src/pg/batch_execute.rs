pub async fn batch_execute<STORE: fastn_wasm::StoreExt>(
    mut caller: wasmtime::Caller<'_, fastn_wasm::Store<STORE>>,
    conn: i32,
    ptr: i32,
    len: i32,
) -> wasmtime::Result<i32> {
    let q = fastn_wasm::helpers::get_str(ptr, len, &mut caller)?;
    let res = caller.data_mut().pg_batch_execute(conn, q).await?;
    fastn_wasm::helpers::send_json(res, &mut caller).await
}

impl<STORE: fastn_wasm::StoreExt> fastn_wasm::Store<STORE> {
    pub async fn pg_batch_execute(
        &mut self,
        conn: i32,
        q: String,
    ) -> wasmtime::Result<Result<(), ft_sys_shared::DbError>> {
        use deadpool_postgres::GenericClient;

        let mut clients = self.clients.lock().await;
        let client = match clients.get_mut(conn as usize) {
            Some(c) => c,
            None => panic!(
                "unknown connection asked: {conn}, have {} connections",
                clients.len()
            ),
        };

        Ok(match client.client.batch_execute(q.as_str()).await {
            Ok(()) => Ok(()),
            Err(e) => Err(fastn_wasm::pg::pg_to_shared(e)),
        })
    }
}
