use std::str::FromStr;

#[derive(Default)]
pub struct HostExports {}

impl fastn_utils::backend_host_export::host::Host for HostExports {
    fn http(
        &mut self,
        request: fastn_utils::backend_host_export::host::Httprequest<'_>,
    ) -> fastn_utils::backend_host_export::host::Httpresponse {
        let url = request.path.to_string();
        let request_method = request.method.to_string();
        let request_body = request.payload.to_string();
        let mut headers = reqwest::header::HeaderMap::new();
        request
            .headers
            .clone()
            .into_iter()
            .for_each(|(header_key, header_val)| {
                headers.insert(
                    reqwest::header::HeaderName::from_str(header_key).unwrap(),
                    reqwest::header::HeaderValue::from_str(header_val).unwrap(),
                );
            });
        let resp = std::thread::spawn(move || {
            let request_client = reqwest::blocking::Client::new();
            match request_method.as_str() {
                "GET" => request_client.get(url).headers(headers),
                "POST" => request_client.post(url).headers(headers).body(request_body),
                "PATCH" => request_client
                    .patch(url)
                    .headers(headers)
                    .body(request_body),
                _ => panic!("METHOD not allowed"),
            }
            .send()
            .unwrap()
            .text()
            .unwrap()
        })
        .join()
        .unwrap();
        fastn_utils::backend_host_export::host::Httpresponse { data: resp }
    }
}

pub struct Context<I, E> {
    pub imports: I,
    pub exports: E,
}

#[derive(thiserror::Error, Debug)]
pub enum WASMError {
    #[error("Wasmtime Error: {}", _0)]
    WasmTime(#[from] wit_bindgen_host_wasmtime_rust::anyhow::Error),

    #[error("JSON Parsing Error: {}", _0)]
    SerdeJson(#[from] serde_json::Error),

    #[error("WasmFunctionInvokeError: {}", _0)]
    WasmFunctionInvoke(String),
}

pub type WasmRunnerResult<T> = std::result::Result<T, WASMError>;

pub async fn handle_wasm(
    req: fastn::http::Request,
    wasm_module: camino::Utf8PathBuf,
    backend_headers: Option<Vec<fastn::package::BackendHeader>>,
) -> fastn::http::Response {
    pub async fn inner(
        req: fastn::http::Request,
        wasm_module: camino::Utf8PathBuf,
        backend_headers: Option<Vec<fastn::package::BackendHeader>>,
    ) -> WasmRunnerResult<actix_web::HttpResponse> {
        let mut wasm_config = wit_bindgen_host_wasmtime_rust::wasmtime::Config::new();
        wasm_config.cache_config_load_default().unwrap();
        wasm_config.wasm_backtrace_details(
            wit_bindgen_host_wasmtime_rust::wasmtime::WasmBacktraceDetails::Disable,
        );

        let engine = wit_bindgen_host_wasmtime_rust::wasmtime::Engine::new(&wasm_config)?;
        let module = wit_bindgen_host_wasmtime_rust::wasmtime::Module::from_file(
            &engine,
            wasm_module.as_str(),
        )?;

        let mut linker: wit_bindgen_host_wasmtime_rust::wasmtime::Linker<
            fastn::wasm::Context<
                fastn::wasm::HostExports,
                fastn_utils::backend_host_import::guest_backend::GuestBackendData,
            >,
        > = wit_bindgen_host_wasmtime_rust::wasmtime::Linker::new(&engine);
        let mut store = wit_bindgen_host_wasmtime_rust::wasmtime::Store::new(
            &engine,
            fastn::wasm::Context {
                imports: fastn::wasm::HostExports {},
                exports: fastn_utils::backend_host_import::guest_backend::GuestBackendData {},
            },
        );

        fastn_utils::backend_host_export::host::add_to_linker(&mut linker, |cx| &mut cx.imports)?;

        let (import, _i) =
            fastn_utils::backend_host_import::guest_backend::GuestBackend::instantiate(
                &mut store,
                &module,
                &mut linker,
                |cx| &mut cx.exports,
            )?;

        let uri = req.uri().to_string();
        // TODO: Fix body
        let b = req.body().to_vec();
        let body_str = if let Ok(b) = std::str::from_utf8(&b) {
            b
        } else {
            ""
        };
        let mut headers = vec![];

        req.headers()
            .iter()
            .for_each(|(header_name, header_value)| {
                headers.push((
                    header_name.as_str().to_string(),
                    header_value
                        .to_str()
                        .expect("Unable to parse header value")
                        .to_string(),
                ));
            });
        if let Some(b_headers) = backend_headers {
            b_headers.into_iter().for_each(|header| {
                let hk = header.header_key;
                headers.push((format!("X-fastn-{hk}"), header.header_value));
            })
        };
        let headers: Vec<(&str, &str)> = headers
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        let request = fastn_utils::backend_host_import::guest_backend::Httprequest {
            path: uri.as_str(),
            headers: &(headers)[..],
            querystring: req.query_string(),
            method: req.method(),
            payload: body_str,
        };
        fastn::time("WASM Guest function").it(match import.handlerequest(&mut store, request) {
            Ok(data) => Ok(actix_web::HttpResponse::Ok()
                .content_type(actix_web::http::header::ContentType::json())
                .status(if data.success {
                    actix_web::http::StatusCode::OK
                } else {
                    actix_web::http::StatusCode::BAD_REQUEST
                })
                .body(data.data)),
            Err(err) => Err(WASMError::WasmFunctionInvoke(err.to_string())),
        })
    }
    fastn::time("WASM Execution: ").it(match inner(req, wasm_module, backend_headers).await {
        Ok(resp) => resp,
        Err(err) => fastn::server_error!("{}", err.to_string()),
    })
}
