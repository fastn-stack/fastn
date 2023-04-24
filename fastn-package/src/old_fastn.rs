#[derive(Default)]
pub struct FastnLibrary {}

pub fn fastn_ftd() -> &'static str {
    include_str!("../fastn.ftd")
}

impl FastnLibrary {
    pub fn get(&self, name: &str, _doc: &ftd::ftd2021::p2::TDoc) -> Option<String> {
        if name == "fastn" {
            Some(format!(
                "{}\n\n-- optional package-data package:\n",
                fastn_ftd()
            ))
        } else {
            // Note: currently we do not allow users to import other modules from FASTN.ftd
            eprintln!("FASTN.ftd can only import `fastn` module");
            None
        }
    }

    pub fn get_with_result(
        &self,
        name: &str,
        doc: &ftd::ftd2021::p2::TDoc,
    ) -> ftd::ftd2021::p1::Result<String> {
        match self.get(name, doc) {
            Some(v) => Ok(v),
            None => ftd::ftd2021::p2::utils::e2(format!("library not found: {}", name), "", 0),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum FastnDocError {
    #[error("fastn.ftd error: {source}")]
    FailedToParse {
        #[from]
        source: fastn_package::initializer::FileAsStringError,
    },
}

// pub async fn doc(path: &camino::Utf8Path) -> Result<ftd::ftd2021::p2::Document, FastnDocError> {
//     {
//         let doc = tokio::fs::read_to_string(path).await?;
//         let lib = fastn_package::old_fastn::FastnLibrary::default();
//         match parse_ftd("fastn", doc.as_str(), &lib) {
//             Ok(v) => Ok(v),
//             Err(e) => Err(fastn_core::Error::PackageError {
//                 message: format!("failed to parse FASTN.ftd: {:?}", &e),
//             }),
//         }
//     }
// }

pub fn parse_ftd(
    name: &str,
    source: &str,
    lib: &fastn_package::old_fastn::FastnLibrary,
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
                // No config in fastn_package::old_fastn::fastnLibrary ignoring processing terms here
                unimplemented!()
            }
        }
    }
    Ok(document)
}
