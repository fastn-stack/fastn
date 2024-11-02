#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub enum Warning {
    // say someone did `-- import: foo as foo`, this is not an error but a warning
    AliasNotNeeded,
}
