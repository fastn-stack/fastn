pub async fn connect(
    mut caller: wasmtime::Caller<'_, fastn_wasm::Store>,
    ptr: i32,
    len: i32,
) -> wasmtime::Result<i32> {
    let db_url = fastn_wasm::helpers::get_str(ptr, len, &mut caller)?;
    println!("sqlite_connect: {db_url}");
    caller.data_mut().sqlite_connect(db_url.as_str()).await
}

impl fastn_wasm::Store {
    pub async fn sqlite_connect(&mut self, db_url: &str) -> wasmtime::Result<i32> {
        let db = rusqlite::Connection::open(if db_url == "default" {
            self.db_url.as_str()
        } else {
            db_url
        })?; // TODO: use rusqlite_to_diesel to convert error

        self.sqlite = Some(std::sync::Arc::new(async_lock::Mutex::new(db)));
        Ok(0)
    }
}
