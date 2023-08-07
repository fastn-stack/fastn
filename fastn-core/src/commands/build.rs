// #[tracing::instrument(skip(config))]
pub async fn build(
    config: &mut fastn_core::Config,
    only_id: Option<&str>,
    base_url: &str,
    ignore_failed: bool,
    test: bool,
) -> fastn_core::Result<()> {
    tokio::fs::create_dir_all(config.build_dir()).await?;

    // Default css and js
    default_build_files(config.root.join(".build"), &config.ftd_edition).await?;

    {
        let documents = get_documents_for_current_package(config).await?;

        match only_id {
            Some(id) => {
                return handle_only_id(id, config, base_url, ignore_failed, test, documents).await
            }
            None => {
                incremental_build(config, documents, base_url, ignore_failed, test).await?;
            }
        }
    }

    // All redirect html files under .build
    if let Some(ref r) = config.package.redirects {
        for (redirect_from, redirect_to) in r.iter() {
            println!(
                "Processing redirect {}/{} -> {}... ",
                config.package.name.as_str(),
                redirect_from.trim_matches('/'),
                redirect_to
            );

            let content = fastn_core::utils::redirect_page_html(redirect_to.as_str());
            let save_file = if redirect_from.as_str().ends_with(".ftd") {
                redirect_from
                    .replace("index.ftd", "index.html")
                    .replace(".ftd", "/index.html")
            } else {
                format!("{}/index.html", redirect_from.trim_matches('/'))
            };

            let save_path = config.root.join(".build").join(save_file.as_str());
            fastn_core::utils::update(save_path, content.as_bytes())
                .await
                .ok();
        }
    }

    config.download_fonts().await
}

mod build_dir {
    pub(crate) fn get_build_content() -> std::io::Result<std::collections::BTreeMap<String, String>>
    {
        let mut b = std::collections::BTreeMap::new();

        for f in find_all_files_recursively(".build") {
            b.insert(
                f.to_string_lossy().to_string().replacen(".build/", "", 1),
                fastn_core::utils::generate_hash(std::fs::read(&f)?),
            );
        }

        Ok(dbg!(b))
    }

    fn find_all_files_recursively(
        dir: impl AsRef<std::path::Path> + std::fmt::Debug,
    ) -> Vec<std::path::PathBuf> {
        let mut files = vec![];
        for entry in std::fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                files.extend(find_all_files_recursively(&path));
            } else {
                files.push(path)
            }
        }
        files
    }
}

mod cache {
    const FILE_NAME: &str = "fastn.cache";

    pub(crate) fn get() -> std::io::Result<Cache> {
        let mut v = match fastn_core::utils::get_cached(FILE_NAME) {
            Some(v) => {
                tracing::debug!("cached hit");
                v
            }
            None => {
                tracing::debug!("cached miss");
                Cache {
                    build_content: std::collections::BTreeMap::new(),
                    documents: std::collections::BTreeMap::new(),
                    assets: std::collections::BTreeMap::new(),
                }
            }
        };
        v.build_content = super::build_dir::get_build_content()?;
        Ok(v)
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug)]
    pub(crate) struct Cache {
        // fastn_version: String, // TODO
        #[serde(skip)]
        pub(crate) build_content: std::collections::BTreeMap<String, String>,
        pub(crate) documents: std::collections::BTreeMap<String, Document>,
        pub(crate) assets: std::collections::BTreeMap<String, File>,
    }

    impl Cache {
        pub(crate) fn cache_it(&self) -> fastn_core::Result<()> {
            fastn_core::utils::cache_it(FILE_NAME, self)?;
            Ok(())
        }
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug)]
    pub(crate) struct File {
        pub(crate) path: String,
        pub(crate) checksum: String,
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug)]
    pub(crate) struct Document {
        pub(crate) file: File,
        pub(crate) html_checksum: String,
        pub(crate) dependencies: Vec<String>,
    }
}

#[tracing::instrument(skip(config, documents))]
async fn incremental_build(
    config: &mut fastn_core::Config,
    documents: std::collections::BTreeMap<String, fastn_core::File>,
    base_url: &str,
    ignore_failed: bool,
    test: bool,
) -> fastn_core::Result<()> {
    // https://fastn.com/rfc/incremental-build/

    let mut c = dbg!(cache::get()?);

    for document in documents.values() {
        handle_file(
            document,
            config,
            base_url,
            ignore_failed,
            test,
            true,
            Some(&mut c),
        )
        .await?;
    }

    // TODO: Handle deleted files (files present in cache/.build but not in documents)

    dbg!(c).cache_it()?;

    Ok(())
}

#[tracing::instrument(skip(config, documents))]
async fn handle_only_id(
    id: &str,
    config: &mut fastn_core::Config,
    base_url: &str,
    ignore_failed: bool,
    test: bool,
    documents: std::collections::BTreeMap<String, fastn_core::File>,
) -> fastn_core::Result<()> {
    for doc in documents.values() {
        if doc.get_id().eq(id) || doc.get_id_with_package().eq(id) {
            return handle_file(doc, config, base_url, ignore_failed, test, false, None).await;
        }
    }

    Err(fastn_core::Error::GenericError(format!(
        "Document {} not found in package {}",
        id,
        config.package.name.as_str()
    )))
}

async fn handle_file(
    document: &fastn_core::File,
    config: &mut fastn_core::Config,
    base_url: &str,
    ignore_failed: bool,
    test: bool,
    build_static_files: bool,
    cache: Option<&mut cache::Cache>,
) -> fastn_core::Result<()> {
    let start = std::time::Instant::now();
    print!("Processing {} ... ", document.get_id_with_package());

    if let Ok(()) = handle_file_(
        document,
        config,
        base_url,
        ignore_failed,
        test,
        build_static_files,
        cache,
    )
    .await
    {
        fastn_core::utils::print_end(
            format!(
                "Processed {}/{}",
                config.package.name.as_str(),
                document.get_id()
            )
            .as_str(),
            start,
        );
    }

    Ok(())
}

fn is_cached(
    cache: &Option<&mut cache::Cache>,
    doc: &fastn_core::Document,
    file_path: &str,
) -> bool {
    let cache = match cache {
        Some(c) => c,
        None => {
            println!("cache miss: no have cache");
            return false;
        }
    };

    let cached_doc = match cache.documents.get(doc.id.as_str()) {
        Some(cached_doc) => cached_doc,
        None => {
            println!("cache miss: no cache entry for {}", doc.id.as_str());
            return false;
        }
    };

    // if it exists, check if the checksums match
    // if they do, return
    dbg!(cached_doc);
    let doc_hash = match cache.build_content.get(file_path) {
        Some(doc_hash) => doc_hash,
        None => {
            println!("cache miss: document not present in .build: {}", file_path);
            return false;
        }
    };

    dbg!(doc_hash);

    if doc_hash != &cached_doc.html_checksum {
        println!("cache miss: html file checksums don't match");
    }

    // if it exists, check if the checksums match
    println!("cache hit");
    true
}

#[tracing::instrument(skip(document, config, cache))]
async fn handle_file_(
    document: &fastn_core::File,
    config: &mut fastn_core::Config,
    base_url: &str,
    ignore_failed: bool,
    test: bool,
    build_static_files: bool,
    cache: Option<&mut cache::Cache>,
) -> fastn_core::Result<()> {
    config.current_document = Some(document.get_id().to_string());
    config.dependencies_during_render = vec![];

    match document {
        fastn_core::File::Ftd(doc) => {
            let file_path = if doc.id.eq("404.ftd") {
                "404.html".to_string()
            } else if doc.id.ends_with("index.ftd") {
                fastn_core::utils::replace_last_n(doc.id.as_str(), 1, "index.ftd", "index.html")
            } else {
                fastn_core::utils::replace_last_n(doc.id.as_str(), 1, ".ftd", "/index.html")
            };

            if is_cached(&cache, doc, file_path.as_str()) {
                return Ok(());
            }

            fastn_core::utils::copy(
                config.root.join(doc.id.as_str()),
                config.root.join(".build").join(doc.id.as_str()),
            )
            .await
            .ok();

            if doc.id.eq("FASTN.ftd") {
                return Ok(());
            }

            let resp = fastn_core::package::package_doc::process_ftd(
                config,
                doc,
                base_url,
                build_static_files,
                test,
                file_path.as_str(),
            )
            .await;
            match (resp, ignore_failed) {
                (Ok(r), _) => {
                    if let Some(cache) = cache {
                        cache.documents.insert(
                            doc.id.to_string(),
                            dbg!(cache::Document {
                                file: cache::File {
                                    path: doc.id.to_string(),
                                    checksum: fastn_core::utils::generate_hash(
                                        doc.content.as_str(),
                                    ),
                                },
                                html_checksum: r.checksum(),
                                dependencies: config.dependencies_during_render.clone(),
                            }),
                        );
                    }
                }
                (_, true) => {
                    print!("Failed ");
                    return Ok(());
                }
                (Err(e), _) => {
                    return Err(e);
                }
            }
        }
        fastn_core::File::Static(sa) => process_static(sa, &config.root, &config.package).await?,
        fastn_core::File::Markdown(_doc) => {
            // TODO: bring this feature back
            print!("Skipped ");
            return Ok(());
        }
        fastn_core::File::Image(main_doc) => {
            process_static(main_doc, &config.root, &config.package).await?;
        }
        fastn_core::File::Code(doc) => {
            process_static(
                &fastn_core::Static {
                    package_name: config.package.name.to_string(),
                    id: doc.id.to_string(),
                    content: doc.content.clone().into_bytes(),
                    base_path: camino::Utf8PathBuf::from(doc.parent_path.as_str()),
                },
                &config.root,
                &config.package,
            )
            .await?;
        }
    }

    Ok(())
}

#[tracing::instrument]
pub async fn default_build_files(
    base_path: camino::Utf8PathBuf,
    ftd_edition: &fastn_core::FTDEdition,
) -> fastn_core::Result<()> {
    if ftd_edition.is_2023() {
        let default_ftd_js_content = ftd::js::all_js_without_test();
        let hashed_ftd_js_name = fastn_core::utils::hashed_default_ftd_js();
        let save_default_ftd_js = base_path.join(hashed_ftd_js_name);
        fastn_core::utils::update(save_default_ftd_js, default_ftd_js_content.as_bytes())
            .await
            .ok();

        let markdown_js_content = ftd::markdown_js();
        let hashed_markdown_js_name = fastn_core::utils::hashed_markdown_js();
        let save_markdown_js = base_path.join(hashed_markdown_js_name);
        fastn_core::utils::update(save_markdown_js, markdown_js_content.as_bytes())
            .await
            .ok();

        let theme_css = ftd::theme_css();
        let hashed_code_themes = fastn_core::utils::hashed_code_theme_css();
        for (theme, file_name) in hashed_code_themes {
            let save_markdown_js = base_path.join(file_name);
            let theme_content = theme_css.get(theme).unwrap();
            fastn_core::utils::update(save_markdown_js, theme_content.as_bytes())
                .await
                .ok();
        }

        let prism_js_content = ftd::prism_js();
        let hashed_prism_js_name = fastn_core::utils::hashed_prism_js();
        let save_prism_js = base_path.join(hashed_prism_js_name);
        fastn_core::utils::update(save_prism_js, prism_js_content.as_bytes())
            .await
            .ok();

        let prism_css_content = ftd::prism_css();
        let hashed_prism_css_name = fastn_core::utils::hashed_prism_css();
        let save_prism_css = base_path.join(hashed_prism_css_name);
        fastn_core::utils::update(save_prism_css, prism_css_content.as_bytes())
            .await
            .ok();
    } else {
        let default_css_content = ftd::css();
        let hashed_css_name = fastn_core::utils::hashed_default_css_name();
        let save_default_css = base_path.join(hashed_css_name);
        fastn_core::utils::update(save_default_css, default_css_content.as_bytes())
            .await
            .ok();

        let default_js_content = format!("{}\n\n{}", ftd::build_js(), fastn_core::fastn_2022_js());
        let hashed_js_name = fastn_core::utils::hashed_default_js_name();
        let save_default_js = base_path.join(hashed_js_name);
        fastn_core::utils::update(save_default_js, default_js_content.as_bytes())
            .await
            .ok();
    }

    Ok(())
}

#[tracing::instrument(skip(config))]
async fn get_documents_for_current_package(
    config: &mut fastn_core::Config,
) -> fastn_core::Result<std::collections::BTreeMap<String, fastn_core::File>> {
    let mut documents = std::collections::BTreeMap::from_iter(
        config
            .get_files(&config.package)
            .await?
            .into_iter()
            .map(|v| (v.get_id().to_string(), v)),
    );

    if let Some(ref sitemap) = config.package.sitemap {
        let new_config = config.clone();
        let get_all_locations = sitemap.get_all_locations();
        let mut files: std::collections::HashMap<String, fastn_core::File> = Default::default();
        for (doc_path, _, url) in get_all_locations {
            let file = {
                let package_name = if let Some(ref url) = url {
                    new_config.find_package_by_id(url).await?.1.name
                } else {
                    config.package.name.to_string()
                };
                let mut file =
                    fastn_core::get_file(package_name, doc_path, config.root.as_path()).await?;
                if let Some(ref url) = url {
                    let url = url.replace("/index.html", "");
                    let extension = if matches!(file, fastn_core::File::Markdown(_)) {
                        "index.md".to_string()
                    } else {
                        "index.ftd".to_string()
                    };

                    file.set_id(format!("{}/{}", url.trim_matches('/'), extension).as_str());
                }
                file
            };
            files.insert(file.get_id().to_string(), file);
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
    sa: &fastn_core::Static,
    base_path: &camino::Utf8Path,
    package: &fastn_core::Package,
) -> fastn_core::Result<()> {
    copy_to_build(sa, base_path, package)?;
    if let Some(original_package) = package.translation_of.as_ref() {
        copy_to_build(sa, base_path, original_package)?;
    }
    return Ok(());

    fn copy_to_build(
        sa: &fastn_core::Static,
        base_path: &camino::Utf8Path,
        package: &fastn_core::Package,
    ) -> fastn_core::Result<()> {
        let build_path = base_path
            .join(".build")
            .join("-")
            .join(package.name.as_str());

        std::fs::create_dir_all(&build_path)?;
        std::fs::write(build_path.join(sa.id.as_str()), &sa.content)?;
        Ok(())
    }
}
