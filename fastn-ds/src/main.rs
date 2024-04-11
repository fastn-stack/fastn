#[tokio::main]
async fn main() {
    let req = ft_sys_shared::Request {
        uri: "/".to_string(),
        method: "get".to_string(),
        headers: vec![],
        body: vec![],
    };

    let module = wasmtime::Module::from_binary(
        &fastn_ds::WASM_ENGINE,
        &tokio::fs::read(
            "../../ft-sdk/sample-wasm/target/wasm32-unknown-unknown/release/sample_wasm.wasm",
        )
        .await
        .unwrap(),
    )
    .unwrap();

    let resp =
        fastn_ds::wasm::process_http_request(req, None, module, Default::default(), "".to_string())
            .await
            .unwrap();

    println!("{:?}", resp);
}
