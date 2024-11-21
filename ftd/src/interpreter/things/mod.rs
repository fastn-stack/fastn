pub(crate) mod component;
pub mod expression;
pub(crate) mod function;
pub(crate) mod kind;
pub(crate) mod or_type;
pub(crate) mod record;
pub(crate) mod value;
pub(crate) mod variable;
pub(crate) mod web_component;

pub type Thing = fastn_resolved::Definition;

pub trait ThingExt {
    fn variable(
        self,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<fastn_resolved::Variable>;
    fn record(
        &self,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<&fastn_resolved::Record>;
    fn web_component(
        &self,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<&fastn_resolved::WebComponentDefinition>;
    fn function(
        &self,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<&fastn_resolved::Function>;
}

impl ThingExt for Thing {
    fn variable(
        self,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<fastn_resolved::Variable> {
        match self {
            ftd::interpreter::Thing::Variable(v) => Ok(v),
            t => ftd::interpreter::utils::e2(
                format!("Expected Variable, found: `{:?}`", t),
                doc_id,
                line_number,
            ),
        }
    }

    fn record(
        &self,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<&fastn_resolved::Record> {
        match self {
            ftd::interpreter::Thing::Record(v) => Ok(v),
            t => ftd::interpreter::utils::e2(
                format!("Expected Record, found: `{:?}`", t),
                doc_id,
                line_number,
            ),
        }
    }

    fn web_component(
        &self,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<&fastn_resolved::WebComponentDefinition> {
        match self {
            ftd::interpreter::Thing::WebComponent(v) => Ok(v),
            t => ftd::interpreter::utils::e2(
                format!("Expected WebComponent, found: `{:?}`", t),
                doc_id,
                line_number,
            ),
        }
    }

    fn function(
        &self,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::interpreter::Result<&fastn_resolved::Function> {
        match self {
            ftd::interpreter::Thing::Function(v) => Ok(v),
            t => ftd::interpreter::utils::e2(
                format!("Expected Function, found: `{:?}`", t),
                doc_id,
                line_number,
            ),
        }
    }
}
