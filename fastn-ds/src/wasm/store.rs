pub struct Store {
    pub req: ft_sys_shared::Request,
    pub clients: std::sync::Arc<async_lock::Mutex<Vec<Conn>>>,
    pub pg_pools: actix_web::web::Data<dashmap::DashMap<String, deadpool_postgres::Pool>>,
    pub sqlite: actix_web::web::Data<async_lock::Mutex<Option<rusqlite::Connection>>>,
    pub response: Option<ft_sys_shared::Request>,
    pub db_url: String,
}

pub struct Conn {
    pub client: deadpool::managed::Object<deadpool_postgres::Manager>,
}

impl Store {
    pub fn new(
        req: ft_sys_shared::Request,
        pg_pools: actix_web::web::Data<dashmap::DashMap<String, deadpool_postgres::Pool>>,
        sqlite: actix_web::web::Data<async_lock::Mutex<Option<rusqlite::Connection>>>,
        db_url: String,
    ) -> Store {
        Self {
            req,
            response: None,
            clients: Default::default(),
            pg_pools,
            db_url,
            sqlite,
        }
    }
}
