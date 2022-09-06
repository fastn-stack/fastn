mod things;

pub use things::expression::Boolean;
pub use things::kind::Kind;
pub use things::property_value::PropertyValue;
pub use things::property_value::Value;
pub use things::variable::Variable;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("P1Error: {}", _0)]
    P1Error(#[from] ftd::p11::Error),

    #[error("InvalidKind: {doc_id}:{line_number} -> {message}")]
    InvalidKind {
        doc_id: String,
        line_number: usize,
        message: String,
    },
}

pub type Result<T> = std::result::Result<T, Error>;
