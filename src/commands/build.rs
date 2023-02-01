pub async fn build(
    config: &mut fastn::Config,
    file: Option<&str>,
    base_url: &str,
    ignore_failed: bool,
) -> fastn::Result<()> {
    fastn::utils::enable_parse_caching(true);

    tokio::fs::create_dir_all(config.build_dir()).await?;
    let documents = get_documents_for_current_package(config).await?;

    // No need to build static files when file is passed during fastn build (no-static behaviour)
    let no_static: bool = file.is_some();

    for main in documents.values() {
        if file.is_some() && file != Some(main.get_id().as_str()) {
            continue;
        }
        config.current_document = Some(main.get_id());
        let start = std::time::Instant::now();

        print!(
            "Processing {}/{} ... ",
            config.package.name.as_str(),
            main.get_id()
        );

        match main {
            fastn::File::Ftd(doc) => {
                if !config.ftd_edition.eq(&fastn::config::FTDEdition::FTD2021)
                    && doc.id.eq("FASTN.ftd")
                {
                    tokio::fs::copy(
                        config.root.join(doc.id.as_str()),
                        config.root.join(".build").join(doc.id.as_str()),
                    )
                    .await?;
                    continue;
                }
                let resp =
                    fastn::package::package_doc::process_ftd(config, doc, base_url, no_static)
                        .await;
                match (resp, ignore_failed) {
                    (Ok(_), _) => (),
                    (_, true) => {
                        println!("Failed");
                        continue;
                    }
                    (Err(e), _) => {
                        return Err(e);
                    }
                }
            }
            fastn::File::Static(sa) => process_static(sa, &config.root, &config.package).await?,
            fastn::File::Markdown(doc) => {
                if !config.ftd_edition.eq(&fastn::config::FTDEdition::FTD2021) {
                    continue;
                }
                let resp = process_markdown(config, doc, base_url, no_static).await;
                match (resp, ignore_failed) {
                    (Ok(r), _) => r,
                    (_, true) => {
                        println!("Failed");
                        continue;
                    }
                    (e, _) => {
                        return e;
                    }
                }
            }
            fastn::File::Image(main_doc) => {
                process_static(main_doc, &config.root, &config.package).await?;
                if config.ftd_edition.eq(&fastn::config::FTDEdition::FTD2021) {
                    let resp = process_image(config, main_doc, base_url, no_static).await;
                    match (resp, ignore_failed) {
                        (Ok(r), _) => r,
                        (_, true) => {
                            println!("Failed");
                            continue;
                        }
                        (e, _) => {
                            return e;
                        }
                    }
                }
            }
            fastn::File::Code(doc) => {
                process_static(
                    &fastn::Static {
                        id: doc.id.to_string(),
                        content: vec![],
                        base_path: camino::Utf8PathBuf::from(doc.parent_path.as_str()),
                    },
                    &config.root,
                    &config.package,
                )
                .await?;
                if config.ftd_edition.eq(&fastn::config::FTDEdition::FTD2021) {
                    let resp = process_code(config, doc, base_url, no_static).await;
                    match (resp, ignore_failed) {
                        (Ok(r), _) => r,
                        (_, true) => {
                            println!("Failed");
                            continue;
                        }
                        (e, _) => {
                            return e;
                        }
                    }
                }
            }
        }
        fastn::utils::print_end(
            format!(
                "Processed {}/{}",
                config.package.name.as_str(),
                main.get_id()
            )
            .as_str(),
            start,
        );
    }

    if !no_static {
        config.download_fonts().await?;
    }
    Ok(())
}

async fn get_documents_for_current_package(
    config: &mut fastn::Config,
) -> fastn::Result<std::collections::BTreeMap<String, fastn::File>> {
    let mut documents = std::collections::BTreeMap::from_iter(
        config
            .get_files(&config.package)
            .await?
            .into_iter()
            .map(|v| (v.get_id(), v)),
    );

    if let Some(ref sitemap) = config.package.sitemap {
        let new_config = config.clone();
        let get_all_locations = sitemap.get_all_locations();
        let mut files: std::collections::HashMap<String, fastn::File> = Default::default();
        for (doc_path, _, url) in get_all_locations {
            let file = {
                let package_name = if let Some(ref url) = url {
                    new_config.find_package_by_id(url).await?.1.name
                } else {
                    config.package.name.to_string()
                };
                let mut file =
                    fastn::get_file(package_name, doc_path, config.root.as_path()).await?;
                if let Some(ref url) = url {
                    let url = url.replace("/index.html", "");
                    let extension = if matches!(file, fastn::File::Markdown(_)) {
                        "index.md".to_string()
                    } else {
                        "index.ftd".to_string()
                    };

                    file.set_id(format!("{}/{}", url.trim_matches('/'), extension).as_str());
                }
                file
            };
            files.insert(file.get_id(), file);
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
    sa: &fastn::Static,
    base_path: &camino::Utf8Path,
    package: &fastn::Package,
) -> fastn::Result<()> {
    copy_to_build(sa, base_path, package)?;
    if let Some(original_package) = package.translation_of.as_ref() {
        copy_to_build(sa, base_path, original_package)?;
    }
    return Ok(());

    fn copy_to_build(
        sa: &fastn::Static,
        base_path: &camino::Utf8Path,
        package: &fastn::Package,
    ) -> fastn::Result<()> {
        let build_path = base_path
            .join(".build")
            .join("-")
            .join(package.name.as_str());

        std::fs::create_dir_all(&build_path)?;
        if let Some((dir, _)) = sa.id.rsplit_once(std::path::MAIN_SEPARATOR) {
            std::fs::create_dir_all(&build_path.join(dir))?;
        }
        std::fs::copy(
            sa.base_path.join(sa.id.as_str()),
            build_path.join(sa.id.as_str()),
        )?;

        Ok(())
    }
}

async fn process_image(
    config: &mut fastn::Config,
    main: &fastn::Static,
    base_url: &str,
    no_static: bool,
) -> fastn::Result<()> {
    let main = convert_to_ftd(config, main)?;

    fastn::package::package_doc::process_ftd(config, &main, base_url, no_static).await?;
    return Ok(());

    fn convert_to_ftd(
        config: &fastn::Config,
        doc: &fastn::Static,
    ) -> fastn::Result<fastn::Document> {
        Ok(fastn::Document {
            package_name: config.package.name.to_string(),
            id: convert_to_ftd_extension(doc.id.as_str())?,
            content: fastn::package_info_image(config, doc, &config.package)?,
            parent_path: doc.base_path.to_string(),
        })
    }

    fn convert_to_ftd_extension(name: &str) -> fastn::Result<String> {
        Ok(format!("{}.ftd", name))
    }
}

async fn process_code(
    config: &mut fastn::Config,
    main: &fastn::Document,
    base_url: &str,
    no_static: bool,
) -> fastn::Result<()> {
    let main = if let Some(main) = convert_to_ftd(config, main)? {
        main
    } else {
        return Ok(());
    };

    fastn::package::package_doc::process_ftd(config, &main, base_url, no_static).await?;
    return Ok(());

    fn convert_to_ftd(
        config: &fastn::Config,
        doc: &fastn::Document,
    ) -> fastn::Result<Option<fastn::Document>> {
        let id = convert_to_ftd_extension(doc.id.as_str())?;
        let ext = fastn::utils::get_extension(doc.id.as_str())?;
        let new_content =
            fastn::package_info_code(config, id.as_str(), doc.content.as_str(), ext.as_str())?;

        let new_doc = {
            let mut new_doc = doc.to_owned();
            new_doc.content = new_content;
            new_doc.id = id;
            new_doc
        };

        Ok(Some(new_doc))
    }

    fn convert_to_ftd_extension(name: &str) -> fastn::Result<String> {
        Ok(format!("{}.ftd", name))
    }
}

async fn process_markdown(
    config: &mut fastn::Config,
    main: &fastn::Document,
    base_url: &str,
    no_static: bool,
) -> fastn::Result<()> {
    let main = if let Some(main) = convert_md_to_ftd(config, main)? {
        main
    } else {
        return Ok(());
    };
    fastn::package::package_doc::process_ftd(config, &main, base_url, no_static).await?;
    return Ok(());

    fn convert_md_to_ftd(
        config: &fastn::Config,
        doc: &fastn::Document,
    ) -> fastn::Result<Option<fastn::Document>> {
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
        let new_content = fastn::package_info_markdown(config, id.as_str(), doc.content.as_str())?;

        let new_doc = {
            let mut new_doc = doc.to_owned();
            new_doc.content = new_content;
            new_doc.id = id;
            new_doc
        };

        Ok(Some(new_doc))
    }

    fn convert_md_to_ftd_extension(name: &str) -> fastn::Result<String> {
        let file_name = if let Some(p1) = name.strip_suffix(".md") {
            p1
        } else {
            return Err(fastn::Error::UsageError {
                message: format!("expected md file found: `{}`", name),
            });
        };

        Ok(format!("{}.ftd", file_name))
    }
}
