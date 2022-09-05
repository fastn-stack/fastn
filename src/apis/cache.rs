// TODO: Ideally we should remove files carefully, there may be some configuration files as well
// We should only remove files which are present in sitemap + FPM.ftd + .packages

// Note: If any read request in the system, It should not delete files.
// In this case need to show the error, can't remove the files.

use itertools::Itertools;

/// Remove path:
async fn remove_file(path: &camino::Utf8PathBuf) -> std::io::Result<()> {
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

pub async fn clear(req: actix_web::HttpRequest) -> actix_web::Result<actix_web::HttpResponse> {
    // Handle options
    // 1. files=<file-path>
    // 2. packages
    //  - main
    //  - all: remove main + .packages
    //  - packages: will remove all packages
    //  - name of the packages like: fpm.dev,foo,c, only name of the packages, entries from .packages folder
    //  - todo: can remove individual packages files, <package-name>/<file path>

    // First let's say clear all

    let config = fpm::time("Config::read()").it(fpm::Config::read(None, false).await.unwrap());
    if config.package.download_base_url.is_none() {
        // Only in case of main package, in case of dependencies we can delete
        return Ok(actix_web::HttpResponse::Ok()
            .body("Cannot delete anything because I cannot download it again".to_string()));
    }

    #[derive(serde::Deserialize)]
    pub struct QueryParams {
        files: Option<String>,
        packages: Option<String>,
    }

    let query = actix_web::web::Query::<QueryParams>::from_query(req.query_string())?;

    // 1. files=<file-path>
    let files = {
        if let Some(files) = query.files.as_ref() {
            files.split(',').collect_vec()
        } else {
            vec![]
        }
    };

    for file in files {
        let file_path = config.root.join(file);
        remove_file(&file_path).await.unwrap();
    }

    // 2. packages
    let packages = {
        if let Some(packages) = query.packages.as_ref() {
            packages.split(',').collect_vec()
        } else {
            vec![]
        }
    };

    for package in packages {
        // package: all, main, packages, package-name
        if package.trim().to_lowercase().eq("all") {
            let mut all = tokio::fs::read_dir(&config.root).await.unwrap();
            while let Some(file) = all.next_entry().await.unwrap() {
                if file.metadata().await.unwrap().is_dir() {
                    tokio::fs::remove_dir_all(file.path()).await.unwrap();
                } else if file.metadata().await.unwrap().is_file() {
                    tokio::fs::remove_file(file.path()).await.unwrap();
                }
            }
        } else if package.trim().to_lowercase().eq("main") {
            // remove only main package content
        } else if package.trim().to_lowercase().eq("packages") {
            tokio::fs::remove_dir_all(&config.packages_root)
                .await
                .unwrap();
        } else {
            remove_file(&config.packages_root.join(package)).await?;
        }
    }

    if !config.root.join("FPM.ftd").exists() {
        fpm::commands::serve::download_init_package(config.package.download_base_url)
            .await
            .unwrap();
    }

    Ok(actix_web::HttpResponse::Ok().body("Done".to_string()))
}
