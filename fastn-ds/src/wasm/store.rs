pub struct Store {
    pub req: ft_sys_shared::Request,
    pub clients: std::sync::Arc<async_lock::Mutex<Vec<Conn>>>,
    pub pg_pools: actix_web::web::Data<dashmap::DashMap<String, deadpool_postgres::Pool>>,
    pub sqlite: actix_web::web::Data<async_lock::Mutex<Option<rusqlite::Connection>>>,
    pub response: Option<Response>,
    pub db_url: String,
}

#[derive(Debug)]
pub enum Response {
    /// When wasm worker sent HTTP response.
    Http(ft_sys_shared::Request),
    /// When wasm worker asked to render and parse a string.
    Ftd(FtdResponse),
}

#[derive(serde::Deserialize, Debug)]
pub struct FtdResponse {
    /// This is the ID of the file, relative to the package in which wasm worker
    /// is present.
    pub ftd: String,
    /// The request data processor will have access to this data as well, so wasm
    /// worker can put some data in it and ftd file can read it back.
    pub request_data: serde_json::Value,
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
