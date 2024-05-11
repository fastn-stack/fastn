pub mod exports;
pub mod helpers;
pub mod macros;
mod store;

pub use store::{Conn, Store};

#[tracing::instrument(skip_all)]
pub async fn process_http_request(
    req: ft_sys_shared::Request,
    ud: Option<ft_sys_shared::UserData>,
    module: wasmtime::Module,
    wasm_pg_pools: actix_web::web::Data<scc::HashMap<String, deadpool_postgres::Pool>>,
    db_url: String,
) -> wasmtime::Result<ft_sys_shared::Request> {
    let path = req.uri.clone();
    let hostn_store = fastn_ds::wasm::Store::new(req, ud, wasm_pg_pools, db_url);
    let mut linker = wasmtime::Linker::new(module.engine());
    hostn_store.register_functions(&mut linker);

    let mut wasm_store = wasmtime::Store::new(module.engine(), hostn_store);

    let instance = match linker.instantiate_async(&mut wasm_store, &module).await {
        Ok(i) => i,
        Err(e) => {
            return Ok(ft_sys_shared::Request::server_error(format!(
                "failed to instantiate wasm module: {e:?}"
            )));
        }
    };

    let (main, mut wasm_store) = get_entrypoint(instance, wasm_store, path)?;
    main.call_async(&mut wasm_store, ()).await?;

    Ok(wasm_store.into_data().response.unwrap_or_else(|| {
        // actix_web::HttpResponse::InternalServerError().body("no response from wasm")
        eprintln!("wasm file returned no http response");
        todo!()
    }))
}

pub fn get_entrypoint(
    instance: wasmtime::Instance,
    mut store: wasmtime::Store<fastn_ds::wasm::Store>,
    path: String,
) -> wasmtime::Result<(
    wasmtime::TypedFunc<(), ()>,
    wasmtime::Store<fastn_ds::wasm::Store>,
)> {
    if let Ok(f) = instance.get_typed_func::<(), ()>(&mut store, "main_ft") {
        return Ok((f, store));
    }
    let endpoint = path_to_endpoint(path)?;
    println!("main_ft not found, trying {endpoint}");
    instance
        .get_typed_func(&mut store, endpoint.as_str())
        .map(|v| (v, store))
}

#[derive(Debug, thiserror::Error)]
enum PathToEndpointError {
    #[error("no wasm file found in path")]
    NoWasm,
    #[error("multiple slashes in path")]
    MultipleSlashes,
}

fn path_to_endpoint(path: String) -> wasmtime::Result<String> {
    match path.split_once(".wasm/") {
        Some((_, l)) => {
            let l = l.trim_end_matches('/');
            if l.contains('/') {
                return Err(PathToEndpointError::MultipleSlashes.into());
            }
            Ok(l.trim_end_matches('/').replace('-', "_") + "__endpoint")
        }
        None => Err(PathToEndpointError::NoWasm.into()),
    }
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
