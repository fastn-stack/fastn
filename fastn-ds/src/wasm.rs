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

#[tracing::instrument(skip_all)]
pub async fn process_http_request(
    req: ft_sys_shared::Request,
    module: wasmtime::Module,
    wasm_pg_pools: actix_web::web::Data<scc::HashMap<String, deadpool_postgres::Pool>>,
    db_url: String,
) -> wasmtime::Result<ft_sys_shared::Request> {
    let path = req.uri.clone();
    let hostn_store = fastn_wasm::Store::new(req, wasm_pg_pools, db_url, Store);
    let mut linker = wasmtime::Linker::new(module.engine());
    hostn_store.register_functions(&mut linker);
    let wasm_store = wasmtime::Store::new(module.engine(), hostn_store);
    let (wasm_store, r) = fastn_utils::handle(wasm_store, module, linker, path).await?;

    if let Some(r) = r {
        return Ok(r);
    }

    Ok(wasm_store
        .into_data()
        .response
        .ok_or(fastn_utils::WasmError::EndpointDidNotReturnResponse)?)
}

pub fn to_response(req: ft_sys_shared::Request) -> actix_web::HttpResponse {
    println!("{req:?}");
    let mut builder = actix_web::HttpResponse::build(req.method.parse().unwrap());
    let mut resp = builder.status(req.method.parse().unwrap()).body(req.body);

    for (k, v) in req.headers {
        resp.headers_mut().insert(
            k.parse().unwrap(),
            actix_http::header::HeaderValue::from_bytes(v.as_slice()).unwrap(),
        );
    }

    resp
}
