pub async fn build(config: &fpm::Config) -> fpm::Result<()> {
    tokio::fs::create_dir_all(config.build_dir()).await?;

    let documents = get_documents_with_config(config).await?;

    for (id, doc_config) in documents.iter() {
        match &doc_config.file {
            fpm::File::Ftd(doc) => {
                process_ftd(&doc, &doc_config.config, id, config.root.as_str()).await?
            }
            fpm::File::Static(sa) => process_static(&sa, id, config.root.as_str()).await?,
            fpm::File::Markdown(doc) => process_markdown(&doc, config).await?,
        }
    }

    /*if config.is_translation_package() {
        let timestamp = fpm::get_timestamp_nanosecond();
        write_track_files(&config, timestamp, &documents).await?;
    }*/

    return Ok(());

    async fn get_documents_with_config(
        config: &fpm::Config,
    ) -> fpm::Result<Vec<(String, DocumentWithConfig)>> {
        let documents = if let Some(ref original) = config.package.translation_of.as_ref() {
            let translation_config = config.add_translation_dependencies().await?;
            let translation_snapshots = config.get_translation_snapshots().await?;
            let mut documents: std::collections::BTreeMap<String, DocumentWithConfig> =
                std::collections::BTreeMap::from_iter(
                    config.get_translation_documents().await?.iter().map(|v| {
                        (
                            v.get_id().replace(&format!("{}/", original.name), ""),
                            DocumentWithConfig {
                                file: v.to_owned(),
                                config: translation_config.clone(),
                                is_current_file: false,
                            },
                        )
                    }),
                );
            documents.extend(
                fpm::get_documents(config)
                    .await?
                    .iter()
                    .filter(|x| translation_snapshots.contains_key(x.get_id().as_str()))
                    .map(|v| {
                        (
                            v.get_id(),
                            DocumentWithConfig {
                                file: v.to_owned(),
                                config: config.clone(),
                                is_current_file: true,
                            },
                        )
                    }),
            );
            documents
        } else {
            std::collections::BTreeMap::from_iter(fpm::get_documents(config).await?.iter().map(
                |v| {
                    (
                        v.get_id(),
                        DocumentWithConfig {
                            file: v.to_owned(),
                            config: config.clone(),
                            is_current_file: true,
                        },
                    )
                },
            ))
        };

        // need to sort it in this order: fpm::File::Static, fpm::File::Markdown, fpm::File::Ftd
        let mut document_vec = documents
            .clone()
            .into_iter()
            .filter(|(_, d)| matches!(d.file, fpm::File::Static(_)))
            .collect::<Vec<(String, DocumentWithConfig)>>();
        document_vec.extend(
            documents
                .clone()
                .into_iter()
                .filter(|(_, d)| matches!(d.file, fpm::File::Markdown(_))),
        );
        document_vec.extend(
            documents
                .clone()
                .into_iter()
                .filter(|(_, d)| matches!(d.file, fpm::File::Ftd(_))),
        );
        Ok(document_vec)
    }

    #[derive(Debug, Clone)]
    struct DocumentWithConfig {
        pub file: fpm::File,
        pub config: fpm::Config,
        pub is_current_file: bool,
    }
}

async fn process_markdown(_doc: &fpm::Document, _config: &fpm::Config) -> fpm::Result<()> {
    // if let Ok(c) = tokio::fs::read_to_string("./FPM/markdown.ftd").await {
    //     c
    // } else {
    //     let d = indoc::indoc! {"
    //         -- import: fpm
    //
    //         -- ftd.text:
    //
    //         $fpm.markdown-content
    //         "};
    //     d.to_string()
    // }
    // todo
    Ok(())
}

async fn process_ftd(
    doc: &fpm::Document,
    config: &fpm::Config,
    id: &str,
    base_path: &str,
) -> fpm::Result<()> {
    use tokio::io::AsyncWriteExt;

    if !(doc.depth == 1 && doc.id.eq("index.ftd")) {
        std::fs::create_dir_all(format!("{}/.build/{}", base_path, id.replace(".ftd", "")))?;
    }
    let file_rel_path = if id.eq("index.ftd") {
        "index.html".to_string()
    } else {
        id.replace(".ftd", "/index.html")
    };
    let lib = fpm::Library::default();
    let b = match ftd::p2::Document::from(&id, doc.content.as_str(), &lib) {
        Ok(v) => v,
        Err(e) => {
            return Err(fpm::Error::ConfigurationError {
                message: format!("failed to parse {:?}", &e),
            });
        }
    };

    let new_file_path = format!("{}/.build/{}", base_path, file_rel_path.as_str());
    let mut f = tokio::fs::File::create(new_file_path.as_str()).await?;

    let ftd_doc = b.to_rt("main", &doc.id);

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
                format!(
                    "{}{}",
                    b.html("main", &doc.id).as_str(),
                    config.get_font_style(),
                )
                .as_str(),
            )
            .as_str()
            .replace("__ftd_js__", ftd::js())
            .as_bytes(),
    )
    .await?;
    println!("Generated {}", file_rel_path.as_str(),);
    Ok(())
}

async fn process_static(sa: &fpm::Static, id: &str, base_path: &str) -> fpm::Result<()> {
    if sa.depth != 1 {
        std::fs::create_dir_all(format!(
            "{}/.build/{}",
            base_path,
            id.rsplit_once("/").unwrap_or_else(|| ("", id)).0
        ))?;
    }
    std::fs::copy(
        format!("{}/{}", sa.base_path, sa.id),
        format!("{}/.build/{}", base_path, id),
    )?;
    Ok(())
}

async fn write_track_files(
    config: &fpm::Config,
    timestamp: u128,
    documents: &Vec<(fpm::File, fpm::Config, bool)>,
) -> fpm::Result<()> {
    let file_names = documents
        .iter()
        .filter(|(_, _, b)| b.clone())
        .map(|(v, _, _)| v.get_id())
        .collect::<Vec<String>>();
    tokio::fs::create_dir_all(config.track_dir()).await?;
    futures::future::join_all(
        file_names
            .into_iter()
            .map(|name| {
                let config = config.clone();
                let timestamp = timestamp.clone();
                tokio::spawn(
                    async move { write_track_file(&config, timestamp, name.as_str()).await },
                )
            })
            .collect::<Vec<tokio::task::JoinHandle<fpm::Result<()>>>>(),
    )
    .await;

    return Ok(());

    async fn write_track_file(
        config: &fpm::Config,
        timestamp: u128,
        name: &str,
    ) -> fpm::Result<()> {
        use tokio::io::AsyncWriteExt;
        if let Some((dir, _)) = name.rsplit_once('/') {
            tokio::fs::create_dir_all(config.root.join(".tracks").join(dir)).await?;
        }
        let file_path = fpm::utils::track_path(name, config.root.as_str());
        let string = if file_path.exists() {
            let existing_doc = tokio::fs::read_to_string(&file_path).await?;
            format!(
                "{}\n\n-- fpm.track: {}\nself-timestamp: {}",
                existing_doc, name, timestamp
            )
        } else {
            format!(
                "-- import: fpm\n\n-- fpm.track: {}\nself-timestamp: {}",
                name, timestamp
            )
        };
        let mut f = tokio::fs::File::create(file_path).await?;
        f.write_all(string.as_bytes()).await?;
        Ok(())
    }

    /*for name in file_names {
        if let Some((dir, _)) = name.rsplit_once('/') {
            tokio::fs::create_dir_all(config.root.join(".tracks").join(dir)).await?;
        }
        let file_path = fpm::utils::track_path(name, config.root.as_str());
        let string = if file_path.exists() {
            let existing_doc = tokio::fs::read_to_string(&file_path).await?;
            format!(
                "{}\n\n-- fpm.track: {}\nself-timestamp: {}",
                existing_doc, name, timestamp
            )
        } else {
            format!(
                "-- import: fpm\n\n-- fpm.track: {}\nself-timestamp: {}",
                name, timestamp
            )
        };
        let mut f = tokio::fs::File::create(file_path).await?;
        f.write_all(string.as_bytes()).await?;
    }*/
}
