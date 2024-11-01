#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Definition {
    Component(fastn_p1::Section),
    Variable(fastn_p1::Section),
    Function(fastn_p1::Section),
    TypeAlias(fastn_p1::Section),
    Record(fastn_p1::Section),
    OrType(fastn_p1::Section),
    Module(fastn_p1::Section),
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Import {
    pub module: fastn_p1::ModuleName,
    pub exports: Option<Export>,
    pub exposing: Option<Export>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum Export {
    All,
    Things(Vec<fastn_p1::AliasableIdentifier>),
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Document {
    pub module_doc: Option<fastn_p1::Span>,
    pub imports: Vec<fastn_p1::unresolved::Import>,
    pub definitions: std::collections::HashMap<fastn_p1::Identifier, Definition>,
    pub content: Vec<fastn_p1::Section>,
    pub errors: Vec<fastn_p1::Spanned<fastn_p1::SingleError>>,
    pub comments: Vec<fastn_p1::Span>,
    pub line_starts: Vec<usize>,
}
