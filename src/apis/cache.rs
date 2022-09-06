// Discussion: https://github.com/FifthTry/fpm/discussions/475
// Docs: TODO

use itertools::Itertools;

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
    let except = except.iter().map(std::path::PathBuf::from).collect_vec();
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

#[derive(Default)]
pub struct QueryParams {
    file: Vec<String>,
    package: Vec<String>,
    all_dependencies: bool,
}

/// http://example.com/?a=1&b=2&c=3 => vec[(a, 1), (b, 2), (c, 3)]
/// TODO: convert it into test case
fn query(uri: &str) -> fpm::Result<Vec<(String, String)>> {
    Ok(url::Url::parse(uri)?
        .query_pairs()
        .into_owned()
        .collect_vec())
}

fn clear_cache_query(uri: &str) -> fpm::Result<QueryParams> {
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

pub async fn clear(req: actix_web::HttpRequest) -> actix_web::Result<actix_web::HttpResponse> {
    let query = clear_cache_query(req.uri().to_string().as_str()).unwrap();
    clear_(query).await.unwrap(); // TODO: Remove unwrap
    Ok(actix_web::HttpResponse::Ok().body("Done".to_string()))
}

pub async fn clear_(query: QueryParams) -> fpm::Result<()> {
    let config = fpm::time("Config::read()").it(fpm::Config::read(None, false).await?);
    if config.package.download_base_url.is_none() {}

    for file in query.file {
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
    for package in query.package {
        if package.eq("main") {
            // TODO: List directories and files other than main
            remove_except(&config.root, &[".packages", ".build"]).await?;
        } else {
            remove(&config.packages_root.join(package)).await?;
        }
    }

    if query.all_dependencies {
        remove_except(&config.packages_root, &[]).await?;
    }

    // Download FPM.ftd again after removing all the content
    if !config.root.join("FPM.ftd").exists() {
        fpm::commands::serve::download_init_package(config.package.download_base_url).await?;
    }

    Ok(())
}
