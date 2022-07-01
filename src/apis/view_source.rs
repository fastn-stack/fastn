pub(crate) async fn view_source(req: actix_web::HttpRequest) -> actix_web::HttpResponse {
    // TODO: Need to remove unwrap
    let path = {
        let mut path: std::path::PathBuf = req.match_info().query("path").parse().unwrap();
        if path.eq(&std::path::PathBuf::new().join("")) {
            path = path.join("/");
        }
        path
    };

    let path = match path.to_str() {
        Some(s) => s,
        None => {
            println!("handle_ftd: Not able to convert path");
            return actix_web::HttpResponse::InternalServerError().body("".as_bytes());
        }
    };

    match handle_view_source(path).await {
        Ok(body) => actix_web::HttpResponse::Ok().body(body),
        Err(e) => {
            println!("new_path: {}, Error: {:?}", path, e);
            actix_web::HttpResponse::InternalServerError().body(e.to_string())
        }
    }
}

async fn handle_view_source(path: &str) -> fpm::Result<Vec<u8>> {
    let mut config = fpm::Config::read2(None, false).await?;
    let file_name = config.get_file_path_and_resolve(path).await?;
    let file = config.get_file_and_package_by_id(path).await?;
    match file {
        fpm::File::Ftd(_) | fpm::File::Markdown(_) | fpm::File::Code(_) => {
            let editor_content = format!(
                "{}\n\n-- source:\n$processor$: fetch-file\npath:{}\n\n-- path: {}\n\n",
                fpm::editor_ftd(),
                file_name,
                file_name,
            );
            let main_document = fpm::Document {
                id: "editor.ftd".to_string(),
                content: editor_content,
                parent_path: config.root.as_str().to_string(),
                package_name: config.package.name.clone(),
            };
            fpm::package_doc::read_ftd(&mut config, &main_document, "/", false).await
        }
        fpm::File::Static(file) | fpm::File::Image(file) => Ok(file.content),
    }
}
