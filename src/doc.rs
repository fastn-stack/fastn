pub fn parse(name: &str, source: &str, lib: &fpm::Library) -> ftd::p1::Result<ftd::p2::Document> {
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
        }
    }
    Ok(document)
}

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
        }
    }
    Ok(document)
}
