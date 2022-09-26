pub(crate) mod kind;
pub(crate) mod record;
pub(crate) mod value;

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum Thing {
    Record(ftd::interpreter2::Record),
}
