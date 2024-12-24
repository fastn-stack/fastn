impl fastn_wasm::Store {
    fn sqlite_connect(&mut self, db_url: &str) -> wasmtime::Result<i32> {
        let db = rusqlite::Connection::open(if db_url == "default" {
            self.db_url.as_str()
        } else {
            db_url
        })?; // TODO: use rusqlite_to_diesel to convert error

        self.sqlite = Some(std::sync::Arc::new(async_lock::Mutex::new(db)));
        Ok(0)
    }
}
