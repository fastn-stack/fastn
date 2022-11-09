static LOCK: once_cell::sync::Lazy<async_lock::RwLock<()>> =
    once_cell::sync::Lazy::new(|| async_lock::RwLock::new(()));

/// path: /-/<package-name>/<file-name>/
/// path: /<file-name>/
async fn serve_file(config: &mut fpm::Config, path: &camino::Utf8Path) -> fpm::http::Response {
    let f = match config.get_file_and_package_by_id(path.as_str()).await {
        Ok(f) => f,
        Err(e) => {
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
        match config.can_read(req, path.as_str()).await {
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
        match config.can_read(req, path.as_str()).await {
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

async fn static_file(file_path: camino::Utf8PathBuf) -> fpm::http::Response {
    if !file_path.exists() {
        return fpm::not_found!("no such static file ({})", file_path);
    }

    match tokio::fs::read(file_path.as_path()).await {
        Ok(r) => fpm::http::ok_with_content_type(r, guess_mime_type(file_path.as_str())),
        Err(e) => {
            fpm::not_found!("FPM-Error: path: {:?}, error: {:?}", file_path, e)
        }
    }
}

async fn serve(req: fpm::http::Request) -> fpm::Result<fpm::http::Response> {
    let _lock = LOCK.read().await;
    let r = format!("{} {}", req.method(), req.path());
    let t = fpm::time(r.as_str());
    println!("{r} started");

    // TODO: remove unwrap
    let path: camino::Utf8PathBuf = req.path().replacen('/', "", 1).parse().unwrap();

    let favicon = camino::Utf8PathBuf::new().join("favicon.ico");
    let response = if path.eq(&favicon) {
        static_file(favicon).await
    } else if path.eq(&camino::Utf8PathBuf::new().join("FPM.ftd")) {
        let config = fpm::time("Config::read()")
            .it(fpm::Config::read(None, false, Some(&req)).await.unwrap());
        serve_fpm_file(&config).await
    } else if path.eq(&camino::Utf8PathBuf::new().join("")) {
        let mut config = fpm::time("Config::read()")
            .it(fpm::Config::read(None, false, Some(&req)).await.unwrap())
            .set_request(req);
        serve_file(&mut config, &path.join("/")).await
    } else if let Some(cr_number) = fpm::cr::get_cr_path_from_url(path.as_str()) {
        let mut config = fpm::time("Config::read()")
            .it(fpm::Config::read(None, false, Some(&req)).await.unwrap());
        serve_cr_file(&req, &mut config, &path, cr_number).await
    } else {
        // url is present in config or not
        // If not present than proxy pass it

        let req_method = req.method().to_string();
        let query_string = req.query_string().to_string();
        let mut config = fpm::time("Config::read()").it(fpm::Config::read(None, false, Some(&req))
            .await
            .unwrap()
            .set_request(req));

        // if start with -/ and mount-point exists so send redirect to mount-point
        // We have to do -/<package-name>/remaining-url/ ==> (<package-name>, remaining-url) ==> (/config.package-name.mount-point/remaining-url/)
        // Get all the dependencies with mount-point if path_start with any package-name so send redirect to mount-point

        if req_method.as_str() == "GET" {
            // if any app name starts with package-name to redirect it to /mount-point/remaining-url/
            for (mp, dep) in config
                .package
                .apps
                .iter()
                .map(|x| (&x.mount_point, &x.package.package))
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
        if file_response.status() == actix_web::http::StatusCode::NOT_FOUND {
            // TODO: Check if path exists in dynamic urls also, otherwise pass to endpoint
            // Already checked in the above method serve_file
            println!("executing proxy: {}", &path);
            let (package_name, url, conf) =
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
                return Ok(fpm::server_error!("request not set"));
            };

            // TODO: read app config and send them to service as header

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
        if file_response.status() == actix_web::http::StatusCode::NOT_FOUND {
            let package = config.find_package_by_id(path.as_str()).await.unwrap().1;
            let wasm_module = config.get_root_for_package(&package).join("backend.wasm");
            // Set the temp path to and do `get_file_and_package_by_id` to retrieve the backend.wasm
            // File for the particular dependency package
            let wasm_module_path = format!("-/{}/backend.wasm", package.name,);
            config
                .get_file_and_package_by_id(wasm_module_path.as_str())
                .await?;
            let req = if let Some(r) = config.request {
                r
            } else {
                return Ok(fpm::server_error!("request not set"));
            };
            fpm::wasm::handle_wasm(req, wasm_module, config.package.backend_headers).await
        } else {
            file_response
        }
    };
    t.it(Ok(response))
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
    let _lock = LOCK.write().await;
    Ok(fpm::apis::cache::clear(&req).await)
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

async fn auth_route(req: fpm::http::Request) -> fpm::Result<fpm::http::Response> {
    let _lock = LOCK.write().await;
    let auth_obj = fpm::auth::github::index(req);
    Ok(auth_obj.await)
}
async fn login_route(req: fpm::http::Request) -> fpm::Result<fpm::http::Response> {
    let _lock = LOCK.write().await;
    let login_obj = fpm::auth::github::login(req);
    Ok(login_obj.await)
}

async fn logout_route(req: fpm::http::Request) -> fpm::Result<fpm::http::Response> {
    let _lock = LOCK.write().await;
    let logout_obj = fpm::auth::github::logout(req);
    Ok(logout_obj)
}
async fn auth_auth_route(req: fpm::http::Request) -> fpm::Result<fpm::http::Response> {
    let _lock = LOCK.write().await;
    //let base_url=format!("{}{}{}",req.connection_info().scheme(),"://",req.connection_info().host());
    let base_url = "http://localhost:8000";
    let uri_string = req.uri();
    let final_url: String = format!("{}{}", base_url.clone(), uri_string.clone().to_string());
    let request_url = url::Url::parse(&final_url.to_string()).unwrap();
    let pairs = request_url.query_pairs();
    let mut code = String::from("");
    let mut state = String::from("");
    for pair in pairs {
        if pair.0 == "code" {
            code = pair.1.to_string();
        }
        if pair.0 == "state" {
            state = pair.1.to_string();
        }
    }
    let auth_obj = fpm::auth::github::auth(
        req,
        fpm::auth::github::AuthRequest {
            code: code.clone(),
            state: state.clone(),
        },
    );
    Ok(auth_obj.await)
}

async fn get_identities_route(req: fpm::http::Request) -> fpm::Result<fpm::http::Response> {
    let _lock = LOCK.write().await;
    let identity_obj = fpm::auth::github::get_identity(req);
    Ok(identity_obj.await)
}
/*async fn get_identities_ctrl(
    req: &fpm::http::Request,
    body: actix_web::web::Bytes,
) -> fpm::Result<fpm::http::Response> {
        let returned_identities=get_dentities(req);
        let identity_obj=
        fpm::auth::github::get_identity_fpm(
        req.cookies(),&returned_identities);
        Ok(identity_obj.await)
}*/
fn get_dentities(req: &fpm::http::Request) -> Vec<fpm::auth::github::UserIdentity> {
    let mut repo_list: Vec<fpm::auth::github::UserIdentity> = Vec::new();
    //let base_url=format!("{}{}{}",req.connection_info().scheme(),"://",req.connection_info().host());
    let base_url = format!("{}{}{}", req.host(), "://", req.host());
    let mut repo_list: Vec<fpm::auth::github::UserIdentity> = Vec::new();
    let uri_string = req.uri();
    let final_url: String = format!("{}{}", base_url.clone(), uri_string.clone().to_string());
    let request_url = url::Url::parse(&final_url.to_string()).unwrap();
    let pairs = request_url.query_pairs();
    for pair in pairs {
        repo_list.push(fpm::auth::github::UserIdentity {
            key: pair.0.to_string(),
            value: 1.to_string(),
        });
    }
    repo_list
}
// clone the fpm repo
// change in the code in serve.rs
// run command `cargo install --path=.`
// After that fpm binary will be available to you
// `fpm serve` or fpm serve --port 8000

// cargo run -- serve, not working even after adding FPM.ftd and index.ftd file

async fn route(
    req: actix_web::HttpRequest,
    body: actix_web::web::Bytes,
) -> fpm::Result<fpm::http::Response> {
    let req = fpm::http::Request::from_actix(req, body);
    /*if req.path()=="/auth/" {
        return auth_route(req.clone()).await;
    }
    else if req.path()=="/auth/login/" {
        return login_route(req.clone()).await;
    }
    else if req.path()=="/auth/logout/" {
        return logout_route(req.clone()).await;
    }
    else if req.path()=="/auth/auth/" {
        return auth_auth_route(req.clone()).await;
    }
    else if req.path()=="/auth/get-identities/" {
        return get_identities_route(req.clone()).await;
    }*/

    match (req.method(), req.path()) {
        ("post", "/-/sync/") if cfg!(feature = "remote") => sync(req).await,
        ("post", "/-/sync2/") if cfg!(feature = "remote") => sync2(req).await,
        ("get", "/-/clone/") if cfg!(feature = "remote") => clone(req).await,
        ("get", t) if t.starts_with("/-/view-src/") => view_source(req).await,
        ("post", "/-/edit/") => edit(req).await,
        ("post", "/-/revert/") => revert(req).await,
        ("get", "/-/editor-sync/") => editor_sync(req).await,
        ("post", "/-/create-cr/") => create_cr(req).await,
        ("get", "/-/create-cr-page/") => create_cr_page(req).await,
        ("post", "/-/clear-cache/") => clear_cache(req).await,
        ("get", "/-/auth/") => auth_route(req).await,
        (_, _) => serve(req).await,
    }
}

pub async fn listen(
    bind_address: &str,
    port: Option<u16>,
    package_download_base_url: Option<String>,
) -> fpm::Result<()> {
    use colored::Colorize;

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

    let app = move || actix_web::App::new().route("/{path:.*}", actix_web::web::route().to(route));

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
// FPM_CONTROLLER=http://127.0.0.1:8000 FPM_INSTANCE_ID=12345 fpm serve 8001
