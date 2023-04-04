mod header;
mod parser;
mod section;
mod sub_section;
mod to_string;

pub use header::Header;
pub use parser::{parse, parse_file_for_global_ids};
pub use section::Section;
pub use sub_section::{SubSection, SubSections};
pub use to_string::to_string;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{doc_id}:{line_number} -> {message}")]
    ParseError {
        message: String,
        doc_id: String,
        line_number: usize,
    },

    #[error("unknown data: {message}, line_number: {line_number}, doc: {doc_id}")]
    UnknownData {
        message: String,
        doc_id: String,
        line_number: usize,
    },

    #[error("missing data: {message}, line_number: {line_number}, doc: {doc_id}")]
    MissingData {
        message: String,
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
