#[derive(Debug)]
pub struct RecordInstance {
    pub name: String,
    pub fields: fastn_js::SetPropertyValue,
    pub prefix: Option<String>,
}
