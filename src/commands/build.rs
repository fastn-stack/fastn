pub async fn build(config: &fpm::Config) -> fpm::Result<()> {
    build_(config, true).await
}

async fn build_(config: &fpm::Config, is_rebuilt_needed: bool) -> fpm::Result<()> {
    use fpm::utils::HasElements;

    if config.build_dir().exists() && !is_rebuilt_needed {
        return Ok(());
    }

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

    process_files(config, &config.original_directory, &documents, true).await
}

async fn build_with_translations(_config: &fpm::Config) -> fpm::Result<()> {
    todo!("build does not yet support translations, only translation-of")
}

#[async_recursion::async_recursion(?Send)]
async fn build_with_original(config: &fpm::Config, original: &fpm::Package) -> fpm::Result<()> {
    // This is the translation package then first build the original package
    {
        // copy original .build to root .build
        // set current dir to original package and build it
        // This would create .build directory inside original package
        // This .build directory would be copied to root .build directory everytime `fpm build` is called
        std::env::set_current_dir(&config.root.join(".packages").join(original.name.as_str()))?;
        let original_config = fpm::Config::read().await?;

        // In this case, if original package is already built then we don't need to built it again
        // as the files in original package is not going to change.
        build_(&original_config, false).await?;

        // copy original .package folder to root .package folder
        // is this needed?
        {
            let src = camino::Utf8PathBuf::from(".packages");
            if src.exists() {
                fpm::copy_dir_all(src, config.root.join(".packages")).await?;
            }
        }
        // set current dir to root again
        std::env::set_current_dir(&config.root)?;
    }

    // Built the root package
    let src = config
        .root
        .join(".packages")
        .join(original.name.as_str())
        .join(".build");
    let dst = config.build_dir();
    fpm::copy_dir_all(src, dst).await?;

    // overwrite all the files in root .build which is present in root
    let original_snapshots = fpm::snapshot::get_latest_snapshots(&config.original_path()?).await?;

    let documents = std::collections::BTreeMap::from_iter(
        fpm::get_documents(config)
            .await?
            .into_iter()
            .filter(|x| original_snapshots.contains_key(x.get_id().as_str()))
            .map(|v| (v.get_id(), v)),
    );

    process_files(config, &config.root, &documents, false).await
}

async fn process_files(
    config: &fpm::Config,
    base_path: &camino::Utf8PathBuf,
    documents: &std::collections::BTreeMap<String, fpm::File>,
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
