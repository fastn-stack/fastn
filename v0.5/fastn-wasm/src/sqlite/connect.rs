pub async fn connect<T: fastn_wasm::StoreExt>(
    mut caller: wasmtime::Caller<'_, fastn_wasm::Store<T>>,
    ptr: i32,
    len: i32,
) -> wasmtime::Result<i32> {
    let db_url = fastn_wasm::helpers::get_str(ptr, len, &mut caller)?;
    println!("sqlite_connect: {db_url}");
    let store = caller.data_mut();
    // fastn_wasm::StoreExt::sqlite_connect(store, db_url.as_str()).await
    store.inner.clone().sqlite_connect(store, db_url.as_str())
    // store.inner.sqlite_connect(store, db_url.as_str())
}
