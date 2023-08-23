// #[tracing::instrument(skip(config))]
pub async fn build(
    config: &mut fastn_core::Config,
    only_id: Option<&str>,
    base_url: &str,
    ignore_failed: bool,
    test: bool,
    check_build: bool,
) -> fastn_core::Result<()> {
    tokio::fs::create_dir_all(config.build_dir()).await?;
    let documents = get_documents_for_current_package(config).await?;

    // Default css and js
    default_build_files(config.root.join(".build"), &config.ftd_edition).await?;

    if let Some(id) = only_id {
        handle_only_id(id, config, base_url, ignore_failed, test, documents).await?;
        return Ok(());
    }

    // All redirect html files under .build
    if let Some(ref r) = config.package.redirects {
        for (redirect_from, redirect_to) in r.iter() {
            println!(
                "Processing redirect {}/{} -> {}... ",
                config.package.name.as_str(),
                redirect_from.trim_matches('/'),
                redirect_to
            );

            let content = fastn_core::utils::redirect_page_html(redirect_to.as_str());
            let save_file = if redirect_from.as_str().ends_with(".ftd") {
                redirect_from
                    .replace("index.ftd", "index.html")
                    .replace(".ftd", "/index.html")
            } else {
                format!("{}/index.html", redirect_from.trim_matches('/'))
            };

            let save_path = config.root.join(".build").join(save_file.as_str());
            fastn_core::utils::update(save_path, content.as_bytes())
                .await
                .ok();
        }
    }

    for document in documents.values() {
        handle_file(document, config, base_url, ignore_failed, test, false).await?;
    }

    config.download_fonts().await?;

    if check_build {
        return fastn_core::post_build_check(config).await;
    }
    Ok(())
}

#[tracing::instrument(skip(config, documents))]
async fn handle_only_id(
    id: &str,
    config: &mut fastn_core::Config,
    base_url: &str,
    ignore_failed: bool,
    test: bool,
    documents: std::collections::BTreeMap<String, fastn_core::File>,
) -> fastn_core::Result<()> {
    for doc in documents.values() {
        if doc.get_id().eq(id) || doc.get_id_with_package().eq(id) {
            return handle_file(doc, config, base_url, ignore_failed, test, true).await;
        }
    }

    Err(fastn_core::Error::GenericError(format!(
        "Document {} not found in package {}",
        id,
        config.package.name.as_str()
    )))
}

async fn handle_file(
    document: &fastn_core::File,
    config: &mut fastn_core::Config,
    base_url: &str,
    ignore_failed: bool,
    test: bool,
    no_static: bool,
) -> fastn_core::Result<()> {
    let start = std::time::Instant::now();
    print!("Processing {} ... ", document.get_id_with_package());

    let process_status =
        handle_file_(document, config, base_url, ignore_failed, test, no_static).await;

    if process_status.is_ok() {
        fastn_core::utils::print_end(
            format!(
                "Processed {}/{}",
                config.package.name.as_str(),
                document.get_id()
            )
            .as_str(),
            start,
        );
    }
    if process_status.is_err() {
        fastn_core::utils::print_error(
            format!(
                "Failed {}/{}",
                config.package.name.as_str(),
                document.get_id()
            )
            .as_str(),
            start,
        );
        return process_status;
    }
    Ok(())
}

#[tracing::instrument(skip(document, config))]
async fn handle_file_(
    document: &fastn_core::File,
    config: &mut fastn_core::Config,
    base_url: &str,
    ignore_failed: bool,
    test: bool,
    no_static: bool,
) -> fastn_core::Result<()> {
    config.current_document = Some(document.get_id().to_string());

    match document {
        fastn_core::File::Ftd(doc) => {
            if !config
                .ftd_edition
                .eq(&fastn_core::config::FTDEdition::FTD2021)
            {
                // Ignore redirect paths
                if let Some(r) = config.package.redirects.as_ref() {
                    if fastn_core::package::redirects::find_redirect(r, doc.id.as_str()).is_some() {
                        println!("Ignored by redirect {}", doc.id.as_str());
                        return Ok(());
                    }
                }

                fastn_core::utils::copy(
                    config.root.join(doc.id.as_str()),
                    config.root.join(".build").join(doc.id.as_str()),
                )
                .await
                .ok();

                if doc.id.eq("FASTN.ftd") {
                    return Ok(());
                }
            }
            let resp = fastn_core::package::package_doc::process_ftd(
                config, doc, base_url, no_static, test,
            )
            .await;
            match (resp, ignore_failed) {
                (Ok(_), _) => (),
                (_, true) => {
                    print!("Failed ");
                    return Ok(());
                }
                (Err(e), _) => {
                    return Err(e);
                }
            }
        }
        fastn_core::File::Static(sa) => process_static(sa, &config.root, &config.package).await?,
        fastn_core::File::Markdown(doc) => {
            if !config
                .ftd_edition
                .eq(&fastn_core::config::FTDEdition::FTD2021)
            {
                // TODO: bring this feature back
                print!("Skipped ");
                return Ok(());
            }
            let resp = process_markdown(config, doc, base_url, no_static, test).await;
            match (resp, ignore_failed) {
                (Ok(r), _) => r,
                (_, true) => {
                    print!("Failed ");
                    return Ok(());
                }
                (e, _) => {
                    return e;
                }
            }
        }
        fastn_core::File::Image(main_doc) => {
            process_static(main_doc, &config.root, &config.package).await?;
            if config
                .ftd_edition
                .eq(&fastn_core::config::FTDEdition::FTD2021)
            {
                let resp = process_image(config, main_doc, base_url, no_static, test).await;
                match (resp, ignore_failed) {
                    (Ok(r), _) => r,
                    (_, true) => {
                        print!("Failed ");
                        return Ok(());
                    }
                    (e, _) => {
                        return e;
                    }
                }
            }
        }
        fastn_core::File::Code(doc) => {
            process_static(
                &fastn_core::Static {
                    package_name: config.package.name.to_string(),
                    id: doc.id.to_string(),
                    content: vec![],
                    base_path: camino::Utf8PathBuf::from(doc.parent_path.as_str()),
                },
                &config.root,
                &config.package,
            )
            .await?;
            if config
                .ftd_edition
                .eq(&fastn_core::config::FTDEdition::FTD2021)
            {
                let resp = process_code(config, doc, base_url, no_static, test).await;
                match (resp, ignore_failed) {
                    (Ok(r), _) => r,
                    (_, true) => {
                        print!("Failed ");
                        return Ok(());
                    }
                    (e, _) => {
                        return e;
                    }
                }
            }
        }
    }

    Ok(())
}

#[tracing::instrument]
pub async fn default_build_files(
    base_path: camino::Utf8PathBuf,
    ftd_edition: &fastn_core::FTDEdition,
) -> fastn_core::Result<()> {
    if ftd_edition.is_2023() {
        let default_ftd_js_content = ftd::js::all_js_without_test();
        let hashed_ftd_js_name = fastn_core::utils::hashed_default_ftd_js();
        let save_default_ftd_js = base_path.join(hashed_ftd_js_name);
        fastn_core::utils::update(save_default_ftd_js, default_ftd_js_content.as_bytes())
            .await
            .ok();

        let markdown_js_content = ftd::markdown_js();
        let hashed_markdown_js_name = fastn_core::utils::hashed_markdown_js();
        let save_markdown_js = base_path.join(hashed_markdown_js_name);
        fastn_core::utils::update(save_markdown_js, markdown_js_content.as_bytes())
            .await
            .ok();

        let theme_css = ftd::theme_css();
        let hashed_code_themes = fastn_core::utils::hashed_code_theme_css();
        for (theme, file_name) in hashed_code_themes {
            let save_markdown_js = base_path.join(file_name);
            let theme_content = theme_css.get(theme).unwrap();
            fastn_core::utils::update(save_markdown_js, theme_content.as_bytes())
                .await
                .ok();
        }

        let prism_js_content = ftd::prism_js();
        let hashed_prism_js_name = fastn_core::utils::hashed_prism_js();
        let save_prism_js = base_path.join(hashed_prism_js_name);
        fastn_core::utils::update(save_prism_js, prism_js_content.as_bytes())
            .await
            .ok();

        let prism_css_content = ftd::prism_css();
        let hashed_prism_css_name = fastn_core::utils::hashed_prism_css();
        let save_prism_css = base_path.join(hashed_prism_css_name);
        fastn_core::utils::update(save_prism_css, prism_css_content.as_bytes())
            .await
            .ok();
    } else {
        let default_css_content = ftd::css();
        let hashed_css_name = fastn_core::utils::hashed_default_css_name();
        let save_default_css = base_path.join(hashed_css_name);
        fastn_core::utils::update(save_default_css, default_css_content.as_bytes())
            .await
            .ok();

        let default_js_content = format!("{}\n\n{}", ftd::build_js(), fastn_core::fastn_2022_js());
        let hashed_js_name = fastn_core::utils::hashed_default_js_name();
        let save_default_js = base_path.join(hashed_js_name);
        fastn_core::utils::update(save_default_js, default_js_content.as_bytes())
            .await
            .ok();
    }

    Ok(())
}

#[tracing::instrument(skip(config))]
async fn get_documents_for_current_package(
    config: &mut fastn_core::Config,
) -> fastn_core::Result<std::collections::BTreeMap<String, fastn_core::File>> {
    let mut documents = std::collections::BTreeMap::from_iter(
        config
            .get_files(&config.package)
            .await?
            .into_iter()
            .map(|v| (v.get_id().to_string(), v)),
    );

    if let Some(ref sitemap) = config.package.sitemap {
        let new_config = config.clone();
        let get_all_locations = sitemap.get_all_locations();
        let mut files: std::collections::HashMap<String, fastn_core::File> = Default::default();
        for (doc_path, _, url) in get_all_locations {
            let file = {
                let package_name = if let Some(ref url) = url {
                    new_config.find_package_by_id(url).await?.1.name
                } else {
                    config.package.name.to_string()
                };
                let mut file =
                    fastn_core::get_file(package_name, doc_path, config.root.as_path()).await?;
                if let Some(ref url) = url {
                    let url = url.replace("/index.html", "");
                    let extension = if matches!(file, fastn_core::File::Markdown(_)) {
                        "index.md".to_string()
                    } else {
                        "index.ftd".to_string()
                    };

                    file.set_id(format!("{}/{}", url.trim_matches('/'), extension).as_str());
                }
                file
            };
            files.insert(file.get_id().to_string(), file);
        }

        config
            .all_packages
            .borrow_mut()
            .extend(new_config.all_packages.into_inner());
        documents.extend(files);
    }

    Ok(documents)
}

async fn process_static(
    sa: &fastn_core::Static,
    base_path: &camino::Utf8Path,
    package: &fastn_core::Package,
) -> fastn_core::Result<()> {
    copy_to_build(sa, base_path, package)?;
    if let Some(original_package) = package.translation_of.as_ref() {
        copy_to_build(sa, base_path, original_package)?;
    }
    return Ok(());

    fn copy_to_build(
        sa: &fastn_core::Static,
        base_path: &camino::Utf8Path,
        package: &fastn_core::Package,
    ) -> fastn_core::Result<()> {
        let build_path = base_path
            .join(".build")
            .join("-")
            .join(package.name.as_str());

        std::fs::create_dir_all(&build_path)?;
        if let Some((dir, _)) = sa.id.rsplit_once('/') {
            std::fs::create_dir_all(build_path.join(dir))?;
        }
        std::fs::copy(
            sa.base_path.join(sa.id.as_str()),
            build_path.join(sa.id.as_str()),
        )?;

        {
            // TODO: need to remove this once download_base_url is removed
            std::fs::create_dir_all(base_path.join(".build"))?;
            if let Some((dir, _)) = sa.id.rsplit_once('/') {
                std::fs::create_dir_all(base_path.join(".build").join(dir))?;
            }

            std::fs::copy(
                sa.base_path.join(sa.id.as_str()),
                base_path.join(".build").join(sa.id.as_str()),
            )?;
        }
        Ok(())
    }
}

async fn process_image(
    config: &mut fastn_core::Config,
    main: &fastn_core::Static,
    base_url: &str,
    no_static: bool,
    test: bool,
) -> fastn_core::Result<()> {
    let main = convert_to_ftd(config, main)?;

    fastn_core::package::package_doc::process_ftd(config, &main, base_url, no_static, test).await?;
    return Ok(());

    fn convert_to_ftd(
        config: &fastn_core::Config,
        doc: &fastn_core::Static,
    ) -> fastn_core::Result<fastn_core::Document> {
        Ok(fastn_core::Document {
            package_name: config.package.name.to_string(),
            id: convert_to_ftd_extension(doc.id.as_str())?,
            content: fastn_core::package_info_image(config, doc, &config.package)?,
            parent_path: doc.base_path.to_string(),
        })
    }

    fn convert_to_ftd_extension(name: &str) -> fastn_core::Result<String> {
        Ok(format!("{}.ftd", name))
    }
}

async fn process_code(
    config: &mut fastn_core::Config,
    main: &fastn_core::Document,
    base_url: &str,
    no_static: bool,
    test: bool,
) -> fastn_core::Result<()> {
    let main = if let Some(main) = convert_to_ftd(config, main)? {
        main
    } else {
        return Ok(());
    };

    fastn_core::package::package_doc::process_ftd(config, &main, base_url, no_static, test).await?;
    return Ok(());

    fn convert_to_ftd(
        config: &fastn_core::Config,
        doc: &fastn_core::Document,
    ) -> fastn_core::Result<Option<fastn_core::Document>> {
        let id = convert_to_ftd_extension(doc.id.as_str())?;
        let ext = fastn_core::utils::get_extension(doc.id.as_str())?;
        let new_content =
            fastn_core::package_info_code(config, id.as_str(), doc.content.as_str(), ext.as_str())?;

        let new_doc = {
            let mut new_doc = doc.to_owned();
            new_doc.content = new_content;
            new_doc.id = id;
            new_doc
        };

        Ok(Some(new_doc))
    }

    fn convert_to_ftd_extension(name: &str) -> fastn_core::Result<String> {
        Ok(format!("{}.ftd", name))
    }
}

async fn process_markdown(
    config: &mut fastn_core::Config,
    main: &fastn_core::Document,
    base_url: &str,
    no_static: bool,
    test: bool,
) -> fastn_core::Result<()> {
    let main = if let Some(main) = convert_md_to_ftd(config, main)? {
        main
    } else {
        return Ok(());
    };
    fastn_core::package::package_doc::process_ftd(config, &main, base_url, no_static, test).await?;
    return Ok(());

    fn convert_md_to_ftd(
        config: &fastn_core::Config,
        doc: &fastn_core::Document,
    ) -> fastn_core::Result<Option<fastn_core::Document>> {
        let doc_id = if doc.id == "README.md"
            && !(camino::Utf8Path::new(format!(".{}index.ftd", std::path::MAIN_SEPARATOR).as_str())
                .exists()
                || camino::Utf8Path::new(
                    format!(".{}index.md", std::path::MAIN_SEPARATOR).as_str(),
                )
                .exists())
        {
            "index.md".to_string()
        } else if !camino::Utf8Path::new(
            format!(
                ".{}{}",
                std::path::MAIN_SEPARATOR,
                convert_md_to_ftd_extension(doc.id.as_str())?
            )
            .as_str(),
        )
        .exists()
        {
            doc.id.to_string()
        } else {
            return Ok(None);
        };
        let id = convert_md_to_ftd_extension(doc_id.as_str())?;
        let new_content =
            fastn_core::package_info_markdown(config, id.as_str(), doc.content.as_str())?;

        let new_doc = {
            let mut new_doc = doc.to_owned();
            new_doc.content = new_content;
            new_doc.id = id;
            new_doc
        };

        Ok(Some(new_doc))
    }

    fn convert_md_to_ftd_extension(name: &str) -> fastn_core::Result<String> {
        let file_name = if let Some(p1) = name.strip_suffix(".md") {
            p1
        } else {
            return Err(fastn_core::Error::UsageError {
                message: format!("expected md file found: `{}`", name),
            });
        };

        Ok(format!("{}.ftd", file_name))
    }
}
