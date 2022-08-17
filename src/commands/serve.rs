pub struct AppState {
    // Set from command line to provide access, mostly for dev purpose.
    pub local_identities: Vec<fpm::user_group::UserIdentity>,
}

async fn serve_files(
    req: &actix_web::HttpRequest,
    config: &mut fpm::Config,
    path: &std::path::Path,
) -> actix_web::HttpResponse {
    let path = match path.to_str() {
        Some(s) => s,
        None => {
            eprintln!("handle_ftd: Not able to convert path");
            return actix_web::HttpResponse::InternalServerError().body("".as_bytes());
        }
    };

    let f = match config.get_file_and_package_by_id(path).await {
        Ok(f) => f,
        Err(e) => {
            eprintln!("FPM-Error: path: {}, {:?}", path, e);
            return actix_web::HttpResponse::InternalServerError().body(e.to_string());
        }
    };

    // Auth Stuff
    // Note: If package does not have sitemap, considering all documents are public.
    // Note: If package have sitemap but does not have any user groups defined,
    // all document should be public, by default behaviour
    // If `f` is static file, considering it public as well, for now.

    if !f.is_static() {
        match config.can_read(req, path) {
            Ok(can_read) => {
                if !can_read {
                    return actix_web::HttpResponse::Unauthorized()
                        .body(format!("You are unauthorized to access: {}", path));
                }
            }
            Err(e) => {
                eprintln!("FPM-Error: can_read error: {}, {:?}", path, e);
                return actix_web::HttpResponse::InternalServerError().body(e.to_string());
            }
        }
    }

    // Get identities from remote(sid)
    // fpm ui, add dependency as auto add, 404 page will come from fpm ui
    // TODO: for image check referer permission

    config.current_document = Some(f.get_id());
    return match f {
        fpm::File::Ftd(main_document) => {
            return match fpm::package_doc::read_ftd(config, &main_document, "/", false).await {
                Ok(r) => actix_web::HttpResponse::Ok().body(r),
                Err(e) => {
                    eprintln!("FPM-Error: path: {}, {:?}", path, e);
                    actix_web::HttpResponse::InternalServerError().body(e.to_string())
                }
            };
        }
        fpm::File::Image(image) => {
            return actix_web::HttpResponse::Ok()
                .content_type(guess_mime_type(image.id.as_str()))
                .body(image.content);
        }
        fpm::File::Static(s) => {
            return actix_web::HttpResponse::Ok().body(s.content);
        }
        _ => {
            eprintln!("FPM unknown handler");
            actix_web::HttpResponse::InternalServerError().body("".as_bytes())
        }
    };
}

fn guess_mime_type(path: &str) -> mime_guess::Mime {
    mime_guess::from_path(path).first_or_octet_stream()
}

async fn serve_fpm_file(config: &fpm::Config) -> actix_web::HttpResponse {
    let response =
        match tokio::fs::read(config.get_root_for_package(&config.package).join("FPM.ftd")).await {
            Ok(res) => res,
            Err(e) => {
                eprintln!("FPM-Error: path: FPM.ftd error: {:?}", e);
                return actix_web::HttpResponse::NotFound().body(e.to_string());
            }
        };
    actix_web::HttpResponse::Ok()
        .content_type("application/octet-stream")
        .body(response)
}

async fn static_file(
    req: &actix_web::HttpRequest,
    file_path: std::path::PathBuf,
) -> actix_web::HttpResponse {
    if !file_path.exists() {
        return actix_web::HttpResponse::NotFound().body("".as_bytes());
    }

    match actix_files::NamedFile::open_async(&file_path).await {
        Ok(r) => r.into_response(req),
        Err(e) => {
            eprintln!("FPM-Error: path: {:?}, error: {:?}", file_path, e);
            actix_web::HttpResponse::NotFound().body(e.to_string())
        }
    }
}

async fn serve(
    req: actix_web::HttpRequest,
    data: actix_web::web::Data<AppState>,
) -> actix_web::HttpResponse {
    // TODO: Need to remove unwrap
    let mut config = fpm::Config::read(None, false).await.unwrap();
    config.set_identities(data.local_identities.clone());

    let path: std::path::PathBuf = req.match_info().query("path").parse().unwrap();

    println!("request for path: {:?}", path);
    let time = std::time::Instant::now();
    let favicon = std::path::PathBuf::new().join("favicon.ico");
    let response = if path.eq(&favicon) {
        static_file(&req, favicon).await
    } else if path.eq(&std::path::PathBuf::new().join("FPM.ftd")) {
        serve_fpm_file(&config).await
    } else if path.eq(&std::path::PathBuf::new().join("")) {
        serve_files(&req, &mut config, &path.join("/")).await
    } else {
        serve_files(&req, &mut config, &path).await
    };
    println!("response time: {:?} for path: {:?}", time.elapsed(), path);
    response
}

#[actix_web::main]
pub async fn fpm_serve(
    bind_address: &str,
    port: Option<u16>,
    identities: Option<String>,
) -> std::io::Result<()> {
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

    fn get_available_port(port: Option<u16>, bind_address: &str) -> Option<std::net::TcpListener> {
        let available_listener =
            |port: u16, bind_address: &str| std::net::TcpListener::bind((bind_address, port));

        if let Some(port) = port {
            return match available_listener(port, bind_address) {
                Ok(l) => Some(l),
                Err(_) => None,
            };
        }

        for x in 8000..9000 {
            match available_listener(x, bind_address) {
                Ok(l) => return Some(l),
                Err(_) => continue,
            }
        }
        None
    }

    let tcp_listener = match get_available_port(port, bind_address) {
        Some(listener) => listener,
        None => {
            eprintln!(
                "{}",
                port.map(|x| format!(
                    r#"provided port {} is not available,
You can try without providing port, it will automatically pick unused port"#,
                    x
                ))
                .unwrap_or_else(|| {
                    "Tried picking port between port 8000 to 9000, not available -:(".to_string()
                })
            );
            return Ok(());
        }
    };

    let app = move || {
        let local_identities = fpm::user_group::parse_identities(
            identities
                .clone()
                .unwrap_or_else(|| "".to_string())
                .as_str(),
        );

        {
            if cfg!(feature = "remote") {
                let json_cfg = actix_web::web::JsonConfig::default()
                    .content_type(|mime| mime == mime_guess::mime::APPLICATION_JSON)
                    .limit(9862416400);

                actix_web::App::new()
                    .app_data(json_cfg)
                    .app_data(actix_web::web::Data::new(AppState { local_identities }))
                    .route("/-/sync/", actix_web::web::post().to(fpm::apis::sync))
                    .route("/-/sync2/", actix_web::web::post().to(fpm::apis::sync2))
                    .route("/-/clone/", actix_web::web::get().to(fpm::apis::clone))
            } else {
                actix_web::App::new()
                    .app_data(actix_web::web::Data::new(AppState { local_identities }))
            }
        }
        .route(
            "/-/view-src/{path:.*}",
            actix_web::web::get().to(fpm::apis::view_source),
        )
        .route("/-/edit/", actix_web::web::post().to(fpm::apis::edit))
        .route(
            "/-/revert/",
            actix_web::web::post().to(fpm::apis::edit::revert),
        )
        .route(
            "/-/editor-sync/",
            actix_web::web::get().to(fpm::apis::edit::sync),
        )
        .route("/{path:.*}", actix_web::web::get().to(serve))
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
        .await
}

// fn authentication(req: &actix_web::HttpRequest) -> bool {
//     false
// }

// cargo install --features controller --path=.
// FPM_CONTROLLER=http://127.0.0.1:8000 FPM_INSTANCE_ID=12345 fpm serve 8001
