pub fn fastn_ftd() -> &'static str {
    include_str!("../fastn.ftd")
}

#[derive(thiserror::Error, Debug)]
pub enum OldFastnParseError {
    #[error("FASTN.ftd is invalid ftd: {source}")]
    FTDError {
        #[from]
        source: ftd::ftd2021::p1::Error,
    },
    #[error("FASTN.ftd imported something other then fastn: {module}")]
    InvalidImport { module: String },
}

pub fn parse_old_fastn(source: &str) -> Result<ftd::ftd2021::p2::Document, OldFastnParseError> {
    let mut s = ftd::ftd2021::interpret("FASTN", source, &None)?;
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
                let source = if module == "fastn" {
                    fastn_ftd()
                } else {
                    return Err(OldFastnParseError::InvalidImport { module });
                };
                s = st.continue_after_import(module.as_str(), source)?;
            }
            ftd::ftd2021::Interpreter::StuckOnForeignVariable { .. } => {
                unimplemented!()
            }
            ftd::ftd2021::Interpreter::CheckID { .. } => {
                // No config in fastn_package::old_fastn::fastnLibrary ignoring processing terms here
                unimplemented!()
            }
        }
    }
    Ok(document)
}
