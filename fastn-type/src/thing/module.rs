#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum ModuleThing {
    Component(ComponentModuleThing),
    Variable(VariableModuleThing),
    Formula(FormulaModuleThing),
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ComponentModuleThing {
    pub name: String,
    pub kind: fastn_type::KindData,
    pub arguments: Vec<fastn_type::Argument>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct VariableModuleThing {
    pub name: String,
    pub kind: fastn_type::KindData,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct FormulaModuleThing {
    pub name: String,
    pub kind: fastn_type::KindData,
}
