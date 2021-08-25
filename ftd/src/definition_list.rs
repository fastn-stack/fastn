#[derive(PartialEq, Debug, Default, Clone, serde_derive::Serialize)]
pub struct DefinitionList {
    pub id: Option<String>,
    pub caption: crate::Rendered,
    pub list: Vec<(crate::Rendered, crate::Rendered)>,
}

impl DefinitionList {
    pub fn to_p1(&self) -> crate::p1::Section {
        todo!()
    }
}
