// Discussion: https://github.com/FifthTry/fpm/discussions/475
// Docs: https://fpm.dev/sitemap/clear-cache/

#[derive(Default, Debug)]
pub struct QueryParams {
    file: Vec<String>,
    package: Vec<String>,
    all_dependencies: bool,
}

fn query(uri: &str) -> fpm::Result<QueryParams> {
    use itertools::Itertools;
    let query = fpm::utils::query(uri)?;
    Ok(QueryParams {
        file: query
            .iter()
            .filter_map(|(key, value)| {
                if key.eq("file") {
                    Some(value.to_string())
                } else {
                    None
                }
            })
            .collect_vec(),
        package: query
            .iter()
            .filter_map(|(key, value)| {
                if key.eq("package") {
                    Some(value.to_string())
                } else {
                    None
                }
            })
            .collect_vec(),
        all_dependencies: query
            .iter()
            .any(|(key, value)| key.eq("all-dependencies") && (value.eq("true") || value.eq("t"))),
    })
}

pub async fn clear(req: &actix_web::HttpRequest) -> actix_web::HttpResponse {
    let query = match query(req.uri().to_string().as_str()) {
        Ok(q) => q,
        Err(err) => {
            eprintln!(
                "FPM-Error: /-/clear-cache/, uri: {:?}, error: {:?}",
                req.uri(),
                err
            );
            return actix_web::HttpResponse::InternalServerError().body(err.to_string());
        }
    };

    if let Err(err) = clear_(&query).await {
        eprintln!(
            "FPM-Error: /-/clear-cache/, query: {:?}, error: {:?}",
            query, err
        );
        return actix_web::HttpResponse::InternalServerError().body(err.to_string());
    }

    actix_web::HttpResponse::Ok().body("Done".to_string())
}

pub async fn clear_(query: &QueryParams) -> fpm::Result<()> {
    let config = fpm::time("Config::read()").it(fpm::Config::read(None, false).await?);
    if config.package.download_base_url.is_none() {
        return Err(fpm::Error::APIResponseError(
            "cannot remove anything, package does not have `download_base_url`".to_string(),
        ));
    }

    // file: file path can be from main package or .packages folder
    for file in query.file.iter() {
        let main_file_path = config.root.join(file.as_str());
        let package_file_path = config.packages_root.join(file.as_str());
        if main_file_path.exists() {
            let path = tokio::fs::canonicalize(main_file_path).await?;
            if path.starts_with(&config.root) {
                fpm::utils::remove(path.as_path()).await?;
            }
        } else if package_file_path.exists() {
            let path = tokio::fs::canonicalize(package_file_path).await?;
            if path.starts_with(&config.root) {
                fpm::utils::remove(path.as_path()).await?;
            }
        } else {
            println!("Not able to remove file from cache: {}", file);
        }
    }

    // package value: main, <package-name>
    for package in query.package.iter() {
        if package.eq("main") {
            // TODO: List directories and files other than main
            fpm::utils::remove_except(&config.root, &[".packages", ".build"]).await?;
        } else {
            let path = tokio::fs::canonicalize(config.packages_root.join(package)).await?;
            if path.starts_with(&config.packages_root) {
                fpm::utils::remove(path.as_path()).await?;
            }
        }
    }

    if query.all_dependencies {
        tokio::fs::remove_dir_all(&config.packages_root).await?;
    }

    // Download FPM.ftd again after removing all the content
    if !config.root.join("FPM.ftd").exists() {
        fpm::commands::serve::download_init_package(config.package.download_base_url).await?;
    }

    Ok(())
}
