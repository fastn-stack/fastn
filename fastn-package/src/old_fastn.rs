pub fn fastn_ftd_2021() -> &'static str {
    include_str!("../fastn_2021.ftd")
}

pub fn parse_old_fastn(
    source: &str,
) -> Result<ftd::ftd2021::p2::Document, fastn_issues::initialization::OldFastnParseError> {
    let mut s = ftd::ftd2021::interpret("FASTN", source, &None)?;
    let document;
    loop {
        match s {
            ftd::ftd2021::Interpreter::Done { document: doc } => {
                document = doc;
                break;
            }
            ftd::ftd2021::Interpreter::StuckOnProcessor { section, .. } => {
                return Err(
                    fastn_issues::initialization::OldFastnParseError::ProcessorUsed {
                        processor: section
                            .header
                            .str("FASTN.ftd", section.line_number, ftd::PROCESSOR_MARKER)
                            .expect("we cant get stuck on processor without processor marker")
                            .to_string(),
                    },
                );
            }
            ftd::ftd2021::Interpreter::StuckOnImport { module, state: st } => {
                let source = if module == "fastn" {
                    fastn_ftd_2021()
                } else {
                    return Err(
                        fastn_issues::initialization::OldFastnParseError::InvalidImport { module },
                    );
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

pub fn get_name(
    doc: ftd::ftd2021::p2::Document,
) -> Result<String, fastn_issues::initialization::GetNameError> {
    let op: Option<PackageTemp> = doc.get(fastn_package::FASTN_PACKAGE_VARIABLE)?;
    match op {
        Some(p) => Ok(p.name),
        None => Err(fastn_issues::initialization::GetNameError::PackageIsNone),
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

#[derive(serde::Deserialize, Debug, Clone, PartialEq)]
pub struct EndpointData {
    pub endpoint: String,
    pub mountpoint: String,
    #[serde(rename = "user-id")]
    pub user_id: Option<bool>,
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
    pub about: Option<String>,
    pub zip: Option<String>,
    #[serde(rename = "download-base-url")]
    pub download_base_url: Option<String>,
    #[serde(rename = "canonical-url")]
    pub canonical_url: Option<String>,
    #[serde(rename = "inherit-auto-imports-from-original")]
    pub import_auto_imports_from_original: bool,
    pub favicon: Option<String>,
    pub endpoint: Vec<EndpointData>,
    pub backend: bool,
    #[serde(rename = "backend-headers")]
    pub backend_headers: Option<Vec<BackendHeader>>,
    pub icon: Option<ftd::ImageSrc>,
    // This will contain the module name through which this package can
    // be accessed when considered as a system's package
    pub system: Option<String>,
    #[serde(rename = "system-is-confidential")]
    pub system_is_confidential: Option<bool>,
    #[serde(rename = "default-language")]
    pub default_language: Option<String>,
    pub lang: Option<String>,
    #[serde(rename = "translation-en")]
    pub translation_en: Option<String>,
    #[serde(rename = "translation-hi")]
    pub translation_hi: Option<String>,
    #[serde(rename = "translation-zh")]
    pub translation_zh: Option<String>,
    #[serde(rename = "translation-es")]
    pub translation_es: Option<String>,
    #[serde(rename = "translation-ar")]
    pub translation_ar: Option<String>,
    #[serde(rename = "translation-pt")]
    pub translation_pt: Option<String>,
    #[serde(rename = "translation-ru")]
    pub translation_ru: Option<String>,
    #[serde(rename = "translation-fr")]
    pub translation_fr: Option<String>,
    #[serde(rename = "translation-de")]
    pub translation_de: Option<String>,
    #[serde(rename = "translation-ja")]
    pub translation_ja: Option<String>,
    #[serde(rename = "translation-bn")]
    pub translation_bn: Option<String>,
    #[serde(rename = "translation-ur")]
    pub translation_ur: Option<String>,
    #[serde(rename = "translation-id")]
    pub translation_id: Option<String>,
    #[serde(rename = "translation-tr")]
    pub translation_tr: Option<String>,
    #[serde(rename = "translation-vi")]
    pub translation_vi: Option<String>,
    #[serde(rename = "translation-it")]
    pub translation_it: Option<String>,
    #[serde(rename = "translation-pl")]
    pub translation_pl: Option<String>,
    #[serde(rename = "translation-th")]
    pub translation_th: Option<String>,
    #[serde(rename = "translation-nl")]
    pub translation_nl: Option<String>,
    #[serde(rename = "translation-ko")]
    pub translation_ko: Option<String>,
    // #[serde(flatten, deserialize_with = "deserialize_languages")]
    // pub other_languages: Option<Vec<Lang>>,
}

// #[derive(serde::Deserialize, Debug, Clone)]
// pub struct Lang {
//     pub lang: String,
//     pub module: String,
// }

// fn deserialize_languages<'de, D>(deserializer: D) -> Result<Option<Vec<Lang>>, D::Error>
// where
//     D: serde::de::Deserializer<'de>,
// {
//     struct LanguageDataVisitor;

//     impl<'de> serde::de::Visitor<'de> for LanguageDataVisitor {
//         type Value = Option<Vec<Lang>>;

//         fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
//             formatter.write_str("a map with language properties")
//         }

//         fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
//         where
//             M: serde::de::MapAccess<'de>,
//         {
//             let mut languages: Vec<Lang> = vec![];

//             while let Some((key, value)) = access.next_entry::<String, String>()? {
//                 dbg!(&key);
//                 if dbg!(key.starts_with("lang-")) {
//                     languages.push(Lang {
//                         lang: key.trim().trim_start_matches("lang-").to_string(),
//                         module: value.trim().to_string(),
//                     });
//                 }
//             }

//             Ok(if languages.is_empty() { None } else { Some(languages) })
//         }
//     }

//     deserializer.deserialize_map(LanguageDataVisitor)
// }
