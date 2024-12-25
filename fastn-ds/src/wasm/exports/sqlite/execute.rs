use crate::wasm::exports::sqlite::query::rusqlite_to_diesel;

pub async fn execute(
    mut caller: wasmtime::Caller<'_, fastn_wasm::Store>,
    ptr: i32,
    len: i32,
) -> wasmtime::Result<i32> {
    let q: fastn_ds::wasm::exports::sqlite::Query =
        fastn_wasm::helpers::get_json(ptr, len, &mut caller)?;
    let res = caller.data_mut().sqlite_execute(q).await?;
    fastn_wasm::helpers::send_json(res, &mut caller).await
}

impl fastn_wasm::Store {
    async fn sqlite_execute(
        &mut self,
        q: fastn_ds::wasm::exports::sqlite::Query,
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
        match conn.execute(q.sql.as_str(), rusqlite::params_from_iter(q.binds)) {
            Ok(cursor) => Ok(Ok(cursor)),
            Err(e) => {
                eprint!("err: {e:?}");
                let e = rusqlite_to_diesel(e);
                eprintln!("err: {e:?}");
                Ok(Err(e))
            }
        }
    }
}
