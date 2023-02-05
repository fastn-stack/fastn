static LOCK: once_cell::sync::Lazy<async_lock::RwLock<()>> =
    once_cell::sync::Lazy::new(|| async_lock::RwLock::new(()));

/// path: /-/<package-name>/<file-name>/
/// path: /<file-name>/
///
#[tracing::instrument(skip_all)]
async fn serve_file(config: &mut fastn_core::Config, path: &camino::Utf8Path) -> fastn_core::http::Response {
    let f = match config.get_file_and_package_by_id(path.as_str()).await {
        Ok(f) => f,
        Err(e) => {
            tracing::error!(
                msg = "fastn-error path not found",
                path = path.as_str(),
                error = %e
            );
            return fastn_core::not_found!("fastn-Error: path: {}, {:?}", path, e);
        }
    };
    // Auth Stuff
    if !f.is_static() {
        let req = if let Some(ref r) = config.request {
            r
        } else {
            return fastn_core::server_error!("request not set");
        };

        match config.can_read(req, path.as_str(), true).await {
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
                return fastn_core::server_error!("fastn-Error: can_read error: {}, {:?}", path, err);
            }
        };
    }

    match f {
        fastn_core::File::Ftd(main_document) => {
            match fastn_core::package::package_doc::read_ftd(config, &main_document, "/", false).await {
                Ok(r) => fastn_core::http::ok_with_content_type(r, mime_guess::mime::TEXT_HTML_UTF_8),
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
        fastn_core::File::Image(image) => {
            fastn_core::http::ok_with_content_type(image.content, guess_mime_type(image.id.as_str()))
        }
        fastn_core::File::Static(s) => fastn_core::http::ok(s.content),
        _ => {
            tracing::error!(msg = "unknown handler", path = path.as_str());
            fastn_core::server_error!("unknown handler")
        }
    }
}

async fn serve_cr_file(
    req: &fastn_core::http::Request,
    config: &mut fastn_core::Config,
    path: &camino::Utf8Path,
    cr_number: usize,
) -> fastn_core::http::Response {
    let _lock = LOCK.read().await;
    let f = match config
        .get_file_and_package_by_cr_id(path.as_str(), cr_number)
        .await
    {
        Ok(f) => f,
        Err(e) => {
            return fastn_core::server_error!("fastn-Error: path: {}, {:?}", path, e);
        }
    };

    // Auth Stuff
    if !f.is_static() {
        match config.can_read(req, path.as_str(), true).await {
            Ok(can_read) => {
                if !can_read {
                    return fastn_core::unauthorised!("You are unauthorized to access: {}", path);
                }
            }
            Err(e) => {
                return fastn_core::server_error!("fastn-Error: can_read error: {}, {:?}", path, e);
            }
        }
    }

    config.current_document = Some(f.get_id());
    match f {
        fastn_core::File::Ftd(main_document) => {
            match fastn_core::package::package_doc::read_ftd(config, &main_document, "/", false).await {
                Ok(r) => fastn_core::http::ok(r),
                Err(e) => {
                    fastn_core::server_error!("fastn-Error: path: {}, {:?}", path, e)
                }
            }
        }
        fastn_core::File::Image(image) => {
            fastn_core::http::ok_with_content_type(image.content, guess_mime_type(image.id.as_str()))
        }
        fastn_core::File::Static(s) => fastn_core::http::ok(s.content),
        _ => {
            fastn_core::server_error!("fastn unknown handler")
        }
    }
}

fn guess_mime_type(path: &str) -> mime_guess::Mime {
    mime_guess::from_path(path).first_or_octet_stream()
}

#[tracing::instrument(skip_all)]
async fn serve_fastn_file(config: &fastn_core::Config) -> fastn_core::http::Response {
    let response = match tokio::fs::read(
        config
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
async fn static_file(file_path: camino::Utf8PathBuf) -> fastn_core::http::Response {
    if !file_path.exists() {
        tracing::error!(msg = "no such static file ({})", path = file_path.as_str());
        return fastn_core::not_found!("no such static file ({})", file_path);
    }

    match tokio::fs::read(file_path.as_path()).await {
        Ok(r) => fastn_core::http::ok_with_content_type(r, guess_mime_type(file_path.as_str())),
        Err(e) => {
            tracing::error!(
                msg = "file-system-error ({})",
                path = file_path.as_str(),
                error = e.to_string()
            );
            fastn_core::not_found!("fastn-Error: path: {:?}, error: {:?}", file_path, e)
        }
    }
}

#[tracing::instrument(skip_all)]
pub async fn serve(
    req: fastn_core::http::Request,
    edition: Option<String>,
    external_js: Vec<String>,
    inline_js: Vec<String>,
    external_css: Vec<String>,
    inline_css: Vec<String>,
) -> fastn_core::Result<fastn_core::http::Response> {
    let _lock = LOCK.read().await;
    // TODO: remove unwrap
    let path: camino::Utf8PathBuf = req.path().replacen('/', "", 1).parse().unwrap();
    let favicon = camino::Utf8PathBuf::new().join("favicon.ico");
    let response = if path.eq(&favicon) {
        static_file(favicon).await
    } else if path.eq(&camino::Utf8PathBuf::new().join("FASTN.ftd")) {
        let config = fastn_core::Config::read(None, false, Some(&req))
            .await
            .unwrap()
            .add_edition(edition)?
            .add_external_js(external_js)
            .add_inline_js(inline_js)
            .add_external_css(external_css)
            .add_inline_css(inline_css);
        serve_fastn_file(&config).await
    } else if path.eq(&camino::Utf8PathBuf::new().join("")) {
        let mut config = fastn_core::Config::read(None, false, Some(&req))
            .await
            .unwrap()
            .add_edition(edition)?
            .set_request(req)
            .add_external_js(external_js)
            .add_inline_js(inline_js)
            .add_external_css(external_css)
            .add_inline_css(inline_css);

        serve_file(&mut config, &path.join("/")).await
    } else if let Some(cr_number) = fastn_core::cr::get_cr_path_from_url(path.as_str()) {
        let mut config = fastn_core::Config::read(None, false, Some(&req))
            .await
            .unwrap()
            .add_edition(edition)?
            .add_external_js(external_js)
            .add_inline_js(inline_js)
            .add_external_css(external_css)
            .add_inline_css(inline_css);
        serve_cr_file(&req, &mut config, &path, cr_number).await
    } else {
        // url is present in config or not
        // If not present than proxy pass it

        let req_method = req.method().to_string();
        let query_string = req.query_string().to_string();
        let mut config = fastn_core::Config::read(None, false, Some(&req))
            .await
            .unwrap()
            .add_edition(edition)?
            .add_external_js(external_js)
            .add_inline_js(inline_js)
            .add_external_css(external_css)
            .add_inline_css(inline_css)
            .set_request(req);

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

        // if request goes with mount-point /todos/api/add-todo/
        // so it should say not found and pass it to proxy
        let file_response = serve_file(&mut config, path.as_path()).await;
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
        if file_response.status() == actix_web::http::StatusCode::NOT_FOUND {
            // TODO: Check if path exists in dynamic urls also, otherwise pass to endpoint
            // Already checked in the above method serve_file
            tracing::info!("executing proxy: path: {}", &path);
            let (package_name, url, mut conf) =
                fastn_core::config::utils::get_clean_url(&config, path.as_str())?;
            let package_name = package_name.unwrap_or_else(|| config.package.name.to_string());

            let host = if let Some(port) = url.port() {
                format!("{}://{}:{}", url.scheme(), url.host_str().unwrap(), port)
            } else {
                format!("{}://{}", url.scheme(), url.host_str().unwrap())
            };
            let req = if let Some(r) = config.request {
                r
            } else {
                tracing::error!(msg = "request not set");
                return Ok(fastn_core::server_error!("request not set"));
            };

            // TODO: read app config and send them to service as header
            // Adjust x-fastn header from based on the platform and the requested field
            if let Some(user_id) = conf.get("user-id") {
                match user_id.split_once('-') {
                    Some((platform, requested_field)) => {
                        if let Some(user_data) = fastn_core::auth::get_user_data_from_cookies(
                            platform,
                            requested_field,
                            req.cookies(),
                        )
                        .await?
                        {
                            conf.insert("X-FASTN-USER-ID".to_string(), user_data);
                        }
                    }
                    _ => return Ok(fastn_core::unauthorised!("invalid user-id provided")),
                }
            }

            return fastn_core::proxy::get_out(
                host.as_str(),
                req,
                url.path(),
                package_name.as_str(),
                &conf,
            )
            .await;
        }

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
    Ok(response)
}

pub(crate) async fn download_init_package(url: Option<String>) -> std::io::Result<()> {
    let mut package = fastn_core::Package::new("unknown-package");
    package.download_base_url = url;
    package
        .http_download_by_id(
            "FASTN.ftd",
            Some(
                &camino::Utf8PathBuf::from_path_buf(std::env::current_dir()?)
                    .expect("fastn-Error: Unable to change path"),
            ),
        )
        .await
        .expect("Unable to find FASTN.ftd file");
    Ok(())
}

pub async fn clear_cache(req: fastn_core::http::Request) -> fastn_core::Result<fastn_core::http::Response> {
    fn is_login(req: &fastn_core::http::Request) -> bool {
        // TODO: Need refactor not happy with this
        req.cookie(fastn_core::auth::AuthProviders::GitHub.as_str())
            .is_some()
            || req
                .cookie(fastn_core::auth::AuthProviders::TeleGram.as_str())
                .is_some()
            || req
                .cookie(fastn_core::auth::AuthProviders::Discord.as_str())
                .is_some()
            || req
                .cookie(fastn_core::auth::AuthProviders::Slack.as_str())
                .is_some()
            || req
                .cookie(fastn_core::auth::AuthProviders::Google.as_str())
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
        return Ok(fastn_core::apis::cache::clear(&req).await);
    }
    // TODO: Remove After Demo, till here

    if !is_login(&req) {
        return Ok(actix_web::HttpResponse::Found()
            .append_header((
                actix_web::http::header::LOCATION,
                "/auth/login/?platform=github".to_string(),
            ))
            .finish());
    }

    let _lock = LOCK.write().await;
    fastn_core::apis::cache::clear(&req).await;
    // TODO: Redirect to Referrer uri
    return Ok(actix_web::HttpResponse::Found()
        .append_header((actix_web::http::header::LOCATION, "/".to_string()))
        .finish());
}

// TODO: Move them to routes folder
async fn sync(req: fastn_core::http::Request) -> fastn_core::Result<fastn_core::http::Response> {
    let _lock = LOCK.write().await;
    fastn_core::apis::sync(&req, req.json()?).await
}

async fn sync2(req: fastn_core::http::Request) -> fastn_core::Result<fastn_core::http::Response> {
    let _lock = LOCK.write().await;
    fastn_core::apis::sync2(&req, req.json()?).await
}

pub async fn clone(req: fastn_core::http::Request) -> fastn_core::Result<fastn_core::http::Response> {
    let _lock = LOCK.read().await;
    fastn_core::apis::clone(req).await
}

pub(crate) async fn view_source(req: fastn_core::http::Request) -> fastn_core::Result<fastn_core::http::Response> {
    let _lock = LOCK.read().await;
    Ok(fastn_core::apis::view_source(&req).await)
}

pub async fn edit(req: fastn_core::http::Request) -> fastn_core::Result<fastn_core::http::Response> {
    let _lock = LOCK.write().await;
    fastn_core::apis::edit(&req, req.json()?).await
}

pub async fn revert(req: fastn_core::http::Request) -> fastn_core::Result<fastn_core::http::Response> {
    let _lock = LOCK.write().await;
    fastn_core::apis::edit::revert(&req, req.json()?).await
}

pub async fn editor_sync(req: fastn_core::http::Request) -> fastn_core::Result<fastn_core::http::Response> {
    let _lock = LOCK.write().await;
    fastn_core::apis::edit::sync(req).await
}

pub async fn create_cr(req: fastn_core::http::Request) -> fastn_core::Result<fastn_core::http::Response> {
    let _lock = LOCK.write().await;
    fastn_core::apis::cr::create_cr(&req, req.json()?).await
}

pub async fn create_cr_page(req: fastn_core::http::Request) -> fastn_core::Result<fastn_core::http::Response> {
    let _lock = LOCK.read().await;
    fastn_core::apis::cr::create_cr_page(req).await
}

struct AppData {
    edition: Option<String>,
    external_js: Vec<String>,
    inline_js: Vec<String>,
    external_css: Vec<String>,
    inline_css: Vec<String>,
}

#[tracing::instrument(skip_all)]
async fn route(
    req: actix_web::HttpRequest,
    body: actix_web::web::Bytes,
    app_data: actix_web::web::Data<AppData>,
) -> fastn_core::Result<fastn_core::http::Response> {
    tracing::info!(method = req.method().as_str(), uri = req.path());
    if req.path().starts_with("/auth/") {
        return fastn_core::auth::routes::handle_auth(
            req,
            app_data.edition.clone(),
            app_data.external_js.clone(),
            app_data.inline_js.clone(),
            app_data.external_css.clone(),
            app_data.inline_css.clone(),
        )
        .await;
    }
    let req = fastn_core::http::Request::from_actix(req, body);
    match (req.method().to_lowercase().as_str(), req.path()) {
        ("post", "/-/sync/") if cfg!(feature = "remote") => sync(req).await,
        ("post", "/-/sync2/") if cfg!(feature = "remote") => sync2(req).await,
        ("get", "/-/clone/") if cfg!(feature = "remote") => clone(req).await,
        ("get", t) if t.starts_with("/-/view-src/") => view_source(req).await,
        ("post", "/-/edit/") => edit(req).await,
        ("post", "/-/revert/") => revert(req).await,
        ("get", "/-/editor-sync/") => editor_sync(req).await,
        ("post", "/-/create-cr/") => create_cr(req).await,
        ("get", "/-/create-cr-page/") => create_cr_page(req).await,
        ("get", "/-/clear-cache/") => clear_cache(req).await,
        ("get", "/-/poll/") => fastn_core::watcher::poll().await,
        (_, _) => {
            serve(
                req,
                app_data.edition.clone(),
                app_data.external_js.clone(),
                app_data.inline_js.clone(),
                app_data.external_css.clone(),
                app_data.inline_css.clone(),
            )
            .await
        }
    }
}

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

    if package_download_base_url.is_some() {
        download_init_package(package_download_base_url).await?;
    }

    if cfg!(feature = "controller") {
        // fastn-controller base path and ec2 instance id (hardcoded for now)
        let fastn_controller: String = std::env::var("FASTN_CONTROLLER")
            .unwrap_or_else(|_| "https://controller.fifthtry.com".to_string());
        let fastn_instance: String =
            std::env::var("FASTN_INSTANCE_ID").expect("FASTN_INSTANCE_ID is required");

        println!("Resolving dependency");
        match crate::controller::resolve_dependencies(fastn_instance, fastn_controller).await {
            Ok(_) => println!("Dependencies resolved"),
            Err(e) => panic!("Error resolving dependencies using controller!!: {:?}", e),
        }
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

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let app = move || {
        actix_web::App::new()
            .app_data(actix_web::web::Data::new(AppData {
                edition: edition.clone(),
                external_js: external_js.clone(),
                inline_js: inline_js.clone(),
                external_css: external_css.clone(),
                inline_css: inline_css.clone(),
            }))
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
