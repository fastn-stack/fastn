static LOCK: once_cell::sync::Lazy<async_lock::RwLock<()>> =
    once_cell::sync::Lazy::new(|| async_lock::RwLock::new(()));

#[tracing::instrument(skip_all)]
fn handle_redirect(
    config: &fastn_core::Config,
    path: &camino::Utf8Path,
) -> Option<fastn_core::http::Response> {
    config
        .package
        .redirects
        .as_ref()
        .and_then(|v| fastn_core::package::redirects::find_redirect(v, path.as_str()))
        .map(|r| fastn_core::http::redirect(r.to_string()))
}

/// path: /-/<package-name>/<file-name>/
/// path: /<file-name>/
#[tracing::instrument(skip_all)]
async fn serve_file(
    config: &mut fastn_core::RequestConfig,
    path: &camino::Utf8Path,
    only_js: bool,
) -> fastn_core::http::Response {
    if let Some(r) = handle_redirect(&config.config, path) {
        return r;
    }

    let path = <&camino::Utf8Path>::clone(&path);

    if let Err(e) = config
        .config
        .package
        .auto_import_language(config.request.cookie("fastn-lang"), None)
    {
        return if !config.config.test_command_running {
            fastn_core::not_found!("fastn-Error: path: {}, {:?}", path, e)
        } else {
            fastn_core::http::not_found_without_warning(format!(
                "fastn-Error: path: {}, {:?}",
                path, e
            ))
        };
    }

    let f = match config.get_file_and_package_by_id(path.as_str()).await {
        Ok(f) => f,
        Err(e) => {
            tracing::error!(
                msg = "fastn-error path not found",
                path = path.as_str(),
                error = %e
            );
            return if !config.config.test_command_running {
                fastn_core::not_found!("fastn-Error: path: {}, {:?}", path, e)
            } else {
                fastn_core::http::not_found_without_warning(format!(
                    "fastn-Error: path: {}, {:?}",
                    path, e
                ))
            };
        }
    };

    // Auth Stuff
    if !f.is_static() {
        match config.can_read(path.as_str(), true).await {
            Ok(can_read) => {
                if !can_read {
                    tracing::error!(
                        msg = "unauthorized-error: can not read",
                        path = path.as_str()
                    );
                    return fastn_core::unauthorised!("You are unauthorized to access: {}", path);
                }
            }
            Err(e) => {
                tracing::error!(msg = "can_read-error", path = path.as_str());
                return fastn_core::server_error!("fastn-Error: can_read error: {}, {:?}", path, e);
            }
        };

        match fastn_core::package::app::can_read(config, path.as_str()).await {
            Ok(can_read) => {
                if !can_read {
                    tracing::error!(
                        msg = "unauthorized-error: can not access app",
                        path = path.as_str()
                    );
                    return fastn_core::unauthorised!("You are unauthorized to access: {}", path);
                }
            }
            Err(err) => {
                tracing::error!(
                    msg = "app::can_read-error: can not access app",
                    path = path.as_str()
                );
                return fastn_core::server_error!(
                    "fastn-Error: can_read error: {}, {:?}",
                    path,
                    err
                );
            }
        };
    }

    let main_document = match f {
        fastn_core::File::Ftd(main_document) => main_document,
        _ => {
            tracing::error!(msg = "unknown handler", path = path.as_str());
            return fastn_core::server_error!("unknown handler");
        }
    };

    if fastn_core::utils::is_ftd_path(path.as_str()) {
        return fastn_core::http::not_found_without_warning(
            "we do not serve ftd file source".to_string(),
        );
    }

    match fastn_core::package::package_doc::read_ftd_(
        config,
        &main_document,
        "/",
        false,
        false,
        only_js,
    )
    .await
    {
        Ok(r) => r.into(),
        Err(e) => {
            tracing::error!(
                msg = "fastn-Error",
                path = path.as_str(),
                error = e.to_string()
            );
            fastn_core::server_error!("fastn-Error: path: {}, {:?}", path, e)
        }
    }
}

fn guess_mime_type(path: &str) -> mime_guess::Mime {
    mime_guess::from_path(path).first_or_octet_stream()
}

#[tracing::instrument(skip_all)]
async fn serve_fastn_file(config: &fastn_core::Config) -> fastn_core::http::Response {
    let response = match config
        .ds
        .read_content(
            &config
                .get_root_for_package(&config.package)
                .join("FASTN.ftd"),
        )
        .await
    {
        Ok(res) => res,
        Err(e) => {
            return fastn_core::not_found!("fastn-Error: path: FASTN.ftd error: {:?}", e);
        }
    };
    fastn_core::http::ok_with_content_type(response, mime_guess::mime::APPLICATION_OCTET_STREAM)
}

#[tracing::instrument(skip_all)]
pub async fn serve(
    config: &fastn_core::Config,
    req: fastn_core::http::Request,
) -> fastn_core::Result<fastn_core::http::Response> {
    if let Some(endpoint_response) = handle_endpoints(config, &req).await {
        return endpoint_response;
    }

    if let Some(default_response) = handle_default_route(&req, config.package.name.as_str()) {
        return default_response;
    }

    if let Some(static_response) =
        handle_static_route(req.path(), config.package.name.as_str(), &config.ds).await
    {
        return static_response;
    }

    serve_helper(config, req, false).await
}

#[tracing::instrument(skip_all)]
pub async fn serve_helper(
    config: &fastn_core::Config,
    req: fastn_core::http::Request,
    only_js: bool,
) -> fastn_core::Result<fastn_core::http::Response> {
    let _lock = LOCK.read().await;

    let mut req_config = fastn_core::RequestConfig::new(config, &req, "", "/");
    let path: camino::Utf8PathBuf = req.path().replacen('/', "", 1).parse()?;

    let mut resp = if path.eq(&camino::Utf8PathBuf::new().join("FASTN.ftd")) {
        serve_fastn_file(config).await
    } else if path.eq(&camino::Utf8PathBuf::new().join("")) {
        serve_file(&mut req_config, &path.join("/"), only_js).await
    } else {
        // url is present in config or not
        // If not present than proxy pass it

        let req_method = req.method().to_string();
        let query_string = req.query_string().to_string();

        // if start with -/ and mount-point exists so send redirect to mount-point
        // We have to do -/<package-name>/remaining-url/ ==> (<package-name>, remaining-url) ==> (/config.package-name.mount-point/remaining-url/)
        // Get all the dependencies with mount-point if path_start with any package-name so send redirect to mount-point
        // fastn_core::file::is_static: checking for static file, if file is static no need to redirect it.
        if req_method.as_str() == "GET" && !fastn_core::file::is_static(path.as_str())? {
            // if any app name starts with package-name to redirect it to /mount-point/remaining-url/
            for (mp, dep) in config
                .package
                .apps
                .iter()
                .map(|x| (&x.mount_point, &x.package))
            {
                if let Some(remaining_path) =
                    fastn_core::config::utils::trim_package_name(path.as_str(), dep.name.as_str())
                {
                    let path = if remaining_path.trim_matches('/').is_empty() {
                        format!("/{}/", mp.trim().trim_matches('/'))
                    } else if query_string.is_empty() {
                        format!(
                            "/{}/{}/",
                            mp.trim().trim_matches('/'),
                            remaining_path.trim_matches('/')
                        )
                    } else {
                        format!(
                            "/{}/{}/?{}",
                            mp.trim().trim_matches('/'),
                            remaining_path.trim_matches('/'),
                            query_string.as_str()
                        )
                    };

                    let mut resp = actix_web::HttpResponse::new(
                        actix_web::http::StatusCode::PERMANENT_REDIRECT,
                    );
                    resp.headers_mut().insert(
                        actix_web::http::header::LOCATION,
                        actix_web::http::header::HeaderValue::from_str(path.as_str()).unwrap(), // TODO:
                    );
                    return Ok(resp);
                }
            }
        }

        let file_response = serve_file(&mut req_config, path.as_path(), only_js).await;
        // If path is not present in sitemap then pass it to proxy
        // TODO: Need to handle other package URL as well, and that will start from `-`
        // and all the static files starts with `-`
        // So task is how to handle proxy urls which are starting with `-/<package-name>/<proxy-path>`
        // In that case need to check whether that url is present in sitemap or not and than proxy
        // pass the url if the file is not static
        // So final check would be file is not static and path is not present in the package's sitemap
        tracing::info!(
            "before executing proxy: file-status: {}, path: {}",
            file_response.status(),
            &path
        );

        // Fallback to WASM execution in case of no successful response
        // TODO: This is hacky. Use the sitemap eventually.
        // if file_response.status() == actix_web::http::StatusCode::NOT_FOUND {
        //     let package = config.find_package_by_id(path.as_str()).await.unwrap().1;
        //     let wasm_module = config.get_root_for_package(&package).join("backend.wasm");
        //     // Set the temp path to and do `get_file_and_package_by_id` to retrieve the backend.wasm
        //     // File for the particular dependency package
        //     let wasm_module_path = format!("-/{}/backend.wasm", package.name,);
        //     config
        //         .get_file_and_package_by_id(wasm_module_path.as_str())
        //         .await?;
        //     let req = if let Some(r) = config.request {
        //         r
        //     } else {
        //         return Ok(fastn_core::server_error!("request not set"));
        //     };
        //     fastn_core::wasm::handle_wasm(req, wasm_module, config.package.backend_headers).await
        // }

        file_response
    };

    for cookie in req_config.processor_set_cookies {
        resp.headers_mut().append(
            actix_web::http::header::SET_COOKIE,
            actix_web::http::header::HeaderValue::from_str(cookie.as_str()).unwrap(),
        );
    }
    Ok(resp)
}

pub(crate) async fn download_init_package(url: &Option<String>) -> std::io::Result<()> {
    let mut package = fastn_core::Package::new("unknown-package");
    package.download_base_url = url.to_owned();
    let current_dir = camino::Utf8PathBuf::from_path_buf(std::env::current_dir()?)
        .expect("fastn-Error: Unable to change path");
    let ds = fastn_ds::DocumentStore::new(current_dir);
    package
        .http_download_by_id("FASTN.ftd", Some(&ds.root()), &ds)
        .await
        .expect("Unable to find FASTN.ftd file");
    Ok(())
}

pub async fn clear_cache(
    config: &fastn_core::Config,
    req: fastn_core::http::Request,
) -> fastn_core::Result<fastn_core::http::Response> {
    #[cfg(feature = "auth")]
    fn is_login(req: &fastn_core::http::Request) -> bool {
        // TODO: Need refactor not happy with this
        req.cookie(fastn_core::auth::AuthProviders::GitHub.as_str())
            .is_some()
    }

    // TODO: Remove After Demo, Need to think about refresh content from github
    #[derive(serde::Deserialize)]
    struct Temp {
        from: Option<String>,
    }
    let from = actix_web::web::Query::<Temp>::from_query(req.query_string())?;
    if from.from.eq(&Some("temp-github".to_string())) {
        let _lock = LOCK.write().await;
        return Ok(fastn_core::apis::cache::clear(config, &req).await);
    }
    // TODO: Remove After Demo, till here

    #[cfg(feature = "auth")]
    if !is_login(&req) {
        return Ok(actix_web::HttpResponse::Found()
            .append_header((
                actix_web::http::header::LOCATION,
                "/auth/login/?platform=github".to_string(),
            ))
            .finish());
    }

    let _lock = LOCK.write().await;
    fastn_core::apis::cache::clear(config, &req).await;
    // TODO: Redirect to Referrer uri
    return Ok(actix_web::HttpResponse::Found()
        .append_header((actix_web::http::header::LOCATION, "/".to_string()))
        .finish());
}

#[derive(serde::Deserialize, Default, Clone)]
pub(crate) struct AppData {
    pub(crate) edition: Option<String>,
    pub(crate) external_js: Vec<String>,
    pub(crate) inline_js: Vec<String>,
    pub(crate) external_css: Vec<String>,
    pub(crate) inline_css: Vec<String>,
}

pub fn handle_default_route(
    req: &fastn_core::http::Request,
    package_name: &str,
) -> Option<fastn_core::Result<fastn_core::http::Response>> {
    if req
        .path()
        .ends_with(fastn_core::utils::hashed_default_css_name())
    {
        return Some(Ok(actix_web::HttpResponse::Ok()
            .content_type(mime_guess::mime::TEXT_CSS)
            .append_header(("Cache-Control", "public, max-age=31536000"))
            .body(ftd::css())));
    } else if req
        .path()
        .ends_with(fastn_core::utils::hashed_default_js_name())
    {
        return Some(Ok(actix_web::HttpResponse::Ok()
            .content_type(mime_guess::mime::TEXT_JAVASCRIPT)
            .append_header(("Cache-Control", "public, max-age=31536000"))
            .body(format!(
                "{}\n\n{}",
                ftd::build_js(),
                fastn_core::fastn_2022_js()
            ))));
    } else if req
        .path()
        .ends_with(fastn_core::utils::hashed_default_ftd_js(package_name))
    {
        return Some(Ok(actix_web::HttpResponse::Ok()
            .content_type(mime_guess::mime::TEXT_JAVASCRIPT)
            .append_header(("Cache-Control", "public, max-age=31536000"))
            .body(ftd::js::all_js_without_test(package_name))));
    } else if req
        .path()
        .ends_with(fastn_core::utils::hashed_markdown_js())
    {
        return Some(Ok(actix_web::HttpResponse::Ok()
            .content_type(mime_guess::mime::TEXT_JAVASCRIPT)
            .append_header(("Cache-Control", "public, max-age=31536000"))
            .body(ftd::markdown_js())));
    } else if let Some(theme) =
        fastn_core::utils::hashed_code_theme_css()
            .iter()
            .find_map(|(theme, url)| {
                if req.path().ends_with(url) {
                    Some(theme)
                } else {
                    None
                }
            })
    {
        let theme_css = ftd::theme_css();
        return theme_css.get(theme).cloned().map(|theme| {
            Ok(actix_web::HttpResponse::Ok()
                .content_type(mime_guess::mime::TEXT_CSS)
                .append_header(("Cache-Control", "public, max-age=31536000"))
                .body(theme))
        });
    } else if req.path().ends_with(fastn_core::utils::hashed_prism_js()) {
        return Some(Ok(actix_web::HttpResponse::Ok()
            .content_type(mime_guess::mime::TEXT_JAVASCRIPT)
            .append_header(("Cache-Control", "public, max-age=31536000"))
            .body(ftd::prism_js())));
    } else if req.path().ends_with(fastn_core::utils::hashed_prism_css()) {
        return Some(Ok(actix_web::HttpResponse::Ok()
            .content_type(mime_guess::mime::TEXT_CSS)
            .append_header(("Cache-Control", "public, max-age=31536000"))
            .body(ftd::prism_css())));
    }

    None
}

#[tracing::instrument(skip_all)]
async fn foo() {
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
}

#[tracing::instrument(skip_all)]
async fn test() -> fastn_core::Result<fastn_core::http::Response> {
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    foo().await;
    Ok(actix_web::HttpResponse::Ok().finish())
}

async fn handle_static_route(
    path: &str,
    package_name: &str,
    ds: &fastn_ds::DocumentStore,
) -> Option<fastn_core::Result<fastn_core::http::Response>> {
    return Some(match handle_static_route_(path, package_name, ds).await? {
        Ok(r) => Ok(r),
        Err(fastn_ds::ReadError::NotFound) => {
            Ok(fastn_core::http::not_found_without_warning("".to_string()))
        }
        Err(e) => Err(e.into()),
    });

    async fn handle_static_route_(
        path: &str,
        package_name: &str,
        ds: &fastn_ds::DocumentStore,
    ) -> Option<Result<fastn_core::http::Response, fastn_ds::ReadError>> {
        if path == "/favicon.ico" {
            return Some(favicon(ds).await);
        }

        if !fastn_core::utils::is_static_path(path) {
            return None;
        }

        // the path can start with slash or -/. If later, it is a static file from our dependencies, so
        // we have to look for them inside .packages.
        let path = match path.strip_prefix("/-/") {
            Some(path) if path.starts_with(package_name) => {
                path.strip_prefix(package_name).unwrap_or(path).to_string()
            }
            Some(path) => format!(".packages/{path}"),
            None => path.to_string(),
        };

        Some(
            static_file(ds, path.strip_prefix('/').unwrap_or(path.as_str()))
                .await
                .map_err(Into::into),
        )
    }

    async fn favicon(
        ds: &fastn_ds::DocumentStore,
    ) -> Result<fastn_core::http::Response, fastn_ds::ReadError> {
        match static_file(ds, "favicon.ico").await {
            Ok(r) => Ok(r),
            Err(fastn_ds::ReadError::NotFound) => Ok(static_file(ds, "static/favicon.ico").await?),
            Err(e) => Err(e),
        }
    }

    #[tracing::instrument(skip(ds))]
    async fn static_file(
        ds: &fastn_ds::DocumentStore,
        path: &str,
    ) -> Result<fastn_core::http::Response, fastn_ds::ReadError> {
        ds.read_content(&fastn_ds::Path::new(path)).await.map(|r| {
            fastn_core::http::ok_with_content_type(r, guess_mime_type(path.to_string().as_str()))
        })
    }
}

async fn handle_endpoints(
    config: &fastn_core::Config,
    req: &fastn_core::http::Request,
) -> Option<fastn_core::Result<fastn_core::http::Response>> {
    let mut endpoint = None;

    for ep in config.package.endpoints.iter() {
        if !req.path().starts_with(ep.mountpoint.as_str()) {
            continue;
        }
        endpoint = Some(ep.clone());
        break;
    }

    let endpoint = match endpoint {
        Some(e) => e,
        None => return None,
    };

    Some(
        fastn_core::proxy::get_out(
            url::Url::parse(
                format!(
                    "{}/{}",
                    endpoint.endpoint.trim_end_matches('/'),
                    req.path().trim_start_matches('/')
                )
                .as_str(),
            )
            .unwrap(),
            req,
            &std::collections::HashMap::new(),
        )
        .await,
    )
}

#[tracing::instrument(skip_all)]
async fn actual_route(
    config: &fastn_core::Config,
    req: actix_web::HttpRequest,
    body: actix_web::web::Bytes,
) -> fastn_core::Result<fastn_core::http::Response> {
    tracing::info!(method = req.method().as_str(), uri = req.path());

    let req = fastn_core::http::Request::from_actix(req, body);

    match (req.method().to_lowercase().as_str(), req.path()) {
        #[cfg(feature = "auth")]
        (_, t) if t.starts_with("/-/auth/") => fastn_core::auth::routes::handle_auth(req).await,
        ("get", "/-/clear-cache/") => clear_cache(config, req).await,
        ("get", "/-/poll/") => fastn_core::watcher::poll().await,
        ("get", "/test/") => test().await,
        (_, _) => serve(config, req).await,
    }
}

#[tracing::instrument(skip_all)]
async fn route(
    req: actix_web::HttpRequest,
    body: actix_web::web::Bytes,
    app_data: actix_web::web::Data<AppData>,
) -> fastn_core::Result<fastn_core::http::Response> {
    let config = fastn_core::Config::read_current(false)
        .await?
        .add_edition(app_data.edition.clone())?
        .add_external_js(app_data.external_js.clone())
        .add_inline_js(app_data.inline_js.clone())
        .add_external_css(app_data.external_css.clone())
        .add_inline_css(app_data.inline_css.clone());

    actual_route(&config, req, body).await
}

//noinspection HttpUrlsUsage
#[allow(clippy::too_many_arguments)]
pub async fn listen(
    bind_address: &str,
    port: Option<u16>,
    package_download_base_url: Option<String>,
    edition: Option<String>,
    external_js: Vec<String>,
    inline_js: Vec<String>,
    external_css: Vec<String>,
    inline_css: Vec<String>,
) -> fastn_core::Result<()> {
    use colored::Colorize;
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    if package_download_base_url.is_some() {
        download_init_package(&package_download_base_url).await?;
    }

    let tcp_listener = match fastn_core::http::get_available_port(port, bind_address) {
        Some(listener) => listener,
        None => {
            eprintln!(
                "{}",
                port.map(|x| format!(
                    r#"Provided port {} is not available.

You can try without providing port, it will automatically pick unused port."#,
                    x.to_string().red()
                ))
                .unwrap_or_else(|| {
                    "Tried picking port between port 8000 to 9000, none are available :-("
                        .to_string()
                })
            );
            std::process::exit(2);
        }
    };

    #[cfg(feature = "auth")]
    if let Ok(auth_enabled) = std::env::var("FASTN_ENABLE_AUTH") {
        if auth_enabled == "true" {
            tracing::info!("running auth related migrations");
            fastn_core::auth::enable_auth()?
        }
    }

    let app = move || {
        actix_web::App::new()
            .app_data(actix_web::web::Data::new(AppData {
                edition: edition.clone(),
                external_js: external_js.clone(),
                inline_js: inline_js.clone(),
                external_css: external_css.clone(),
                inline_css: inline_css.clone(),
            }))
            .wrap(actix_web::middleware::Compress::default())
            .wrap(fastn_core::catch_panic::CatchPanic::default())
            .wrap(
                actix_web::middleware::Logger::new(
                    r#""%r" %Ts %s %b %a "%{Referer}i" "%{User-Agent}i""#,
                )
                .log_target(""),
            )
            .route("/{path:.*}", actix_web::web::route().to(route))
    };

    println!("### Server Started ###");
    println!(
        "Go to: http://{}:{}",
        bind_address,
        tcp_listener.local_addr()?.port()
    );
    actix_web::HttpServer::new(app)
        .listen(tcp_listener)?
        .run()
        .await?;
    Ok(())
}

// cargo install --features controller --path=.
// FASTN_CONTROLLER=http://127.0.0.1:8000 FASTN_INSTANCE_ID=12345 fastn serve 8001
// TRACING=INFO fastn serve
