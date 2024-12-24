pub struct Store<T: StoreExt> {
    pub req: ft_sys_shared::Request,
    pub clients: std::sync::Arc<async_lock::Mutex<Vec<Conn>>>,
    pub pg_pools: actix_web::web::Data<scc::HashMap<String, deadpool_postgres::Pool>>,
    pub sqlite: Option<std::sync::Arc<async_lock::Mutex<rusqlite::Connection>>>,
    pub response: Option<ft_sys_shared::Request>,
    pub db_url: String,
    pub inner: T,
}

pub trait StoreExt: Send + Clone {
    fn sqlite_connect<T: StoreExt>(
        &self,
        store: &mut Store<T>,
        db_url: &str,
    ) -> wasmtime::Result<i32>;
}

pub struct Conn {
    pub client: deadpool::managed::Object<deadpool_postgres::Manager>,
}

impl<T: StoreExt> Store<T> {
    pub fn new(
        req: ft_sys_shared::Request,
        pg_pools: actix_web::web::Data<scc::HashMap<String, deadpool_postgres::Pool>>,
        db_url: String,
        inner: T,
    ) -> Store<T> {
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
