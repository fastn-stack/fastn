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

    process_files(config, &config.original_directory, &documents).await
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
    let mut documents = std::collections::BTreeMap::from_iter(
        fpm::paths_to_files(files, original_path.as_path())
            .await?
            .into_iter()
            .map(|v| (v.get_id(), v)),
    );

    // Fetch all files from the current package
    // ignore all those documents/files which is not in original package
    // Overwrite the files having same name/id
    documents.extend(
        fpm::get_documents(config)
            .await?
            .into_iter()
            .filter(|x| original_snapshots.contains_key(x.get_id().as_str()))
            .map(|v| (v.get_id(), v)),
    );

    // Process all the files collected from original and root package
    process_files(config, &config.root, &documents).await
}

async fn process_files(
    config: &fpm::Config,
    base_path: &camino::Utf8PathBuf,
    documents: &std::collections::BTreeMap<String, fpm::File>,
) -> fpm::Result<()> {
    for (id, file) in documents.iter() {
        match file {
            fpm::File::Ftd(doc) => process_ftd(doc, config, id, base_path.as_str()).await?,
            fpm::File::Static(sa) => process_static(sa, id, base_path.as_str()).await?,
            fpm::File::Markdown(doc) => process_markdown(doc, config).await?,
        }
        println!("Processed {}", id);
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
    let lib = fpm::Library {
        config: config.clone(),
    };
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
