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
) -> Result<((), wasmtime::Store<S>), ApplyMigrationError> {
    let ep = match instance.get_typed_func::<(), i32>(&mut store, "migration__entrypoint") {
        Ok(v) => v,
        Err(e) => {
            println!("failed to get migration__entrypoint ({e}), proceeding without migration");
            return Ok(((), store));
        }
    };

    let i = ep.call_async(&mut store, ()).await?;
    if i != 0 {
        return Err(ApplyMigrationError::MigrationFailed(i));
    }

    Ok(((), store))
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
