#![deny(unused_crate_dependencies)]

extern crate self as ftd_p1;

pub type Map<T> = std::collections::BTreeMap<String, T>;

#[cfg(test)]
#[macro_use]
mod test;

pub(crate) mod header;
mod parser;
mod section;
pub mod utils;

pub use header::{AccessModifier, BlockRecordHeader, Header, Headers, SectionHeader, KV};
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
}

pub type Result<T> = std::result::Result<T, Error>;
