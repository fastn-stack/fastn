use std::iter::FromIterator;

pub async fn build(
    config: &fpm::Config,
    file: Option<&str>,
    base_url: &str,
    ignore_failed: bool,
) -> fpm::Result<()> {
    use fpm::utils::HasElements;
    use itertools::Itertools;

    tokio::fs::create_dir_all(config.build_dir()).await?;
    // let skip_failed = ignore_failed.unwrap_or(false);
    // Process static assets for the dependencies
    let dependencies = if let Some(package) = config.package.translation_of.as_ref() {
        let mut deps = package
            .get_flattened_dependencies()
            .into_iter()
            .unique_by(|dep| dep.package.name.clone())
            .collect_vec();
        deps.extend(
            config
                .package
                .get_flattened_dependencies()
                .into_iter()
                .unique_by(|dep| dep.package.name.clone()),
        );
        deps
    } else {
        config
            .package
            .get_flattened_dependencies()
            .into_iter()
            .unique_by(|dep| dep.package.name.clone())
            .collect_vec()
    };
    let mut asset_documents = std::collections::HashMap::new();
    asset_documents.insert(
        config.package.name.clone(),
        config.package.get_assets_doc(config, base_url).await?,
    );
    for dep in &dependencies {
        asset_documents.insert(
            dep.package.name.clone(),
            dep.package.get_assets_doc(config, base_url).await?,
        );
    }

    match (
        config.package.translation_of.as_ref(),
        config.package.translations.has_elements(),
    ) {
        (Some(_), true) => {
            // No package can be both a translation of something and has its own
            // translations, when building `config` we ensured this was rejected
            unreachable!()
        }
        (Some(original), false) => {
            build_with_original(
                config,
                original,
                file,
                base_url,
                ignore_failed,
                &asset_documents,
            )
            .await
        }
        (None, false) => {
            build_simple(config, file, base_url, ignore_failed, &asset_documents).await
        }
        (None, true) => {
            build_with_translations(config, file, base_url, ignore_failed, &asset_documents).await
        }
    }?;

    for dep in dependencies {
        let static_files = std::collections::BTreeMap::from_iter(
            fpm::get_documents(config, &dep.package)
                .await?
                .into_iter()
                .filter(|file_instance| {
                    matches!(file_instance, fpm::File::Static(_))
                        || matches!(file_instance, fpm::File::Code(_))
                        || matches!(file_instance, fpm::File::Image(_))
                })
                .collect::<Vec<fpm::File>>()
                .into_iter()
                .map(|v| (v.get_id(), v)),
        );
        process_files(
            config,
            &dep.package,
            &static_files,
            file,
            base_url,
            ignore_failed,
            &asset_documents,
        )
        .await?;
    }
    Ok(())
}

async fn build_simple(
    config: &fpm::Config,
    file: Option<&str>,
    base_url: &str,
    skip_failed: bool,
    asset_documents: &std::collections::HashMap<String, String>,
) -> fpm::Result<()> {
    let documents = std::collections::BTreeMap::from_iter(
        fpm::get_documents(config, &config.package)
            .await?
            .into_iter()
            .map(|v| (v.get_id(), v)),
    );
    process_files(
        config,
        &config.package,
        &documents,
        file,
        base_url,
        skip_failed,
        asset_documents,
    )
    .await
}

async fn build_with_translations(
    config: &fpm::Config,
    process_file: Option<&str>,
    base_url: &str,
    skip_failed: bool,
    asset_documents: &std::collections::HashMap<String, String>,
) -> fpm::Result<()> {
    use std::io::Write;

    let documents = std::collections::BTreeMap::from_iter(
        fpm::get_documents(config, &config.package)
            .await?
            .into_iter()
            .map(|v| (v.get_id(), v)),
    );

    let message = fpm::available_languages(config)?;

    for file in documents.values() {
        if process_file.is_some() && process_file != Some(file.get_id().as_str()) {
            continue;
        }
        fpm::process_file(
            config,
            &config.package,
            file,
            None,
            Some(message.as_str()),
            Default::default(),
            base_url,
            skip_failed,
            asset_documents,
        )
        .await?;
    }
    // Add /-/translation-status page
    {
        let translation_status = fpm::Document {
            id: "-/translation-status.ftd".to_string(),
            content: fpm::original_package_status(config)?,
            parent_path: config.root.as_str().to_string(),
            package_name: config.package.name.clone(),
        };

        print!("Processing translation-status.ftd ... ");
        let start = std::time::Instant::now();
        std::io::stdout().flush()?;

        process_ftd(
            config,
            &translation_status,
            None,
            None,
            Default::default(),
            base_url,
            asset_documents,
        )
        .await?;
        fpm::utils::print_end("Processed translation-status.ftd", start);
    }
    Ok(())
}

/// This is the translation package
/// First fetch all files from the original package
/// Then overwrite it with file in root package having same name/id as in original package
/// ignore all those documents/files which is not in original package
async fn build_with_original(
    config: &fpm::Config,
    _original: &fpm::Package,
    file: Option<&str>,
    base_url: &str,
    skip_failed: bool,
    asset_documents: &std::collections::HashMap<String, String>,
) -> fpm::Result<()> {
    use std::io::Write;

    // This is the translation package
    // Fetch all files from the original package
    let original_path = config.original_path()?;
    let original_snapshots = fpm::snapshot::get_latest_snapshots(&original_path).await?;
    let files = original_snapshots
        .keys()
        .into_iter()
        .map(|v| original_path.join(v))
        .collect::<Vec<camino::Utf8PathBuf>>();
    // documents contains all the files in original package
    let original_documents = std::collections::BTreeMap::from_iter(
        fpm::paths_to_files(config.package.name.as_str(), files, original_path.as_path())
            .await?
            .into_iter()
            .map(|v| (v.get_id(), v)),
    );

    // Fetch all files from the current package
    // ignore all those documents/files which is not in original package
    // Overwrite the files having same name/id

    let translated_documents = std::collections::BTreeMap::from_iter(
        fpm::get_documents(config, &config.package)
            .await?
            .into_iter()
            .filter(|x| original_snapshots.contains_key(x.get_id().as_str()))
            .map(|v| (v.get_id(), v)),
    );

    let translated_documents = fpm::TranslatedDocument::get_translated_document(
        config,
        original_documents,
        translated_documents,
    )
    .await?;

    // Process all the files collected from original and root package
    for translated_document in translated_documents.values() {
        if file.is_some()
            && file
                != Some(
                    match &translated_document {
                        fpm::TranslatedDocument::Missing { original } => original.get_id(),
                        fpm::TranslatedDocument::NeverMarked { translated, .. } => {
                            translated.get_id()
                        }
                        fpm::TranslatedDocument::Outdated { translated, .. } => translated.get_id(),
                        fpm::TranslatedDocument::UptoDate { translated, .. } => translated.get_id(),
                    }
                    .as_str(),
                )
        {
            continue;
        }
        translated_document
            .html(config, base_url, skip_failed, asset_documents)
            .await?;
    }

    // Add /-/translation-status page
    {
        let translation_status = fpm::Document {
            id: "-/translation-status.ftd".to_string(),
            content: fpm::translation_package_status(config)?,
            parent_path: config.root.as_str().to_string(),
            package_name: config.package.name.clone(),
        };
        print!("Processing translation-status.ftd ... ");
        let start = std::time::Instant::now();
        std::io::stdout().flush()?;
        process_ftd(
            config,
            &translation_status,
            None,
            None,
            Default::default(),
            base_url,
            asset_documents,
        )
        .await?;
        fpm::utils::print_end("Processed translation-status.ftd", start);
    }
    Ok(())
}

async fn process_files(
    config: &fpm::Config,
    package: &fpm::Package,
    documents: &std::collections::BTreeMap<String, fpm::File>,
    file: Option<&str>,
    base_url: &str,
    skip_failed: bool,
    asset_documents: &std::collections::HashMap<String, String>,
) -> fpm::Result<()> {
    for f in documents.values() {
        if file.is_some() && file != Some(f.get_id().as_str()) {
            continue;
        }
        process_file(
            config,
            package,
            f,
            None,
            None,
            Default::default(),
            base_url,
            skip_failed,
            asset_documents,
        )
        .await?
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn process_file(
    config: &fpm::Config,
    package: &fpm::Package,
    main: &fpm::File,
    fallback: Option<&fpm::File>,
    message: Option<&str>,
    translated_data: fpm::TranslationData,
    base_url: &str,
    skip_failed: bool,
    asset_documents: &std::collections::HashMap<String, String>,
) -> fpm::Result<()> {
    use std::io::Write;

    let start = std::time::Instant::now();
    if let Some(fallback) = fallback {
        print!(
            "Processing {}/{} ... ",
            package.name.as_str(),
            main.get_id()
        );
        std::io::stdout().flush()?;
        match (main, fallback) {
            (fpm::File::Ftd(main_doc), fpm::File::Ftd(fallback_doc)) => {
                let resp = process_ftd(
                    config,
                    main_doc,
                    Some(fallback_doc),
                    message,
                    translated_data,
                    base_url,
                    asset_documents,
                )
                .await;
                match (resp, skip_failed) {
                    (Ok(r), _) => r,
                    (_, true) => {
                        println!("Failed");
                        return Ok(());
                    }
                    (e, _) => {
                        return e;
                    }
                }
            }
            (fpm::File::Static(main_sa), fpm::File::Static(_)) => {
                process_static(main_sa, &config.root, package).await?
            }
            (fpm::File::Code(main_doc), fpm::File::Code(fallback_doc)) => {
                process_static(
                    &fpm::Static {
                        id: main_doc.id.to_string(),
                        base_path: camino::Utf8PathBuf::from(main_doc.parent_path.as_str()),
                    },
                    &config.root,
                    package,
                )
                .await?;
                let resp = process_code(
                    config,
                    main_doc,
                    Some(fallback_doc),
                    message,
                    translated_data,
                    base_url,
                    asset_documents,
                )
                .await;
                match (resp, skip_failed) {
                    (Ok(r), _) => r,
                    (_, true) => {
                        println!("Failed");
                        return Ok(());
                    }
                    (e, _) => {
                        return e;
                    }
                }
            }
            (fpm::File::Image(main_doc), fpm::File::Image(fallback_doc)) => {
                process_static(main_doc, &config.root, package).await?;
                let resp = process_image(
                    config,
                    main_doc,
                    Some(fallback_doc),
                    message,
                    translated_data,
                    base_url,
                    package,
                    asset_documents,
                )
                .await;
                match (resp, skip_failed) {
                    (Ok(r), _) => r,
                    (_, true) => {
                        println!("Failed");
                        return Ok(());
                    }
                    (e, _) => {
                        return e;
                    }
                }
            }
            (fpm::File::Markdown(main_doc), fpm::File::Markdown(fallback_doc)) => {
                let resp = process_markdown(
                    config,
                    main_doc,
                    Some(fallback_doc),
                    message,
                    translated_data,
                    base_url,
                    asset_documents,
                )
                .await;
                match (resp, skip_failed) {
                    (Ok(r), _) => r,
                    (_, true) => {
                        println!("Failed");
                        return Ok(());
                    }
                    (e, _) => {
                        return e;
                    }
                }
            }
            (main_file, fallback_file) => {
                return Err(fpm::Error::UsageError {
                    message: format!(
                        "{:?} has not the same type as {:?}",
                        main_file, fallback_file
                    ),
                })
            }
        }
        fpm::utils::print_end(
            format!("Processed {}/{}", package.name.as_str(), main.get_id()).as_str(),
            start,
        );
        return Ok(());
    }
    print!(
        "Processing {}/{} ... ",
        package.name.as_str(),
        main.get_id()
    );
    std::io::stdout().flush()?;
    match main {
        fpm::File::Ftd(doc) => {
            let resp = process_ftd(
                config,
                doc,
                None,
                message,
                translated_data,
                base_url,
                asset_documents,
            )
            .await;
            match (resp, skip_failed) {
                (Ok(r), _) => r,
                (_, true) => {
                    println!("Failed");
                    return Ok(());
                }
                (e, _) => {
                    return e;
                }
            }
        }
        fpm::File::Static(sa) => process_static(sa, &config.root, package).await?,
        fpm::File::Markdown(doc) => {
            let resp = process_markdown(
                config,
                doc,
                None,
                message,
                translated_data,
                base_url,
                asset_documents,
            )
            .await;
            match (resp, skip_failed) {
                (Ok(r), _) => r,
                (_, true) => {
                    println!("Failed");
                    return Ok(());
                }
                (e, _) => {
                    return e;
                }
            }
        }
        fpm::File::Image(main_doc) => {
            process_static(main_doc, &config.root, package).await?;
            let resp = process_image(
                config,
                main_doc,
                None,
                message,
                translated_data,
                base_url,
                package,
                asset_documents,
            )
            .await;
            match (resp, skip_failed) {
                (Ok(r), _) => r,
                (_, true) => {
                    println!("Failed");
                    return Ok(());
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
                    base_path: camino::Utf8PathBuf::from(doc.parent_path.as_str()),
                },
                &config.root,
                package,
            )
            .await?;
            let resp = process_code(
                config,
                doc,
                None,
                message,
                translated_data,
                base_url,
                asset_documents,
            )
            .await;
            match (resp, skip_failed) {
                (Ok(r), _) => r,
                (_, true) => {
                    println!("Failed");
                    return Ok(());
                }
                (e, _) => {
                    return e;
                }
            }
        }
    }
    fpm::utils::print_end(
        format!("Processed {}/{}", package.name.as_str(), main.get_id()).as_str(),
        start,
    );
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn process_image(
    config: &fpm::Config,
    main: &fpm::Static,
    fallback: Option<&fpm::Static>,
    message: Option<&str>,
    translated_data: fpm::TranslationData,
    base_url: &str,
    package: &fpm::Package,
    asset_documents: &std::collections::HashMap<String, String>,
) -> fpm::Result<()> {
    let main = convert_to_ftd(config, main, package)?;
    if let Some(d) = fallback {
        return process_ftd(
            config,
            &main,
            Some(&convert_to_ftd(config, d, package)?),
            message,
            translated_data,
            base_url,
            asset_documents,
        )
        .await;
    }

    return process_ftd(
        config,
        &main,
        None,
        message,
        translated_data,
        base_url,
        asset_documents,
    )
    .await;

    fn convert_to_ftd(
        config: &fpm::Config,
        doc: &fpm::Static,
        package: &fpm::Package,
    ) -> fpm::Result<fpm::Document> {
        Ok(fpm::Document {
            package_name: package.name.to_string(),
            id: convert_to_ftd_extension(doc.id.as_str())?,
            content: fpm::package_info_image(config, doc, package)?,
            parent_path: doc.base_path.to_string(),
        })
    }

    fn convert_to_ftd_extension(name: &str) -> fpm::Result<String> {
        Ok(format!("{}.ftd", name))
    }
}

async fn process_code(
    config: &fpm::Config,
    main: &fpm::Document,
    fallback: Option<&fpm::Document>,
    message: Option<&str>,
    translated_data: fpm::TranslationData,
    base_url: &str,
    asset_documents: &std::collections::HashMap<String, String>,
) -> fpm::Result<()> {
    let main = if let Some(main) = convert_to_ftd(config, main)? {
        main
    } else {
        return Ok(());
    };
    if let Some(d) = fallback {
        match convert_to_ftd(config, d)? {
            Some(d) => {
                return process_ftd(
                    config,
                    &main,
                    Some(&d),
                    message,
                    translated_data,
                    base_url,
                    asset_documents,
                )
                .await;
            }
            None => {
                return Ok(());
            }
        }
    };

    return process_ftd(
        config,
        &main,
        None,
        message,
        translated_data,
        base_url,
        asset_documents,
    )
    .await;

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
    config: &fpm::Config,
    main: &fpm::Document,
    fallback: Option<&fpm::Document>,
    message: Option<&str>,
    translated_data: fpm::TranslationData,
    base_url: &str,
    asset_documents: &std::collections::HashMap<String, String>,
) -> fpm::Result<()> {
    let main = if let Some(main) = convert_md_to_ftd(config, main)? {
        main
    } else {
        return Ok(());
    };
    if let Some(d) = fallback {
        match convert_md_to_ftd(config, d)? {
            Some(d) => {
                return process_ftd(
                    config,
                    &main,
                    Some(&d),
                    message,
                    translated_data,
                    base_url,
                    asset_documents,
                )
                .await;
            }
            None => {
                return Ok(());
            }
        }
    };

    return process_ftd(
        config,
        &main,
        None,
        message,
        translated_data,
        base_url,
        asset_documents,
    )
    .await;

    fn convert_md_to_ftd(
        config: &fpm::Config,
        doc: &fpm::Document,
    ) -> fpm::Result<Option<fpm::Document>> {
        let doc_id = if doc.id == "README.md"
            && !(std::path::Path::new(format!(".{}index.ftd", std::path::MAIN_SEPARATOR).as_str())
                .exists()
                || std::path::Path::new(format!(".{}index.md", std::path::MAIN_SEPARATOR).as_str())
                    .exists())
        {
            "index.md".to_string()
        } else if !std::path::Path::new(
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

// todo: place this function with its dependent function in different file.
pub(crate) async fn get_ftd(
    config: &fpm::Config,
    main: &fpm::Document,
    base_url: &str,
    asset_documents: &std::collections::HashMap<String, String>,
) -> fpm::Result<String> {
    let new_main = fpm::Document {
        content: config
            .package
            .get_prefixed_body(main.content.as_str(), &main.id, true),
        id: main.id.to_owned(),
        parent_path: main.parent_path.to_owned(),
        package_name: main.package_name.to_owned(),
    };

    return get_html(config, &new_main, base_url, asset_documents).await;

    async fn get_html(
        config: &fpm::Config,
        main: &fpm::Document,
        base_url: &str,
        asset_documents: &std::collections::HashMap<String, String>,
    ) -> fpm::Result<String> {
        let lib = fpm::Library {
            config: config.clone(),
            markdown: None,
            document_id: main.id.clone(),
            translated_data: Default::default(),
            asset_documents: asset_documents.to_owned(),
            base_url: base_url.to_string(),
        };

        let main_ftd_doc = match ftd::p2::Document::from(
            main.id_with_package().as_str(),
            main.content.as_str(),
            &lib,
        ) {
            Ok(v) => v,
            Err(e) => {
                return Err(fpm::Error::PackageError {
                    message: format!("failed to parse {:?}", &e),
                });
            }
        };
        let doc_title = match &main_ftd_doc.title() {
            Some(x) => x.original.clone(),
            _ => main.id.as_str().to_string(),
        };
        let ftd_doc = main_ftd_doc.to_rt("main", &main.id);
        Ok(replace_markers(
            fpm::ftd_html(),
            config,
            main,
            doc_title.as_str(),
            base_url,
            &ftd_doc,
        ))
    }
}

async fn process_ftd(
    config: &fpm::Config,
    main: &fpm::Document,
    fallback: Option<&fpm::Document>,
    message: Option<&str>,
    translated_data: fpm::TranslationData,
    base_url: &str,
    asset_documents: &std::collections::HashMap<String, String>,
) -> fpm::Result<()> {
    if main.id.eq("FPM.ftd") {
        if config.is_translation_package() {
            use std::io::Write;

            let original_snapshots =
                fpm::snapshot::get_latest_snapshots(&config.original_path()?).await?;
            let translation_status =
                fpm::translation::get_translation_status_counts(&original_snapshots, &config.root)?;
            let content = std::fs::read_to_string(config.root.join(main.id.as_str()))?;
            let fpm = {
                let mut translation_status_summary = format!(
                    indoc::indoc! {"
                        -- fpm.translation-status-summary:
                        never-marked: {never_marked}
                        missing: {missing}
                        out-dated: {out_dated}
                        upto-date: {upto_date}
                    "},
                    never_marked = translation_status.never_marked,
                    missing = translation_status.missing,
                    out_dated = translation_status.out_dated,
                    upto_date = translation_status.upto_date,
                );
                if let Some(last_modified_on) = translation_status.last_modified_on {
                    translation_status_summary = format!(
                        indoc::indoc! {"
                            {translation_status_summary}last-modified-on: {last_modified_on}
                        "},
                        translation_status_summary = translation_status_summary,
                        last_modified_on = last_modified_on
                    );
                }

                format!(
                    indoc::indoc! {"
                    {content}

                    {translation_status_count}

                    "},
                    content = content,
                    translation_status_count = translation_status_summary
                )
            };

            let mut file =
                std::fs::File::create(config.root.join(".build").join(main.id.as_str()))?;
            file.write_all(fpm.as_bytes())?;
        } else {
            std::fs::copy(
                config.root.join(main.id.as_str()),
                config.root.join(".build").join(main.id.as_str()),
            )?;
        }
    }
    let main = {
        let mut main = main.to_owned();
        if main.id.eq("FPM.ftd") {
            main.id = "-.ftd".to_string();
            let path = config.root.join("FPM").join("info.ftd");
            main.content = if path.is_file() {
                std::fs::read_to_string(path)?
            } else {
                let package_info_package = match config
                    .package
                    .get_dependency_for_interface(fpm::PACKAGE_INFO_INTERFACE)
                    .or_else(|| {
                        config
                            .package
                            .get_dependency_for_interface(fpm::PACKAGE_THEME_INTERFACE)
                    }) {
                    Some(dep) => dep.package.name.as_str(),
                    None => fpm::PACKAGE_INFO_INTERFACE,
                };
                config.package.get_prefixed_body(
                    format!(
                        "-- import: {}/package-info as pi\n\n-- pi.package-info-page:",
                        package_info_package
                    )
                    .as_str(),
                    &main.id,
                    true,
                )
            }
        }
        main
    };

    if !main.id.eq("index.ftd") {
        std::fs::create_dir_all(config.root.join(".build").join(main.id_to_path()))?;
    }
    let file_rel_path = if main.id.eq("index.ftd") {
        "index.html".to_string()
    } else {
        main.id.replace(
            ".ftd",
            format!("{}index.html", std::path::MAIN_SEPARATOR).as_str(),
        )
    };

    let new_file_path = config.root.join(".build").join(file_rel_path);

    let (fallback, message, final_main) = if main.id.eq("-.ftd") {
        (None, None, main)
    } else {
        let new_main = fpm::Document {
            content: config
                .package
                .get_prefixed_body(main.content.as_str(), &main.id, true),
            id: main.id,
            parent_path: main.parent_path,
            package_name: main.package_name,
        };
        let new_fallback = fallback.map(|fb| fpm::Document {
            content: config
                .package
                .get_prefixed_body(fb.content.as_str(), &fb.id, true),
            ..fb.to_owned()
        });
        (new_fallback, message, new_main)
    };
    match (fallback, message) {
        (Some(fallback), Some(message)) => {
            write_with_fallback(
                config,
                &final_main,
                fallback,
                new_file_path.as_str(),
                message,
                translated_data,
                base_url,
                asset_documents,
            )
            .await?
        }
        (None, Some(message)) => {
            write_with_message(
                config,
                &final_main,
                new_file_path.as_str(),
                message,
                translated_data,
                base_url,
                asset_documents,
            )
            .await?
        }
        _ => {
            write_default(
                config,
                &final_main,
                new_file_path.as_str(),
                base_url,
                asset_documents,
            )
            .await?
        }
    }

    return Ok(());

    async fn write_default(
        config: &fpm::Config,
        main: &fpm::Document,
        new_file_path: &str,
        base_url: &str,
        asset_documents: &std::collections::HashMap<String, String>,
    ) -> fpm::Result<()> {
        use tokio::io::AsyncWriteExt;

        let lib = fpm::Library {
            config: config.clone(),
            markdown: None,
            document_id: main.id.clone(),
            translated_data: Default::default(),
            asset_documents: asset_documents.to_owned(),
            base_url: base_url.to_string(),
        };

        let main_ftd_doc = match ftd::p2::Document::from(
            main.id_with_package().as_str(),
            main.content.as_str(),
            &lib,
        ) {
            Ok(v) => v,
            Err(e) => {
                return Err(fpm::Error::PackageError {
                    message: format!("failed to parse {:?}", &e),
                });
            }
        };
        let doc_title = match &main_ftd_doc.title() {
            Some(x) => x.original.clone(),
            _ => main.id.as_str().to_string(),
        };
        let ftd_doc = main_ftd_doc.to_rt("main", &main.id);

        let mut f = tokio::fs::File::create(new_file_path).await?;
        f.write_all(
            replace_markers(
                fpm::ftd_html(),
                config,
                main,
                doc_title.as_str(),
                base_url,
                &ftd_doc,
            )
            .as_bytes(),
        )
        .await?;
        Ok(())
    }

    async fn write_with_message(
        config: &fpm::Config,
        main: &fpm::Document,
        new_file_path: &str,
        message: &str,
        translated_data: fpm::TranslationData,
        base_url: &str,
        asset_documents: &std::collections::HashMap<String, String>,
    ) -> fpm::Result<()> {
        use tokio::io::AsyncWriteExt;

        let lib = fpm::Library {
            config: config.clone(),
            markdown: None,
            document_id: main.id.clone(),
            translated_data,
            asset_documents: asset_documents.to_owned(),
            base_url: base_url.to_string(),
        };

        let main_ftd_doc = match ftd::p2::Document::from(
            main.id_with_package().as_str(),
            main.content.as_str(),
            &lib,
        ) {
            Ok(v) => v,
            Err(e) => {
                return Err(fpm::Error::PackageError {
                    message: format!("failed to parse {:?}", &e),
                });
            }
        };

        let doc_title = match &main_ftd_doc.title() {
            Some(x) => x.original.clone(),
            _ => main.id.as_str().to_string(),
        };
        let main_rt_doc = main_ftd_doc.to_rt("main", &main.id);

        let message_ftd_doc = match ftd::p2::Document::from("message", message, &lib) {
            Ok(v) => v,
            Err(e) => {
                return Err(fpm::Error::PackageError {
                    message: format!("failed to parse {:?}", &e),
                });
            }
        };
        let message_rt_doc = message_ftd_doc.to_rt("message", &main.id);

        let mut f = tokio::fs::File::create(new_file_path).await?;

        f.write_all(
            replace_markers(
                fpm::with_message()
                    .replace(
                        "__ftd_data_message__",
                        serde_json::to_string_pretty(&message_rt_doc.data)
                            .expect("failed to convert document to json")
                            .as_str(),
                    )
                    .replace(
                        "__ftd_external_children_message__",
                        serde_json::to_string_pretty(&message_rt_doc.external_children)
                            .expect("failed to convert document to json")
                            .as_str(),
                    )
                    .replace(
                        "__message__",
                        format!("{}{}", message_rt_doc.html, config.get_font_style(),).as_str(),
                    )
                    .as_str(),
                config,
                main,
                doc_title.as_str(),
                base_url,
                &main_rt_doc,
            )
            .as_bytes(),
        )
        .await?;
        Ok(())
    }
    #[allow(clippy::too_many_arguments)]
    async fn write_with_fallback(
        config: &fpm::Config,
        main: &fpm::Document,
        fallback: fpm::Document,
        new_file_path: &str,
        message: &str,
        translated_data: fpm::TranslationData,
        base_url: &str,
        asset_documents: &std::collections::HashMap<String, String>,
    ) -> fpm::Result<()> {
        use tokio::io::AsyncWriteExt;
        let lib = fpm::Library {
            config: config.clone(),
            markdown: None,
            document_id: main.id.clone(),
            translated_data,
            asset_documents: asset_documents.to_owned(),
            base_url: base_url.to_string(),
        };

        let main_ftd_doc = match ftd::p2::Document::from(
            main.id_with_package().as_str(),
            main.content.as_str(),
            &lib,
        ) {
            Ok(v) => v,
            Err(e) => {
                return Err(fpm::Error::PackageError {
                    message: format!("failed to parse {:?}", &e),
                });
            }
        };
        let main_rt_doc = main_ftd_doc.to_rt("main", &main.id);

        let message_ftd_doc = match ftd::p2::Document::from("message", message, &lib) {
            Ok(v) => v,
            Err(e) => {
                return Err(fpm::Error::PackageError {
                    message: format!("failed to parse {:?}", &e),
                });
            }
        };

        let doc_title = match &main_ftd_doc.title() {
            Some(x) => x.original.clone(),
            _ => main.id.as_str().to_string(),
        };
        let message_rt_doc = message_ftd_doc.to_rt("message", &main.id);

        let fallback_ftd_doc = match ftd::p2::Document::from(
            main.id_with_package().as_str(),
            fallback.content.as_str(),
            &lib,
        ) {
            Ok(v) => v,
            Err(e) => {
                return Err(fpm::Error::PackageError {
                    message: format!("failed to parse {:?}", &e),
                });
            }
        };
        let fallback_rt_doc = fallback_ftd_doc.to_rt("fallback", &fallback.id);

        let mut f = tokio::fs::File::create(new_file_path).await?;

        f.write_all(
            replace_markers(
                fpm::with_fallback()
                    .replace(
                        "__ftd_data_message__",
                        serde_json::to_string_pretty(&message_rt_doc.data)
                            .expect("failed to convert document to json")
                            .as_str(),
                    )
                    .replace(
                        "__ftd_external_children_message__",
                        serde_json::to_string_pretty(&message_rt_doc.external_children)
                            .expect("failed to convert document to json")
                            .as_str(),
                    )
                    .replace(
                        "__message__",
                        format!("{}{}", message_rt_doc.html, config.get_font_style(),).as_str(),
                    )
                    .replace(
                        "__ftd_data_fallback__",
                        serde_json::to_string_pretty(&fallback_rt_doc.data)
                            .expect("failed to convert document to json")
                            .as_str(),
                    )
                    .replace(
                        "__ftd_external_children_fallback__",
                        serde_json::to_string_pretty(&fallback_rt_doc.external_children)
                            .expect("failed to convert document to json")
                            .as_str(),
                    )
                    .replace(
                        "__fallback__",
                        format!("{}{}", fallback_rt_doc.html, config.get_font_style(),).as_str(),
                    )
                    .as_str(),
                config,
                main,
                doc_title.as_str(),
                base_url,
                &main_rt_doc,
            )
            .as_bytes(),
        )
        .await?;
        Ok(())
    }
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

fn replace_markers(
    s: &str,
    config: &fpm::Config,
    main: &fpm::Document,
    title: &str,
    base_url: &str,
    main_rt: &ftd::Document,
) -> String {
    s.replace("__ftd_doc_title__", title)
        .replace(
            "__ftd_canonical_url__",
            config
                .package
                .generate_canonical_url(main.id_to_path().as_str())
                .as_str(),
        )
        .replace("__ftd_js__", fpm::ftd_js().as_str())
        .replace("__ftd_body_events__", main_rt.body_events.as_str())
        .replace("__ftd_css__", fpm::ftd_css())
        .replace("__fpm_js__", fpm::fpm_js())
        .replace(
            "__ftd_data_main__",
            fpm::font::escape(
                serde_json::to_string_pretty(&main_rt.data)
                    .expect("failed to convert document to json")
                    .as_str(),
            )
            .as_str(),
        )
        .replace(
            "__ftd_external_children_main__",
            fpm::font::escape(
                serde_json::to_string_pretty(&main_rt.external_children)
                    .expect("failed to convert document to json")
                    .as_str(),
            )
            .as_str(),
        )
        .replace(
            "__main__",
            format!("{}{}", main_rt.html, config.get_font_style(),).as_str(),
        )
        .replace("__base_url__", base_url)
}
