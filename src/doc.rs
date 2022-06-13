// TODO: make async
pub async fn parse<'a>(
    name: &str,
    source: &str,
    lib: &'a fpm::Library,
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
                let value = lib
                    .process(&section, &state.tdoc(&mut Default::default()))
                    .await?;
                s = state.continue_after_processor(&section, value)?;
            }
            ftd::Interpreter::StuckOnImport {
                module,
                state: mut st,
            } => {
                let source = if module.eq("fpm/time") {
                    st.add_foreign_variable_prefix(module.as_str());
                    "".to_string()
                } else {
                    lib.get_with_result(module.as_str(), &st.tdoc(&mut Default::default()))?
                };
                s = st.continue_after_import(module.as_str(), source.as_str())?;
            }
            ftd::Interpreter::StuckOnForeignVariable { variable, state } => {
                let value = resolve_foreign_variable(variable.as_str(), name)?;
                s = state.continue_after_variable(variable.as_str(), value)?
            }
        }
    }
    Ok(document)
}

fn resolve_foreign_variable(variable: &str, doc_name: &str) -> ftd::p1::Result<ftd::Value> {
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
        _ => ftd::e2(format!("{} not found", variable).as_str(), doc_name, 0),
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
                let value = resolve_foreign_variable(variable.as_str(), name)?;
                s = state.continue_after_variable(variable.as_str(), value)?
            }
        }
    }
    Ok(document)
}
