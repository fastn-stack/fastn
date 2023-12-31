impl fastn_core::Package {
    #[tracing::instrument(skip(self))]
    pub(crate) async fn fs_fetch_by_file_name(
        &self,
        name: &str,
        package_root: Option<&camino::Utf8PathBuf>,
        ds: &fastn_ds::DocumentStore,
    ) -> fastn_core::Result<Vec<u8>> {
        tracing::info!(document = name);
        let package_root = self.package_root_with_default(package_root)?;

        let file_path = package_root.join(name.trim_start_matches('/'));
        // Issue 1: Need to remove / from the start of the name
        match ds.read_content(&file_path, None).await {
            Ok(content) => Ok(content),
            Err(err) => {
                tracing::error!(
                    msg = "file-read-error: file not found",
                    path = file_path.as_str()
                );
                Err(Err(err)?)
            }
        }
    }

    pub(crate) fn package_root_with_default(
        &self,
        package_root: Option<&camino::Utf8PathBuf>,
    ) -> fastn_core::Result<camino::Utf8PathBuf> {
        tracing::info!(package = self.name);
        if let Some(package_root) = package_root {
            Ok(package_root.to_owned())
        } else {
            match self.fastn_path.as_ref() {
                Some(path) if path.parent().is_some() => Ok(path.parent().unwrap().to_path_buf()),
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

    #[tracing::instrument(skip(self))]
    pub(crate) async fn fs_fetch_by_id(
        &self,
        id: &str,
        package_root: Option<&camino::Utf8PathBuf>,
        ds: &fastn_ds::DocumentStore,
    ) -> fastn_core::Result<(String, Vec<u8>)> {
        if fastn_core::file::is_static(id)? {
            if let Ok(data) = self.fs_fetch_by_file_name(id, package_root, ds).await {
                return Ok((id.to_string(), data));
            }
        } else {
            for name in file_id_to_names(id) {
                if let Ok(data) = self
                    .fs_fetch_by_file_name(name.as_str(), package_root, ds)
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

    #[tracing::instrument(skip_all)]
    async fn http_fetch_by_file_name(&self, name: &str) -> fastn_core::Result<Vec<u8>> {
        let base = self.download_base_url.as_ref().ok_or_else(|| {
            let message = format!(
                "package base not found. Package: {}, File: {}",
                &self.name, name
            );
            tracing::error!(msg = message);
            fastn_core::Error::PackageError { message }
        })?;

        crate::http::construct_url_and_get(
            format!("{}/{}", base.trim_end_matches('/'), name.trim_matches('/')).as_str(),
        )
        .await
    }

    #[tracing::instrument(skip_all)]
    async fn http_fetch_by_id(&self, id: &str) -> fastn_core::Result<(String, Vec<u8>)> {
        if fastn_core::file::is_static(id)? {
            if let Ok(data) = self.http_fetch_by_file_name(id).await {
                return Ok((id.to_string(), data));
            }
        } else {
            for name in file_id_to_names(id) {
                if let Ok(data) = self.http_fetch_by_file_name(name.as_str()).await {
                    return Ok((name, data));
                }
            }
        }

        let message = format!(
            "http-error: file not found for id: {}. package: {}",
            id, &self.name
        );
        tracing::error!(document = id, msg = message);
        Err(fastn_core::Error::PackageError { message })
    }

    #[tracing::instrument(skip(self))]
    pub(crate) async fn http_download_by_id(
        &self,
        id: &str,
        package_root: Option<&camino::Utf8PathBuf>,
    ) -> fastn_core::Result<(String, Vec<u8>)> {
        tracing::info!(document = id);
        let package_root = self.package_root_with_default(package_root)?;

        let (file_path, data) = self.http_fetch_by_id(id).await?;
        fastn_core::utils::write(
            &package_root,
            file_path.trim_start_matches('/'),
            data.as_slice(),
        )
        .await?;

        Ok((file_path, data))
    }

    #[tracing::instrument(skip_all)]
    pub(crate) async fn http_download_by_file_name(
        &self,
        file_path: &str,
        package_root: Option<&camino::Utf8PathBuf>,
    ) -> fastn_core::Result<Vec<u8>> {
        let package_root = self.package_root_with_default(package_root)?;

        let data = self.http_fetch_by_file_name(file_path).await?;
        fastn_core::utils::write(&package_root, file_path, data.as_slice()).await?;

        Ok(data)
    }

    pub(crate) async fn resolve_by_file_name(
        &self,
        file_path: &str,
        package_root: Option<&camino::Utf8PathBuf>,
        restore_default: bool,
        ds: &fastn_ds::DocumentStore,
    ) -> fastn_core::Result<Vec<u8>> {
        if let Ok(response) = self
            .fs_fetch_by_file_name(file_path, package_root, ds)
            .await
        {
            return Ok(response);
        }
        if let Ok(response) = self
            .http_download_by_file_name(file_path, package_root)
            .await
        {
            return Ok(response);
        }

        if !restore_default {
            return Err(fastn_core::Error::PackageError {
                message: format!(
                    "fs_fetch_by_id:: Corresponding file not found for id: {}. Package: {}",
                    file_path, &self.name
                ),
            });
        }

        let new_file_path = match file_path.rsplit_once('.') {
            Some((remaining, ext))
                if mime_guess::MimeGuess::from_ext(ext)
                    .first_or_octet_stream()
                    .to_string()
                    .starts_with("image/")
                    && remaining.ends_with("-dark") =>
            {
                format!("{}.{}", remaining.trim_end_matches("-dark"), ext)
            }
            _ => {
                return Err(fastn_core::Error::PackageError {
                    message: format!(
                        "fs_fetch_by_id:: Corresponding file not found for id: {}. Package: {}",
                        file_path, &self.name
                    ),
                })
            }
        };

        let root = self.package_root_with_default(package_root)?;

        if let Ok(response) = self
            .fs_fetch_by_file_name(new_file_path.as_str(), package_root, ds)
            .await
        {
            tokio::fs::copy(root.join(new_file_path), root.join(file_path)).await?;
            return Ok(response);
        }

        match self
            .http_download_by_file_name(new_file_path.as_str(), package_root)
            .await
        {
            Ok(response) => {
                tokio::fs::copy(root.join(new_file_path), root.join(file_path)).await?;
                Ok(response)
            }
            Err(e) => Err(e),
        }
    }

    #[tracing::instrument(skip(self))]
    pub(crate) async fn resolve_by_id(
        &self,
        id: &str,
        package_root: Option<&camino::Utf8PathBuf>,
        config_package_name: &str,
        ds: &fastn_ds::DocumentStore,
    ) -> fastn_core::Result<(String, Vec<u8>)> {
        tracing::info!(id = id);
        if let Ok(response) = self.fs_fetch_by_id(id, package_root, ds).await {
            return Ok(response);
        }

        if config_package_name.ne(&self.name) {
            if let Ok(response) = self.http_download_by_id(id, package_root).await {
                return Ok(response);
            }
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
                        "fs_fetch_by_id:: Corresponding file not found for id: {}. Package: {}",
                        id, &self.name
                    ),
                });
            }
        };

        let root = self.package_root_with_default(package_root)?;
        if let Ok(response) = self.fs_fetch_by_id(new_id.as_str(), package_root, ds).await {
            if config_package_name.ne(&self.name) {
                fastn_core::utils::write(&root, id.trim_start_matches('/'), response.1.as_slice())
                    .await?;
            }
            return Ok(response);
        }

        if config_package_name.eq(&self.name) {
            tracing::error!(id = id, msg = "id error: can not get the dark");
            return Err(fastn_core::Error::PackageError {
                message: format!(
                    "fs_fetch_by_id:: Corresponding file not found for id: {}. Package: {}",
                    id, &self.name
                ),
            });
        }

        match self
            .http_download_by_id(new_id.as_str(), package_root)
            .await
        {
            Ok(response) => {
                if config_package_name.ne(&self.name) {
                    fastn_core::utils::write(
                        &root,
                        id.trim_start_matches('/'),
                        response.1.as_slice(),
                    )
                    .await?;
                }
                Ok(response)
            }
            Err(e) => Err(e),
        }
    }
}

pub(crate) fn file_id_to_names(id: &str) -> Vec<String> {
    let id = id.replace("/index.html", "/").replace("index.html", "/");
    if id.eq("/") {
        return vec![
            "index.ftd".to_string(),
            "README.md".to_string(),
            "index.md".to_string(),
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
        // Todo: removing `md` file support for now
        // format!("{}.md", id),
        // format!("{}/README.md", id),
        // format!("{}/index.md", id),
    ]);
    ids
}

pub enum FTDResult {
    Html(Vec<u8>),
    Redirect { url: String, code: i32 },
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
        }
    }

    pub fn checksum(&self) -> String {
        fastn_core::utils::generate_hash(self.html())
    }
}

impl From<FTDResult> for fastn_core::http::Response {
    fn from(val: FTDResult) -> Self {
        match val {
            FTDResult::Html(body) => {
                fastn_core::http::ok_with_content_type(body, mime_guess::mime::TEXT_HTML_UTF_8)
            }
            FTDResult::Redirect { url, code } => fastn_core::http::redirect_with_code(url, code),
        }
    }
}

#[tracing::instrument(skip_all)]
pub(crate) async fn read_ftd(
    config: &mut fastn_core::RequestConfig,
    main: &fastn_core::Document,
    base_url: &str,
    download_assets: bool,
    test: bool,
) -> fastn_core::Result<FTDResult> {
    read_ftd_(config, main, base_url, download_assets, test, false).await
}

#[tracing::instrument(skip_all)]
pub(crate) async fn read_ftd_(
    config: &mut fastn_core::RequestConfig,
    main: &fastn_core::Document,
    base_url: &str,
    download_assets: bool,
    test: bool,
    only_js: bool,
) -> fastn_core::Result<FTDResult> {
    tracing::info!(document = main.id);
    match config.config.ftd_edition {
        fastn_core::FTDEdition::FTD2021 => {
            unimplemented!()
        }
        fastn_core::FTDEdition::FTD2022 => {
            read_ftd_2022(config, main, base_url, download_assets, test).await
        }
        fastn_core::FTDEdition::FTD2023 => {
            read_ftd_2023(config, main, base_url, download_assets, only_js).await
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
) -> fastn_core::Result<FTDResult> {
    let font_style = config.config.get_font_style();
    let c = &config.config.clone();

    let current_package = config
        .config
        .all_packages
        .borrow()
        .get(main.package_name.as_str())
        .unwrap_or(&config.config.package)
        .to_owned();

    config.document_id = main.id.clone();
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
) -> fastn_core::Result<FTDResult> {
    let package_name = config.config.package.name.to_string();
    let c = &config.config.clone();

    let current_package = config
        .config
        .all_packages
        .borrow()
        .get(main.package_name.as_str())
        .unwrap_or(&config.config.package)
        .to_owned();

    config.document_id = main.id.clone();
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
        let ssr_body = fastn_js::ssr_with_js_string(
            &package_name,
            format!("{js_ftd_script}\n{js_document_script}").as_str(),
        );

        fastn_core::utils::replace_markers_2023(
            js_document_script.as_str(),
            js_ast_data.scripts.join("").as_str(),
            ssr_body.as_str(),
            config.config.get_font_style().as_str(),
            ftd::ftd_js_css(),
            base_url,
            c,
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
) -> fastn_core::Result<FTDResult> {
    let build_dir = config.config.build_dir();
    let response = read_ftd(config, main, base_url, build_static_files, test).await?;
    fastn_core::utils::overwrite(&build_dir, file_path, &response.html()).await?;

    Ok(response)
}
