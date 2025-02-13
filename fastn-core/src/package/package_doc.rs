impl fastn_core::Package {
    pub(crate) async fn fs_fetch_by_file_name(
        &self,
        name: &str,
        package_root: Option<&fastn_ds::Path>,
        ds: &fastn_ds::DocumentStore,
        session_id: &Option<String>,
    ) -> fastn_core::Result<Vec<u8>> {
        let package_root = self.package_root_with_default(package_root)?;

        let file_path = package_root.join(name.trim_start_matches('/'));
        // Issue 1: Need to remove / from the start of the name
        match ds.read_content(&file_path, session_id).await {
            Ok(content) => Ok(content),
            Err(err) => {
                tracing::error!(
                    msg = "file-read-error: file not found",
                    path = file_path.to_string()
                );
                Err(Err(err)?)
            }
        }
    }

    pub(crate) fn package_root_with_default(
        &self,
        package_root: Option<&fastn_ds::Path>,
    ) -> fastn_core::Result<fastn_ds::Path> {
        if let Some(package_root) = package_root {
            Ok(package_root.to_owned())
        } else {
            match self.fastn_path.as_ref() {
                Some(path) if path.parent().is_some() => Ok(path.parent().unwrap()),
                _ => {
                    tracing::error!(
                        msg = "package root not found. Package: {}",
                        package = self.name,
                    );
                    Err(fastn_core::Error::PackageError {
                        message: format!("package root not found. Package: {}", &self.name),
                    })
                }
            }
        }
    }

    pub(crate) async fn get_manifest(
        &self,
        ds: &fastn_ds::DocumentStore,
        session_id: &Option<String>,
    ) -> fastn_core::Result<Option<fastn_core::Manifest>> {
        let manifest_path = self
            .fastn_path
            .as_ref()
            .and_then(|path| path.parent())
            .map(|parent| parent.join(fastn_core::manifest::MANIFEST_FILE));
        let manifest: Option<fastn_core::Manifest> = if let Some(manifest_path) = manifest_path {
            match ds.read_content(&manifest_path, session_id).await {
                Ok(manifest_bytes) => match serde_json::de::from_slice(manifest_bytes.as_slice()) {
                    Ok(manifest) => Some(manifest),
                    Err(_) => None,
                },
                Err(_) => None,
            }
        } else {
            None
        };

        Ok(manifest)
    }

    #[tracing::instrument(skip(self, ds))]
    pub(crate) async fn fs_fetch_by_id(
        &self,
        id: &str,
        package_root: Option<&fastn_ds::Path>,
        ds: &fastn_ds::DocumentStore,
        session_id: &Option<String>,
    ) -> fastn_core::Result<(String, Vec<u8>)> {
        let id = id.trim_start_matches(&format!("/-/{}", self.name));
        if fastn_core::file::is_static(id)? {
            if let Ok(data) = self
                .fs_fetch_by_file_name(id, package_root, ds, session_id)
                .await
            {
                return Ok((id.to_string(), data));
            }
        } else {
            for name in file_id_to_names(id) {
                if let Ok(data) = self
                    .fs_fetch_by_file_name(name.as_str(), package_root, ds, session_id)
                    .await
                {
                    return Ok((name, data));
                }
            }
        }

        tracing::error!(
            msg = "fs-error: file not found",
            document = id,
            package = self.name
        );
        Err(fastn_core::Error::PackageError {
            message: format!(
                "fs_fetch_by_id:: Corresponding file not found for id: {}. Package: {}",
                id, &self.name
            ),
        })
    }

    // #[cfg(feature = "use-config-json")]
    // pub(crate) async fn fs_fetch_by_id(
    //     &self,
    //     id: &str,
    //     package_root: Option<&fastn_ds::Path>,
    //     ds: &fastn_ds::DocumentStore,
    //     session_id: &Option<String>,
    // ) -> fastn_core::Result<(String, Vec<u8>)> {
    //     let new_id = if fastn_core::file::is_static(id)? {
    //         if !self.files.contains(&id.trim_start_matches('/').to_string()) {
    //             return Err(fastn_core::Error::PackageError {
    //                 message: format!(
    //                     "fs_fetch_by_id:: Corresponding file not found for id: {}. Package: {} 3",
    //                     id, &self.name
    //                 ),
    //             });
    //         }
    //
    //         Some(id.to_string())
    //     } else {
    //         file_id_to_names(id)
    //             .iter()
    //             .find(|id| self.files.contains(id))
    //             .map(|id| id.to_string())
    //     };
    //     if let Some(id) = new_id {
    //         if let Ok(data) = self
    //             .fs_fetch_by_file_name(id.as_str(), package_root, ds, session_id)
    //             .await
    //         {
    //             return Ok((id.to_string(), data));
    //         }
    //     }
    //
    //     tracing::error!(
    //         msg = "fs-error: file not found",
    //         document = id,
    //         package = self.name
    //     );
    //     Err(fastn_core::Error::PackageError {
    //         message: format!(
    //             "fs_fetch_by_id:: Corresponding file not found for id: {}. Package: {} 2",
    //             id, &self.name
    //         ),
    //     })
    // }

    pub(crate) async fn resolve_by_file_name(
        &self,
        file_path: &str,
        package_root: Option<&fastn_ds::Path>,
        ds: &fastn_ds::DocumentStore,
        session_id: &Option<String>,
    ) -> fastn_core::Result<Vec<u8>> {
        let manifest = self.get_manifest(ds, session_id).await?;

        let new_file_path = match &manifest {
            Some(manifest) if manifest.files.contains_key(file_path) => file_path.to_string(),
            Some(manifest) => {
                let new_file_path = match file_path.rsplit_once('.') {
                    Some((remaining, ext))
                        if mime_guess::MimeGuess::from_ext(ext)
                            .first_or_octet_stream()
                            .to_string()
                            .starts_with("image/") =>
                    {
                        if remaining.ends_with("-dark") {
                            format!(
                                "{}.{}",
                                remaining.trim_matches('/').trim_end_matches("-dark"),
                                ext
                            )
                        } else {
                            format!("{}-dark.{}", remaining.trim_matches('/'), ext)
                        }
                    }
                    _ => {
                        tracing::error!(
                            file_path = file_path,
                            msg = "file_path error: can not get the dark"
                        );
                        return Err(fastn_core::Error::PackageError {
                            message: format!(
                                "fs_fetch_by_file_name:: Corresponding file not found for file_path: {}. Package: {}",
                                file_path, &self.name
                            ),
                        });
                    }
                };

                if !manifest.files.contains_key(&new_file_path) {
                    tracing::error!(
                        file_path = file_path,
                        msg = "file_path error: can not get the dark"
                    );
                    return Err(fastn_core::Error::PackageError {
                        message: format!(
                            "fs_fetch_by_file_name:: Corresponding file not found for file_path: {}. Package: {}",
                            file_path, &self.name
                        ),
                    });
                }

                new_file_path
            }
            None => file_path.to_string(),
        };

        self.fs_fetch_by_file_name(&new_file_path, package_root, ds, session_id)
            .await
    }

    #[tracing::instrument(skip(self, ds))]
    pub(crate) async fn resolve_by_id(
        &self,
        id: &str,
        package_root: Option<&fastn_ds::Path>,
        config_package_name: &str,
        ds: &fastn_ds::DocumentStore,
        session_id: &Option<String>,
    ) -> fastn_core::Result<(String, Vec<u8>)> {
        if config_package_name.eq(&self.name) {
            if fastn_core::file::is_static(id)? {
                if let Ok(data) = self
                    .fs_fetch_by_file_name(id, package_root, ds, session_id)
                    .await
                {
                    return Ok((id.to_string(), data));
                }

                let new_id = match id.rsplit_once('.') {
                    Some((remaining, ext))
                        if mime_guess::MimeGuess::from_ext(ext)
                            .first_or_octet_stream()
                            .to_string()
                            .starts_with("image/") =>
                    {
                        if remaining.ends_with("-dark") {
                            format!(
                                "{}.{}",
                                remaining.trim_matches('/').trim_end_matches("-dark"),
                                ext
                            )
                        } else {
                            format!("{}-dark.{}", remaining.trim_matches('/'), ext)
                        }
                    }
                    _ => {
                        tracing::error!(id = id, msg = "id error: can not get the dark");
                        return Err(fastn_core::Error::PackageError {
                            message: format!(
                                "fs_fetch_by_id:: Corresponding file not found for id: {}. Package: {} 1",
                                id, &self.name
                            ),
                        });
                    }
                };

                if let Ok(data) = self
                    .fs_fetch_by_file_name(&new_id, package_root, ds, session_id)
                    .await
                {
                    return Ok((new_id.to_string(), data));
                }
            } else {
                for name in file_id_to_names(id) {
                    tracing::info!("attempting non static file: {}", name);
                    if let Ok(data) = self
                        .fs_fetch_by_file_name(name.as_str(), package_root, ds, session_id)
                        .await
                    {
                        return Ok((name, data));
                    }
                }
            }
        }

        tracing::info!("resolve_by_id: not found in the package. Trying fs_fetch_by_id");
        self.fs_fetch_by_id(id, package_root, ds, session_id).await
    }
}

pub(crate) fn file_id_to_names(id: &str) -> Vec<String> {
    let id = id.replace("/index.html", "/").replace("index.html", "/");
    if id.eq("/") {
        return vec![
            "index.ftd".to_string(),
            "README.md".to_string(),
            "index.md".to_string(),
            "index.html".to_string(),
        ];
    }
    let mut ids = vec![];
    if !id.ends_with('/') {
        ids.push(id.trim_matches('/').to_string());
    }
    let id = id.trim_matches('/').to_string();
    ids.extend([
        format!("{}.ftd", id),
        format!("{}/index.ftd", id),
        format!("{}/index.html", id),
        // Todo: removing `md` file support for now
        // format!("{}.md", id),
        // format!("{}/README.md", id),
        // format!("{}/index.md", id),
    ]);
    ids
}

pub enum FTDResult {
    Html(Vec<u8>),
    Redirect {
        url: String,
        code: u16,
    },
    Json(Vec<u8>),
    Response {
        response: Vec<u8>,
        status_code: i64,
        content_type: mime_guess::Mime,
        headers: fastn_resolved::Map<fastn_resolved::PropertyValue>,
    },
}

impl FTDResult {
    pub fn html(&self) -> Vec<u8> {
        match self {
            FTDResult::Html(d) => d.to_vec(),
            FTDResult::Redirect { url, .. } => {
                // Note: this is a hack to redirect to a html page, we can not handle code in this
                // case
                fastn_core::utils::redirect_page_html(url).into_bytes()
            }
            FTDResult::Json(_d) => todo!("json not yet handled"),
            FTDResult::Response { .. } => todo!("response not yet handled"),
        }
    }

    pub fn checksum(&self) -> String {
        fastn_core::utils::generate_hash(self.html())
    }
}

#[tracing::instrument(skip_all)]
pub async fn read_ftd(
    config: &mut fastn_core::RequestConfig,
    main: &fastn_core::Document,
    base_url: &str,
    download_assets: bool,
    test: bool,
    preview_session_id: &Option<String>,
) -> fastn_core::Result<FTDResult> {
    read_ftd_(
        config,
        main,
        base_url,
        download_assets,
        test,
        false,
        preview_session_id,
    )
    .await
}

#[tracing::instrument(skip_all)]
pub(crate) async fn read_ftd_(
    config: &mut fastn_core::RequestConfig,
    main: &fastn_core::Document,
    base_url: &str,
    download_assets: bool,
    test: bool,
    only_js: bool,
    preview_session_id: &Option<String>,
) -> fastn_core::Result<FTDResult> {
    tracing::info!(document = main.id);
    match config.config.ftd_edition {
        fastn_core::FTDEdition::FTD2022 => {
            read_ftd_2022(
                config,
                main,
                base_url,
                download_assets,
                test,
                preview_session_id,
            )
            .await
        }
        fastn_core::FTDEdition::FTD2023 => {
            read_ftd_2023(
                config,
                main,
                base_url,
                download_assets,
                only_js,
                preview_session_id,
            )
            .await
        }
    }
}

#[tracing::instrument(name = "read_ftd_2022", skip_all)]
pub(crate) async fn read_ftd_2022(
    config: &mut fastn_core::RequestConfig,
    main: &fastn_core::Document,
    base_url: &str,
    download_assets: bool,
    test: bool,
    preview_session_id: &Option<String>,
) -> fastn_core::Result<FTDResult> {
    let font_style = config.config.get_font_style();
    let c = &config.config.clone();

    let current_package = config
        .config
        .find_package_else_default(main.package_name.as_str(), None);

    config.document_id.clone_from(&main.id);
    config.base_url = base_url.to_string();

    // Get Prefix Body => [AutoImports + Actual Doc content]
    let mut doc_content =
        current_package.get_prefixed_body(main.content.as_str(), main.id.as_str(), true);
    // Fix aliased imports to full path (if any)
    doc_content = current_package.fix_imports_in_body(doc_content.as_str(), main.id.as_str())?;

    let line_number = doc_content.split('\n').count() - main.content.split('\n').count();
    let main_ftd_doc = match fastn_core::doc::interpret_helper(
        main.id_with_package().as_str(),
        doc_content.as_str(),
        config,
        base_url,
        download_assets,
        line_number,
        preview_session_id,
    )
    .await
    {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(msg = "failed to parse", doc = main.id.as_str());
            return Err(fastn_core::Error::PackageError {
                message: format!("failed to parse {:?}", &e),
            });
        }
    };

    if let Some((url, code)) = main_ftd_doc.get_redirect()? {
        return Ok(FTDResult::Redirect { url, code });
    }

    if let Some((response, content_type, status_code)) = main_ftd_doc.get_response()? {
        return Ok(FTDResult::Response {
            response: response.into(),
            content_type: content_type.parse().unwrap(), // TODO: Remove unwrap()
            status_code,
            headers: Default::default(),
        });
    }

    if let Some(v) = main_ftd_doc.get_json()? {
        return Ok(FTDResult::Json(v));
    }

    let executor = ftd::executor::ExecuteDoc::from_interpreter(main_ftd_doc)?;
    let node = ftd::node::NodeData::from_rt(executor);
    let html_ui = ftd::html::HtmlUI::from_node_data(node, "main", test)?;

    let file_content = fastn_core::utils::replace_markers_2022(
        fastn_core::ftd_html(),
        html_ui,
        c,
        main.id_to_path().as_str(),
        font_style.as_str(),
        base_url,
        preview_session_id,
    )
    .await;

    Ok(FTDResult::Html(file_content.into()))
}

#[allow(clippy::await_holding_refcell_ref)]
#[tracing::instrument(name = "read_ftd_2023", skip_all)]
pub(crate) async fn read_ftd_2023(
    config: &mut fastn_core::RequestConfig,
    main: &fastn_core::Document,
    base_url: &str,
    download_assets: bool,
    only_js: bool,
    preview_session_id: &Option<String>,
) -> fastn_core::Result<FTDResult> {
    let package_name = config.config.package.name.to_string();
    let c = &config.config.clone();

    let current_package = config
        .config
        .find_package_else_default(main.package_name.as_str(), None);

    config.document_id.clone_from(&main.id);
    config.base_url = base_url.to_string();

    // Get Prefix Body => [AutoImports + Actual Doc content]
    let mut doc_content =
        current_package.get_prefixed_body(main.content.as_str(), main.id.as_str(), true);
    // Fix aliased imports to full path (if any)
    doc_content = current_package.fix_imports_in_body(doc_content.as_str(), main.id.as_str())?;

    let line_number = doc_content.split('\n').count() - main.content.split('\n').count();
    let main_ftd_doc = match fastn_core::doc::interpret_helper(
        main.id_with_package().as_str(),
        doc_content.as_str(),
        config,
        base_url,
        download_assets,
        line_number,
        preview_session_id,
    )
    .await
    {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(msg = "failed to parse", doc = main.id.as_str());
            return Err(fastn_core::Error::PackageError {
                message: format!("failed to parse {:?}", &e),
            });
        }
    };
    if let Some((url, code)) = main_ftd_doc.get_redirect()? {
        return Ok(FTDResult::Redirect { url, code });
    }
    if let Some((response, content_type, status_code)) = main_ftd_doc.get_response()? {
        return Ok(FTDResult::Response {
            response: response.into(),
            content_type: content_type.parse().unwrap(), // TODO: Remove unwrap()
            status_code,
            headers: Default::default(),
        });
    }
    if let Some(data) = main_ftd_doc.get_json()? {
        return Ok(FTDResult::Json(data));
    }

    let js_ast_data = ftd::js::document_into_js_ast(main_ftd_doc);
    let js_document_script = fastn_js::to_js(js_ast_data.asts.as_slice(), package_name.as_str());
    let js_ftd_script = fastn_js::to_js(
        ftd::js::default_bag_into_js_ast().as_slice(),
        package_name.as_str(),
    );
    let file_content = if only_js {
        fastn_js::ssr_raw_string_without_test(
            &package_name,
            format!("{js_ftd_script}\n{js_document_script}").as_str(),
        )
    } else {
        let ssr_body = if config.request.is_bot() {
            fastn_js::ssr_with_js_string(
                &package_name,
                format!("{js_ftd_script}\n{js_document_script}").as_str(),
            )?
        } else {
            EMPTY_HTML_BODY.to_string()
        };

        fastn_core::utils::replace_markers_2023(
            js_document_script.as_str(),
            js_ast_data.scripts.join("").as_str(),
            ssr_body.as_str(),
            config.config.get_font_style().as_str(),
            ftd::ftd_js_css(),
            base_url,
            c,
            preview_session_id,
        )
        .await
    };

    Ok(FTDResult::Html(file_content.into()))
}

pub(crate) async fn process_ftd(
    config: &mut fastn_core::RequestConfig,
    main: &fastn_core::Document,
    base_url: &str,
    build_static_files: bool,
    test: bool,
    file_path: &str,
    preview_session_id: &Option<String>,
) -> fastn_core::Result<FTDResult> {
    let build_dir = config.config.build_dir();
    let response = read_ftd(
        config,
        main,
        base_url,
        build_static_files,
        test,
        preview_session_id,
    )
    .await?;
    fastn_core::utils::overwrite(&build_dir, file_path, &response.html(), &config.config.ds)
        .await?;

    Ok(response)
}

const EMPTY_HTML_BODY: &str = "<body></body><style id=\"styles\"></style>";
