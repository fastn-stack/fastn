use std::str::FromStr;

#[derive(Default)]
pub struct HostExports {}

impl fpm_utils::backend_host_export::host::Host for HostExports {
    fn http(
        &mut self,
        request: fpm_utils::backend_host_export::host::Httprequest<'_>,
    ) -> fpm_utils::backend_host_export::host::Httpresponse {
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
                _ => panic!(""),
            }
            .send()
            .unwrap()
            .text()
            .unwrap()
        })
        .join()
        .unwrap();
        fpm_utils::backend_host_export::host::Httpresponse { data: resp }
    }
}

pub struct Context<I, E> {
    pub imports: I,
    pub exports: E,
}

pub async fn handle_wasm(
    req: &actix_web::HttpRequest,
    body: actix_web::web::Bytes, // TODO: Not liking it, It should be fetched from request only
    wasm_module: camino::Utf8PathBuf,
) -> actix_web::Result<actix_web::HttpResponse> {
    pub async fn inner(
        req: &actix_web::HttpRequest,
        body: actix_web::web::Bytes, // TODO: Not liking it, It should be fetched from request only
        wasm_module: camino::Utf8PathBuf,
    ) -> actix_web::Result<actix_web::HttpResponse> {
        let mut wasm_config = wit_bindgen_host_wasmtime_rust::wasmtime::Config::new();
        wasm_config.cache_config_load_default().unwrap();
        wasm_config.wasm_backtrace_details(
            wit_bindgen_host_wasmtime_rust::wasmtime::WasmBacktraceDetails::Disable,
        );

        let engine = wit_bindgen_host_wasmtime_rust::wasmtime::Engine::new(&wasm_config).unwrap();
        let module = wit_bindgen_host_wasmtime_rust::wasmtime::Module::from_file(
            &engine,
            wasm_module.as_str(),
        )
        .unwrap();

        let mut linker: wit_bindgen_host_wasmtime_rust::wasmtime::Linker<
            fpm::wasm::Context<
                fpm::wasm::HostExports,
                fpm_utils::backend_host_import::guest_backend::GuestBackendData,
            >,
        > = wit_bindgen_host_wasmtime_rust::wasmtime::Linker::new(&engine);
        let mut store = wit_bindgen_host_wasmtime_rust::wasmtime::Store::new(
            &engine,
            fpm::wasm::Context {
                imports: fpm::wasm::HostExports {},
                exports: fpm_utils::backend_host_import::guest_backend::GuestBackendData {},
            },
        );

        fpm_utils::backend_host_export::host::add_to_linker(&mut linker, |cx| &mut cx.imports)
            .unwrap();

        let (import, _i) =
            fpm_utils::backend_host_import::guest_backend::GuestBackend::instantiate(
                &mut store,
                &module,
                &mut linker,
                |cx| &mut cx.exports,
            )
            .expect("Unable to run");
        // TODO: Handle the error efficiently

        let uri = req.uri().to_string();
        let b = body.to_vec();
        let body_str = if let Ok(b) = std::str::from_utf8(&b) {
            b
        } else {
            ""
        };
        let request = fpm_utils::backend_host_import::guest_backend::Httprequest {
            path: uri.as_str(),
            headers: &(&req.headers().iter().fold(
                vec![],
                |mut accumulator, (header_name, header_value)| {
                    accumulator.push((header_name.as_str(), header_value.to_str().expect("msg")));
                    accumulator
                },
            ))[..],
            querystring: req.query_string(),
            method: req.method().as_str(),
            payload: body_str,
        };
        fpm::time("WASM Guest function").it(match import.handlerequest(&mut store, request) {
            Ok(data) => Ok(actix_web::HttpResponse::Ok()
                .content_type(actix_web::http::header::ContentType::json())
                .status(actix_web::http::StatusCode::OK)
                .body(data)),
            Err(err) => fpm::apis::error(
                err.to_string(),
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            ),
        })
    }
    fpm::time("WASM Execution: ").it(inner(req, body, wasm_module).await)
}
