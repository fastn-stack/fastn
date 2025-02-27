pub async fn execute<STORE: fastn_wasm::StoreExt>(
    mut caller: wasmtime::Caller<'_, fastn_wasm::Store<STORE>>,
    ptr: i32,
    len: i32,
) -> wasmtime::Result<i32> {
    let q: fastn_wasm::sqlite::Query = fastn_wasm::helpers::get_json(ptr, len, &mut caller)?;
    let res = caller.data_mut().sqlite_execute(q).await?;
    fastn_wasm::helpers::send_json(res, &mut caller).await
}

impl<STORE: fastn_wasm::StoreExt> fastn_wasm::Store<STORE> {
    async fn sqlite_execute(
        &mut self,
        q: fastn_wasm::sqlite::Query,
    ) -> wasmtime::Result<Result<usize, ft_sys_shared::DbError>> {
        let conn = if let Some(ref mut conn) = self.sqlite {
            conn
        } else {
            eprintln!("sqlite connection not found");
            return Ok(Err(ft_sys_shared::DbError::UnableToSendCommand(
                "connection not found".to_string(),
            )));
        };

        let conn = conn.lock().await;
        println!("execute: {q:?}");
        match conn.execute(q.sql.as_str(), q.binds) {
            Ok(cursor) => Ok(Ok(cursor)),
            Err(fastn_wasm::SQLError::Rusqlite(e)) => {
                eprint!("err: {e:?}");
                let e = fastn_wasm::sqlite::query::rusqlite_to_diesel(e);
                eprintln!("err: {e:?}");
                Ok(Err(e))
            }
            Err(fastn_wasm::SQLError::InvalidQuery(e)) => {
                Ok(Err(ft_sys_shared::DbError::UnableToSendCommand(e)))
            } // Todo: Handle error message
        }
    }
}
