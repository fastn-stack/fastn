// NOTE: Dependency-aware hash generation moved to fastn-cache crate

fn cached_parse(
    id: &str,
    source: &str,
    line_number: usize,
    enable_cache: bool,
    _dependencies: Option<&[String]>, // Dependencies from previous compilation if available
) -> ftd::interpreter::Result<ftd::interpreter::ParsedDocument> {
    #[derive(serde::Deserialize, serde::Serialize)]
    struct C {
        hash: String,
        dependencies: Vec<String>,
        doc: ftd::interpreter::ParsedDocument,
    }

    // Only use cache if explicitly enabled via --enable-cache flag
    if enable_cache {
        if let Some(c) = fastn_core::utils::get_cached::<C>(id) {
            // Simple content hash check for now (dependency-aware logic in fastn-cache)
            let current_hash = fastn_core::utils::generate_hash(source);

            if c.hash == current_hash {
                eprintln!("🚀 PERF: CACHE HIT (simple hash) for: {}", id);
                return Ok(c.doc);
            }
            // eprintln!("🔥 PERF: Cache invalidated (content changed) for: {}", id);
        } else {
            // eprintln!("🔥 PERF: Cache miss (no previous cache) for: {}", id);
        }
    } else {
        // eprintln!("🔥 PERF: Caching DISABLED (use --enable-cache to enable)");
    }

    let doc = ftd::interpreter::ParsedDocument::parse_with_line_number(id, source, line_number)?;

    // Cache with empty dependencies for now (will be updated later with real dependencies)
    if enable_cache {
        let initial_hash = fastn_core::utils::generate_hash(source);
        fastn_core::utils::cache_it(
            id,
            C {
                doc,
                hash: initial_hash,
                dependencies: vec![], // Will be updated after compilation with real dependencies
            },
        )
        .map(|v| v.doc)
    } else {
        Ok(doc)
    }
}

// NOTE: Cache dependency updates will be handled by fastn-cache crate
// This function is temporarily simplified during migration

pub fn package_dependent_builtins(
    config: &fastn_core::Config,
    req_path: &str,
) -> ftd::interpreter::HostBuiltins {
    [
        fastn_core::host_builtins::app_path(config, req_path),
        fastn_core::host_builtins::main_package(config),
        fastn_core::host_builtins::app_mounts(config),
    ]
}

#[tracing::instrument(skip(lib))]
pub async fn interpret_helper(
    name: &str,
    source: &str,
    lib: &mut fastn_core::Library2022,
    base_url: &str,
    download_assets: bool,
    line_number: usize,
    preview_session_id: &Option<String>,
) -> ftd::interpreter::Result<ftd::interpreter::Document> {
    let doc = cached_parse(name, source, line_number, lib.config.enable_cache, None)?;

    let builtin_overrides = package_dependent_builtins(&lib.config, lib.request.path());
    let mut s = ftd::interpreter::interpret_with_line_number(name, doc, Some(builtin_overrides))?;
    lib.module_package_map.insert(
        name.trim_matches('/').to_string(),
        lib.config.package.name.to_string(),
    );
    let document;
    loop {
        match s {
            ftd::interpreter::Interpreter::Done { document: doc } => {
                tracing::info!("done");
                document = doc;
                break;
            }
            ftd::interpreter::Interpreter::StuckOnImport {
                module,
                state: mut st,
                caller_module,
            } => {
                tracing::info!("stuck on import: {module}");
                // TODO: also check if module in in dependencies of this package
                let caller_module = if module.starts_with("inherited-")
                    || caller_module.starts_with("inherited-")
                {
                    // We want to use the main package name as the caller_module for this as the
                    // inherited- package's provided-via path can only be read from the main
                    // package.
                    name
                } else {
                    &caller_module
                };

                let (source, path, foreign_variable, foreign_function, ignore_line_numbers) =
                    resolve_import_2022(
                        lib,
                        &mut st,
                        module.as_str(),
                        caller_module,
                        preview_session_id,
                    )
                    .await?;
                tracing::info!("import resolved: {module} -> {path}");
                lib.dependencies_during_render.push(path);
                let doc = cached_parse(
                    module.as_str(),
                    source.as_str(),
                    ignore_line_numbers,
                    lib.config.enable_cache,
                    None,
                )?;
                s = st.continue_after_import(
                    module.as_str(),
                    doc,
                    foreign_variable,
                    foreign_function,
                    ignore_line_numbers,
                )?;
            }
            ftd::interpreter::Interpreter::StuckOnProcessor {
                state,
                ast,
                module,
                processor,
                ..
            } => {
                tracing::info!("stuck on processor: {processor}");
                let doc = state.get_current_processing_module().ok_or(
                    ftd::interpreter::Error::ValueNotFound {
                        doc_id: module,
                        line_number: ast.line_number(),
                        message: "Cannot find the module".to_string(),
                    },
                )?;
                let line_number = ast.line_number();
                let value = lib
                    .process(
                        ast.clone(),
                        processor,
                        &mut state.tdoc(doc.as_str(), line_number)?,
                        preview_session_id,
                    )
                    .await?;
                s = state.continue_after_processor(value, ast)?;
            }
            ftd::interpreter::Interpreter::StuckOnForeignVariable {
                state,
                module,
                variable,
                caller_module,
            } => {
                tracing::info!("stuck on foreign variable: {variable}");
                let value = resolve_foreign_variable2022(
                    variable.as_str(),
                    module.as_str(),
                    lib,
                    base_url,
                    download_assets,
                    caller_module.as_str(),
                    preview_session_id,
                )
                .await?;
                s = state.continue_after_variable(module.as_str(), variable.as_str(), value)?;
            }
        }
    }

    // Update cache with collected dependencies for always-correct future caching
    // Note: We don't have access to original source here, which is a limitation
    // For now, log the dependencies that were collected
    if lib.config.enable_cache && !lib.dependencies_during_render.is_empty() {
        eprintln!(
            "🔥 PERF: Collected {} dependencies for future cache invalidation",
            lib.dependencies_during_render.len()
        );
        for dep in &lib.dependencies_during_render {
            eprintln!("  📁 Dependency: {}", dep);
        }
    }

    Ok(document)
}

/// returns: (source, path, foreign_variable, foreign_function, ignore_line_numbers)
#[allow(clippy::type_complexity)]
#[tracing::instrument(skip(lib, _state))]
pub async fn resolve_import_2022(
    lib: &mut fastn_core::Library2022,
    _state: &mut ftd::interpreter::InterpreterState,
    module: &str,
    caller_module: &str,
    session_id: &Option<String>,
) -> ftd::interpreter::Result<(String, String, Vec<String>, Vec<String>, usize)> {
    let current_package = lib.get_current_package(caller_module)?;
    let source = if module.eq("fastn/time") {
        (
            "".to_string(),
            "$fastn$/time.ftd".to_string(),
            vec!["time".to_string()],
            vec![],
            0,
        )
    } else if module.eq("fastn/processors") {
        (
            fastn_core::processor_ftd().to_string(),
            "$fastn$/processors.ftd".to_string(),
            vec![],
            vec![
                "figma-typo-token".to_string(),
                "figma-cs-token".to_string(),
                "figma-cs-token-old".to_string(),
                "http".to_string(),
                "get-data".to_string(),
                "toc".to_string(),
                "sitemap".to_string(),
                "full-sitemap".to_string(),
                "request-data".to_string(),
                "document-readers".to_string(),
                "document-writers".to_string(),
                "user-groups".to_string(),
                "user-group-by-id".to_string(),
                "get-identities".to_string(),
                "document-id".to_string(),
                "document-full-id".to_string(),
                "document-suffix".to_string(),
                "document-name".to_string(),
                "user-details".to_string(),
                "fastn-apps".to_string(),
                "is-reader".to_string(),
                "sql-query".to_string(),
                "sql-execute".to_string(),
                "sql-batch".to_string(),
                "package-query".to_string(),
                "pg".to_string(),
                "package-tree".to_string(),
                "fetch-file".to_string(),
                "query".to_string(),
                "current-language".to_string(),
                "current-url".to_string(),
                "translation-info".to_string(),
            ],
            0,
        )
    } else if module.ends_with("assets") {
        let foreign_variable = vec!["files".to_string()];

        if module.starts_with(current_package.name.as_str()) {
            (
                current_package.get_font_ftd().unwrap_or_default(),
                format!("{name}/-/assets.ftd", name = current_package.name),
                foreign_variable,
                vec![],
                0,
            )
        } else {
            let mut font_ftd = "".to_string();
            let mut path = "".to_string();
            for (alias, package) in current_package.aliases() {
                if module.starts_with(alias) {
                    lib.push_package_under_process(module, package, session_id)
                        .await?;
                    font_ftd = lib
                        .config
                        .all_packages
                        .get(package.name.as_str())
                        .unwrap()
                        .get()
                        .get_font_ftd()
                        .unwrap_or_default();
                    path = format!("{name}/-/fonts.ftd", name = package.name);
                    break;
                }
            }
            (font_ftd, path, foreign_variable, vec![], 0)
        }
    } else {
        let (content, path, ignore_line_numbers) = lib
            .get_with_result(module, caller_module, session_id)
            .await?;
        (
            content,
            path,
            vec![],
            vec![
                "figma-typo-token".to_string(),
                "figma-cs-token".to_string(),
                "figma-cs-token-old".to_string(),
                "http".to_string(),
                "sql-query".to_string(),
                "sql-execute".to_string(),
                "sql-batch".to_string(),
                "package-query".to_string(),
                "pg".to_string(),
                "toc".to_string(),
                "include".to_string(),
                "get-data".to_string(),
                "sitemap".to_string(),
                "full-sitemap".to_string(),
                "user-groups".to_string(),
                "document-readers".to_string(),
                "document-writers".to_string(),
                "user-group-by-id".to_string(),
                "get-identities".to_string(),
                "document-id".to_string(),
                "document-full-id".to_string(),
                "document-name".to_string(),
                "document-suffix".to_string(),
                "package-id".to_string(),
                "package-tree".to_string(),
                "fetch-file".to_string(),
                "get-version-data".to_string(),
                "cr-meta".to_string(),
                "request-data".to_string(),
                "user-details".to_string(),
                "fastn-apps".to_string(),
                "is-reader".to_string(),
                "current-language".to_string(),
                "current-url".to_string(),
                "translation-info".to_string(),
            ],
            ignore_line_numbers,
        )
    };
    Ok(source)
}

#[tracing::instrument(name = "fastn_core::stuck-on-foreign-variable", err, skip(lib))]
pub async fn resolve_foreign_variable2022(
    variable: &str,
    doc_name: &str,
    lib: &mut fastn_core::Library2022,
    base_url: &str,
    download_assets: bool,
    caller_module: &str,
    preview_session_id: &Option<String>,
) -> ftd::interpreter::Result<fastn_resolved::Value> {
    let package = lib.get_current_package(caller_module)?;
    if let Ok(value) = resolve_ftd_foreign_variable_2022(variable, doc_name) {
        return Ok(value);
    }

    if variable.starts_with("files.") {
        let files = variable.trim_start_matches("files.").to_string();
        let package_name = doc_name.trim_end_matches("/assets").to_string();

        if package.name.eq(&package_name)
            && let Ok(value) = get_assets_value(
                doc_name,
                &package,
                files.as_str(),
                lib,
                base_url,
                download_assets,
                preview_session_id,
            )
            .await
        {
            return Ok(value);
        }
        for (alias, package) in package.aliases() {
            if alias.eq(&package_name) {
                let package = lib
                    .config
                    .find_package_else_default(package.name.as_str(), Some(package.to_owned()));
                if let Ok(value) = get_assets_value(
                    doc_name,
                    &package,
                    files.as_str(),
                    lib,
                    base_url,
                    download_assets,
                    preview_session_id,
                )
                .await
                {
                    return Ok(value);
                }
            }
        }
    }

    return ftd::interpreter::utils::e2(format!("{variable} not found 2").as_str(), doc_name, 0);

    async fn get_assets_value(
        module: &str,
        package: &fastn_core::Package,
        files: &str,
        lib: &mut fastn_core::RequestConfig,
        base_url: &str,
        download_assets: bool, // true: in case of `fastn build`
        preview_session_id: &Option<String>,
    ) -> ftd::ftd2021::p1::Result<fastn_resolved::Value> {
        lib.push_package_under_process(module, package, preview_session_id)
            .await?;
        let _base_url = base_url.trim_end_matches('/');

        // remove :type=module when used with js files
        let mut files = files
            .rsplit_once(':')
            .map_or(files.to_string(), |(f, _)| f.to_string());

        let light = {
            if let Some(f) = files.strip_suffix(".light") {
                files = f.to_string();
                true
            } else {
                false
            }
        };
        let dark = {
            if light {
                false
            } else if let Some(f) = files.strip_suffix(".dark") {
                files = f.to_string();
                true
            } else {
                false
            }
        };

        match files.rsplit_once('.') {
            Some((file, ext))
                if mime_guess::MimeGuess::from_ext(ext)
                    .first_or_octet_stream()
                    .to_string()
                    .starts_with("image/") =>
            {
                let light_mode = format!("/-/{}/{}.{}", package.name, file.replace('.', "/"), ext)
                    .trim_start_matches('/')
                    .to_string();

                let light_path = format!("{}.{}", file.replace('.', "/"), ext);
                if download_assets
                    && !lib
                        .downloaded_assets
                        .contains_key(&format!("{}/{}", package.name, light_path))
                {
                    let start = std::time::Instant::now();
                    let light = package
                        .resolve_by_file_name(
                            light_path.as_str(),
                            None,
                            &lib.config.ds,
                            preview_session_id,
                        )
                        .await
                        .map_err(|e| ftd::ftd2021::p1::Error::ParseError {
                            message: e.to_string(),
                            doc_id: lib.document_id.to_string(),
                            line_number: 0,
                        })?;
                    print!("Processing {}/{} ... ", package.name.as_str(), light_path);
                    fastn_core::utils::write(
                        &lib.config.build_dir().join("-").join(package.name.as_str()),
                        light_path.as_str(),
                        light.as_slice(),
                        &lib.config.ds,
                        preview_session_id,
                    )
                    .await
                    .map_err(|e| ftd::ftd2021::p1::Error::ParseError {
                        message: e.to_string(),
                        doc_id: lib.document_id.to_string(),
                        line_number: 0,
                    })?;
                    lib.downloaded_assets.insert(
                        format!("{}/{}", package.name, light_path),
                        light_mode.to_string(),
                    );
                    fastn_core::utils::print_end(
                        format!("Processed {}/{}", package.name.as_str(), light_path).as_str(),
                        start,
                    );
                }

                if light {
                    return Ok(fastn_resolved::Value::String {
                        text: light_mode.trim_start_matches('/').to_string(),
                    });
                }

                let mut dark_mode = if file.ends_with("-dark") {
                    light_mode.clone()
                } else {
                    format!(
                        "/-/{}/{}-dark.{}",
                        package.name,
                        file.replace('.', "/"),
                        ext
                    )
                    .trim_start_matches('/')
                    .to_string()
                };

                let dark_path = format!("{}-dark.{}", file.replace('.', "/"), ext);
                if download_assets && !file.ends_with("-dark") {
                    let start = std::time::Instant::now();
                    if let Some(dark) = lib
                        .downloaded_assets
                        .get(&format!("{}/{}", package.name, dark_path))
                    {
                        dark_mode = dark.to_string();
                    } else if let Ok(dark) = package
                        .resolve_by_file_name(
                            dark_path.as_str(),
                            None,
                            &lib.config.ds,
                            preview_session_id,
                        )
                        .await
                    {
                        print!("Processing {}/{} ... ", package.name.as_str(), dark_path);
                        fastn_core::utils::write(
                            &lib.config.build_dir().join("-").join(package.name.as_str()),
                            dark_path.as_str(),
                            dark.as_slice(),
                            &lib.config.ds,
                            preview_session_id,
                        )
                        .await
                        .map_err(|e| {
                            ftd::ftd2021::p1::Error::ParseError {
                                message: e.to_string(),
                                doc_id: lib.document_id.to_string(),
                                line_number: 0,
                            }
                        })?;
                        fastn_core::utils::print_end(
                            format!("Processed {}/{}", package.name.as_str(), dark_path).as_str(),
                            start,
                        );
                    } else {
                        dark_mode.clone_from(&light_mode);
                    }
                    lib.downloaded_assets.insert(
                        format!("{}/{}", package.name, dark_path),
                        dark_mode.to_string(),
                    );
                }

                if dark {
                    return Ok(fastn_resolved::Value::String {
                        text: dark_mode.trim_start_matches('/').to_string(),
                    });
                }
                #[allow(deprecated)]
                Ok(fastn_resolved::Value::Record {
                    name: "ftd#image-src".to_string(),
                    fields: std::array::IntoIter::new([
                        (
                            "light".to_string(),
                            fastn_resolved::PropertyValue::Value {
                                value: fastn_resolved::Value::String { text: light_mode },
                                is_mutable: false,
                                line_number: 0,
                            },
                        ),
                        (
                            "dark".to_string(),
                            fastn_resolved::PropertyValue::Value {
                                value: fastn_resolved::Value::String { text: dark_mode },
                                is_mutable: false,
                                line_number: 0,
                            },
                        ),
                    ])
                    .collect(),
                })
            }
            Some((file, ext)) => {
                download(
                    lib,
                    download_assets,
                    package,
                    format!("{}.{}", file.replace('.', "/"), ext).as_str(),
                    preview_session_id,
                )
                .await?;
                Ok(fastn_resolved::Value::String {
                    text: format!("-/{}/{}.{}", package.name, file.replace('.', "/"), ext),
                })
            }
            None => {
                download(
                    lib,
                    download_assets,
                    package,
                    files.as_str(),
                    preview_session_id,
                )
                .await?;
                Ok(fastn_resolved::Value::String {
                    text: format!("-/{}/{}", package.name, files),
                })
            }
        }
    }
}

async fn download(
    lib: &mut fastn_core::Library2022,
    download_assets: bool,
    package: &fastn_core::Package,
    path: &str,
    preview_session_id: &Option<String>,
) -> ftd::ftd2021::p1::Result<()> {
    if download_assets
        && !lib
            .downloaded_assets
            .contains_key(&format!("{}/{}", package.name, path))
    {
        let start = std::time::Instant::now();
        let data = package
            .resolve_by_file_name(path, None, &lib.config.ds, preview_session_id)
            .await
            .map_err(|e| ftd::ftd2021::p1::Error::ParseError {
                message: e.to_string(),
                doc_id: lib.document_id.to_string(),
                line_number: 0,
            })?;
        print!("Processing {}/{} ... ", package.name, path);
        fastn_core::utils::write(
            &lib.config.build_dir().join("-").join(package.name.as_str()),
            path,
            data.as_slice(),
            &lib.config.ds,
            preview_session_id,
        )
        .await
        .map_err(|e| ftd::ftd2021::p1::Error::ParseError {
            message: e.to_string(),
            doc_id: lib.document_id.to_string(),
            line_number: 0,
        })?;
        lib.downloaded_assets.insert(
            format!("{}/{}", package.name, path),
            format!("-/{}/{}", package.name, path),
        );
        fastn_core::utils::print_end(
            format!("Processed {}/{}", package.name, path).as_str(),
            start,
        );
    }

    Ok(())
}

pub async fn resolve_foreign_variable2(
    variable: &str,
    doc_name: &str,
    state: &ftd::ftd2021::InterpreterState,
    lib: &mut fastn_core::Library2,
    base_url: &str,
    download_assets: bool,
    session_id: &Option<String>,
) -> ftd::ftd2021::p1::Result<ftd::Value> {
    lib.packages_under_process
        .truncate(state.document_stack.len());
    let package = lib.get_current_package()?;
    if let Ok(value) = resolve_ftd_foreign_variable(variable, doc_name) {
        return Ok(value);
    }

    if let Some((package_name, files)) = variable.split_once("/assets#files.") {
        if package.name.eq(package_name)
            && let Ok(value) =
                get_assets_value(&package, files, lib, base_url, download_assets, session_id).await
        {
            return Ok(value);
        }
        for (alias, package) in package.aliases() {
            if alias.eq(package_name)
                && let Ok(value) =
                    get_assets_value(package, files, lib, base_url, download_assets, session_id)
                        .await
            {
                return Ok(value);
            }
        }
    }

    return ftd::ftd2021::p2::utils::e2(format!("{variable} not found 2").as_str(), doc_name, 0);

    async fn get_assets_value(
        package: &fastn_core::Package,
        files: &str,
        lib: &mut fastn_core::Library2,
        base_url: &str,
        download_assets: bool, // true: in case of `fastn build`
        session_id: &Option<String>,
    ) -> ftd::ftd2021::p1::Result<ftd::Value> {
        lib.push_package_under_process(package)?;
        let base_url = base_url.trim_end_matches('/');
        let mut files = files.to_string();
        let light = {
            if let Some(f) = files.strip_suffix(".light") {
                files = f.to_string();
                true
            } else {
                false
            }
        };
        let dark = {
            if light {
                false
            } else if let Some(f) = files.strip_suffix(".dark") {
                files = f.to_string();
                true
            } else {
                false
            }
        };

        match files.rsplit_once('.') {
            Some((file, ext))
                if mime_guess::MimeGuess::from_ext(ext)
                    .first_or_octet_stream()
                    .to_string()
                    .starts_with("image/") =>
            {
                let light_mode = format!(
                    "{base_url}/-/{}/{}.{}",
                    package.name,
                    file.replace('.', "/"),
                    ext
                )
                .trim_start_matches('/')
                .to_string();

                let light_path = format!("{}.{}", file.replace('.', "/"), ext);
                if download_assets
                    && !lib
                        .config
                        .downloaded_assets
                        .contains_key(&format!("{}/{}", package.name, light_path))
                {
                    let start = std::time::Instant::now();
                    let light = package
                        .resolve_by_file_name(
                            light_path.as_str(),
                            None,
                            &lib.config.config.ds,
                            session_id,
                        )
                        .await
                        .map_err(|e| ftd::ftd2021::p1::Error::ParseError {
                            message: e.to_string(),
                            doc_id: lib.document_id.to_string(),
                            line_number: 0,
                        })?;
                    print!("Processing {}/{} ... ", package.name.as_str(), light_path);
                    fastn_core::utils::write(
                        &lib.config
                            .config
                            .build_dir()
                            .join("-")
                            .join(package.name.as_str()),
                        light_path.as_str(),
                        light.as_slice(),
                        &lib.config.config.ds,
                        session_id,
                    )
                    .await
                    .map_err(|e| ftd::ftd2021::p1::Error::ParseError {
                        message: e.to_string(),
                        doc_id: lib.document_id.to_string(),
                        line_number: 0,
                    })?;
                    lib.config.downloaded_assets.insert(
                        format!("{}/{}", package.name, light_path),
                        light_mode.to_string(),
                    );
                    fastn_core::utils::print_end(
                        format!("Processed {}/{}", package.name.as_str(), light_path).as_str(),
                        start,
                    );
                }

                if light {
                    return Ok(ftd::Value::String {
                        text: light_mode,
                        source: ftd::TextSource::Header,
                    });
                }

                let mut dark_mode = if file.ends_with("-dark") {
                    light_mode.clone()
                } else {
                    format!(
                        "{base_url}/-/{}/{}-dark.{}",
                        package.name,
                        file.replace('.', "/"),
                        ext
                    )
                    .trim_start_matches('/')
                    .to_string()
                };

                let dark_path = format!("{}-dark.{}", file.replace('.', "/"), ext);
                if download_assets && !file.ends_with("-dark") {
                    let start = std::time::Instant::now();
                    if let Some(dark) = lib
                        .config
                        .downloaded_assets
                        .get(&format!("{}/{}", package.name, dark_path))
                    {
                        dark_mode = dark.to_string();
                    } else if let Ok(dark) = package
                        .resolve_by_file_name(
                            dark_path.as_str(),
                            None,
                            &lib.config.config.ds,
                            session_id,
                        )
                        .await
                    {
                        print!("Processing {}/{} ... ", package.name.as_str(), dark_path);
                        fastn_core::utils::write(
                            &lib.config
                                .config
                                .build_dir()
                                .join("-")
                                .join(package.name.as_str()),
                            dark_path.as_str(),
                            dark.as_slice(),
                            &lib.config.config.ds,
                            session_id,
                        )
                        .await
                        .map_err(|e| {
                            ftd::ftd2021::p1::Error::ParseError {
                                message: e.to_string(),
                                doc_id: lib.document_id.to_string(),
                                line_number: 0,
                            }
                        })?;
                        fastn_core::utils::print_end(
                            format!("Processed {}/{}", package.name.as_str(), dark_path).as_str(),
                            start,
                        );
                    } else {
                        dark_mode.clone_from(&light_mode);
                    }
                    lib.config.downloaded_assets.insert(
                        format!("{}/{}", package.name, dark_path),
                        dark_mode.to_string(),
                    );
                }

                if dark {
                    return Ok(ftd::Value::String {
                        text: dark_mode,
                        source: ftd::TextSource::Header,
                    });
                }
                #[allow(deprecated)]
                Ok(ftd::Value::Record {
                    name: "ftd#image-src".to_string(),
                    fields: std::array::IntoIter::new([
                        (
                            "light".to_string(),
                            ftd::PropertyValue::Value {
                                value: ftd::Value::String {
                                    text: light_mode,
                                    source: ftd::TextSource::Header,
                                },
                            },
                        ),
                        (
                            "dark".to_string(),
                            ftd::PropertyValue::Value {
                                value: ftd::Value::String {
                                    text: dark_mode,
                                    source: ftd::TextSource::Header,
                                },
                            },
                        ),
                    ])
                    .collect(),
                })
            }
            Some((file, ext)) => Ok(ftd::Value::String {
                text: format!("-/{}/{}.{}", package.name, file.replace('.', "/"), ext),
                source: ftd::TextSource::Header,
            }),
            None => Ok(ftd::Value::String {
                text: format!("-/{}/{}", package.name, files),
                source: ftd::TextSource::Header,
            }),
        }
    }
}

// No need to make async since this is pure.
pub fn parse_ftd(
    name: &str,
    source: &str,
    lib: &fastn_core::FastnLibrary,
) -> ftd::ftd2021::p1::Result<ftd::ftd2021::p2::Document> {
    let mut s = ftd::ftd2021::interpret(name, source, &None)?;
    let document;
    loop {
        match s {
            ftd::ftd2021::Interpreter::Done { document: doc } => {
                document = doc;
                break;
            }
            ftd::ftd2021::Interpreter::StuckOnProcessor { .. } => {
                unimplemented!()
            }
            ftd::ftd2021::Interpreter::StuckOnImport { module, state: st } => {
                let source = lib.get_with_result(
                    module.as_str(),
                    &st.tdoc(&mut Default::default(), &mut Default::default()),
                )?;
                s = st.continue_after_import(module.as_str(), source.as_str())?;
            }
            ftd::ftd2021::Interpreter::StuckOnForeignVariable { .. } => {
                unimplemented!()
            }
            ftd::ftd2021::Interpreter::CheckID { .. } => {
                // No config in fastn_core::FastnLibrary ignoring processing terms here
                unimplemented!()
            }
        }
    }
    Ok(document)
}

fn resolve_ftd_foreign_variable(
    variable: &str,
    doc_name: &str,
) -> ftd::ftd2021::p1::Result<ftd::Value> {
    match variable.strip_prefix("fastn/time#") {
        Some("now-str") => Ok(ftd::Value::String {
            text: std::str::from_utf8(
                std::process::Command::new("date")
                    .output()
                    .expect("failed to execute process")
                    .stdout
                    .as_slice(),
            )
            .unwrap()
            .to_string(),
            source: ftd::TextSource::Header,
        }),
        _ => ftd::ftd2021::p2::utils::e2(format!("{variable} not found 3").as_str(), doc_name, 0),
    }
}

fn resolve_ftd_foreign_variable_2022(
    variable: &str,
    doc_name: &str,
) -> ftd::ftd2021::p1::Result<fastn_resolved::Value> {
    match variable.strip_prefix("fastn/time#") {
        Some("now-str") => Ok(fastn_resolved::Value::String {
            text: std::str::from_utf8(
                std::process::Command::new("date")
                    .output()
                    .expect("failed to execute process")
                    .stdout
                    .as_slice(),
            )
            .unwrap()
            .to_string(),
        }),
        _ => ftd::ftd2021::p2::utils::e2(format!("{variable} not found 3").as_str(), doc_name, 0),
    }
}
