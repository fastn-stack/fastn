pub async fn handle<S: Send>(
    mut wasm_store: wasmtime::Store<S>,
    module: wasmtime::Module,
    linker: wasmtime::Linker<S>,
    path: String,
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

    let (wasm_store, r) = crate::apply_migration(instance, wasm_store).await;

    if let Err(e) = r {
        return Ok((
            wasm_store,
            Some(ft_sys_shared::Request::server_error(format!(
                "failed to apply migration: {e:?}"
            ))),
        ));
    };

    let (main, mut wasm_store) = crate::get_entrypoint(instance, wasm_store, path)?;
    main.call_async(&mut wasm_store, ()).await?;

    Ok((wasm_store, None))
}

#[derive(Debug, thiserror::Error)]
pub enum ApplyMigrationError {
    #[error("failed to get migration__entrypoint: {0}")]
    GetMigrationEntrypoint(#[from] wasmtime::Error),
    #[error("migration failed: {0}")]
    MigrationFailed(i32),
}

pub async fn apply_migration<S: Send>(
    instance: wasmtime::Instance,
    mut store: wasmtime::Store<S>,
) -> (wasmtime::Store<S>, Result<(), ApplyMigrationError>) {
    let ep = match instance.get_typed_func::<(), i32>(&mut store, "migration__entrypoint") {
        Ok(v) => v,
        Err(e) => {
            println!("failed to get migration__entrypoint ({e}), proceeding without migration");
            return (store, Ok(()));
        }
    };

    let i = match ep.call_async(&mut store, ()).await {
        Ok(v) => v,
        Err(e) => return (store, Err(e.into())),
    };
    if i != 0 {
        return (store, Err(ApplyMigrationError::MigrationFailed(i)));
    }

    (store, Ok(()))
}

pub fn get_entrypoint<S: Send>(
    instance: wasmtime::Instance,
    mut store: wasmtime::Store<S>,
    path: String,
) -> wasmtime::Result<(wasmtime::TypedFunc<(), ()>, wasmtime::Store<S>)> {
    if let Ok(f) = instance.get_typed_func::<(), ()>(&mut store, "main_ft") {
        return Ok((f, store));
    }
    let entrypoint = path_to_entrypoint(path.as_str())?;
    println!("main_ft not found, trying {entrypoint}");
    instance
        .get_typed_func(&mut store, entrypoint.as_str())
        .map(|v| (v, store))
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
