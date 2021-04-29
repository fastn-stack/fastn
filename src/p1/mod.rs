mod header;
mod parser;
mod section;
mod sub_section;
mod to_string;

pub use header::Header;
pub use parser::parse;
pub use section::Section;
pub use sub_section::{SubSection, SubSections};
pub use to_string::to_string;

use thiserror::Error as Error_;

#[derive(Error_, Debug)]
pub enum Error {
    #[error("invalid input: {message}")]
    InvalidInput { message: String, context: String },

    #[error("file was empty")]
    EmptyFile,
    #[error("key not found: {key}")]
    NotFound { key: String },
    #[error("cant parse integer")]
    CantParseInt {
        #[from]
        source: std::num::ParseIntError,
    },
    #[error("cant parse bool")]
    CantParseBool,
    #[error("cant parse float")]
    CantParseFloat {
        #[from]
        source: std::num::ParseFloatError,
    },
}

pub type Result<T> = std::result::Result<T, Error>;
