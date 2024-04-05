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
        .map(|r| fastn_core::http::permanent_redirect(r.to_string()))
}

/// path: /-/<package-name>/<file-name>/
/// path: /<file-name>/
#[tracing::instrument(skip_all)]
async fn serve_file(
    config: &mut fastn_core::RequestConfig,
    path: &camino::Utf8Path,
    only_js: bool,
) -> fastn_core::http::Response {
    let path = <&camino::Utf8Path>::clone(&path);

    if let Err(e) = config
        .config
        .package
        .auto_import_language(config.request.cookie("fastn-lang"), None)
    {
        return if config.config.test_command_running {
            fastn_core::http::not_found_without_warning(format!(
                "fastn-Error: path: {}, {:?}",
                path, e
            ))
        } else {
            fastn_core::not_found!("fastn-Error: path: {}, {:?}", path, e)
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
            return if config.config.test_command_running {
                fastn_core::http::not_found_without_warning(format!(
                    "fastn-Error: path: {}, {:?}",
                    path, e
                ))
            } else {
                fastn_core::not_found!("fastn-Error: path: {}, {:?}", path, e)
            };
        }
    };

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
    only_js: bool,
) -> fastn_core::Result<fastn_core::http::Response> {
    if let Some(endpoint_response) = handle_endpoints(config, &req).await {
        return endpoint_response;
    }

    if let Some(app_response) = handle_apps(config, &req).await {
        return app_response;
    }

    if let Some(default_response) = handle_default_route(&req, config.package.name.as_str()) {
        return default_response;
    }

    let path: camino::Utf8PathBuf = req.path().replacen('/', "", 1).parse()?;

    if let Some(r) = handle_redirect(config, &path) {
        return Ok(r);
    }

    if fastn_core::utils::is_static_path(req.path()) {
        return handle_static_route(req.path(), config.package.name.as_str(), &config.ds).await;
    }

    let req_config = fastn_core::RequestConfig::new(config, &req, "", "/");

    serve_helper(req_config, only_js, path).await
}

#[tracing::instrument(skip_all)]
pub async fn serve_helper(
    mut req_config: fastn_core::RequestConfig,
    only_js: bool,
    path: camino::Utf8PathBuf,
) -> fastn_core::Result<fastn_core::http::Response> {
    let mut resp = if req_config.request.path() == "/" {
        serve_file(&mut req_config, &path.join("/"), only_js).await
    } else {
        // url is present in config or not
        // If not present than proxy pass it

        let query_string = req_config.request.query_string().to_string();

        // if start with -/ and mount-point exists so send redirect to mount-point
        // We have to do -/<package-name>/remaining-url/ ==> (<package-name>, remaining-url) ==> (/config.package-name.mount-point/remaining-url/)
        // Get all the dependencies with mount-point if path_start with any package-name so send redirect to mount-point
        // fastn_core::file::is_static: checking for static file, if file is static no need to redirect it.
        // if any app name starts with package-name to redirect it to /mount-point/remaining-url/
        for (mp, dep) in req_config
            .config
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

                let mut resp =
                    actix_web::HttpResponse::new(actix_web::http::StatusCode::PERMANENT_REDIRECT);
                resp.headers_mut().insert(
                    actix_web::http::header::LOCATION,
                    actix_web::http::header::HeaderValue::from_str(path.as_str()).unwrap(), // TODO:
                );
                return Ok(resp);
            }
        }

        let file_response = serve_file(&mut req_config, path.as_path(), only_js).await;

        tracing::info!(
            "before executing proxy: file-status: {}, path: {}",
            file_response.status(),
            &path
        );

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

async fn handle_static_route(
    path: &str,
    package_name: &str,
    ds: &fastn_ds::DocumentStore,
) -> fastn_core::Result<fastn_core::http::Response> {
    return match handle_static_route_(path, package_name, ds).await {
        Ok(r) => Ok(r),
        Err(fastn_ds::ReadError::NotFound) => handle_not_found_image(path, package_name, ds).await,
        Err(e) => Err(e.into()),
    };

    async fn handle_static_route_(
        path: &str,
        package_name: &str,
        ds: &fastn_ds::DocumentStore,
    ) -> Result<fastn_core::http::Response, fastn_ds::ReadError> {
        if path == "/favicon.ico" {
            return favicon(ds).await;
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

        static_file(ds, path.strip_prefix('/').unwrap_or(path.as_str()))
            .await
            .map_err(Into::into)
    }

    async fn handle_not_found_image(
        path: &str,
        package_name: &str,
        ds: &fastn_ds::DocumentStore,
    ) -> fastn_core::Result<fastn_core::http::Response> {
        // todo: handle dark images using manifest
        if let Some(new_file_path) = generate_dark_image_path(path) {
            return handle_static_route_(new_file_path.as_str(), package_name, ds)
                .await
                .or_else(|e| {
                    if let fastn_ds::ReadError::NotFound = e {
                        Ok(fastn_core::http::not_found_without_warning("".to_string()))
                    } else {
                        Err(e.into())
                    }
                });
        }

        Ok(fastn_core::http::not_found_without_warning("".to_string()))
    }

    fn generate_dark_image_path(path: &str) -> Option<String> {
        match path.rsplit_once('.') {
            Some((remaining, ext))
                if mime_guess::MimeGuess::from_ext(ext)
                    .first_or_octet_stream()
                    .to_string()
                    .starts_with("image/") =>
            {
                Some(if remaining.ends_with("-dark") {
                    format!("{}.{}", remaining.trim_end_matches("-dark"), ext)
                } else {
                    format!("{}-dark.{}", remaining, ext)
                })
            }
            _ => None,
        }
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
    let matched_endpoint = config
        .package
        .endpoints
        .iter()
        .find(|ep| req.path().starts_with(ep.mountpoint.trim_end_matches('/')));

    let endpoint = match matched_endpoint {
        Some(e) => e,
        None => return None,
    };

    let response = match config
        .ds
        .http(
            url::Url::parse(
                format!(
                    "{}/{}",
                    endpoint.endpoint.trim_end_matches('/'),
                    req.path()
                        .trim_start_matches(endpoint.mountpoint.trim_end_matches('/'))
                        .trim_start_matches('/')
                )
                .as_str(),
            )
            .unwrap(),
            req,
            &std::collections::HashMap::new(),
        )
        .await
        .map_err(fastn_core::Error::DSHttpError)
    {
        Ok(response) => response,
        Err(e) => return Some(Err(e)),
    };

    let actix_response = fastn_core::http::ResponseBuilder::from_reqwest(response).await;
    Some(Ok(actix_response))
}

async fn handle_apps(
    config: &fastn_core::Config,
    req: &fastn_core::http::Request,
) -> Option<fastn_core::Result<fastn_core::http::Response>> {
    let matched_app = config.package.apps.iter().find(|a| {
        req.path().starts_with(
            a.end_point
                .clone()
                .unwrap_or_default()
                .trim_end_matches('/'),
        )
    });

    let _app = match matched_app {
        Some(e) => e,
        None => return None,
    };

    // app.package.endpoints
    // app.package.apps

    // see if app.pack

    None
}

#[tracing::instrument(skip_all)]
async fn actual_route(
    config: &fastn_core::Config,
    req: actix_web::HttpRequest,
    body: actix_web::web::Bytes,
) -> fastn_core::Result<fastn_core::http::Response> {
    tracing::info!(method = req.method().as_str(), uri = req.path());
    let req = fastn_core::http::Request::from_actix(req, body);

    serve(config, req, false).await
}

#[tracing::instrument(skip_all)]
async fn route(
    req: actix_web::HttpRequest,
    body: actix_web::web::Bytes,
    config: actix_web::web::Data<std::sync::Arc<fastn_core::Config>>,
) -> fastn_core::Result<fastn_core::http::Response> {
    actual_route(&config, req, body).await
}

#[allow(clippy::too_many_arguments)]
pub async fn listen(
    config: std::sync::Arc<fastn_core::Config>,
    bind_address: &str,
    port: Option<u16>,
) -> fastn_core::Result<()> {
    use colored::Colorize;
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

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

    let app = move || {
        actix_web::App::new()
            .app_data(actix_web::web::Data::new(std::sync::Arc::clone(&config)))
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
