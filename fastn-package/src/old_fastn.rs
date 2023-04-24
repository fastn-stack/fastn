pub fn fastn_ftd() -> &'static str {
    include_str!("../fastn_2021.ftd")
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
    #[error("FASTN.ftd tried to use a processor: {processor}")]
    ProcessorUsed { processor: String },
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
            ftd::ftd2021::Interpreter::StuckOnProcessor { section, .. } => {
                return Err(OldFastnParseError::ProcessorUsed {
                    processor: section
                        .header
                        .str("FASTN.ftd", section.line_number, ftd::PROCESSOR_MARKER)
                        .expect("we cant get stuck on processor without processor marker")
                        .to_string(),
                })
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
                unreachable!("we never register any foreign variable so we cant come here")
            }
            ftd::ftd2021::Interpreter::CheckID { .. } => {
                unimplemented!()
            }
        }
    }
    Ok(document)
}

#[derive(thiserror::Error, Debug)]
pub enum GetNameError {
    #[error("Can't find fastn.package in FASTN.ftd, must be impossible: {source}")]
    CantFindPackage {
        #[from]
        source: ftd::ftd2021::p1::Error,
    },
    #[error("fastn.package was not initialised")]
    PackageIsNone,
}

pub fn get_name(doc: ftd::ftd2021::p2::Document) -> Result<String, GetNameError> {
    let op: Option<PackageTemp> = doc.get("fastn#package")?;
    match op {
        Some(p) => Ok(p.name),
        None => Err(GetNameError::PackageIsNone),
    }
}

/// Backend Header is a struct that is used to read and store the backend-header from the FASTN.ftd file
#[derive(serde::Deserialize, Debug, Clone)]
pub struct BackendHeader {
    #[serde(rename = "header-key")]
    pub header_key: String,
    #[serde(rename = "header-value")]
    pub header_value: String,
}

/// PackageTemp is a struct that is used for mapping the `fastn.package` data in FASTN.ftd file. It is
/// not used elsewhere in program, it is immediately converted to `fastn_core::Package` struct during
/// deserialization process
#[derive(serde::Deserialize, Debug, Clone)]
pub struct PackageTemp {
    pub name: String,
    pub versioned: bool,
    #[serde(rename = "translation-of")]
    pub translation_of: Option<String>,
    #[serde(rename = "translation")]
    pub translations: Vec<String>,
    #[serde(rename = "language")]
    pub language: Option<String>,
    pub about: Option<String>,
    pub zip: Option<String>,
    #[serde(rename = "download-base-url")]
    pub download_base_url: Option<String>,
    #[serde(rename = "canonical-url")]
    pub canonical_url: Option<String>,
    #[serde(rename = "inherit-auto-imports-from-original")]
    pub import_auto_imports_from_original: bool,
    #[serde(rename = "favicon")]
    pub favicon: Option<String>,
    #[serde(rename = "endpoint")]
    pub endpoint: Option<String>,
    #[serde(rename = "backend")]
    pub backend: bool,
    #[serde(rename = "backend-headers")]
    pub backend_headers: Option<Vec<BackendHeader>>,
    #[serde(rename = "icon")]
    pub icon: Option<ftd::ImageSrc>,
}
