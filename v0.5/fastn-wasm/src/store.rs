pub struct Store {
    pub req: ft_sys_shared::Request,
    pub clients: std::sync::Arc<async_lock::Mutex<Vec<Conn>>>,
    pub pg_pools: actix_web::web::Data<scc::HashMap<String, deadpool_postgres::Pool>>,
    pub sqlite: Option<std::sync::Arc<async_lock::Mutex<rusqlite::Connection>>>,
    pub response: Option<ft_sys_shared::Request>,
    pub db_url: String,
}

pub struct Conn {
    pub client: deadpool::managed::Object<deadpool_postgres::Manager>,
}

impl Store {
    pub fn new(
        req: ft_sys_shared::Request,
        pg_pools: actix_web::web::Data<scc::HashMap<String, deadpool_postgres::Pool>>,
        db_url: String,
    ) -> Store {
        Self {
            req,
            response: None,
            clients: Default::default(),
            pg_pools,
            db_url,
            sqlite: None,
        }
    }
}

impl Store {
    pub fn to_http(&self) -> ft_sys_shared::Request {
        self.req.clone()
    }

    pub fn store_response(&mut self, r: ft_sys_shared::Request) {
        self.response = Some(r);
    }
}
