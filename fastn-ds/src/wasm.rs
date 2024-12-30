pub struct Store;

impl fastn_wasm::StoreExt for Store {
    fn get_db_url(&self, store_db_url: &str, db_url: &str) -> String {
        if db_url == "default" {
            store_db_url
        } else {
            db_url
        }
        .to_string()
    }

    fn connection_open(
        &self,
        store_db_url: &str,
        db_url: &str,
    ) -> wasmtime::Result<Box<dyn fastn_wasm::ConnectionExt>> {
        let conn = Connection(rusqlite::Connection::open(
            self.get_db_url(store_db_url, db_url),
        )?);

        Ok(Box::new(conn))
    }
}

pub struct Connection(rusqlite::Connection);

impl fastn_wasm::ConnectionExt for Connection {
    fn prepare(&self, sql: &str) -> Result<rusqlite::Statement, fastn_wasm::ExecuteError> {
        self.0
            .prepare(sql)
            .map_err(fastn_wasm::ExecuteError::Rusqlite)
    }

    fn execute(
        &self,
        query: &str,
        binds: Vec<fastn_wasm::Value>,
    ) -> Result<usize, fastn_wasm::ExecuteError> {
        self.0
            .execute(query, rusqlite::params_from_iter(binds))
            .map_err(fastn_wasm::ExecuteError::Rusqlite)
    }

    fn execute_batch(&self, query: &str) -> Result<(), fastn_wasm::ExecuteError> {
        self.0
            .execute_batch(query)
            .map_err(fastn_wasm::ExecuteError::Rusqlite)
    }
}
