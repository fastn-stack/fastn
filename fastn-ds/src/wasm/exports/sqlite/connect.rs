pub async fn connect(
    mut caller: wasmtime::Caller<'_, fastn_ds::wasm::Store>,
    ptr: i32,
    len: i32,
) -> wasmtime::Result<i32> {
    let db_url = fastn_ds::wasm::helpers::get_str(ptr, len, &mut caller).await?;
    println!("sqlite_connect: {db_url}");

    todo!()
}
