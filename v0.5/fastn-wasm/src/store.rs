pub struct Store<STORE: StoreExt> {
    pub req: ft_sys_shared::Request,
    pub clients: std::sync::Arc<async_lock::Mutex<Vec<Conn>>>,
    pub pg_pools: actix_web::web::Data<scc::HashMap<String, deadpool_postgres::Pool>>,
    pub sqlite: Option<std::sync::Arc<async_lock::Mutex<Box<dyn ConnectionExt>>>>,
    pub response: Option<ft_sys_shared::Request>,
    pub db_url: String,
    pub inner: STORE,
}
pub trait StoreExt: Send {
    fn connection_open(
        &self,
        store_db_url: &str,
        db_url: &str,
    ) -> wasmtime::Result<Box<dyn ConnectionExt>>;
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
pub enum ExecuteError {
    Rusqlite(rusqlite::Error),
    InvalidQuery(String),
}

pub trait ConnectionExt: Send {
    fn prepare(&self, sql: &str) -> Result<rusqlite::Statement, ExecuteError>;
    fn execute(&self, query: &str, binds: Vec<Value>) -> Result<usize, ExecuteError>;
    fn execute_batch(&self, query: &str) -> Result<(), ExecuteError>;
}

pub type Value = ft_sys_shared::SqliteRawValue;
