pub async fn parse<'a>(
    name: &str,
    source: &str,
    lib: &'a fpm::Library,
    base_url: &str,
    current_package: Option<&fpm::Package>,
) -> ftd::p1::Result<ftd::p2::Document> {
    let mut s = ftd::interpret(name, source)?;

    let mut packages_under_process = vec![current_package
        .map(|v| v.to_owned())
        .unwrap_or_else(|| lib.config.package.clone())];
    let document;
    loop {
        match s {
            ftd::Interpreter::Done { document: doc } => {
                document = doc;
                break;
            }
            ftd::Interpreter::StuckOnProcessor { state, section } => {
                let value = lib
                    .process(
                        &section,
                        &state.tdoc(&mut Default::default(), &mut Default::default()),
                    )
                    .await?;
                s = state.continue_after_processor(&section, value)?;
            }
            ftd::Interpreter::StuckOnImport {
                module,
                state: mut st,
            } => {
                packages_under_process.truncate(st.document_stack.len());
                let source = if module.eq("fpm/time") {
                    st.add_foreign_variable_prefix(module.as_str(), vec![module.to_string()]);
                    packages_under_process.push(
                        packages_under_process
                            .last()
                            .ok_or_else(|| ftd::p1::Error::ParseError {
                                message: "The processing document stack is empty".to_string(),
                                doc_id: "".to_string(),
                                line_number: 0,
                            })?
                            .clone(),
                    );
                    "".to_string()
                } else if module.ends_with("assets") {
                    st.add_foreign_variable_prefix(
                        module.as_str(),
                        vec![format!("{}#files", module)],
                    );

                    packages_under_process.push(
                        packages_under_process
                            .last()
                            .ok_or_else(|| ftd::p1::Error::ParseError {
                                message: "The processing document stack is empty".to_string(),
                                doc_id: "".to_string(),
                                line_number: 0,
                            })?
                            .clone(),
                    );

                    let current_package = packages_under_process.last().ok_or_else(|| {
                        ftd::p1::Error::ParseError {
                            message: "The processing document stack is empty".to_string(),
                            doc_id: "".to_string(),
                            line_number: 0,
                        }
                    })?;

                    if module.starts_with(current_package.name.as_str()) {
                        current_package.get_font_ftd().unwrap_or_default()
                    } else {
                        let mut font_ftd = "".to_string();
                        for (alias, package) in current_package.aliases() {
                            if module.starts_with(alias) {
                                font_ftd = package.get_font_ftd().unwrap_or_default();
                                break;
                            }
                        }
                        font_ftd
                    }
                } else {
                    lib.get_with_result(module.as_str(), &mut packages_under_process)
                        .await?
                };
                s = st.continue_after_import(module.as_str(), source.as_str())?;
            }
            ftd::Interpreter::StuckOnForeignVariable { variable, state } => {
                packages_under_process.truncate(state.document_stack.len());
                let current_package =
                    packages_under_process
                        .last()
                        .ok_or_else(|| ftd::p1::Error::ParseError {
                            message: "The processing document stack is empty".to_string(),
                            doc_id: "".to_string(),
                            line_number: 0,
                        })?;
                let value = resolve_foreign_variable(
                    variable.as_str(),
                    name,
                    current_package,
                    lib,
                    base_url,
                )?;
                s = state.continue_after_variable(variable.as_str(), value)?
            }
            ftd::Interpreter::CheckID { .. } => {
                // This function was used in build.rs
                // Not using build.rs anymore (build2.rs is used currently)
                // so ignoring processing terms here
                // s = st.continue_after_checking_id(None, index)?;
                unimplemented!();
            }
        }
    }
    Ok(document)
}

pub async fn interpret_helper<'a>(
    name: &str,
    source: &str,
    lib: &'a mut fpm::Library2,
) -> ftd::interpreter2::Result<ftd::interpreter2::Document> {
    let mut s = ftd::interpreter2::interpret(name, source)?;
    let document;
    loop {
        match s {
            ftd::interpreter2::Interpreter::Done { document: doc } => {
                document = doc;
                break;
            }
            ftd::interpreter2::Interpreter::StuckOnImport {
                module,
                state: mut st,
            } => {
                let source = resolve_import_2022(lib, &mut st, module.as_str()).await?;
                s = st.continue_after_import(module.as_str(), source.as_str())?;
            }
        }
    }
    Ok(document)
}

pub async fn parse2<'a>(
    name: &str,
    source: &str,
    lib: &'a mut fpm::Library2,
    base_url: &str,
    download_assets: bool,
) -> ftd::p1::Result<ftd::p2::Document> {
    let mut s = ftd::interpret(name, source)?;

    let document;
    loop {
        match s {
            ftd::Interpreter::Done { document: doc } => {
                document = doc;
                break;
            }
            ftd::Interpreter::StuckOnProcessor { state, section } => {
                if lib.is_lazy_processor(
                    &section,
                    &state.tdoc(&mut Default::default(), &mut Default::default()),
                )? {
                    s = state.continue_after_storing_section(&section)?;
                } else {
                    let value = lib
                        .process(
                            &section,
                            &state.tdoc(&mut Default::default(), &mut Default::default()),
                        )
                        .await?;
                    s = state.continue_after_processor(&section, value)?;
                }
            }
            ftd::Interpreter::StuckOnImport {
                module,
                state: mut st,
            } => {
                let source = resolve_import(lib, &mut st, module.as_str()).await?;
                s = st.continue_after_import(module.as_str(), source.as_str())?;
            }
            ftd::Interpreter::StuckOnForeignVariable { variable, state } => {
                let value = resolve_foreign_variable2(
                    variable.as_str(),
                    name,
                    &state,
                    lib,
                    base_url,
                    download_assets,
                )
                .await?;
                s = state.continue_after_variable(variable.as_str(), value)?
            }
            ftd::Interpreter::CheckID {
                replace_blocks,
                state: st,
            } => {
                // No config in ftd::ExampleLibrary using dummy global_ids map for debugging
                let mut mapped_replace_blocks: Vec<
                    ftd::ReplaceLinkBlock<std::collections::HashMap<String, String>>,
                > = vec![];

                for (captured_id_set, source, ln) in replace_blocks.iter() {
                    let mut id_map: std::collections::HashMap<String, String> =
                        std::collections::HashMap::new();
                    for id in captured_id_set {
                        let link = lib
                            .config
                            .global_ids
                            .get(id)
                            .ok_or_else(|| ftd::p1::Error::ForbiddenUsage {
                                message: fpm::warning!("id: {} not found while linking", id),
                                doc_id: st.id.clone(),
                                line_number: *ln,
                            })?
                            .to_string();
                        id_map.insert(id.to_string(), link);
                    }
                    mapped_replace_blocks.push((id_map, source.to_owned(), ln.to_owned()));
                }

                s = st.continue_after_checking_id(mapped_replace_blocks)?;
            }
        }
    }
    Ok(document)
}

pub async fn resolve_import<'a>(
    lib: &'a mut fpm::Library2,
    state: &mut ftd::InterpreterState,
    module: &str,
) -> ftd::p1::Result<String> {
    lib.packages_under_process
        .truncate(state.document_stack.len());
    let current_package = lib.get_current_package()?;
    let source = if module.eq("fpm/time") {
        state.add_foreign_variable_prefix(module, vec![module.to_string()]);
        lib.push_package_under_process(&current_package).await?;
        "".to_string()
    } else if module.ends_with("assets") {
        state.add_foreign_variable_prefix(module, vec![format!("{}#files", module)]);

        if module.starts_with(current_package.name.as_str()) {
            lib.push_package_under_process(&current_package).await?;
            lib.get_current_package()?
                .get_font_ftd()
                .unwrap_or_default()
        } else {
            let mut font_ftd = "".to_string();
            for (alias, package) in current_package.aliases() {
                if module.starts_with(alias) {
                    lib.push_package_under_process(package).await?;
                    font_ftd = lib
                        .config
                        .all_packages
                        .borrow()
                        .get(package.name.as_str())
                        .unwrap()
                        .get_font_ftd()
                        .unwrap_or_default();
                    break;
                }
            }
            font_ftd
        }
    } else {
        lib.get_with_result(module).await?
    };

    Ok(source)
}

pub async fn resolve_import_2022<'a>(
    lib: &'a mut fpm::Library2,
    state: &mut ftd::interpreter2::InterpreterState,
    module: &str,
) -> ftd::interpreter2::Result<String> {
    lib.packages_under_process
        .truncate(state.document_stack.len());
    let source = lib.get_with_result(module).await?;
    Ok(source)
}

pub async fn resolve_foreign_variable2(
    variable: &str,
    doc_name: &str,
    state: &ftd::InterpreterState,
    lib: &mut fpm::Library2,
    base_url: &str,
    download_assets: bool,
) -> ftd::p1::Result<ftd::Value> {
    lib.packages_under_process
        .truncate(state.document_stack.len());
    let package = lib.get_current_package()?;
    if let Ok(value) = resolve_ftd_foreign_variable(variable, doc_name) {
        return Ok(value);
    }

    if let Some((package_name, files)) = variable.split_once("/assets#files.") {
        if package.name.eq(package_name) {
            if let Ok(value) =
                get_assets_value(&package, files, lib, base_url, download_assets).await
            {
                return Ok(value);
            }
        }
        for (alias, package) in package.aliases() {
            if alias.eq(package_name) {
                if let Ok(value) =
                    get_assets_value(package, files, lib, base_url, download_assets).await
                {
                    return Ok(value);
                }
            }
        }
    }

    return ftd::p2::utils::e2(format!("{} not found 2", variable).as_str(), doc_name, 0);

    async fn get_assets_value(
        package: &fpm::Package,
        files: &str,
        lib: &mut fpm::Library2,
        base_url: &str,
        download_assets: bool, // true: in case of `fpm build`
    ) -> ftd::p1::Result<ftd::Value> {
        lib.push_package_under_process(package).await?;
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
                );

                let light_path = format!("{}.{}", file.replace('.', "/"), ext);
                if download_assets
                    && !lib
                        .config
                        .downloaded_assets
                        .contains_key(&format!("{}/{}", package.name, light_path))
                {
                    let start = std::time::Instant::now();
                    let light = package
                        .resolve_by_file_name(light_path.as_str(), None, false)
                        .await
                        .map_err(|e| ftd::p1::Error::ParseError {
                            message: e.to_string(),
                            doc_id: lib.document_id.to_string(),
                            line_number: 0,
                        })?;
                    fpm::utils::write(
                        &lib.config.build_dir().join("-").join(package.name.as_str()),
                        light_path.as_str(),
                        light.as_slice(),
                    )
                    .await
                    .map_err(|e| ftd::p1::Error::ParseError {
                        message: e.to_string(),
                        doc_id: lib.document_id.to_string(),
                        line_number: 0,
                    })?;
                    lib.config.downloaded_assets.insert(
                        format!("{}/{}", package.name, light_path),
                        light_mode.to_string(),
                    );
                    fpm::utils::print_end(
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
                        .resolve_by_file_name(dark_path.as_str(), None, false)
                        .await
                    {
                        fpm::utils::write(
                            &lib.config.build_dir().join("-").join(package.name.as_str()),
                            dark_path.as_str(),
                            dark.as_slice(),
                        )
                        .await
                        .map_err(|e| ftd::p1::Error::ParseError {
                            message: e.to_string(),
                            doc_id: lib.document_id.to_string(),
                            line_number: 0,
                        })?;
                        fpm::utils::print_end(
                            format!("Processed {}/{}", package.name.as_str(), dark_path).as_str(),
                            start,
                        );
                    } else {
                        dark_mode = light_mode.clone();
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
                text: format!("/-/{}/{}.{}", package.name, file.replace('.', "/"), ext),
                source: ftd::TextSource::Header,
            }),
            None => Ok(ftd::Value::String {
                text: format!("/-/{}/{}", package.name, files),
                source: ftd::TextSource::Header,
            }),
        }
    }
}

fn resolve_foreign_variable(
    variable: &str,
    doc_name: &str,
    package: &fpm::Package,
    lib: &fpm::Library,
    base_url: &str,
) -> ftd::p1::Result<ftd::Value> {
    if let Ok(value) = resolve_ftd_foreign_variable(variable, doc_name) {
        return Ok(value);
    }

    if let Some((package_name, files)) = variable.split_once("/assets#files.") {
        if package.name.eq(package_name) {
            if let Ok(value) = get_assets_value(package, files, lib, doc_name, base_url) {
                return Ok(value);
            }
        }
        for (alias, package) in package.aliases() {
            if alias.eq(package_name) {
                if let Ok(value) = get_assets_value(package, files, lib, doc_name, base_url) {
                    return Ok(value);
                }
            }
        }
    }

    return ftd::p2::utils::e2(format!("{} not found 1", variable).as_str(), doc_name, 0);

    fn get_assets_value(
        package: &fpm::Package,
        files: &str,
        lib: &fpm::Library,
        doc_name: &str,
        base_url: &str,
    ) -> ftd::p1::Result<ftd::Value> {
        let base_url = base_url.trim_end_matches('/');
        let mut files = files.to_string();
        let path = lib.config.get_root_for_package(package);
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
                    .starts_with("image/")
                    && path
                        .join(format!("{}.{}", file.replace('.', "/"), ext))
                        .exists() =>
            {
                let light_mode = format!(
                    "{base_url}/-/{}/{}.{}",
                    package.name,
                    file.replace('.', "/"),
                    ext
                );
                if light {
                    return Ok(ftd::Value::String {
                        text: light_mode,
                        source: ftd::TextSource::Header,
                    });
                }
                let dark_mode = if path
                    .join(format!("{}-dark.{}", file.replace('.', "/"), ext))
                    .exists()
                {
                    format!(
                        "{base_url}/-/{}/{}-dark.{}",
                        package.name,
                        file.replace('.', "/"),
                        ext
                    )
                } else {
                    light_mode.clone()
                };

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
            Some((file, ext))
                if path
                    .join(format!("{}.{}", file.replace('.', "/"), ext))
                    .exists() =>
            {
                Ok(ftd::Value::String {
                    text: format!("/-/{}/{}.{}", package.name, file.replace('.', "/"), ext),
                    source: ftd::TextSource::Header,
                })
            }
            None if path.join(&files).exists() => Ok(ftd::Value::String {
                text: format!("/-/{}/{}", package.name, files),
                source: ftd::TextSource::Header,
            }),
            _ => ftd::p2::utils::e2(format!("{} not found 2", files).as_str(), doc_name, 0),
        }
    }
}

// No need to make async since this is pure.
pub fn parse_ftd(
    name: &str,
    source: &str,
    lib: &fpm::FPMLibrary,
) -> ftd::p1::Result<ftd::p2::Document> {
    let mut s = ftd::interpret(name, source)?;
    let document;
    loop {
        match s {
            ftd::Interpreter::Done { document: doc } => {
                document = doc;
                break;
            }
            ftd::Interpreter::StuckOnProcessor { state, section } => {
                let value = lib.process(
                    &section,
                    &state.tdoc(&mut Default::default(), &mut Default::default()),
                )?;
                s = state.continue_after_processor(&section, value)?;
            }
            ftd::Interpreter::StuckOnImport { module, state: st } => {
                let source = lib.get_with_result(
                    module.as_str(),
                    &st.tdoc(&mut Default::default(), &mut Default::default()),
                )?;
                s = st.continue_after_import(module.as_str(), source.as_str())?;
            }
            ftd::Interpreter::StuckOnForeignVariable { variable, state } => {
                let value = resolve_ftd_foreign_variable(variable.as_str(), name)?;
                s = state.continue_after_variable(variable.as_str(), value)?
            }
            ftd::Interpreter::CheckID { .. } => {
                // No config in fpm::FPMLibrary ignoring processing terms here
                unimplemented!()
            }
        }
    }
    Ok(document)
}

fn resolve_ftd_foreign_variable(variable: &str, doc_name: &str) -> ftd::p1::Result<ftd::Value> {
    match variable.strip_prefix("fpm/time#") {
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
        _ => ftd::p2::utils::e2(format!("{} not found 3", variable).as_str(), doc_name, 0),
    }
}
