#[derive(serde::Deserialize, Debug, Clone)]
pub struct AutoImport {
    pub path: String,
    #[serde(rename = "as")]
    pub alias: Option<String>,
}
