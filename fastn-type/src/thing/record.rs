#[derive(Debug, Default, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Record {
    pub name: String,
    pub fields: Vec<Field>,
    pub line_number: usize,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Field {
    pub name: String,
    pub kind: fastn_type::KindData,
    pub mutable: bool,
    pub value: Option<fastn_type::PropertyValue>,
    pub line_number: usize,
    pub access_modifier: AccessModifier,
}

#[derive(Debug, Default, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub enum AccessModifier {
    #[default]
    Public,
    Private,
}
