pub struct Store<STORE: StoreExt> {
    pub req: ft_sys_shared::Request,
    pub clients: std::sync::Arc<async_lock::Mutex<Vec<Conn>>>,
    pub pg_pools: actix_web::web::Data<scc::HashMap<String, deadpool_postgres::Pool>>,
    pub sqlite: Option<std::sync::Arc<async_lock::Mutex<Box<dyn ConnectionExt>>>>,
    pub response: Option<ft_sys_shared::Request>,
    pub db_url: String,
    pub inner: STORE,
}

pub struct StoreImpl;
impl StoreExt for StoreImpl {}

pub trait StoreExt: Send {
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
    ) -> wasmtime::Result<Box<dyn ConnectionExt>> {
        let conn = rusqlite::Connection::open(self.get_db_url(store_db_url, db_url))?;
        Ok(Box::new(conn))
    }
}

pub struct Conn {
    pub client: deadpool::managed::Object<deadpool_postgres::Manager>,
}

impl<STORE: StoreExt> Store<STORE> {
    pub fn new(
        req: ft_sys_shared::Request,
        pg_pools: actix_web::web::Data<scc::HashMap<String, deadpool_postgres::Pool>>,
        db_url: String,
        inner: STORE,
    ) -> Store<STORE> {
        Self {
            req,
            response: None,
            clients: Default::default(),
            pg_pools,
            db_url,
            sqlite: None,
            inner,
        }
    }
}

#[derive(Debug)]
pub enum SQLError {
    Rusqlite(rusqlite::Error),
    InvalidQuery(String),
}

pub trait ConnectionExt: Send {
    fn prepare(&self, sql: &str) -> Result<rusqlite::Statement, SQLError>;
    fn execute(
        &self,
        query: &str,
        binds: Vec<ft_sys_shared::SqliteRawValue>,
    ) -> Result<usize, SQLError>;
    fn execute_batch(&self, query: &str) -> Result<(), SQLError>;
}

impl fastn_wasm::ConnectionExt for rusqlite::Connection {
    fn prepare(&self, sql: &str) -> Result<rusqlite::Statement, fastn_wasm::SQLError> {
        self.prepare(sql).map_err(fastn_wasm::SQLError::Rusqlite)
    }

    fn execute(
        &self,
        query: &str,
        binds: Vec<ft_sys_shared::SqliteRawValue>,
    ) -> Result<usize, fastn_wasm::SQLError> {
        self.execute(query, rusqlite::params_from_iter(binds))
            .map_err(fastn_wasm::SQLError::Rusqlite)
    }

    fn execute_batch(&self, query: &str) -> Result<(), fastn_wasm::SQLError> {
        self.execute_batch(query)
            .map_err(fastn_wasm::SQLError::Rusqlite)
    }
}
