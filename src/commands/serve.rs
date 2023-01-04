static LOCK: once_cell::sync::Lazy<async_lock::RwLock<()>> =
    once_cell::sync::Lazy::new(|| async_lock::RwLock::new(()));

/// path: /-/<package-name>/<file-name>/
/// path: /<file-name>/
///
#[tracing::instrument(skip_all)]
async fn serve_file(config: &mut fpm::Config, path: &camino::Utf8Path) -> fpm::http::Response {
    let f = match config.get_file_and_package_by_id(path.as_str()).await {
        Ok(f) => f,
        Err(e) => {
            tracing::error!(
                msg = "fpm-error path not found",
                path = path.as_str(),
                error = %e
            );
            return fpm::not_found!("FPM-Error: path: {}, {:?}", path, e);
        }
    };
    // Auth Stuff
    if !f.is_static() {
        let req = if let Some(ref r) = config.request {
            r
        } else {
            return fpm::server_error!("request not set");
        };

        match config.can_read(req, path.as_str(), true).await {
            Ok(can_read) => {
                if !can_read {
                    tracing::error!(
                        msg = "unauthorized-error: can not read",
                        path = path.as_str()
                    );
                    return fpm::unauthorised!("You are unauthorized to access: {}", path);
                }
            }
            Err(e) => {
                tracing::error!(msg = "can_read-error", path = path.as_str());
                return fpm::server_error!("FPM-Error: can_read error: {}, {:?}", path, e);
            }
        };

        match fpm::package::app::can_read(config, path.as_str()).await {
            Ok(can_read) => {
                if !can_read {
                    tracing::error!(
                        msg = "unauthorized-error: can not access app",
                        path = path.as_str()
                    );
                    return fpm::unauthorised!("You are unauthorized to access: {}", path);
                }
            }
            Err(err) => {
                tracing::error!(
                    msg = "app::can_read-error: can not access app",
                    path = path.as_str()
                );
                return fpm::server_error!("FPM-Error: can_read error: {}, {:?}", path, err);
            }
        };
    }

    match f {
        fpm::File::Ftd(main_document) => {
            match fpm::package::package_doc::read_ftd(config, &main_document, "/", false).await {
                Ok(r) => fpm::http::ok_with_content_type(r, mime_guess::mime::TEXT_HTML_UTF_8),
                Err(e) => {
                    fpm::server_error!("FPM-Error: path: {}, {:?}", path, e)
                }
            }
        }
        fpm::File::Image(image) => {
            fpm::http::ok_with_content_type(image.content, guess_mime_type(image.id.as_str()))
        }
        fpm::File::Static(s) => fpm::http::ok(s.content),
        _ => {
            fpm::server_error!("unknown handler")
        }
    }
}

async fn serve_cr_file(
    req: &fpm::http::Request,
    config: &mut fpm::Config,
    path: &camino::Utf8Path,
    cr_number: usize,
) -> fpm::http::Response {
    let _lock = LOCK.read().await;
    let f = match config
        .get_file_and_package_by_cr_id(path.as_str(), cr_number)
        .await
    {
        Ok(f) => f,
        Err(e) => {
            return fpm::server_error!("FPM-Error: path: {}, {:?}", path, e);
        }
    };

    // Auth Stuff
    if !f.is_static() {
        match config.can_read(req, path.as_str(), true).await {
            Ok(can_read) => {
                if !can_read {
                    return fpm::unauthorised!("You are unauthorized to access: {}", path);
                }
            }
            Err(e) => {
                return fpm::server_error!("FPM-Error: can_read error: {}, {:?}", path, e);
            }
        }
    }

    config.current_document = Some(f.get_id());
    match f {
        fpm::File::Ftd(main_document) => {
            match fpm::package::package_doc::read_ftd(config, &main_document, "/", false).await {
                Ok(r) => fpm::http::ok(r),
                Err(e) => {
                    fpm::server_error!("FPM-Error: path: {}, {:?}", path, e)
                }
            }
        }
        fpm::File::Image(image) => {
            fpm::http::ok_with_content_type(image.content, guess_mime_type(image.id.as_str()))
        }
        fpm::File::Static(s) => fpm::http::ok(s.content),
        _ => {
            fpm::server_error!("FPM unknown handler")
        }
    }
}

fn guess_mime_type(path: &str) -> mime_guess::Mime {
    mime_guess::from_path(path).first_or_octet_stream()
}

#[tracing::instrument(skip_all)]
async fn serve_fpm_file(config: &fpm::Config) -> fpm::http::Response {
    let response =
        match tokio::fs::read(config.get_root_for_package(&config.package).join("FPM.ftd")).await {
            Ok(res) => res,
            Err(e) => {
                return fpm::not_found!("FPM-Error: path: FPM.ftd error: {:?}", e);
            }
        };
    fpm::http::ok_with_content_type(response, mime_guess::mime::APPLICATION_OCTET_STREAM)
}

#[tracing::instrument(skip_all)]
async fn static_file(file_path: camino::Utf8PathBuf) -> fpm::http::Response {
    if !file_path.exists() {
        tracing::error!(msg = "no such static file ({})", path = file_path.as_str());
        return fpm::not_found!("no such static file ({})", file_path);
    }

    match tokio::fs::read(file_path.as_path()).await {
        Ok(r) => fpm::http::ok_with_content_type(r, guess_mime_type(file_path.as_str())),
        Err(e) => {
            tracing::error!(
                msg = "file-system-error ({})",
                path = file_path.as_str(),
                error = e.to_string()
            );
            fpm::not_found!("FPM-Error: path: {:?}, error: {:?}", file_path, e)
        }
    }
}

#[tracing::instrument(skip_all)]
pub async fn serve(
    req: fpm::http::Request,
    edition: Option<String>,
    external_js: Vec<String>,
    inline_js: Vec<String>,
    external_css: Vec<String>,
    inline_css: Vec<String>,
) -> fpm::Result<fpm::http::Response> {
    let _lock = LOCK.read().await;
    // TODO: remove unwrap
    let path: camino::Utf8PathBuf = req.path().replacen('/', "", 1).parse().unwrap();
    let favicon = camino::Utf8PathBuf::new().join("favicon.ico");
    let response = if path.eq(&favicon) {
        static_file(favicon).await
    } else if path.eq(&camino::Utf8PathBuf::new().join("FPM.ftd")) {
        let config = fpm::Config::read(None, false, Some(&req))
            .await
            .unwrap()
            .add_edition(edition)?
            .add_external_js(external_js)
            .add_inline_js(inline_js)
            .add_external_css(external_css)
            .add_inline_css(inline_css);
        serve_fpm_file(&config).await
    } else if path.eq(&camino::Utf8PathBuf::new().join("")) {
        let mut config = fpm::Config::read(None, false, Some(&req))
            .await
            .unwrap()
            .add_edition(edition)?
            .set_request(req)
            .add_external_js(external_js)
            .add_inline_js(inline_js)
            .add_external_css(external_css)
            .add_inline_css(inline_css);

        serve_file(&mut config, &path.join("/")).await
    } else if let Some(cr_number) = fpm::cr::get_cr_path_from_url(path.as_str()) {
        let mut config = fpm::Config::read(None, false, Some(&req))
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
        let mut config = fpm::Config::read(None, false, Some(&req))
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
        // fpm::file::is_static: checking for static file, if file is static no need to redirect it.
        if req_method.as_str() == "GET" && !fpm::file::is_static(path.as_str())? {
            // if any app name starts with package-name to redirect it to /mount-point/remaining-url/
            for (mp, dep) in config
                .package
                .apps
                .iter()
                .map(|x| (&x.mount_point, &x.package))
            {
                if let Some(remaining_path) =
                    fpm::config::utils::trim_package_name(path.as_str(), dep.name.as_str())
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
                fpm::config::utils::get_clean_url(&config, path.as_str())?;
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
                return Ok(fpm::server_error!("request not set"));
            };

            // TODO: read app config and send them to service as header
            // Adjust x-fpm header from based on the platform and the requested field
            if let Some(user_id) = conf.get("user-id") {
                match user_id.split_once('-') {
                    Some((platform, requested_field)) => {
                        if let Some(user_data) = fpm::auth::get_user_data_from_cookies(
                            platform,
                            requested_field,
                            req.cookies(),
                        )
                        .await?
                        {
                            conf.insert("X-FPM-USER-ID".to_string(), user_data);
                        }
                    }
                    _ => return Ok(fpm::unauthorised!("invalid user-id provided")),
                }
            }

            return fpm::proxy::get_out(
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
        //         return Ok(fpm::server_error!("request not set"));
        //     };
        //     fpm::wasm::handle_wasm(req, wasm_module, config.package.backend_headers).await
        // }

        file_response
    };
    Ok(response)
}

pub(crate) async fn download_init_package(url: Option<String>) -> std::io::Result<()> {
    let mut package = fpm::Package::new("unknown-package");
    package.download_base_url = url;
    package
        .http_download_by_id(
            "FPM.ftd",
            Some(
                &camino::Utf8PathBuf::from_path_buf(std::env::current_dir()?)
                    .expect("FPM-Error: Unable to change path"),
            ),
        )
        .await
        .expect("Unable to find FPM.ftd file");
    Ok(())
}

pub async fn clear_cache(req: fpm::http::Request) -> fpm::Result<fpm::http::Response> {
    fn is_login(req: &fpm::http::Request) -> bool {
        // TODO: Need refactor not happy with this
        req.cookie(fpm::auth::AuthProviders::GitHub.as_str())
            .is_some()
            || req
                .cookie(fpm::auth::AuthProviders::TeleGram.as_str())
                .is_some()
            || req
                .cookie(fpm::auth::AuthProviders::Discord.as_str())
                .is_some()
            || req
                .cookie(fpm::auth::AuthProviders::Slack.as_str())
                .is_some()
            || req
                .cookie(fpm::auth::AuthProviders::Google.as_str())
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
        return Ok(fpm::apis::cache::clear(&req).await);
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
    fpm::apis::cache::clear(&req).await;
    // TODO: Redirect to Referrer uri
    return Ok(actix_web::HttpResponse::Found()
        .append_header((actix_web::http::header::LOCATION, "/".to_string()))
        .finish());
}

// TODO: Move them to routes folder
async fn sync(req: fpm::http::Request) -> fpm::Result<fpm::http::Response> {
    let _lock = LOCK.write().await;
    fpm::apis::sync(&req, req.json()?).await
}

async fn sync2(req: fpm::http::Request) -> fpm::Result<fpm::http::Response> {
    let _lock = LOCK.write().await;
    fpm::apis::sync2(&req, req.json()?).await
}

pub async fn clone(req: fpm::http::Request) -> fpm::Result<fpm::http::Response> {
    let _lock = LOCK.read().await;
    fpm::apis::clone(req).await
}

pub(crate) async fn view_source(req: fpm::http::Request) -> fpm::Result<fpm::http::Response> {
    let _lock = LOCK.read().await;
    Ok(fpm::apis::view_source(&req).await)
}

pub async fn edit(req: fpm::http::Request) -> fpm::Result<fpm::http::Response> {
    let _lock = LOCK.write().await;
    fpm::apis::edit(&req, req.json()?).await
}

pub async fn revert(req: fpm::http::Request) -> fpm::Result<fpm::http::Response> {
    let _lock = LOCK.write().await;
    fpm::apis::edit::revert(&req, req.json()?).await
}

pub async fn editor_sync(req: fpm::http::Request) -> fpm::Result<fpm::http::Response> {
    let _lock = LOCK.write().await;
    fpm::apis::edit::sync(req).await
}

pub async fn create_cr(req: fpm::http::Request) -> fpm::Result<fpm::http::Response> {
    let _lock = LOCK.write().await;
    fpm::apis::cr::create_cr(&req, req.json()?).await
}

pub async fn create_cr_page(req: fpm::http::Request) -> fpm::Result<fpm::http::Response> {
    let _lock = LOCK.read().await;
    fpm::apis::cr::create_cr_page(req).await
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
) -> fpm::Result<fpm::http::Response> {
    tracing::info!(method = req.method().as_str(), uri = req.path());
    if req.path().starts_with("/auth/") {
        return fpm::auth::routes::handle_auth(
            req,
            app_data.edition.clone(),
            app_data.external_js.clone(),
            app_data.inline_js.clone(),
            app_data.external_css.clone(),
            app_data.inline_css.clone(),
        )
        .await;
    }
    let req = fpm::http::Request::from_actix(req, body);
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
        ("get", "/-/poll/") => fpm::watcher::poll().await,
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
) -> fpm::Result<()> {
    use colored::Colorize;
    dotenv::dotenv().ok();
    if package_download_base_url.is_some() {
        download_init_package(package_download_base_url).await?;
    }

    if cfg!(feature = "controller") {
        // fpm-controller base path and ec2 instance id (hardcoded for now)
        let fpm_controller: String = std::env::var("FPM_CONTROLLER")
            .unwrap_or_else(|_| "https://controller.fifthtry.com".to_string());
        let fpm_instance: String =
            std::env::var("FPM_INSTANCE_ID").expect("FPM_INSTANCE_ID is required");

        println!("Resolving dependency");
        match crate::controller::resolve_dependencies(fpm_instance, fpm_controller).await {
            Ok(_) => println!("Dependencies resolved"),
            Err(e) => panic!("Error resolving dependencies using controller!!: {:?}", e),
        }
    }

    let tcp_listener = match fpm::http::get_available_port(port, bind_address) {
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

    let app = move || {
        actix_web::App::new()
            .app_data(actix_web::web::Data::new(AppData {
                edition: edition.clone(),
                external_js: external_js.clone(),
                inline_js: inline_js.clone(),
                external_css: external_css.clone(),
                inline_css: inline_css.clone(),
            }))
            .route("/{path:.*}", actix_web::web::route().to(route))
    };

    println!("### Server Started ###");
    println!(
        "Go to: http://{}:{}",
        bind_address,
        tcp_listener.local_addr()?.port()
    );

    tracing().await;
    println!("### Configured tracing ###");

    actix_web::HttpServer::new(app)
        .listen(tcp_listener)?
        .run()
        .await?;
    Ok(())
}

async fn tracing() {
    use tracing_subscriber::layer::SubscriberExt;
    tracing_forest::worker_task()
        .set_global(true)
        .build_with(|_layer: tracing_forest::ForestLayer<_, _>| {
            tracing_subscriber::Registry::default()
                .with(tracing_forest::ForestLayer::default())
                .with(tracing_forest::util::LevelFilter::INFO)
        })
        // .build_on(|subscriber| subscriber.with(tracing_forest::util::LevelFilter::INFO))
        .on(async {}) // this statement is needed, without this logs are getting printed
        .await;
}

// cargo install --features controller --path=.
// FPM_CONTROLLER=http://127.0.0.1:8000 FPM_INSTANCE_ID=12345 fpm serve 8001
