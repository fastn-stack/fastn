pub struct Store<STORE: StoreExt> {
    pub req: ft_sys_shared::Request,
    pub clients: std::sync::Arc<async_lock::Mutex<Vec<Conn>>>,
    pub pg_pools: std::sync::Arc<scc::HashMap<String, deadpool_postgres::Pool>>,
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
        mut req: ft_sys_shared::Request,
        pg_pools: std::sync::Arc<scc::HashMap<String, deadpool_postgres::Pool>>,
        db_url: String,
        inner: STORE,
        mountpoint: String,
        app_mounts: std::collections::HashMap<String, String>,
    ) -> Store<STORE> {
        req.headers
            .push((FASTN_APP_URL_HEADER.to_string(), mountpoint.into_bytes()));

        let app_mounts = serde_json::to_string(&app_mounts).unwrap();
        req.headers
            .push((FASTN_APP_URLS_HEADER.to_string(), app_mounts.into_bytes()));

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

/// `app-url` is the path on which the app is installed.
///
/// if in `FASTN.ftd`, we have:
///
/// ```ftd
/// -- import: fastn
/// -- fastn.package: hello-world
///
/// -- fastn.dependency: my-app.com
///
/// -- fastn.app: my-app.com
/// url: /foo/
/// ```
///
/// then the `app-url` is `/foo/`.
pub const FASTN_APP_URL_HEADER: &str = "x-fastn-app-url";

/// A JSON object that contains the app-urls of `fastn.app`s
///
/// If in FASTN.ftd, we have:
///
/// ```ftd
/// -- import: fastn
/// -- fastn.package: hello-world
///
/// -- fastn.app: Auth App
/// package: lets-auth.fifthtry.site
/// mount-point: /-/auth/
///
/// -- fastn.app: Let's Talk App
/// package: lets-talk.fifthtry.site
/// mount-point: /talk/
/// ```
///
/// Then the value will be a JSON string:
///
/// ```json
/// { "lets-auth": "/-/auth/", "lets-talk": "/talk/" }
/// ```
///
/// NOTE: `lets-auth.fifthtry.site` and `lets-talk.fifthtry.site` are required to be a system
/// package. The names `lets-auth` and `lets-talk` are taken from their `system` field
pub const FASTN_APP_URLS_HEADER: &str = "x-fastn-app-urls";
