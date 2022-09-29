pub(crate) mod expression;
pub(crate) mod kind;
pub(crate) mod record;
pub(crate) mod value;
pub(crate) mod variable;

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum Thing {
    Record(ftd::interpreter2::Record),
    Variable(ftd::interpreter2::Variable),
}
