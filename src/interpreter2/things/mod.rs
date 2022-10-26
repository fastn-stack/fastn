pub(crate) mod component;
pub mod default;
pub(crate) mod expression;
pub(crate) mod function;
pub(crate) mod kind;
pub(crate) mod record;
pub(crate) mod value;
pub(crate) mod variable;

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum Thing {
    Record(ftd::interpreter2::Record),
    Variable(ftd::interpreter2::Variable),
    Component(ftd::interpreter2::ComponentDefinition),
    Function(ftd::interpreter2::Function),
}

impl Thing {
    pub fn line_number(&self) -> usize {
        match self {
            Thing::Record(r) => r.line_number,
            Thing::Variable(v) => v.line_number,
            Thing::Component(c) => c.line_number,
            Thing::Function(f) => f.line_number,
        }
    }

    pub(crate) fn variable(
        self,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::interpreter2::Result<ftd::interpreter2::Variable> {
        match self {
            ftd::interpreter2::Thing::Variable(v) => Ok(v),
            t => ftd::interpreter2::utils::e2(
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
    ) -> ftd::interpreter2::Result<ftd::interpreter2::Record> {
        match self {
            ftd::interpreter2::Thing::Record(v) => Ok(v),
            t => ftd::interpreter2::utils::e2(
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
    ) -> ftd::interpreter2::Result<ftd::interpreter2::Function> {
        match self {
            ftd::interpreter2::Thing::Function(v) => Ok(v),
            t => ftd::interpreter2::utils::e2(
                format!("Expected Function, found: `{:?}`", t),
                doc_id,
                line_number,
            ),
        }
    }
}
