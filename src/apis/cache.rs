// Discussion: https://github.com/FifthTry/fpm/discussions/475
// Docs: TODO

/// Remove path: It can be directory or file
async fn remove(path: &camino::Utf8PathBuf) -> std::io::Result<()> {
    if path.is_file() {
        tokio::fs::remove_file(path).await?;
    } else if path.is_dir() {
        tokio::fs::remove_dir_all(path).await?
    } else if path.is_symlink() {
        // TODO:
        // It can be a directory or a file
    }
    Ok(())
}

/// Remove from provided `root` except given list
async fn remove_except(root: &camino::Utf8PathBuf, except: &[&str]) -> fpm::Result<()> {
    use itertools::Itertools;
    let except = except
        .iter()
        .map(|x| root.join(x))
        .map(|x| x.into_std_path_buf())
        .collect_vec();
    let mut all = tokio::fs::read_dir(root).await?;
    while let Some(file) = all.next_entry().await? {
        if except.contains(&file.path()) {
            continue;
        }
        if file.metadata().await?.is_dir() {
            tokio::fs::remove_dir_all(file.path()).await?;
        } else if file.metadata().await?.is_file() {
            tokio::fs::remove_file(file.path()).await?;
        }
    }
    Ok(())
}

#[derive(Default, Debug)]
pub struct QueryParams {
    file: Vec<String>,
    package: Vec<String>,
    all_dependencies: bool,
}

/// /api/?a=1&b=2&c=3 => vec[(a, 1), (b, 2), (c, 3)]
fn query(uri: &str) -> fpm::Result<Vec<(String, String)>> {
    use itertools::Itertools;
    Ok(
        url::Url::parse(format!("https://fifthtry.com/{}", uri).as_str())?
            .query_pairs()
            .into_owned()
            .collect_vec(),
    )
}

fn clear_cache_query(uri: &str) -> fpm::Result<QueryParams> {
    use itertools::Itertools;
    let query = query(uri)?;
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

pub async fn clear(req: actix_web::HttpRequest) -> actix_web::HttpResponse {
    let query = match clear_cache_query(req.uri().to_string().as_str()) {
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
    if config.package.download_base_url.is_none() {}

    // file: file path can be from main package or .packages folder
    for file in query.file.iter() {
        let main_file_path = config.root.join(file.as_str());
        let package_file_path = config.packages_root.join(file.as_str());
        if main_file_path.exists() {
            remove(&main_file_path).await?;
        } else if package_file_path.exists() {
            remove(&package_file_path).await?;
        } else {
            println!("Not able to remove file from cache: {}", file);
        }
    }

    // package value: main, <package-name>
    for package in query.package.iter() {
        if package.eq("main") {
            // TODO: List directories and files other than main
            remove_except(&config.root, &[".packages", ".build"]).await?;
        } else {
            remove(&config.packages_root.join(package)).await?;
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

#[cfg(test)]
mod tests {
    #[test]
    fn query() {
        assert_eq!(
            super::query("/api/?a=1&b=2&c=3").unwrap(),
            vec![
                ("a".to_string(), "1".to_string()),
                ("b".to_string(), "2".to_string()),
                ("c".to_string(), "3".to_string())
            ]
        )
    }
}
