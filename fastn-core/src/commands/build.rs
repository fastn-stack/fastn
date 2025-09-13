// #[tracing::instrument(skip(config))]
#[allow(clippy::too_many_arguments)]
pub async fn build(
    config: &fastn_core::Config,
    only_id: Option<&str>,
    base_url: &str,
    ignore_failed: bool,
    test: bool,
    check_build: bool,
    zip_url: Option<&str>,
    preview_session_id: &Option<String>,
) -> fastn_core::Result<()> {
    let build_dir = config.ds.root().join(".build");
    // Default css and js
    default_build_files(
        build_dir.clone(),
        &config.ftd_edition,
        &config.package.name,
        &config.ds,
    )
    .await?;

    {
        let documents = get_documents_for_current_package(config).await?;
        let zip_url = zip_url.map_or_else(|| config.package.zip.clone(), |z| Some(z.to_string()));

        fastn_core::manifest::write_manifest_file(config, &build_dir, zip_url, &None).await?;

        match only_id {
            Some(id) => {
                return handle_only_id(
                    id,
                    config,
                    base_url,
                    ignore_failed,
                    test,
                    documents,
                    preview_session_id,
                )
                .await;
            }
            None => {
                incremental_build(
                    config,
                    &documents,
                    base_url,
                    ignore_failed,
                    test,
                    preview_session_id,
                )
                .await?;
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

            let save_path = config.ds.root().join(".build").join(save_file.as_str());
            fastn_core::utils::update(&save_path, content.as_bytes(), &config.ds)
                .await
                .ok();
        }
    }

    if !test {
        config.download_fonts(&None).await?;
    }

    if check_build {
        return fastn_core::post_build_check(config).await;
    }

    Ok(())
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

        Ok(b)
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
    use super::is_virtual_dep;

    const FILE_NAME: &str = "fastn.cache";

    pub(crate) fn get() -> std::io::Result<(bool, Cache)> {
        let (cache_hit, mut v) = match fastn_core::utils::get_cached(FILE_NAME) {
            Some(v) => {
                tracing::debug!("cached hit");
                (true, v)
            }
            None => {
                tracing::debug!("cached miss");
                (
                    false,
                    Cache {
                        build_content: std::collections::BTreeMap::new(),
                        ftd_cache: std::collections::BTreeMap::new(),
                        documents: std::collections::BTreeMap::new(),
                        file_checksum: std::collections::BTreeMap::new(),
                    },
                )
            }
        };
        v.build_content = super::build_dir::get_build_content()?;
        Ok((cache_hit, v))
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug)]
    pub(crate) struct Cache {
        // fastn_version: String, // TODO
        #[serde(skip)]
        pub(crate) build_content: std::collections::BTreeMap<String, String>,
        #[serde(skip)]
        pub(crate) ftd_cache: std::collections::BTreeMap<String, Option<String>>,
        pub(crate) documents: std::collections::BTreeMap<String, Document>,
        pub(crate) file_checksum: std::collections::BTreeMap<String, String>,
    }

    impl Cache {
        pub(crate) fn cache_it(&self) -> fastn_core::Result<()> {
            fastn_core::utils::cache_it(FILE_NAME, self)?;
            Ok(())
        }
        pub(crate) fn get_file_hash(&mut self, path: &str) -> fastn_core::Result<String> {
            if is_virtual_dep(path) {
                // these are virtual file, they don't exist on disk, and hash only changes when
                // fastn source changes
                return Ok("hello".to_string());
            }
            match self.ftd_cache.get(path) {
                Some(Some(v)) => Ok(v.to_owned()),
                Some(None) => Err(fastn_core::Error::GenericError(path.to_string())),
                None => {
                    let hash = match fastn_core::utils::get_ftd_hash(path) {
                        Ok(v) => v,
                        Err(e) => {
                            self.ftd_cache.insert(path.to_string(), None);
                            return Err(e);
                        }
                    };
                    self.ftd_cache.insert(path.to_string(), Some(hash.clone()));
                    Ok(hash)
                }
            }
        }
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
    pub(crate) struct File {
        pub(crate) path: String,
        pub(crate) checksum: String,
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
    pub(crate) struct Document {
        pub(crate) html_checksum: String,
        pub(crate) dependencies: Vec<String>,
    }
}

fn get_dependency_name_without_package_name(package_name: &str, dependency_name: &str) -> String {
    if let Some(remaining) = dependency_name.strip_prefix(&format!("{package_name}/")) {
        remaining.to_string()
    } else {
        dependency_name.to_string()
    }
    .trim_end_matches('/')
    .to_string()
}

fn is_virtual_dep(path: &str) -> bool {
    let path = std::path::Path::new(path);

    path.starts_with("$fastn$/")
        || path.ends_with("/-/fonts.ftd")
        || path.ends_with("/-/assets.ftd")
}

#[allow(clippy::too_many_arguments)]
async fn handle_dependency_file(
    config: &fastn_core::Config,
    cache: &mut cache::Cache,
    documents: &std::collections::BTreeMap<String, fastn_core::File>,
    base_url: &str,
    ignore_failed: bool,
    test: bool,
    name_without_package_name: String,
    processed: &mut Vec<String>,
    preview_session_id: &Option<String>,
) -> fastn_core::Result<()> {
    for document in documents.values() {
        if remove_extension(document.get_id()).eq(name_without_package_name.as_str())
            || remove_extension(&document.get_id_with_package())
                .eq(name_without_package_name.as_str())
        {
            let id = document.get_id().to_string();
            if processed.contains(&id) {
                continue;
            }
            handle_file(
                document,
                config,
                base_url,
                ignore_failed,
                test,
                true,
                Some(cache),
                preview_session_id,
            )
            .await?;
            processed.push(id);
        }
    }

    Ok(())
}

// removes deleted documents from cache and build folder
async fn remove_deleted_documents(
    config: &fastn_core::Config,
    c: &mut cache::Cache,
    documents: &std::collections::BTreeMap<String, fastn_core::File>,
) -> fastn_core::Result<()> {
    use itertools::Itertools;

    let removed_documents = c
        .documents
        .keys()
        .filter(|cached_document_id| {
            for document in documents.values() {
                if remove_extension(document.get_id()).eq(cached_document_id.as_str())
                    || remove_extension(&document.get_id_with_package())
                        .eq(cached_document_id.as_str())
                {
                    return false;
                }
            }

            true
        })
        .map(|id| id.to_string())
        .collect_vec();

    for removed_doc_id in &removed_documents {
        let folder_path = config.build_dir().join(removed_doc_id);
        let folder_parent = folder_path.parent();
        let file_path = &folder_path.with_extension("ftd");

        config.ds.remove(file_path).await?;
        config.ds.remove(&folder_path).await?;

        // If the parent folder of the file's output folder is also empty, delete it as well.
        if let Some(folder_parent) = folder_parent
            && config
                .ds
                .get_all_file_path(&folder_parent, &[])
                .await
                .is_empty()
        {
            config.ds.remove(&folder_parent).await?;
        }

        c.documents.remove(removed_doc_id);
    }

    Ok(())
}

#[tracing::instrument(skip(config, documents))]
async fn incremental_build(
    config: &fastn_core::Config,
    documents: &std::collections::BTreeMap<String, fastn_core::File>,
    base_url: &str,
    ignore_failed: bool,
    test: bool,
    preview_session_id: &Option<String>,
) -> fastn_core::Result<()> {
    // https://fastn.com/rfc/incremental-build/
    use itertools::Itertools;

    let (cache_hit, mut c) = cache::get()?;

    let mut processed: Vec<String> = vec![];

    if cache_hit {
        let mut unresolved_dependencies = vec![];
        let mut resolved_dependencies: Vec<String> = vec![];
        let mut resolving_dependencies: Vec<String> = vec![];

        // Sort documents by ID for deterministic processing order
        let mut sorted_documents: Vec<_> = documents.values().collect();
        sorted_documents.sort_by_key(|doc| doc.get_id());
        
        for file in sorted_documents {
            // copy static files
            if file.is_static() {
                handle_file(
                    file,
                    config,
                    base_url,
                    ignore_failed,
                    test,
                    true,
                    Some(&mut c),
                    preview_session_id,
                )
                .await?;
                continue;
            }

            unresolved_dependencies.push(remove_extension(file.get_id()));
        }

        // Sort dependencies for deterministic processing order (stable test output)
        unresolved_dependencies.sort();

        while let Some(unresolved_dependency) = unresolved_dependencies.pop() {
            // println!("Current UR: {}", unresolved_dependency.as_str());
            if let Some(doc) = c.documents.get(unresolved_dependency.as_str()) {
                // println!(
                //     "[INCREMENTAL BUILD][CACHE FOUND] Processing: {}",
                //     &unresolved_dependency
                // );

                let mut own_resolved_dependencies: Vec<String> = vec![];

                let dependencies: Vec<String> = doc
                    .dependencies
                    .iter()
                    .map(|dep| get_dependency_name_without_package_name(&config.package.name, dep))
                    .collect_vec();

                for dep in &dependencies {
                    if resolved_dependencies.contains(dep) || dep.eq(&unresolved_dependency) {
                        own_resolved_dependencies.push(dep.to_string());
                        continue;
                    }
                    unresolved_dependencies.push(dep.to_string());
                }

                // Sort after adding new dependencies for deterministic processing
                unresolved_dependencies.sort();

                // println!(
                //     "[INCREMENTAL] [R]: {} [RV]: {} [UR]: {} [ORD]: {}",
                //     &resolved_dependencies.len(),
                //     &resolving_dependencies.len(),
                //     &unresolved_dependencies.len(),
                //     own_resolved_dependencies.len(),
                // );

                if own_resolved_dependencies.eq(&dependencies) {
                    handle_dependency_file(
                        config,
                        &mut c,
                        documents,
                        base_url,
                        ignore_failed,
                        test,
                        unresolved_dependency.to_string(),
                        &mut processed,
                        preview_session_id,
                    )
                    .await?;

                    resolved_dependencies.push(unresolved_dependency.to_string());
                    if unresolved_dependencies.is_empty()
                        && let Some(resolving_dependency) = resolving_dependencies.pop()
                    {
                        if resolving_dependency.eq(&unresolved_dependency.as_str()) {
                            // println!("[INCREMENTAL][CIRCULAR]: {}", &unresolved_dependency);
                            continue;
                        }
                        unresolved_dependencies.push(resolving_dependency);
                    }
                } else {
                    // println!("Adding to RD: {}", unresolved_dependency.as_str());
                    resolving_dependencies.push(unresolved_dependency.to_string());
                }
            } else {
                if is_virtual_dep(&unresolved_dependency) {
                    resolved_dependencies.push(unresolved_dependency.clone());
                } else {
                    // println!("Not found in cache UR: {}", unresolved_dependency.as_str());

                    handle_dependency_file(
                        config,
                        &mut c,
                        documents,
                        base_url,
                        ignore_failed,
                        test,
                        unresolved_dependency.to_string(),
                        &mut processed,
                        preview_session_id,
                    )
                    .await?;

                    resolved_dependencies.push(unresolved_dependency.clone());
                }
                if unresolved_dependencies.is_empty()
                    && let Some(resolving_dependency) = resolving_dependencies.pop()
                {
                    if resolving_dependency.eq(&unresolved_dependency.as_str()) {
                        // println!("[INCREMENTAL][CIRCULAR]: {}", &unresolved_dependency);
                        continue;
                    }
                    unresolved_dependencies.push(resolving_dependency);
                }
            }
        }

        remove_deleted_documents(config, &mut c, documents).await?;
    } else {
        for document in documents.values() {
            let id = document.get_id().to_string();
            if processed.contains(&id) {
                continue;
            }
            handle_file(
                document,
                config,
                base_url,
                ignore_failed,
                test,
                true,
                Some(&mut c),
                preview_session_id,
            )
            .await?;
            processed.push(id);
        }
    }

    c.cache_it()?;

    Ok(())
}

#[tracing::instrument(skip(config, documents))]
async fn handle_only_id(
    id: &str,
    config: &fastn_core::Config,
    base_url: &str,
    ignore_failed: bool,
    test: bool,
    documents: std::collections::BTreeMap<String, fastn_core::File>,
    preview_session_id: &Option<String>,
) -> fastn_core::Result<()> {
    for doc in documents.values() {
        if doc.get_id().eq(id) || doc.get_id_with_package().eq(id) {
            return handle_file(
                doc,
                config,
                base_url,
                ignore_failed,
                test,
                false,
                None,
                preview_session_id,
            )
            .await;
        }
    }

    Err(fastn_core::Error::GenericError(format!(
        "Document {} not found in package {}",
        id,
        config.package.name.as_str()
    )))
}

#[allow(clippy::too_many_arguments)]
async fn handle_file(
    document: &fastn_core::File,
    config: &fastn_core::Config,
    base_url: &str,
    ignore_failed: bool,
    test: bool,
    build_static_files: bool,
    cache: Option<&mut cache::Cache>,
    preview_session_id: &Option<String>,
) -> fastn_core::Result<()> {
    let start = std::time::Instant::now();
    print!("Processing {} ... ", document.get_id_with_package());
    let package_name = config.package.name.to_string();
    let process_status = handle_file_(
        document,
        config,
        base_url,
        ignore_failed,
        test,
        build_static_files,
        cache,
        preview_session_id,
    )
    .await;
    if process_status.is_ok() {
        fastn_core::utils::print_end(
            format!("Processed {}/{}", package_name.as_str(), document.get_id()).as_str(),
            start,
        );
    }
    if process_status.is_err() {
        fastn_core::utils::print_error(
            format!("Failed {}/{}", package_name.as_str(), document.get_id()).as_str(),
            start,
        );
        return process_status;
    }
    Ok(())
}

fn is_cached<'a>(
    cache: Option<&'a mut cache::Cache>,
    doc: &fastn_core::Document,
    file_path: &str,
) -> (Option<&'a mut cache::Cache>, bool) {
    let cache: &mut cache::Cache = match cache {
        Some(c) => c,
        None => {
            // println!("cache miss: no have cache");
            return (cache, false);
        }
    };

    let id = remove_extension(doc.id.as_str());

    let cached_doc: cache::Document = match cache.documents.get(id.as_str()).cloned() {
        Some(cached_doc) => cached_doc,
        None => {
            // println!("cache miss: no cache entry for {}", id.as_str());
            return (Some(cache), false);
        }
    };

    // if it exists, check if the checksums match
    // if they do, return
    // dbg!(&cached_doc);
    let doc_hash = match cache.build_content.get(file_path) {
        Some(doc_hash) => doc_hash,
        None => {
            // println!("cache miss: document not present in .build: {}", file_path);
            return (Some(cache), false);
        }
    };

    // dbg!(doc_hash);

    if doc_hash != &cached_doc.html_checksum {
        // println!("cache miss: html file checksums don't match");
        return (Some(cache), false);
    }

    let file_checksum = match cache.file_checksum.get(id.as_str()).cloned() {
        Some(file_checksum) => file_checksum,
        None => {
            // println!("cache miss: no cache entry for {}", id.as_str());
            return (Some(cache), false);
        }
    };

    if file_checksum != fastn_core::utils::generate_hash(doc.content.as_str()) {
        // println!("cache miss: ftd file checksums don't match");
        return (Some(cache), false);
    }

    for dep in &cached_doc.dependencies {
        let file_checksum = match cache.file_checksum.get(dep) {
            None => {
                // println!("cache miss: file {} not present in cache", dep);
                return (Some(cache), false);
            }
            Some(file_checksum) => file_checksum.clone(),
        };

        let current_hash = match cache.get_file_hash(dep.as_str()) {
            Ok(hash) => hash,
            Err(_) => {
                // println!("cache miss: dependency {} not present current folder", dep);
                return (Some(cache), false);
            }
        };

        if file_checksum != current_hash {
            // println!("cache miss: dependency {} checksums don't match", dep);
            return (Some(cache), false);
        }
    }

    // println!("cache hit");
    (Some(cache), true)
}

fn remove_extension(id: &str) -> String {
    if id.ends_with("/index.ftd") {
        fastn_core::utils::replace_last_n(id, 1, "/index.ftd", "")
    } else {
        fastn_core::utils::replace_last_n(id, 1, ".ftd", "")
    }
}

#[tracing::instrument(skip(document, config, cache))]
#[allow(clippy::too_many_arguments)]
async fn handle_file_(
    document: &fastn_core::File,
    config: &fastn_core::Config,
    base_url: &str,
    ignore_failed: bool,
    test: bool,
    build_static_files: bool,
    cache: Option<&mut cache::Cache>,
    preview_session_id: &Option<String>,
) -> fastn_core::Result<()> {
    match document {
        fastn_core::File::Ftd(doc) => {
            let file_path = if doc.id.eq("404.ftd") {
                "404.html".to_string()
            } else if doc.id.ends_with("index.ftd") {
                fastn_core::utils::replace_last_n(doc.id.as_str(), 1, "index.ftd", "index.html")
            } else {
                fastn_core::utils::replace_last_n(doc.id.as_str(), 1, ".ftd", "/index.html")
            };

            let (cache, is_cached) = is_cached(cache, doc, file_path.as_str());
            if is_cached {
                return Ok(());
            }

            fastn_core::utils::copy(
                &config.ds.root().join(doc.id.as_str()),
                &config.ds.root().join(".build").join(doc.id.as_str()),
                &config.ds,
            )
            .await
            .ok();

            if doc.id.eq("FASTN.ftd") {
                return Ok(());
            }

            let (resp, dependencies) = {
                let req = fastn_core::http::Request::default();
                let mut req_config =
                    fastn_core::RequestConfig::new(config, &req, doc.id.as_str(), base_url);
                req_config.current_document = Some(document.get_id().to_string());

                let result = fastn_core::package::package_doc::process_ftd(
                    &mut req_config,
                    doc,
                    base_url,
                    build_static_files,
                    test,
                    file_path.as_str(),
                    preview_session_id,
                )
                .await;

                // Extract dependencies before the scope ends
                (result, req_config.dependencies_during_render)
            };

            match (resp, ignore_failed) {
                (Ok(r), _) => {
                    // Use collected dependencies for proper incremental build cache invalidation
                    // Dependencies were extracted from req_config scope above
                    if let Some(cache) = cache {
                        cache.documents.insert(
                            remove_extension(doc.id.as_str()),
                            cache::Document {
                                html_checksum: r.checksum(),
                                dependencies,
                            },
                        );
                        cache.file_checksum.insert(
                            remove_extension(doc.id.as_str()),
                            fastn_core::utils::generate_hash(doc.content.as_str()),
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
        fastn_core::File::Static(sa) => {
            process_static(sa, &config.ds.root(), &config.package, &config.ds).await?
        }
        fastn_core::File::Markdown(_doc) => {
            // TODO: bring this feature back
            print!("Skipped ");
            return Ok(());
        }
        fastn_core::File::Image(main_doc) => {
            process_static(main_doc, &config.ds.root(), &config.package, &config.ds).await?;
        }
        fastn_core::File::Code(doc) => {
            process_static(
                &fastn_core::Static {
                    package_name: config.package.name.to_string(),
                    id: doc.id.to_string(),
                    content: doc.content.clone().into_bytes(),
                    base_path: doc.parent_path.clone(),
                },
                &config.ds.root(),
                &config.package,
                &config.ds,
            )
            .await?;
        }
    }

    Ok(())
}

#[tracing::instrument]
pub async fn default_build_files(
    base_path: fastn_ds::Path,
    ftd_edition: &fastn_core::FTDEdition,
    package_name: &str,
    ds: &fastn_ds::DocumentStore,
) -> fastn_core::Result<()> {
    if ftd_edition.is_2023() {
        let default_ftd_js_content = ftd::js::all_js_without_test(package_name);
        let hashed_ftd_js_name = fastn_core::utils::hashed_default_ftd_js(package_name);
        let save_default_ftd_js = base_path.join(hashed_ftd_js_name);
        fastn_core::utils::update(&save_default_ftd_js, default_ftd_js_content.as_bytes(), ds)
            .await
            .ok();

        let markdown_js_content = ftd::markdown_js();
        let hashed_markdown_js_name = fastn_core::utils::hashed_markdown_js();
        let save_markdown_js = base_path.join(hashed_markdown_js_name);
        fastn_core::utils::update(&save_markdown_js, markdown_js_content.as_bytes(), ds)
            .await
            .ok();

        let theme_css = ftd::theme_css();
        let hashed_code_themes = fastn_core::utils::hashed_code_theme_css();
        for (theme, file_name) in hashed_code_themes {
            let save_markdown_js = base_path.join(file_name);
            let theme_content = theme_css.get(theme).unwrap();
            fastn_core::utils::update(&save_markdown_js, theme_content.as_bytes(), ds)
                .await
                .ok();
        }

        let prism_js_content = ftd::prism_js();
        let hashed_prism_js_name = fastn_core::utils::hashed_prism_js();
        let save_prism_js = base_path.join(hashed_prism_js_name);
        fastn_core::utils::update(&save_prism_js, prism_js_content.as_bytes(), ds)
            .await
            .ok();

        let prism_css_content = ftd::prism_css();
        let hashed_prism_css_name = fastn_core::utils::hashed_prism_css();
        let save_prism_css = base_path.join(hashed_prism_css_name);
        fastn_core::utils::update(&save_prism_css, prism_css_content.as_bytes(), ds)
            .await
            .ok();
    } else {
        let default_css_content = ftd::css();
        let hashed_css_name = fastn_core::utils::hashed_default_css_name();
        let save_default_css = base_path.join(hashed_css_name);
        fastn_core::utils::update(&save_default_css, default_css_content.as_bytes(), ds)
            .await
            .ok();

        let default_js_content = format!("{}\n\n{}", ftd::build_js(), fastn_core::fastn_2022_js());
        let hashed_js_name = fastn_core::utils::hashed_default_js_name();
        let save_default_js = base_path.join(hashed_js_name);
        fastn_core::utils::update(&save_default_js, default_js_content.as_bytes(), ds)
            .await
            .ok();
    }

    Ok(())
}

#[tracing::instrument(skip(config))]
async fn get_documents_for_current_package(
    config: &fastn_core::Config,
) -> fastn_core::Result<std::collections::BTreeMap<String, fastn_core::File>> {
    let mut documents = std::collections::BTreeMap::from_iter(
        config
            .get_files(&config.package, &None)
            .await?
            .into_iter()
            .map(|v| (v.get_id().to_string(), v)),
    );

    if let Some(ref sitemap) = config.package.sitemap {
        let get_all_locations = sitemap.get_all_locations();
        let mut files: std::collections::HashMap<String, fastn_core::File> = Default::default();
        for (doc_path, _, url) in get_all_locations {
            let file = {
                let package_name = if let Some(ref url) = url {
                    config.find_package_by_id(url).await?.1.name
                } else {
                    config.package.name.to_string()
                };
                let mut file = fastn_core::get_file(
                    &config.ds,
                    package_name,
                    doc_path,
                    &config.ds.root(),
                    &None,
                )
                .await?;
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

        documents.extend(files);
    }

    Ok(documents)
}

async fn process_static(
    sa: &fastn_core::Static,
    base_path: &fastn_ds::Path,
    package: &fastn_core::Package,
    ds: &fastn_ds::DocumentStore,
) -> fastn_core::Result<()> {
    copy_to_build(sa, base_path, package, ds).await?;
    if let Some(original_package) = package.translation_of.as_ref() {
        copy_to_build(sa, base_path, original_package, ds).await?;
    }
    return Ok(());

    async fn copy_to_build(
        sa: &fastn_core::Static,
        base_path: &fastn_ds::Path,
        package: &fastn_core::Package,
        ds: &fastn_ds::DocumentStore,
    ) -> fastn_core::Result<()> {
        let build_path = base_path
            .join(".build")
            .join("-")
            .join(package.name.as_str());

        let full_file_path = build_path.join(sa.id.as_str());
        ds.write_content(&full_file_path, &sa.content).await?;

        {
            // TODO: need to remove this once download_base_url is removed
            let content = ds
                .read_content(&sa.base_path.join(sa.id.as_str()), &None)
                .await?;
            ds.write_content(&base_path.join(".build").join(sa.id.as_str()), &content)
                .await?;
        }
        Ok(())
    }
}
