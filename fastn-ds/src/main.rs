#[tokio::main]
async fn main() {
    let req = ft_sys_shared::Request {
        uri: "/".to_string(),
        method: "get".to_string(),
        headers: vec![],
        body: vec![],
    };

    let module = wasmtime::Module::from_binary(
        &fastn_wasm::WASM_ENGINE,
        &tokio::fs::read(
            "../../ft-sdk/sample-wasm/target/wasm32-unknown-unknown/release/sample_wasm.wasm",
        )
        .await
        .unwrap(),
    )
    .unwrap();

    let store = fastn_wasm::Store::new(
        req,
        Default::default(),
        "".to_string(),
        fastn_wasm::StoreImpl,
    );
    let resp = fastn_wasm::process_http_request("/", module, store)
        .await
        .unwrap();

    println!("{:?}", resp);
}
