#[derive(serde::Deserialize, Debug, Clone)]
pub struct AutoImport {
    pub path: String,
    pub alias: Option<String>,
    pub exposing: Vec<String>,
}
