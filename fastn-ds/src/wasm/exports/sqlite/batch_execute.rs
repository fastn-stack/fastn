pub async fn batch_execute(
    mut caller: wasmtime::Caller<'_, fastn_ds::wasm::Store>,
    ptr: i32,
    len: i32,
) -> wasmtime::Result<i32> {
    let q = fastn_ds::wasm::helpers::get_str(ptr, len, &mut caller).await?;
    let res = caller.data_mut().sqlite_batch_execute(q).await?;
    fastn_ds::wasm::helpers::send_json(res, &mut caller).await
}

impl fastn_ds::wasm::Store {
    pub async fn sqlite_batch_execute(
        &mut self,
        q: String,
    ) -> wasmtime::Result<Result<(), ft_sys_shared::DbError>> {
        let conn = if let Some(ref mut conn) = self.sqlite {
            conn
        } else {
            todo!()
        };

        let conn = conn.lock().await;

        Ok(match conn.execute_batch(q.as_str()) {
            Ok(()) => Ok(()),
            Err(_e) => todo!(),
        })
    }
}
