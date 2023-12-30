use ftd::ftd2021;
pub use ftd::ftd2021::di::definition::Definition;
pub use ftd::ftd2021::di::import::Import;
pub use ftd::ftd2021::di::invocation::Invocation;
pub use ftd::ftd2021::di::main::DI;
pub use ftd::ftd2021::di::property::Property;
#[cfg(test)]
pub use ftd::ftd2021::di::property::Source;
pub use ftd::ftd2021::di::record::Record;

#[cfg(test)]
#[macro_use]
mod test;

mod definition;
mod import;
mod invocation;
mod main;
mod property;
mod record;
mod utils;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("P1Error: {}", _0)]
    P1Error(#[from] ftd::p1::Error),

    #[error("ASTParseError: {doc_id}:{line_number} -> {message}")]
    ParseError {
        message: String,
        doc_id: String,
        line_number: usize,
    },
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn parse_error<T, S1>(m: S1, doc_id: &str, line_number: usize) -> ftd2021::di::Result<T>
where
    S1: Into<String>,
{
    Err(Error::ParseError {
        message: m.into(),
        doc_id: doc_id.to_string(),
        line_number,
    })
}
