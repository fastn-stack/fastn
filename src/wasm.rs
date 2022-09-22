#[derive(Default)]
pub struct HostExports {}

pub struct Context<I, E> {
    pub imports: I,
    pub exports: E,
}

pub async fn handle_wasm(
    req: &actix_web::HttpRequest,
    wasm_module: camino::Utf8PathBuf,
) -> actix_web::Result<actix_web::HttpResponse> {
    pub async fn inner(
        req: &actix_web::HttpRequest,
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
    fpm::time("WASM Execution: ").it(inner(req, wasm_module).await)
}
