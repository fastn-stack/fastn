// For now all
pub async fn clear(req: actix_web::HttpRequest) -> actix_web::Result<actix_web::HttpResponse> {
    // Ideally we should remove files carefully, there may be some configuration files as well
    // We should only remove files which are present in sitemap + FPM.ftd + .packages

    // First let's say clear all

    let config = fpm::time("Config::read()").it(fpm::Config::read(None, false).await.unwrap());
    if config.package.download_base_url.is_none() {
        // Only in case of main package, in case of dependencies we can delete
        return Ok(actix_web::HttpResponse::Ok()
            .body("Cannot delete anything because I cannot download it again".to_string()));
    }

    // #[derive(serde::Deserialize)]
    // pub struct QueryParams {
    //     clear: Option<String>, // all, dep, main
    // }
    //
    // let query = actix_web::web::Query::<QueryParams>::from_query(req.query_string())?;

    let mut all = tokio::fs::read_dir(&config.root).await.unwrap();

    while let Some(file) = all.next_entry().await.unwrap() {
        if file.metadata().await.unwrap().is_dir() {
            tokio::fs::remove_dir_all(file.path()).await.unwrap();
        } else if file.metadata().await.unwrap().is_file() {
            tokio::fs::remove_file(file.path()).await.unwrap();
        } else if file.metadata().await.unwrap().is_symlink() {
            // tokio::fs::don't know the function name(file.path()).await.unwrap();
        }
    }

    if !config.root.join("FPM.ftd").exists() {
        fpm::commands::serve::download_init_package(config.package.download_base_url)
            .await
            .unwrap();
    }

    // std::fs::remove_file()
    Ok(actix_web::HttpResponse::Ok().body("Done".to_string()))
}

/*

files=a,c,...
all default
.packages/a
.packages/a/b
.packages

clear=all
clear=dependencies
documents=a,c...
packages=a,b

download-base-url: FPM.ftd
*/
