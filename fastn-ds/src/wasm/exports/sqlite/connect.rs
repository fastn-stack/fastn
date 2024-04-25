pub async fn connect(
    mut caller: wasmtime::Caller<'_, fastn_ds::wasm::Store>,
    ptr: i32,
    len: i32,
) -> wasmtime::Result<i32> {
    let db_url = fastn_ds::wasm::helpers::get_str(ptr, len, &mut caller).await?;
    println!("sqlite_connect: {db_url}");

    caller.data_mut().sqlite_connect(db_url.as_str()).await
}

impl fastn_ds::wasm::Store {
    pub async fn sqlite_connect(&mut self, db_url: &str) -> wasmtime::Result<i32> {
        assert_eq!(db_url, "default", "we currently only support default");
        let mut conn = self.sqlite.as_ref().lock().await;
        if conn.is_some() {
            return Ok(0);
        }

        // TODO: convert this to an error instead of assert
        assert_eq!(
            self.db_url.starts_with("sqlite"),
            true,
            "only sqlite is supported"
        );

        let db = rusqlite::Connection::open(self.db_url.as_str())?;
        *conn = Some(db);

        Ok(0)
    }
}
