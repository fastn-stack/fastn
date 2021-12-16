pub async fn build(config: &fpm::Config) -> fpm::Result<()> {
    tokio::fs::create_dir_all(config.build_dir()).await?;

    match (
        config.package.translation_of.as_ref(),
        config.package.translations.is_empty(),
    ) {
        (Some(_), true) => {
            // No package can be both a translation of something and has its own
            // translations, when building `config` we ensured this was rejected
            unreachable!()
        }
        (Some(original), false) => build_with_original(config, original),
        (None, true) => build_simple(config),
        (None, false) => build_with_translations(config),
    }
}

async fn build_simple(_config: &fpm::Config) -> fpm::Result<()> {
    todo!()
}

async fn build_with_translations(_config: &fpm::Config) -> fpm::Result<()> {
    todo!("build does not yet support translations, only translation-of")
}

async fn build_with_original(config: &fpm::Config) -> fpm::Result<()> {
    let translation_snapshots = if let Some(ref original) = config.package.translation_of.as_ref() {
        let translation_config = config.read_translation().await?;
        let documents = config
            .get_translation_documents()
            .await?
            .iter()
            .map(|v| {
                (
                    v.get_id()
                        .replace(&format!(".packages/{}/", original.name), ""),
                    v.to_owned(),
                )
            })
            .collect::<Vec<(String, fpm::File)>>();

        std::env::set_current_dir(
            &config
                .original_directory
                .join(".packages")
                .join(original.name.as_str()),
        )?;
        process_files(
            &translation_config,
            &config.original_directory,
            documents,
            true,
        )
        .await?;

        // is it needed??
        fpm::utils::copy_dir_all(".packages", config.original_directory.join(".packages")).await?;
        std::env::set_current_dir(&config.original_directory)?;

        Some(fpm::snapshot::get_latest_snapshots(&config.original_path()?).await?)
    } else {
        None
    };

    let documents = {
        let mut documents = fpm::get_root_documents(config)
            .await?
            .into_iter()
            .map(|v| (v.get_id(), v))
            .collect::<Vec<(String, fpm::File)>>();
        if let Some(translation_snapshots) = translation_snapshots {
            documents = documents
                .into_iter()
                .filter(|(_, x)| translation_snapshots.contains_key(x.get_id().as_str()))
                .collect::<Vec<(String, fpm::File)>>();
        }
        documents
    };
    process_files(
        config,
        &config.original_directory,
        documents,
        !config.is_translation_package(),
    )
    .await?;

    Ok(())
}

async fn process_files(
    config: &fpm::Config,
    base_path: &camino::Utf8PathBuf,
    documents: Vec<(String, fpm::File)>,
    print_log: bool,
) -> fpm::Result<()> {
    // TODO: why is only ftd files being logged? Log all files or none.
    for (id, file) in documents.iter() {
        match file {
            fpm::File::Ftd(doc) => {
                process_ftd(doc, config, id, base_path.as_str(), print_log).await?
            }
            fpm::File::Static(sa) => process_static(sa, id, base_path.as_str()).await?,
            fpm::File::Markdown(doc) => process_markdown(doc, config).await?,
        }
    }
    Ok(())
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
    print_log: bool,
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
    let b = match ftd::p2::Document::from(id, doc.content.as_str(), &lib) {
        Ok(v) => v,
        Err(e) => {
            return Err(fpm::Error::PackageError {
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
    if print_log {
        println!("Generated {}", file_rel_path.as_str(),);
    }
    Ok(())
}

async fn process_static(sa: &fpm::Static, id: &str, base_path: &str) -> fpm::Result<()> {
    if sa.depth != 1 {
        std::fs::create_dir_all(format!(
            "{}/.build/{}",
            base_path,
            id.rsplit_once("/").unwrap_or(("", id)).0
        ))?;
    }
    std::fs::copy(
        format!("{}/{}", sa.base_path, sa.id),
        format!("{}/.build/{}", base_path, id),
    )?;
    Ok(())
}
