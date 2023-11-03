pub(crate) async fn edit_source(req: &fastn_core::http::Request) -> fastn_core::http::Response {
    // TODO: Need to remove unwrap
    let path = {
        let mut path: camino::Utf8PathBuf =
            req.path().replacen("/-/edit-src/", "", 1).parse().unwrap();
        if path.eq(&camino::Utf8PathBuf::new().join("")) {
            path = path.join("/");
        }
        path
    };

    match handle_view_source(req, path.as_str()).await {
        Ok(body) => fastn_core::http::ok(body),
        Err(e) => {
            fastn_core::server_error!("new_path: {}, Error: {:?}", path, e)
        }
    }
}

async fn handle_view_source(
    req: &fastn_core::http::Request,
    path: &str,
) -> fastn_core::Result<Vec<u8>> {
    let config = fastn_core::Config::read(None, false).await?;
    let mut req_config = fastn_core::RequestConfig::new(&config, req, "editor-source.ftd", "/");

    let file_name = config.get_file_path_and_resolve(path).await?;
    let file = req_config.get_file_and_package_by_id(path).await?;

    match file {
        fastn_core::File::Ftd(_) | fastn_core::File::Markdown(_) | fastn_core::File::Code(_) => {
            let editor_ftd = fastn_core::package_editor_source(&config, file_name.as_str())?;
            let main_document = fastn_core::Document {
                id: "editor-source.ftd".to_string(),
                content: editor_ftd,
                parent_path: config.root.as_str().to_string(),
                package_name: config.package.name.clone(),
            };
            fastn_core::package::package_doc::read_ftd(
                &mut req_config,
                &main_document,
                "/",
                false,
                false,
            )
            .await
            .map(|r| r.html())
        }
        fastn_core::File::Static(ref file) | fastn_core::File::Image(ref file) => {
            Ok(file.content.to_owned())
        }
    }
}
