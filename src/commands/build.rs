use std::iter::FromIterator;

pub async fn build(
    config: &mut fpm::Config,
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

    if config.package.versioned {
        fpm::version::build_version(config, file, base_url, ignore_failed, &asset_documents)
            .await?;
    } else {
        match (
            config.package.translation_of.is_some(),
            config.package.translations.has_elements(),
        ) {
            (true, true) => {
                // No package can be both a translation of something and has its own
                // translations, when building `config` we ensured this was rejected
                unreachable!()
            }
            (true, false) => {
                build_with_original(config, file, base_url, ignore_failed, &asset_documents).await
            }
            (false, false) => {
                build_simple(config, file, base_url, ignore_failed, &asset_documents).await
            }
            (false, true) => {
                build_with_translations(config, file, base_url, ignore_failed, &asset_documents)
                    .await
            }
        }?;
    }

    for dep in dependencies {
        let static_files = std::collections::BTreeMap::from_iter(
            config
                .get_files(&dep.package)
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
            true,
        )
        .await?;
    }
    Ok(())
}

async fn build_simple(
    config: &mut fpm::Config,
    file: Option<&str>,
    base_url: &str,
    skip_failed: bool,
    asset_documents: &std::collections::HashMap<String, String>,
) -> fpm::Result<()> {
    let mut documents = std::collections::BTreeMap::from_iter(
        config
            .get_files(&config.package)
            .await?
            .into_iter()
            .map(|v| (v.get_id(), v)),
    );

    if let Some(ref sitemap) = config.sitemap {
        let get_all_locations = sitemap.get_all_locations();
        let mut files: std::collections::HashMap<String, fpm::File> = Default::default();
        for (doc_path, _, url) in get_all_locations {
            let file = {
                let mut file = fpm::get_file(
                    config.package.name.to_string(),
                    doc_path,
                    config.root.as_path(),
                )
                .await?;
                if let Some(ref url) = url {
                    file.set_id(format!("{}/index.ftd", url).as_str());
                }
                file
            };
            files.insert(file.get_id(), file);
        }

        documents.extend(files);
    }

    process_files(
        config,
        &config.package.clone(),
        &documents,
        file,
        base_url,
        skip_failed,
        asset_documents,
        false,
    )
    .await
}

async fn build_with_translations(
    config: &mut fpm::Config,
    process_file: Option<&str>,
    base_url: &str,
    skip_failed: bool,
    asset_documents: &std::collections::HashMap<String, String>,
) -> fpm::Result<()> {
    use std::io::Write;

    let mut documents = std::collections::BTreeMap::from_iter(
        config
            .get_files(&config.package)
            .await?
            .into_iter()
            .map(|v| (v.get_id(), v)),
    );

    if let Some(ref sitemap) = config.sitemap {
        let get_all_locations = sitemap.get_all_locations();
        let mut files: std::collections::HashMap<String, fpm::File> = Default::default();
        for (doc_path, _, url) in get_all_locations {
            let file = {
                let mut file = fpm::get_file(
                    config.package.name.to_string(),
                    doc_path,
                    config.root.as_path(),
                )
                .await?;
                if let Some(ref url) = url {
                    file.set_id(format!("{}/index.ftd", url).as_str());
                }
                file
            };
            files.insert(file.get_id(), file);
        }

        documents.extend(files);
    }

    let message = fpm::available_languages(config)?;

    for file in documents.values() {
        if process_file.is_some() && process_file != Some(file.get_id().as_str()) {
            continue;
        }
        config.current_document = Some(file.get_id());
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
            None,
            false,
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
    config: &mut fpm::Config,
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
    let mut original_documents = std::collections::BTreeMap::from_iter(
        fpm::paths_to_files(config.package.name.as_str(), files, original_path.as_path())
            .await?
            .into_iter()
            .map(|v| (v.get_id(), v)),
    );

    let mut translated_documents = std::collections::BTreeMap::from_iter(
        config
            .get_files(&config.package)
            .await?
            .into_iter()
            .filter(|x| original_snapshots.contains_key(x.get_id().as_str()))
            .map(|v| (v.get_id(), v)),
    );

    if let Some(ref sitemap) = config.sitemap {
        let get_all_locations = sitemap.get_all_locations();
        let mut files: std::collections::HashMap<String, fpm::File> = Default::default();
        let mut translation_files: std::collections::HashMap<String, fpm::File> =
            Default::default();
        for (doc_path, translation_doc_path, url) in get_all_locations {
            let file = {
                let mut file = fpm::get_file(
                    config.package.name.to_string(),
                    doc_path,
                    config.root.as_path(),
                )
                .await?;
                if let Some(ref url) = url {
                    file.set_id(format!("{}/index.ftd", url).as_str());
                }
                file
            };
            files.insert(file.get_id(), file);
            if let Some(translation_doc_path) = translation_doc_path {
                let file = {
                    let mut file = fpm::get_file(
                        config.package.name.to_string(),
                        translation_doc_path,
                        config.root.as_path(),
                    )
                    .await?;
                    if let Some(ref url) = url {
                        file.set_id(format!("{}/index.ftd", url).as_str());
                    }
                    file
                };
                translation_files.insert(file.get_id(), file);
            }
        }

        original_documents.extend(files);
        translated_documents.extend(translation_files);
    }

    // Fetch all files from the current package
    // ignore all those documents/files which is not in original package
    // Overwrite the files having same name/id

    let translated_documents = fpm::TranslatedDocument::get_translated_document(
        config,
        original_documents,
        translated_documents,
    )
    .await?;

    // Process all the files collected from original and root package
    for translated_document in translated_documents.values() {
        let id = match translated_document {
            fpm::TranslatedDocument::Missing { ref original } => original.get_id(),
            fpm::TranslatedDocument::NeverMarked { ref translated, .. } => translated.get_id(),
            fpm::TranslatedDocument::Outdated { ref translated, .. } => translated.get_id(),
            fpm::TranslatedDocument::UptoDate { ref translated, .. } => translated.get_id(),
        };
        if file.is_some() && file != Some(id.as_str()) {
            continue;
        }
        config.current_document = Some(id);
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

#[allow(clippy::too_many_arguments)]
async fn process_files(
    config: &mut fpm::Config,
    package: &fpm::Package,
    documents: &std::collections::BTreeMap<String, fpm::File>,
    file: Option<&str>,
    base_url: &str,
    skip_failed: bool,
    asset_documents: &std::collections::HashMap<String, String>,
    copy_only: bool,
) -> fpm::Result<()> {
    for f in documents.values() {
        if file.is_some() && file != Some(f.get_id().as_str()) {
            continue;
        }
        config.current_document = Some(f.get_id());
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
            None,
            copy_only,
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
    original_id: Option<String>,
    copy_only: bool,
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
                process_static(main_sa, &config.root, package, original_id).await?
            }
            (fpm::File::Code(main_doc), fpm::File::Code(fallback_doc)) => {
                process_static(
                    &fpm::Static {
                        id: main_doc.id.to_string(),
                        base_path: camino::Utf8PathBuf::from(main_doc.parent_path.as_str()),
                    },
                    &config.root,
                    package,
                    original_id,
                )
                .await?;
                if !copy_only {
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
            }
            (fpm::File::Image(main_doc), fpm::File::Image(fallback_doc)) => {
                process_static(main_doc, &config.root, package, original_id).await?;
                if !copy_only {
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
        fpm::File::Static(sa) => process_static(sa, &config.root, package, original_id).await?,
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
            process_static(main_doc, &config.root, package, original_id).await?;
            if !copy_only {
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
        }
        fpm::File::Code(doc) => {
            process_static(
                &fpm::Static {
                    id: doc.id.to_string(),
                    base_path: camino::Utf8PathBuf::from(doc.parent_path.as_str()),
                },
                &config.root,
                package,
                original_id,
            )
            .await?;
            if !copy_only {
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

    std::fs::create_dir_all(config.root.join(".build").join(main.id_to_path()))?;

    let file_rel_path = if main.id.contains("index.ftd") {
        main.id.replace("index.ftd", "index.html")
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
            fpm::utils::replace_markers(
                fpm::ftd_html(),
                config,
                main.id_to_path().as_str(),
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
            fpm::utils::replace_markers(
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
                main.id_to_path().as_str(),
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
            fpm::utils::replace_markers(
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
                main.id_to_path().as_str(),
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
    original_id: Option<String>,
) -> fpm::Result<()> {
    copy_to_build(sa, base_path, package, &original_id)?;
    if let Some(original_package) = package.translation_of.as_ref() {
        copy_to_build(sa, base_path, original_package, &original_id)?;
    }
    return Ok(());

    fn copy_to_build(
        sa: &fpm::Static,
        base_path: &camino::Utf8Path,
        package: &fpm::Package,
        original_id: &Option<String>,
    ) -> fpm::Result<()> {
        let build_path = base_path
            .join(".build")
            .join("-")
            .join(package.name.as_str());
        let original_id = if let Some(id) = original_id {
            id.as_str()
        } else {
            sa.id.as_str()
        };

        std::fs::create_dir_all(&build_path)?;
        if let Some((dir, _)) = sa.id.rsplit_once(std::path::MAIN_SEPARATOR) {
            std::fs::create_dir_all(&build_path.join(dir))?;
        }
        std::fs::copy(
            sa.base_path.join(original_id),
            build_path.join(sa.id.as_str()),
        )?;

        Ok(())
    }
}
