pub(crate) async fn view_source(req: &fastn_core::http::Request) -> fastn_core::http::Response {
    // TODO: Need to remove unwrap
    let path = {
        let mut path: camino::Utf8PathBuf =
            req.path().replacen("/-/view-src/", "", 1).parse().unwrap();
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

async fn handle_view_source(req: &fastn_core::http::Request, path: &str) -> fastn_core::Result<Vec<u8>> {
    let mut config = fastn_core::Config::read(None, false, Some(req)).await?;
    let file_name = config.get_file_path_and_resolve(path).await?;
    let file = config.get_file_and_package_by_id(path).await?;

    match file {
        fastn_core::File::Ftd(_) | fastn_core::File::Markdown(_) | fastn_core::File::Code(_) => {
            let snapshots = fastn_core::snapshot::get_latest_snapshots(&config.root).await?;
            let diff = get_diff(&file, &snapshots).await;
            let editor_ftd = fastn_core::package_info_editor(&config, file_name.as_str(), diff)?;
            let main_document = fastn_core::Document {
                id: "editor.ftd".to_string(),
                content: editor_ftd,
                parent_path: config.root.as_str().to_string(),
                package_name: config.package.name.clone(),
            };
            fastn_core::package::package_doc::read_ftd(&mut config, &main_document, "/", false).await
        }
        fastn_core::File::Static(ref file) | fastn_core::File::Image(ref file) => Ok(file.content.to_owned()),
    }
}

pub(crate) async fn get_diff(
    doc: &fastn_core::File,
    snapshots: &std::collections::BTreeMap<String, u128>,
) -> fastn_core::Result<Option<String>> {
    if let Some(timestamp) = snapshots.get(&doc.get_id()) {
        let path = fastn_core::utils::history_path(&doc.get_id(), &doc.get_base_path(), timestamp);
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
