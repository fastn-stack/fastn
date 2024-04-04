pub struct Store {
    pub req: ft_sys_shared::Request,
    pub ud: Option<ft_sys_shared::UserData>,
    pub clients: std::sync::Arc<async_lock::Mutex<Vec<Conn>>>,
    pub wasm_pg_pools: actix_web::web::Data<dashmap::DashMap<String, deadpool_postgres::Pool>>,
    pub response: Option<ft_sys_shared::Request>,
    pub db_url: String,
}

pub struct Conn {
    pub client: deadpool::managed::Object<deadpool_postgres::Manager>,
}

impl Store {
    pub fn new(
        req: ft_sys_shared::Request,
        ud: Option<ft_sys_shared::UserData>,
        wasm_pg_pools: actix_web::web::Data<dashmap::DashMap<String, deadpool_postgres::Pool>>,
        db_url: String,
    ) -> Store {
        Self {
            req,
            ud,
            response: None,
            clients: Default::default(),
            wasm_pg_pools,
            db_url,
        }
    }
}
