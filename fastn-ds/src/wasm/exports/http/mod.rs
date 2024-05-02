mod get_request;
mod send_request;
mod send_response;

pub use get_request::get_request;
pub use send_request::send_request;
pub use send_response::send_response;

pub async fn send_ftd(
    mut caller: wasmtime::Caller<'_, fastn_ds::wasm::Store>,
    ptr: i32,
    len: i32,
) -> wasmtime::Result<()> {
    let r = fastn_ds::wasm::helpers::get_json(ptr, len, &mut caller)?;
    caller
        .data_mut()
        .store_response(fastn_ds::wasm::Response::Ftd(r));
    Ok(())
}
