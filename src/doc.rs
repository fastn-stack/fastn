// TODO: make async
pub async fn parse<'a>(
    name: &str,
    source: &str,
    lib: &'a fpm::Library,
    base_url: &str,
) -> ftd::p1::Result<ftd::p2::Document> {
    let mut s = ftd::interpret(name, source)?;
    let mut packages_under_process = vec![&lib.config.package];
    let document;
    loop {
        match s {
            ftd::Interpreter::Done { document: doc } => {
                document = doc;
                break;
            }
            ftd::Interpreter::StuckOnProcessor { state, section } => {
                let value = lib
                    .process(&section, &state.tdoc(&mut Default::default()))
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
                    packages_under_process.push(packages_under_process.last().ok_or_else(
                        || ftd::p1::Error::ParseError {
                            message: "The processing document stack is empty".to_string(),
                            doc_id: "".to_string(),
                            line_number: 0,
                        },
                    )?);
                    "".to_string()
                } else if module.ends_with("assets") {
                    st.add_foreign_variable_prefix(
                        module.as_str(),
                        vec![format!("{}#files", module)],
                    );

                    packages_under_process.push(packages_under_process.last().ok_or_else(
                        || ftd::p1::Error::ParseError {
                            message: "The processing document stack is empty".to_string(),
                            doc_id: "".to_string(),
                            line_number: 0,
                        },
                    )?);

                    let current_package = packages_under_process.last().ok_or_else(|| {
                        ftd::p1::Error::ParseError {
                            message: "The processing document stack is empty".to_string(),
                            doc_id: "".to_string(),
                            line_number: 0,
                        }
                    })?;

                    if module.starts_with(current_package.name.as_str()) {
                        current_package
                            .get_font_ftd()
                            .unwrap_or_else(|| "".to_string())
                    } else {
                        let mut font_ftd = "".to_string();
                        for (alias, package) in current_package.aliases() {
                            if module.starts_with(alias) {
                                font_ftd = package.get_font_ftd().unwrap_or_else(|| "".to_string());
                                break;
                            }
                        }
                        font_ftd
                    }
                } else {
                    lib.get_with_result(module.as_str(), &mut packages_under_process)?
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
        }
    }
    Ok(document)
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

    return ftd::e2(format!("{} not found 1", variable).as_str(), doc_name, 0);

    fn get_assets_value(
        package: &fpm::Package,
        files: &str,
        lib: &fpm::Library,
        doc_name: &str,
        base_url: &str,
    ) -> ftd::p1::Result<ftd::Value> {
        let base_url = base_url.trim_end_matches('/');
        let path = lib.config.get_root_for_package(package);
        match files.rsplit_once(".") {
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
            None if path.join(files).exists() => Ok(ftd::Value::String {
                text: format!("/-/{}/{}", package.name, files),
                source: ftd::TextSource::Header,
            }),
            _ => ftd::e2(format!("{} not found 2", files).as_str(), doc_name, 0),
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
                let value = lib.process(&section, &state.tdoc(&mut Default::default()))?;
                s = state.continue_after_processor(&section, value)?;
            }
            ftd::Interpreter::StuckOnImport { module, state: st } => {
                let source =
                    lib.get_with_result(module.as_str(), &st.tdoc(&mut Default::default()))?;
                s = st.continue_after_import(module.as_str(), source.as_str())?;
            }
            ftd::Interpreter::StuckOnForeignVariable { variable, state } => {
                let value = resolve_ftd_foreign_variable(variable.as_str(), name)?;
                s = state.continue_after_variable(variable.as_str(), value)?
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
        _ => ftd::e2(format!("{} not found 3", variable).as_str(), doc_name, 0),
    }
}
