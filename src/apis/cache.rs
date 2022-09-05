// TODO: Ideally we should remove files carefully, there may be some configuration files as well
// We should only remove files which are present in sitemap + FPM.ftd + .packages

// Note: If any read request in the system, It should not delete files.
// In this case need to show the error, can't remove the files.

use itertools::Itertools;

/// Remove path: It can be directory or file
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

/// Remove content from root except provided values
async fn remove_except(root: &camino::Utf8PathBuf, except: &[&str]) -> fpm::Result<()> {
    let except = except
        .into_iter()
        .map(|x| std::path::PathBuf::from(x))
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

#[derive(serde::Deserialize)]
pub struct QueryParams {
    files: Option<String>,
    packages: Option<String>,
}

pub async fn clear(req: actix_web::HttpRequest) -> actix_web::Result<actix_web::HttpResponse> {
    let query = actix_web::web::Query::<QueryParams>::from_query(req.query_string())?;
    clear_(query.0).await.unwrap(); // TODO: Remove unwrap
    Ok(actix_web::HttpResponse::Ok().body("Done".to_string()))
}

pub async fn clear_(query: QueryParams) -> fpm::Result<()> {
    // Handle options
    // 1. files=<file-path>
    // 2. packages
    //  - main
    //  - all: remove main + .packages
    //  - packages: will remove all packages
    //  - name of the packages like: fpm.dev,foo,c, only name of the packages, entries from .packages folder
    //  - can remove individual packages files, <package-name>/<file path>

    // First let's say clear all

    let config = fpm::time("Config::read()").it(fpm::Config::read(None, false).await?);
    if config.package.download_base_url.is_none() {
        // If this is present, so `fpm` will not be able to serve after removing the content
        // Only in case of main package, in case of dependencies we can delete
        // return Ok(actix_web::HttpResponse::Ok()
        //     .body("Cannot delete anything because I cannot download it again".to_string()));
    }

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
        // package value: all, main, packages, <package-name>, <package-file-path>
        if package.trim().to_lowercase().eq("all") {
            remove_except(&config.root, &[]).await?;
        } else if package.trim().to_lowercase().eq("main") {
            remove_except(&config.root, &[".packages", ".build"]).await?;
        } else if package.trim().to_lowercase().eq("packages") {
            tokio::fs::remove_dir_all(&config.packages_root).await?;
        } else {
            remove_file(&config.packages_root.join(package)).await?;
        }
    }

    // Download FPM.ftd again after removing all the content
    if !config.root.join("FPM.ftd").exists() {
        fpm::commands::serve::download_init_package(config.package.download_base_url)
            .await
            .unwrap();
    }

    Ok(())
}
