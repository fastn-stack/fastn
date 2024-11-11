pub async fn process(
    value: ftd_ast::VariableValue,
    kind: fastn_type::Kind,
    doc: &ftd::interpreter::TDoc<'_>,
    req_config: &mut fastn_core::RequestConfig,
) -> ftd::interpreter::Result<ftd::interpreter::Value> {
    let current_language = req_config.config.package.current_language_meta()?;
    let available_languages = req_config.config.package.available_languages_meta()?;
    doc.from_json(
        &LanguageData::new(current_language, available_languages),
        &kind,
        &value,
    )
}

#[derive(Default, Debug, serde::Serialize)]
pub struct LanguageData {
    #[serde(rename = "current-language")]
    pub current_language: LanguageMeta,
    #[serde(rename = "available-languages")]
    pub available_languages: Vec<LanguageMeta>,
}

#[derive(Default, Debug, serde::Serialize)]
pub struct LanguageMeta {
    pub id: String,
    pub id3: String,
    pub human: String,
    #[serde(rename = "is-current")]
    pub is_current: bool,
}

impl LanguageData {
    pub fn new(current_language: LanguageMeta, available_languages: Vec<LanguageMeta>) -> Self {
        LanguageData {
            current_language,
            available_languages,
        }
    }
}
