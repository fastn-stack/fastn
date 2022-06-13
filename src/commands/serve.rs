async fn handle_ftd(config: &mut fpm::Config, path: std::path::PathBuf) -> actix_web::HttpResponse {
    use itertools::Itertools;
    let dependencies = if let Some(package) = config.package.translation_of.as_ref() {
        let mut deps = package
            .get_flattened_dependencies()
            .into_iter()
            .unique_by(|dep| dep.package.name.clone())
            .collect_vec();
        deps.extend(
            config
                .package
                .get_flattened_dependencies()
                .into_iter()
                .unique_by(|dep| dep.package.name.clone()),
        );
        deps
    } else {
        config
            .package
            .get_flattened_dependencies()
            .into_iter()
            .unique_by(|dep| dep.package.name.clone())
            .collect_vec()
    };

    let mut asset_documents = std::collections::HashMap::new();
    asset_documents.insert(
        config.package.name.clone(),
        config.package.get_assets_doc(config, "/").await.unwrap(),
    );

    for dep in &dependencies {
        asset_documents.insert(
            dep.package.name.clone(),
            dep.package.get_assets_doc(config, "/").await.unwrap(),
        );
    }

    let new_path = match path.to_str() {
        Some(s) => s.replace("-/", ""),
        None => {
            println!("handle_ftd: Not able to convert path");
            return actix_web::HttpResponse::InternalServerError().body("".as_bytes());
        }
    };

    let dep_package = find_dep_package(config, &dependencies, &new_path);

    let f = match config.get_file_by_id(&new_path, dep_package).await {
        Ok(f) => f,
        Err(e) => {
            println!("path: {}, Error: {:?}", new_path, e);
            return actix_web::HttpResponse::InternalServerError().body("".as_bytes());
        }
    };

    config.current_document = Some(f.get_id());
    return match f {
        fpm::File::Ftd(main_document) => {
            return match fpm::commands::build::process_ftd(
                config,
                &main_document,
                None,
                None,
                Default::default(),
                "/",
                &asset_documents,
                false,
            )
            .await
            {
                Ok(r) => actix_web::HttpResponse::Ok().body(r),
                Err(e) => actix_web::HttpResponse::InternalServerError().body(e.to_string()),
            };
        }
        _ => actix_web::HttpResponse::InternalServerError().body("".as_bytes()),
    };

    fn find_dep_package<'a>(
        config: &'a fpm::Config,
        dep: &'a [fpm::Dependency],
        file_path: &'a str,
    ) -> &'a fpm::Package {
        dep.iter()
            .find(|d| file_path.starts_with(&d.package.name))
            .map(|x| &x.package)
            .unwrap_or(&config.package)
    }
}

async fn handle_dash(
    req: &actix_web::HttpRequest,
    config: &fpm::Config,
    path: std::path::PathBuf,
) -> actix_web::HttpResponse {
    let new_path = match path.to_str() {
        Some(s) => s.replace("-/", ""),
        None => {
            println!("handle_dash: Not able to convert path");
            return actix_web::HttpResponse::InternalServerError().body("".as_bytes());
        }
    };

    let file_path = if new_path.starts_with(&config.package.name) {
        std::path::PathBuf::new().join(
            new_path
                .strip_prefix(&(config.package.name.to_string() + "/"))
                .unwrap(),
        )
    } else {
        std::path::PathBuf::new().join(".packages").join(new_path)
    };

    server_static_file(req, file_path).await
}

async fn server_static_file(
    req: &actix_web::HttpRequest,
    file_path: std::path::PathBuf,
) -> actix_web::HttpResponse {
    if !file_path.exists() {
        return actix_web::HttpResponse::NotFound().body("".as_bytes());
    }

    match actix_files::NamedFile::open_async(file_path).await {
        Ok(r) => r.into_response(req),
        Err(_e) => actix_web::HttpResponse::NotFound().body("TODO".as_bytes()),
    }
}
async fn serve_static(req: actix_web::HttpRequest) -> actix_web::HttpResponse {
    let mut config = fpm::Config::read(None).await.unwrap();
    let path: std::path::PathBuf = req.match_info().query("path").parse().unwrap();

    let favicon = std::path::PathBuf::new().join("favicon.ico");
    if path.starts_with("-/") {
        handle_dash(&req, &config, path).await
    } else if path.eq(&favicon) {
        server_static_file(&req, favicon).await
    } else if path.eq(&std::path::PathBuf::new().join("")) {
        handle_ftd(&mut config, path.join("index")).await
    } else {
        handle_ftd(&mut config, path).await
    }
}

#[actix_web::main]
pub async fn serve(port: &str) -> std::io::Result<()> {
    println!("### Server Started ###");
    println!("Go to: http://127.0.0.1:{}", port);
    actix_web::HttpServer::new(|| {
        actix_web::App::new().route("/{path:.*}", actix_web::web::get().to(serve_static))
    })
    .bind(format!("127.0.0.1:{}", port))?
    .run()
    .await
}
