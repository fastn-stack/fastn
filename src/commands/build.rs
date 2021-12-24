pub async fn build(config: &fpm::Config) -> fpm::Result<()> {
    use fpm::utils::HasElements;

    tokio::fs::create_dir_all(config.build_dir()).await?;

    match (
        config.package.translation_of.as_ref(),
        config.package.translations.has_elements(),
    ) {
        (Some(_), true) => {
            // No package can be both a translation of something and has its own
            // translations, when building `config` we ensured this was rejected
            unreachable!()
        }
        (Some(original), false) => build_with_original(config, original).await,
        (None, false) => build_simple(config).await,
        (None, true) => build_with_translations(config).await,
    }
}

async fn build_simple(config: &fpm::Config) -> fpm::Result<()> {
    let documents = std::collections::BTreeMap::from_iter(
        fpm::get_documents(config)
            .await?
            .into_iter()
            .map(|v| (v.get_id(), v)),
    );

    process_files(config, &documents).await
}

async fn build_with_translations(_config: &fpm::Config) -> fpm::Result<()> {
    todo!("build does not yet support translations, only translation-of")
}

/// This is the translation package
/// First fetch all files from the original package
/// Then overwrite it with file in root package having same name/id as in original package
/// ignore all those documents/files which is not in original package
async fn build_with_original(config: &fpm::Config, _original: &fpm::Package) -> fpm::Result<()> {
    // This is the translation package
    // Fetch all files from the original package
    let original_path = config.original_path()?;
    let original_snapshots = fpm::snapshot::get_latest_snapshots(&original_path).await?;
    let files = original_snapshots
        .keys()
        .into_iter()
        .map(|v| original_path.join(v).into_std_path_buf())
        .collect::<Vec<std::path::PathBuf>>();

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
        translated_document.html(config).await?;
    }
    Ok(())
}

async fn process_files(
    config: &fpm::Config,
    documents: &std::collections::BTreeMap<String, fpm::File>,
) -> fpm::Result<()> {
    for file in documents.values() {
        process_file(config, file, None, None, Default::default()).await?
    }
    Ok(())
}

pub(crate) async fn process_file(
    config: &fpm::Config,
    main: &fpm::File,
    fallback: Option<&fpm::File>,
    message: Option<&str>,
    translated_data: fpm::TranslationData,
) -> fpm::Result<()> {
    if let Some(fallback) = fallback {
        match (main, fallback) {
            (fpm::File::Ftd(main_doc), fpm::File::Ftd(fallback_doc)) => {
                process_ftd(
                    config,
                    main_doc,
                    Some(fallback_doc),
                    message,
                    translated_data,
                )
                .await?
            }
            (fpm::File::Static(main_sa), fpm::File::Static(_)) => {
                process_static(main_sa, &config.root).await?
            }
            (fpm::File::Markdown(main_sa), fpm::File::Markdown(_)) => {
                process_markdown(main_sa, config).await?
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
        fpm::File::Ftd(doc) => process_ftd(config, doc, None, message, translated_data).await?,
        fpm::File::Static(sa) => process_static(sa, &config.root).await?,
        fpm::File::Markdown(doc) => process_markdown(doc, config).await?,
    }
    println!("Processed {}", main.get_id());
    Ok(())
}

async fn process_markdown(_doc: &fpm::Document, _config: &fpm::Config) -> fpm::Result<()> {
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
) -> fpm::Result<()> {
    if !main.id.eq("index.ftd") {
        std::fs::create_dir_all(config.root.join(".build").join(main.id.replace(".ftd", "")))?;
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

    match (fallback, message) {
        (Some(fallback), Some(message)) => {
            write_with_fallback(
                config,
                main,
                fallback,
                new_file_path.as_str(),
                message,
                translated_data,
            )
            .await?
        }
        (None, Some(message)) => {
            write_with_message(
                config,
                main,
                new_file_path.as_str(),
                message,
                translated_data,
            )
            .await?
        }
        _ => write_default(config, main, new_file_path.as_str()).await?,
    }

    return Ok(());

    async fn write_default(
        config: &fpm::Config,
        main: &fpm::Document,
        new_file_path: &str,
    ) -> fpm::Result<()> {
        use tokio::io::AsyncWriteExt;

        let lib = fpm::Library {
            config: config.clone(),
            markdown: None,
            document_id: main.id.clone(),
            translated_data: Default::default(),
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
        let ftd_doc = main_ftd_doc.to_rt("main", &main.id);

        let mut f = tokio::fs::File::create(new_file_path).await?;
        f.write_all(
            ftd::html()
                .replace(
                    "__ftd_data__",
                    serde_json::to_string_pretty(&ftd_doc.data)
                        .expect("failed to convert document to json")
                        .as_str(),
                )
                .replace(
                    "__ftd_external_children__",
                    serde_json::to_string_pretty(&ftd_doc.external_children)
                        .expect("failed to convert document to json")
                        .as_str(),
                )
                .replace(
                    "__ftd__",
                    format!("{}{}", ftd_doc.html, config.get_font_style(),).as_str(),
                )
                .as_str()
                .replace("__ftd_js__", ftd::js())
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
    ) -> fpm::Result<()> {
        use tokio::io::AsyncWriteExt;

        let lib = fpm::Library {
            config: config.clone(),
            markdown: None,
            document_id: main.id.clone(),
            translated_data,
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
        let message_rt_doc = message_ftd_doc.to_rt("message", &main.id);

        let mut f = tokio::fs::File::create(new_file_path).await?;

        f.write_all(
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
                .replace(
                    "__ftd_data_main__",
                    serde_json::to_string_pretty(&main_rt_doc.data)
                        .expect("failed to convert document to json")
                        .as_str(),
                )
                .replace(
                    "__ftd_external_children_main__",
                    serde_json::to_string_pretty(&main_rt_doc.external_children)
                        .expect("failed to convert document to json")
                        .as_str(),
                )
                .replace(
                    "__main__",
                    format!("{}{}", main_rt_doc.html, config.get_font_style(),).as_str(),
                )
                .as_str()
                .replace("__ftd_js__", ftd::js())
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
    ) -> fpm::Result<()> {
        use tokio::io::AsyncWriteExt;

        let lib = fpm::Library {
            config: config.clone(),
            markdown: None,
            document_id: main.id.clone(),
            translated_data,
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
                .replace(
                    "__ftd_data_main__",
                    serde_json::to_string_pretty(&main_rt_doc.data)
                        .expect("failed to convert document to json")
                        .as_str(),
                )
                .replace(
                    "__ftd_external_children_main__",
                    serde_json::to_string_pretty(&main_rt_doc.external_children)
                        .expect("failed to convert document to json")
                        .as_str(),
                )
                .replace(
                    "__main__",
                    format!("{}{}", main_rt_doc.html, config.get_font_style(),).as_str(),
                )
                .as_str()
                .replace("__ftd_js__", ftd::js())
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
