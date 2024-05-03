pub struct Store {
    pub req: ft_sys_shared::Request,
    pub ud: Option<ft_sys_shared::UserData>,
    pub clients: std::sync::Arc<async_lock::Mutex<Vec<Conn>>>,
    pub pg_pools: actix_web::web::Data<scc::HashMap<String, deadpool_postgres::Pool>>,
    pub sqlite: Option<std::sync::Arc<async_lock::Mutex<rusqlite::Connection>>>,
    pub response: Option<Response>,
    pub db_url: String,
}

#[derive(Debug)]
pub enum Response {
    /// When wasm worker sent HTTP response.
    Http(ft_sys_shared::Request),
}

pub struct Conn {
    pub client: deadpool::managed::Object<deadpool_postgres::Manager>,
}

impl Store {
    pub fn new(
        req: ft_sys_shared::Request,
        ud: Option<ft_sys_shared::UserData>,
        pg_pools: actix_web::web::Data<scc::HashMap<String, deadpool_postgres::Pool>>,
        db_url: String,
    ) -> Store {
        Self {
            req,
            ud,
            response: None,
            clients: Default::default(),
            pg_pools,
            db_url,
            sqlite: None,
        }
    }
}
