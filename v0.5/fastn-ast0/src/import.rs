#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Import {
    pub module: fastn_p1::ModuleName,
    pub exports: Option<Export>,
    pub exposing: Option<Exposing>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum Export {
    All,
    Things(Vec<fastn_p1::AliasableIdentifier>),
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum Exposing {
    All,
    Things(Vec<fastn_p1::AliasableIdentifier>),
}
