pub(crate) async fn view_source(req: &fpm::http::Request) -> fpm::http::Response {
    // TODO: Need to remove unwrap
    let path = {
        let mut path: camino::Utf8PathBuf = req
            .path()
            .replacen("/-/view-source/", "", 1)
            .parse()
            .unwrap();
        if path.eq(&camino::Utf8PathBuf::new().join("")) {
            path = path.join("/");
        }
        path
    };

    match handle_view_source(path.as_str()).await {
        Ok(body) => fpm::http::ok(body),
        Err(e) => {
            fpm::server_error!("new_path: {}, Error: {:?}", path, e)
        }
    }
}

async fn handle_view_source(path: &str) -> fpm::Result<Vec<u8>> {
    let mut config = fpm::Config::read(None, false).await?;
    let file_name = config.get_file_path_and_resolve(path).await?;
    let file = config.get_file_and_package_by_id(path).await?;

    match file {
        fpm::File::Ftd(_) | fpm::File::Markdown(_) | fpm::File::Code(_) => {
            let snapshots = fpm::snapshot::get_latest_snapshots(&config.root).await?;
            let diff = get_diff(&file, &snapshots).await;
            let editor_ftd = fpm::package_info_editor(&config, file_name.as_str(), diff)?;
            let main_document = fpm::Document {
                id: "editor.ftd".to_string(),
                content: editor_ftd,
                parent_path: config.root.as_str().to_string(),
                package_name: config.package.name.clone(),
            };
            fpm::package_doc::read_ftd(&mut config, &main_document, "/", false).await
        }
        fpm::File::Static(ref file) | fpm::File::Image(ref file) => Ok(file.content.to_owned()),
    }
}

pub(crate) async fn get_diff(
    doc: &fpm::File,
    snapshots: &std::collections::BTreeMap<String, u128>,
) -> fpm::Result<Option<String>> {
    if let Some(timestamp) = snapshots.get(&doc.get_id()) {
        let path = fpm::utils::history_path(&doc.get_id(), &doc.get_base_path(), timestamp);
        let content = tokio::fs::read_to_string(&doc.get_full_path()).await?;

        let existing_doc = tokio::fs::read_to_string(&path).await?;
        if content.eq(&existing_doc) {
            return Ok(None);
        }
        let patch = diffy::create_patch(&existing_doc, &content);

        return Ok(Some(patch.to_string().replace("---", "\\---")));
    }
    Ok(None)
}
