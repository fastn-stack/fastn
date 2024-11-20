#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum ModuleThing {
    Component(ComponentModuleThing),
    Variable(VariableModuleThing),
    Formula(FormulaModuleThing),
}

impl ModuleThing {
    pub fn component(
        name: String,
        kind: fastn_resolved::KindData,
        arguments: Vec<fastn_resolved::Argument>,
    ) -> Self {
        ModuleThing::Component(ComponentModuleThing::new(name, kind, arguments))
    }

    pub fn variable(name: String, kind: fastn_resolved::KindData) -> Self {
        ModuleThing::Variable(VariableModuleThing::new(name, kind))
    }

    pub fn function(name: String, kind: fastn_resolved::KindData) -> Self {
        ModuleThing::Formula(FormulaModuleThing::new(name, kind))
    }

    pub fn get_kind(&self) -> fastn_resolved::KindData {
        match self {
            fastn_resolved::ModuleThing::Component(c) => c.kind.clone(),
            fastn_resolved::ModuleThing::Variable(v) => v.kind.clone(),
            fastn_resolved::ModuleThing::Formula(f) => f.kind.clone(),
        }
    }

    pub fn get_name(&self) -> String {
        match self {
            fastn_resolved::ModuleThing::Component(c) => c.name.clone(),
            fastn_resolved::ModuleThing::Variable(v) => v.name.clone(),
            fastn_resolved::ModuleThing::Formula(f) => f.name.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ComponentModuleThing {
    pub name: String,
    pub kind: fastn_resolved::KindData,
    pub arguments: Vec<fastn_resolved::Argument>,
}

impl ComponentModuleThing {
    pub fn new(
        name: String,
        kind: fastn_resolved::KindData,
        arguments: Vec<fastn_resolved::Argument>,
    ) -> Self {
        ComponentModuleThing {
            name,
            kind,
            arguments,
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct FormulaModuleThing {
    pub name: String,
    pub kind: fastn_resolved::KindData,
}

impl FormulaModuleThing {
    pub fn new(name: String, kind: fastn_resolved::KindData) -> Self {
        FormulaModuleThing { name, kind }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct VariableModuleThing {
    pub name: String,
    pub kind: fastn_resolved::KindData,
}

impl VariableModuleThing {
    pub fn new(name: String, kind: fastn_resolved::KindData) -> Self {
        VariableModuleThing { name, kind }
    }
}
