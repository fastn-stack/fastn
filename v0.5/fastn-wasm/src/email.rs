pub async fn send<S: Send>(
    mut caller: wasmtime::Caller<'_, S>,
    ptr: i32,
    len: i32,
) -> wasmtime::Result<i32> {
    let e: ft_sys_shared::Email = fastn_wasm::helpers::get_json(ptr, len, &mut caller)?;
    tracing::info!("sending email: {:?}: {}", e.to, e.subject);
    let response = ft_sys_shared::EmailHandle::new("yo yo".to_string());
    fastn_wasm::helpers::send_json(response, &mut caller).await
}

pub async fn cancel<S: Send>(
    mut caller: wasmtime::Caller<'_, S>,
    ptr: i32,
    len: i32,
) -> wasmtime::Result<()> {
    let e: ft_sys_shared::EmailHandle = fastn_wasm::helpers::get_json(ptr, len, &mut caller)?;
    tracing::info!("cancelling email: {}", e.inner());
    Ok(())
}
