#[derive(Debug)]
pub struct OrType {
    pub name: String,
    pub variants: fastn_js::SetPropertyValue,
    pub prefix: Option<String>,
}
