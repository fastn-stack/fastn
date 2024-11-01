mod parser;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Document {
    pub module_doc: Option<fastn_parser::Span>,
    pub imports: Vec<fastn_parser::unresolved::Import>,
    pub definitions: std::collections::HashMap<fastn_parser::Identifier, Definition>,
    pub content: Vec<fastn_parser::Section>,
    pub errors: Vec<fastn_parser::Spanned<fastn_parser::Error>>,
    pub comments: Vec<fastn_parser::Span>,
    pub line_starts: Vec<usize>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Definition {
    Component(fastn_parser::Section),
    Variable(fastn_parser::Section),
    Function(fastn_parser::Section),
    TypeAlias(fastn_parser::Section),
    Record(fastn_parser::Section),
    OrType(fastn_parser::Section),
    Module(fastn_parser::Section),
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Import {
    pub module: fastn_parser::ModuleName,
    pub exports: Option<Export>,
    pub exposing: Option<Export>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum Export {
    All,
    Things(Vec<fastn_parser::AliasableIdentifier>),
}
