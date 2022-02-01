use std::iter::FromIterator;

pub async fn build(
    config: &fpm::Config,
    file: Option<&str>,
    base_url: Option<&str>,
    ignore_failed: bool,
) -> fpm::Result<()> {
    use fpm::utils::HasElements;

    tokio::fs::create_dir_all(config.build_dir()).await?;
    // let skip_failed = ignore_failed.unwrap_or(false);

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
            build_with_original(config, original, file, base_url, ignore_failed).await
        }
        (None, false) => build_simple(config, file, base_url, ignore_failed).await,
        (None, true) => build_with_translations(config, file, base_url, ignore_failed).await,
    }
}

async fn build_simple(
    config: &fpm::Config,
    file: Option<&str>,
    base_url: Option<&str>,
    skip_failed: bool,
) -> fpm::Result<()> {
    let documents = std::collections::BTreeMap::from_iter(
        fpm::get_documents(config)
            .await?
            .into_iter()
            .map(|v| (v.get_id(), v)),
    );

    process_files(config, &documents, file, base_url, skip_failed).await
}

async fn build_with_translations(
    config: &fpm::Config,
    _file: Option<&str>,
    base_url: Option<&str>,
    skip_failed: bool,
) -> fpm::Result<()> {
    let documents = std::collections::BTreeMap::from_iter(
        fpm::get_documents(config)
            .await?
            .into_iter()
            .map(|v| (v.get_id(), v)),
    );

    let message = fpm::available_languages(config)?;

    for file in documents.values() {
        fpm::process_file(
            config,
            file,
            None,
            Some(message.as_str()),
            Default::default(),
            base_url,
            skip_failed,
        )
        .await?;
    }
    // Add /FPM/translation-status page
    {
        let translation_status = fpm::Document {
            id: "FPM/translation-status.ftd".to_string(),
            content: fpm::original_package_status(config)?,
            parent_path: config.root.as_str().to_string(),
        };

        process_ftd(
            config,
            &translation_status,
            None,
            None,
            Default::default(),
            base_url,
        )
        .await?;
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
    _file: Option<&str>,
    base_url: Option<&str>,
    skip_failed: bool,
) -> fpm::Result<()> {
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
        fpm::paths_to_files(files, original_path.as_path())
            .await?
            .into_iter()
            .map(|v| (v.get_id(), v)),
    );

    // Fetch all files from the current package
    // ignore all those documents/files which is not in original package
    // Overwrite the files having same name/id

    let translated_documents = std::collections::BTreeMap::from_iter(
        fpm::get_documents(config)
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
        translated_document
            .html(config, base_url, skip_failed)
            .await?;
    }

    // Add /FPM/translation-status page
    {
        let translation_status = fpm::Document {
            id: "FPM/translation-status.ftd".to_string(),
            content: fpm::translation_package_status(config)?,
            parent_path: config.root.as_str().to_string(),
        };

        process_ftd(
            config,
            &translation_status,
            None,
            None,
            Default::default(),
            base_url,
        )
        .await?;
    }
    Ok(())
}

async fn process_files(
    config: &fpm::Config,
    documents: &std::collections::BTreeMap<String, fpm::File>,
    file: Option<&str>,
    base_url: Option<&str>,
    skip_failed: bool,
) -> fpm::Result<()> {
    for f in documents.values() {
        if file.is_some() && file != Some(f.get_id().as_str()) {
            continue;
        }
        process_file(
            config,
            f,
            None,
            None,
            Default::default(),
            base_url,
            skip_failed,
        )
        .await?
    }
    Ok(())
}

pub(crate) async fn process_file(
    config: &fpm::Config,
    main: &fpm::File,
    fallback: Option<&fpm::File>,
    message: Option<&str>,
    translated_data: fpm::TranslationData,
    base_url: Option<&str>,
    skip_failed: bool,
) -> fpm::Result<()> {
    if let Some(fallback) = fallback {
        match (main, fallback) {
            (fpm::File::Ftd(main_doc), fpm::File::Ftd(fallback_doc)) => {
                let resp = process_ftd(
                    config,
                    main_doc,
                    Some(fallback_doc),
                    message,
                    translated_data,
                    base_url,
                )
                .await;
                match (resp, skip_failed) {
                    (Ok(r), _) => r,
                    (_, true) => {
                        println!("Failed to process {}", main.get_id());
                        return Ok(());
                    }
                    (_, _) => {
                        return Err(fpm::Error::UsageError {
                            message: format!("{:?} Unable to process the document", main.get_id()),
                        })
                    }
                }
            }
            (fpm::File::Static(main_sa), fpm::File::Static(_)) => {
                process_static(main_sa, &config.root).await?
            }
            (fpm::File::Markdown(main_sa), fpm::File::Markdown(_)) => {
                process_markdown(main_sa, config, base_url).await?
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
        println!("Processed {}", main.get_id());
        return Ok(());
    }
    match main {
        fpm::File::Ftd(doc) => {
            let resp = process_ftd(config, doc, None, message, translated_data, base_url).await;
            match (resp, skip_failed) {
                (Ok(r), _) => r,
                (_, true) => {
                    println!("Failed to process {}", main.get_id());
                    return Ok(());
                }
                (_, _) => {
                    return Err(fpm::Error::UsageError {
                        message: format!("{:?} Unable to process the document", main.get_id()),
                    })
                }
            }
        }
        fpm::File::Static(sa) => process_static(sa, &config.root).await?,
        fpm::File::Markdown(doc) => process_markdown(doc, config, base_url).await?,
    }
    println!("Processed {}", main.get_id());
    Ok(())
}

async fn process_markdown(
    _doc: &fpm::Document,
    _config: &fpm::Config,
    _base_url: Option<&str>,
) -> fpm::Result<()> {
    /*if _doc.id == "README.md"
        && !(std::path::Path::new(
        format!(".{}index.ftd", std::path::MAIN_SEPARATOR).as_str(),
    )
        .exists()
        || std::path::Path::new(
        format!(".{}index.md", std::path::MAIN_SEPARATOR).as_str(),
    )
        .exists())
    {
        "index.md".to_string()
    } else {
        _doc.id.to_string()
    }*/
    // if let Ok(c) = tokio::fs::read_to_string("./FPM/markdown.ftd").await {
    //     c
    // } else {
    //     fpm::default_markdown().to_string()
    // }
    // todo
    Ok(())
}

async fn process_ftd(
    config: &fpm::Config,
    main: &fpm::Document,
    fallback: Option<&fpm::Document>,
    message: Option<&str>,
    translated_data: fpm::TranslationData,
    base_url: Option<&str>,
) -> fpm::Result<()> {
    if main.id.eq("FPM.ftd") {
        if config.is_translation_package() {
            use std::io::Write;

            let original_snapshots =
                fpm::snapshot::get_latest_snapshots(&config.original_path()?).await?;
            let translation_status =
                fpm::translation::get_translation_status_count(&original_snapshots, &config.root)?;
            let content = std::fs::read_to_string(config.root.join(main.id.as_str()))?;
            let fpm = {
                let mut translation_status_count = format!(
                    indoc::indoc! {"
                        -- fpm.translation-status-count:
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
                    translation_status_count = format!(
                        indoc::indoc! {"
                            {translation_status_count}last-modified-on: {last_modified_on}
                        "},
                        translation_status_count = translation_status_count,
                        last_modified_on = last_modified_on
                    );
                }

                format!(
                    indoc::indoc! {"
                    {content}

                    {translation_status_count}

                    "},
                    content = content,
                    translation_status_count = translation_status_count
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

    let (fallback, message, final_main) = if main.id.eq("FPM.ftd") {
        let mut main = main.to_owned();
        main.content = include_str!("../../ftd/info.ftd").to_string();
        (None, None, main)
    } else {
        (fallback, message, main.to_owned())
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
            )
            .await?
        }
        _ => write_default(config, &final_main, new_file_path.as_str(), base_url).await?,
    }

    return Ok(());

    async fn write_default(
        config: &fpm::Config,
        main: &fpm::Document,
        new_file_path: &str,
        base_url: Option<&str>,
    ) -> fpm::Result<()> {
        use tokio::io::AsyncWriteExt;

        let lib = fpm::Library {
            config: config.clone(),
            markdown: None,
            document_id: main.id.clone(),
            translated_data: Default::default(),
            current_package: std::sync::Arc::new(std::sync::Mutex::new(vec![config
                .package
                .clone()])),
        };

        let main_ftd_doc =
            match ftd::p2::Document::from(main.id.as_str(), main.content.as_str(), &lib) {
                Ok(v) => v,
                Err(e) => {
                    return Err(fpm::Error::PackageError {
                        message: format!("failed to parse {:?}", &e),
                    });
                }
            };
        let doc_title = match &main_ftd_doc.title() {
            Some(x) => x.rendered.clone(),
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
        base_url: Option<&str>,
    ) -> fpm::Result<()> {
        use tokio::io::AsyncWriteExt;

        let lib = fpm::Library {
            config: config.clone(),
            markdown: None,
            document_id: main.id.clone(),
            translated_data,
            current_package: std::sync::Arc::new(std::sync::Mutex::new(vec![config
                .package
                .clone()])),
        };

        let main_ftd_doc =
            match ftd::p2::Document::from(main.id.as_str(), main.content.as_str(), &lib) {
                Ok(v) => v,
                Err(e) => {
                    return Err(fpm::Error::PackageError {
                        message: format!("failed to parse {:?}", &e),
                    });
                }
            };

        let doc_title = match &main_ftd_doc.title() {
            Some(x) => x.rendered.clone(),
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

    async fn write_with_fallback(
        config: &fpm::Config,
        main: &fpm::Document,
        fallback: &fpm::Document,
        new_file_path: &str,
        message: &str,
        translated_data: fpm::TranslationData,
        base_url: Option<&str>,
    ) -> fpm::Result<()> {
        use tokio::io::AsyncWriteExt;

        let lib = fpm::Library {
            config: config.clone(),
            markdown: None,
            document_id: main.id.clone(),
            translated_data,
            current_package: std::sync::Arc::new(std::sync::Mutex::new(vec![config
                .package
                .clone()])),
        };

        let main_ftd_doc =
            match ftd::p2::Document::from(main.id.as_str(), main.content.as_str(), &lib) {
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
            Some(x) => x.rendered.clone(),
            _ => main.id.as_str().to_string(),
        };
        let message_rt_doc = message_ftd_doc.to_rt("message", &main.id);

        let fallback_ftd_doc =
            match ftd::p2::Document::from(main.id.as_str(), fallback.content.as_str(), &lib) {
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

async fn process_static(sa: &fpm::Static, base_path: &camino::Utf8Path) -> fpm::Result<()> {
    if let Some((dir, _)) = sa.id.rsplit_once(std::path::MAIN_SEPARATOR) {
        std::fs::create_dir_all(base_path.join(".build").join(dir))?;
    }
    std::fs::copy(
        sa.base_path.join(sa.id.as_str()),
        base_path.join(".build").join(sa.id.as_str()),
    )?;
    Ok(())
}

fn replace_markers(
    s: &str,
    config: &fpm::Config,
    main: &fpm::Document,
    title: &str,
    base_url: Option<&str>,
    main_rt: &ftd::Document,
) -> String {
    let mut s = s
        .replace("__ftd_doc_title__", title)
        .replace(
            "__ftd_canonical_url__",
            config
                .package
                .generate_canonical_url(main.id_to_path().as_str())
                .as_str(),
        )
        .replace("__ftd_js__", fpm::ftd_js())
        .replace("__ftd_css__", fpm::ftd_css())
        .replace("__fpm_js__", fpm::fpm_js())
        .replace(
            "__ftd_data_main__",
            serde_json::to_string_pretty(&main_rt.data)
                .expect("failed to convert document to json")
                .as_str(),
        )
        .replace(
            "__ftd_external_children_main__",
            serde_json::to_string_pretty(&main_rt.external_children)
                .expect("failed to convert document to json")
                .as_str(),
        )
        .replace(
            "__main__",
            format!("{}{}", main_rt.html, config.get_font_style(),).as_str(),
        );
    s = match base_url {
        Some(u) => s.replace("__base_url__", u),
        None => s.replace("<base href=\"__base_url__\">", ""),
    };
    s
}
