#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("ast: {0}")]
    Ast(#[from] ftd_ast::Error),

    #[error("ftd-p1: {name}")]
    NotAComponent {
        name: String,
        usage_document: ftd_tc::DocumentID,
        usage_line: usize,
        found: Box<ftd_tc::Qualified<ftd_tc::Type>>,
    },
}

pub type Result<T> = std::result::Result<T, Error>;
