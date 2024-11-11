pub(crate) mod component;
pub mod default;
pub mod expression;
pub(crate) mod function;
pub(crate) mod kind;
pub(crate) mod or_type;
pub(crate) mod record;
pub(crate) mod value;
pub(crate) mod variable;
pub(crate) mod web_component;

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum ModuleThing {
    Component(ComponentModuleThing),
    Variable(VariableModuleThing),
    Formula(FormulaModuleThing),
}

impl ModuleThing {
    pub fn component(
        name: String,
        kind: fastn_type::KindData,
        arguments: Vec<ftd::interpreter::Argument>,
    ) -> Self {
        ModuleThing::Component(ComponentModuleThing::new(name, kind, arguments))
    }

    pub fn variable(name: String, kind: fastn_type::KindData) -> Self {
        ModuleThing::Variable(VariableModuleThing::new(name, kind))
    }

    pub fn function(name: String, kind: fastn_type::KindData) -> Self {
        ModuleThing::Formula(FormulaModuleThing::new(name, kind))
    }

    pub fn get_kind(&self) -> fastn_type::KindData {
        match self {
            ftd::interpreter::ModuleThing::Component(c) => c.kind.clone(),
            ftd::interpreter::ModuleThing::Variable(v) => v.kind.clone(),
            ftd::interpreter::ModuleThing::Formula(f) => f.kind.clone(),
        }
    }

    pub fn get_name(&self) -> String {
        match self {
            ftd::interpreter::ModuleThing::Component(c) => c.name.clone(),
            ftd::interpreter::ModuleThing::Variable(v) => v.name.clone(),
            ftd::interpreter::ModuleThing::Formula(f) => f.name.clone(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct VariableModuleThing {
    pub name: String,
    pub kind: fastn_type::KindData,
}

impl VariableModuleThing {
    pub fn new(name: String, kind: fastn_type::KindData) -> Self {
        VariableModuleThing { name, kind }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct FormulaModuleThing {
    pub name: String,
    pub kind: fastn_type::KindData,
}

impl FormulaModuleThing {
    pub fn new(name: String, kind: fastn_type::KindData) -> Self {
        FormulaModuleThing { name, kind }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ComponentModuleThing {
    pub name: String,
    pub kind: fastn_type::KindData,
    pub arguments: Vec<ftd::interpreter::Argument>,
}

impl ComponentModuleThing {
    pub fn new(
        name: String,
        kind: fastn_type::KindData,
        arguments: Vec<ftd::interpreter::Argument>,
    ) -> Self {
        ComponentModuleThing {
            name,
            kind,
            arguments,
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum Thing {
    Record(ftd::interpreter::Record),
    OrType(ftd::interpreter::OrType),
    OrTypeWithVariant {
        or_type: String,
        variant: ftd::interpreter::OrTypeVariant,
    },
    Variable(ftd::interpreter::Variable),
    Component(ftd::interpreter::ComponentDefinition),
    WebComponent(ftd::interpreter::WebComponentDefinition),
    Function(ftd::interpreter::Function),
    Export {
        from: String,
        to: String,
        line_number: usize,
    },
}

impl Thing {
    pub(crate) fn name(&self) -> String {
        match self {
            ftd::interpreter::Thing::Record(r) => r.name.clone(),
            ftd::interpreter::Thing::OrType(o) => o.name.clone(),
            ftd::interpreter::Thing::OrTypeWithVariant { or_type, .. } => or_type.clone(),
            ftd::interpreter::Thing::Variable(v) => v.name.to_string(),
            ftd::interpreter::Thing::Component(c) => c.name.to_string(),
            ftd::interpreter::Thing::Function(f) => f.name.to_string(),
            ftd::interpreter::Thing::WebComponent(w) => w.name.to_string(),
            ftd::interpreter::Thing::Export { to, .. } => to.to_string(),
        }
    }

    pub fn line_number(&self) -> usize {
        match self {
            Thing::Record(r) => r.line_number,
            Thing::Variable(v) => v.line_number,
            Thing::Component(c) => c.line_number,
            Thing::Function(f) => f.line_number,
            Thing::OrType(o) => o.line_number,
            Thing::OrTypeWithVariant { variant, .. } => variant.line_number(),
            Thing::WebComponent(w) => w.line_number,
            Thing::Export { line_number, .. } => *line_number,
        }
    }

    pub fn variable(
        self,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<ftd::interpreter::Variable> {
        match self {
            ftd::interpreter::Thing::Variable(v) => Ok(v),
            t => ftd::interpreter::utils::e2(
                format!("Expected Variable, found: `{:?}`", t),
                doc_id,
                line_number,
            ),
        }
    }

    pub(crate) fn record(
        self,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<ftd::interpreter::Record> {
        match self {
            ftd::interpreter::Thing::Record(v) => Ok(v),
            t => ftd::interpreter::utils::e2(
                format!("Expected Record, found: `{:?}`", t),
                doc_id,
                line_number,
            ),
        }
    }

    pub(crate) fn function(
        self,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<ftd::interpreter::Function> {
        match self {
            ftd::interpreter::Thing::Function(v) => Ok(v),
            t => ftd::interpreter::utils::e2(
                format!("Expected Function, found: `{:?}`", t),
                doc_id,
                line_number,
            ),
        }
    }

    pub(crate) fn component(self) -> Option<ftd::interpreter::ComponentDefinition> {
        match self {
            ftd::interpreter::Thing::Component(v) => Some(v),
            _ => None,
        }
    }
}
