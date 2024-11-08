#[derive(Debug, Default, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct OrType {
    pub name: String,
    pub variants: Vec<OrTypeVariant>,
    pub line_number: usize,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum OrTypeVariant {
    AnonymousRecord(fastn_type::Record),
    Regular(fastn_type::Field),
    Constant(fastn_type::Field),
}
