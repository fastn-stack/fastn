#[derive(Debug)]
pub struct OrType {
    pub name: String,
    pub variant: fastn_js::SetPropertyValue,
    pub prefix: Option<String>,
}
