#[derive(PartialEq, Debug, Default, Clone, Serialize)]
pub struct DefinitionList {
    pub caption: crate::Rendered,
    pub list: Vec<(crate::Rendered, crate::Rendered)>,
}

impl ToString for DefinitionList {
    fn to_string(&self) -> String {
        todo!()
    }
}

impl DefinitionList {
    pub fn to_p1(&self) -> crate::p1::Section {
        todo!()
    }
}
