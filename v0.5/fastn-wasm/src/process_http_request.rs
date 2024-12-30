#[tracing::instrument(skip_all)]
pub async fn process_http_request<STORE: fastn_wasm::StoreExt>(
    path: &str,
    module: wasmtime::Module,
    store: fastn_wasm::Store<STORE>,
) -> wasmtime::Result<ft_sys_shared::Request> {
    let mut linker = wasmtime::Linker::new(module.engine());
    store.register_functions(&mut linker);
    let wasm_store = wasmtime::Store::new(module.engine(), store);
    let (wasm_store, r) = handle(wasm_store, module, linker, path).await?;
    if let Some(r) = r {
        return Ok(r);
    }

    Ok(wasm_store
        .into_data()
        .response
        .ok_or(WasmError::EndpointDidNotReturnResponse)?)
}

pub async fn handle<S: Send>(
    mut wasm_store: wasmtime::Store<S>,
    module: wasmtime::Module,
    linker: wasmtime::Linker<S>,
    path: &str,
) -> wasmtime::Result<(wasmtime::Store<S>, Option<ft_sys_shared::Request>)> {
    let instance = match linker.instantiate_async(&mut wasm_store, &module).await {
        Ok(i) => i,
        Err(e) => {
            return Ok((
                wasm_store,
                Some(ft_sys_shared::Request::server_error(format!(
                    "failed to instantiate wasm module: {e:?}"
                ))),
            ));
        }
    };

    let (mut wasm_store, main) = get_entrypoint(instance, wasm_store, path);

    let main = match main {
        Ok(v) => v,
        Err(e) => {
            return Ok((
                wasm_store,
                Some(ft_sys_shared::Request {
                    uri: "server-error".to_string(),
                    method: "404".to_string(),
                    headers: vec![],
                    body: format!("no endpoint found for {path}: {e:?}").into_bytes(),
                }),
            ));
        }
    };
    main.call_async(&mut wasm_store, ()).await?;

    Ok((wasm_store, None))
}

pub fn get_entrypoint<S: Send>(
    instance: wasmtime::Instance,
    mut store: wasmtime::Store<S>,
    path: &str,
) -> (
    wasmtime::Store<S>,
    wasmtime::Result<wasmtime::TypedFunc<(), ()>>,
) {
    let entrypoint = match path_to_entrypoint(path) {
        Ok(v) => v,
        Err(e) => return (store, Err(e)),
    };
    let r = instance.get_typed_func(&mut store, entrypoint.as_str());
    (store, r)
}

#[derive(Debug, thiserror::Error)]
pub enum PathToEndpointError {
    #[error("no wasm file found in path")]
    NoWasm,
}

#[derive(Debug, thiserror::Error)]
pub enum WasmError {
    #[error("endpoint did not return response")]
    EndpointDidNotReturnResponse,
}

pub fn path_to_entrypoint(path: &str) -> wasmtime::Result<String> {
    let path = path.split_once('?').map(|(f, _)| f).unwrap_or(path);
    match path.split_once(".wasm/") {
        Some((_, l)) => {
            let l = l.trim_end_matches('/').replace('/', "_");
            Ok(l.trim_end_matches('/').replace('-', "_") + "__entrypoint")
        }
        None => Err(PathToEndpointError::NoWasm.into()),
    }
}
