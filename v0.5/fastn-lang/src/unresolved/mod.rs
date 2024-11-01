mod parser;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Document {
    pub module_doc: Option<fastn_lang::Span>,
    pub imports: Vec<fastn_lang::unresolved::Import>,
    pub definitions: std::collections::HashMap<fastn_lang::Identifier, Definition>,
    pub content: Vec<fastn_lang::Section>,
    pub errors: Vec<fastn_lang::Spanned<fastn_lang::Error>>,
    pub comments: Vec<fastn_lang::Span>,
    pub line_starts: Vec<usize>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Definition {
    Component(fastn_lang::Section),
    Variable(fastn_lang::Section),
    Function(fastn_lang::Section),
    TypeAlias(fastn_lang::Section),
    Record(fastn_lang::Section),
    OrType(fastn_lang::Section),
    Module(fastn_lang::Section),
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Import {
    pub module: fastn_lang::ModuleName,
    pub exports: Option<Export>,
    pub exposing: Option<Export>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum Export {
    All,
    Things(Vec<fastn_lang::AliasableIdentifier>),
}
