use crate::wasm::exports::sqlite::query::rusqlite_to_diesel;

pub async fn batch_execute(
    mut caller: wasmtime::Caller<'_, fastn_wasm::Store>,
    ptr: i32,
    len: i32,
) -> wasmtime::Result<i32> {
    let q = fastn_wasm::helpers::get_str(ptr, len, &mut caller)?;
    let res = caller.data_mut().sqlite_batch_execute(q).await?;
    fastn_wasm::helpers::send_json(res, &mut caller).await
}

impl fastn_wasm::Store {
    pub async fn sqlite_batch_execute(
        &mut self,
        q: String,
    ) -> wasmtime::Result<Result<(), ft_sys_shared::DbError>> {
        let conn = if let Some(ref mut conn) = self.sqlite {
            conn
        } else {
            eprintln!("sqlite connection not found");
            return Ok(Err(ft_sys_shared::DbError::UnableToSendCommand(
                "no db connection".to_string(),
            )));
        };

        let conn = conn.lock().await;

        println!("batch: {q:?}");
        Ok(match conn.execute_batch(q.as_str()) {
            Ok(()) => Ok(()),
            Err(e) => {
                eprint!("err: {e:?}");
                let e = rusqlite_to_diesel(e);
                eprintln!("err: {e:?}");
                return Ok(Err(e));
            }
        })
    }
}
