pub async fn process(
    value: ftd::ast::VariableValue,
    kind: ftd::interpreter::Kind,
    doc: &ftd::interpreter::TDoc<'_>,
    req_config: &mut fastn_core::RequestConfig,
) -> ftd::interpreter::Result<ftd::interpreter::Value> {
    let current_language =
        req_config
            .config
            .package
            .current_language_meta()
            .unwrap_or(LanguageMeta {
                id: "en".to_string(),
                id3: "eng".to_string(),
                human: "English".to_string(),
                is_active: true,
            });
    dbg!(&current_language);
    let available_languages = req_config.config.package.available_languages_meta();
    dbg!(&available_languages);
    let result = LanguageData {
        current_language,
        available_languages,
    };
    doc.from_json(&result, &kind, &value)
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
    #[serde(rename = "is-active")]
    pub is_active: bool,
}
