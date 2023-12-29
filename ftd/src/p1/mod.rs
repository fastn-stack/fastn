#[cfg(test)]
#[macro_use]
mod test;

pub(crate) mod header;
mod parser;
mod section;
pub mod utils;

pub use header::{AccessModifier, Header, Headers, Section as HSection, KV};
pub use parser::{parse, parse_with_line_number};
pub use section::Body;
pub use section::Section;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{doc_id}:{line_number} -> SectionNotFound")]
    SectionNotFound { doc_id: String, line_number: usize },

    #[error("{doc_id}:{line_number} -> MoreThanOneCaption")]
    MoreThanOneCaption { doc_id: String, line_number: usize },

    #[error("{doc_id}:{line_number} -> {message}")]
    ParseError {
        message: String,
        doc_id: String,
        line_number: usize,
    },

    #[error("{doc_id}:{line_number} -> MoreThanOneHeader for key {key}")]
    MoreThanOneHeader {
        key: String,
        doc_id: String,
        line_number: usize,
    },

    #[error("{doc_id}:{line_number} -> HeaderNotFound for key {key}")]
    HeaderNotFound {
        key: String,
        doc_id: String,
        line_number: usize,
    },

    #[error("forbidden usage: {message}, line_number: {line_number}, doc: {doc_id}")]
    ForbiddenUsage {
        message: String,
        doc_id: String,
        line_number: usize,
    },

    #[error("key not found: {key}, line number: {line_number}, doc: {doc_id}")]
    NotFound {
        doc_id: String,
        line_number: usize,
        key: String,
    },

    #[error("got more than one sub-sections: {key}, line number: {line_number}, doc: {doc_id}")]
    MoreThanOneSubSections {
        key: String,
        doc_id: String,
        line_number: usize,
    },

    #[error("serde error: {source}")]
    Serde {
        #[from]
        source: serde_json::Error,
    },

    #[error("syntect error: {source}")]
    Syntect {
        #[from]
        source: syntect::Error,
    },
}

pub type Result<T> = std::result::Result<T, Error>;
