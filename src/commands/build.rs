pub async fn build(
    config: &mut fpm::Config,
    file: Option<&str>,
    base_url: &str,
    ignore_failed: bool,
) -> fpm::Result<()> {
    tokio::fs::create_dir_all(config.build_dir()).await?;
    let documents = get_documents_for_current_package(config).await?;

    // No need to build static files when file is passed during fpm build (no-static behaviour)
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
            fpm::File::Ftd(doc) => {
                let resp =
                    fpm::package::package_doc::process_ftd(config, doc, base_url, no_static).await;
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
            fpm::File::Static(sa) => process_static(sa, &config.root, &config.package).await?,
            fpm::File::Markdown(doc) => {
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
            fpm::File::Image(main_doc) => {
                process_static(main_doc, &config.root, &config.package).await?;
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
            fpm::File::Code(doc) => {
                process_static(
                    &fpm::Static {
                        id: doc.id.to_string(),
                        content: vec![],
                        base_path: camino::Utf8PathBuf::from(doc.parent_path.as_str()),
                    },
                    &config.root,
                    &config.package,
                )
                .await?;
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
        fpm::utils::print_end(
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
    config: &mut fpm::Config,
) -> fpm::Result<std::collections::BTreeMap<String, fpm::File>> {
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
        let mut files: std::collections::HashMap<String, fpm::File> = Default::default();
        for (doc_path, _, url) in get_all_locations {
            let file = {
                let package_name = if let Some(ref url) = url {
                    new_config.find_package_by_id(url).await?.1.name
                } else {
                    config.package.name.to_string()
                };
                let mut file = fpm::get_file(package_name, doc_path, config.root.as_path()).await?;
                if let Some(ref url) = url {
                    let url = url.replace("/index.html", "");
                    let extension = if matches!(file, fpm::File::Markdown(_)) {
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
    sa: &fpm::Static,
    base_path: &camino::Utf8Path,
    package: &fpm::Package,
) -> fpm::Result<()> {
    copy_to_build(sa, base_path, package)?;
    if let Some(original_package) = package.translation_of.as_ref() {
        copy_to_build(sa, base_path, original_package)?;
    }
    return Ok(());

    fn copy_to_build(
        sa: &fpm::Static,
        base_path: &camino::Utf8Path,
        package: &fpm::Package,
    ) -> fpm::Result<()> {
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
    config: &mut fpm::Config,
    main: &fpm::Static,
    base_url: &str,
    no_static: bool,
) -> fpm::Result<()> {
    let main = convert_to_ftd(config, main)?;

    fpm::package::package_doc::process_ftd(config, &main, base_url, no_static).await?;
    return Ok(());

    fn convert_to_ftd(config: &fpm::Config, doc: &fpm::Static) -> fpm::Result<fpm::Document> {
        Ok(fpm::Document {
            package_name: config.package.name.to_string(),
            id: convert_to_ftd_extension(doc.id.as_str())?,
            content: fpm::package_info_image(config, doc, &config.package)?,
            parent_path: doc.base_path.to_string(),
        })
    }

    fn convert_to_ftd_extension(name: &str) -> fpm::Result<String> {
        Ok(format!("{}.ftd", name))
    }
}

async fn process_code(
    config: &mut fpm::Config,
    main: &fpm::Document,
    base_url: &str,
    no_static: bool,
) -> fpm::Result<()> {
    let main = if let Some(main) = convert_to_ftd(config, main)? {
        main
    } else {
        return Ok(());
    };

    fpm::package::package_doc::process_ftd(config, &main, base_url, no_static).await?;
    return Ok(());

    fn convert_to_ftd(
        config: &fpm::Config,
        doc: &fpm::Document,
    ) -> fpm::Result<Option<fpm::Document>> {
        let id = convert_to_ftd_extension(doc.id.as_str())?;
        let ext = fpm::utils::get_extension(doc.id.as_str())?;
        let new_content =
            fpm::package_info_code(config, id.as_str(), doc.content.as_str(), ext.as_str())?;

        let new_doc = {
            let mut new_doc = doc.to_owned();
            new_doc.content = new_content;
            new_doc.id = id;
            new_doc
        };

        Ok(Some(new_doc))
    }

    fn convert_to_ftd_extension(name: &str) -> fpm::Result<String> {
        Ok(format!("{}.ftd", name))
    }
}

async fn process_markdown(
    config: &mut fpm::Config,
    main: &fpm::Document,
    base_url: &str,
    no_static: bool,
) -> fpm::Result<()> {
    let main = if let Some(main) = convert_md_to_ftd(config, main)? {
        main
    } else {
        return Ok(());
    };
    fpm::package::package_doc::process_ftd(config, &main, base_url, no_static).await?;
    return Ok(());

    fn convert_md_to_ftd(
        config: &fpm::Config,
        doc: &fpm::Document,
    ) -> fpm::Result<Option<fpm::Document>> {
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
        let new_content = fpm::package_info_markdown(config, id.as_str(), doc.content.as_str())?;

        let new_doc = {
            let mut new_doc = doc.to_owned();
            new_doc.content = new_content;
            new_doc.id = id;
            new_doc
        };

        Ok(Some(new_doc))
    }

    fn convert_md_to_ftd_extension(name: &str) -> fpm::Result<String> {
        let file_name = if let Some(p1) = name.strip_suffix(".md") {
            p1
        } else {
            return Err(fpm::Error::UsageError {
                message: format!("expected md file found: `{}`", name),
            });
        };

        Ok(format!("{}.ftd", file_name))
    }
}
