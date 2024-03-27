#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("ast: {0}")]
    Ast(#[from] ftd_ast::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
