extern crate self as fpm;

mod build;
mod check;
mod config;
mod dependency;
mod document;
mod library;
mod sync;
mod utils;

pub use build::build;
pub use check::check;
pub use config::Package;
pub use dependency::{Dependency, DependencyProvider};
pub(crate) use document::{process_dir, Document};
pub use library::Library;
pub use sync::sync;
pub(crate) use utils::get_timestamp_nanosecond;

pub fn fpm_ftd() -> &'static str {
    include_str!("../fpm.ftd")
}

pub fn never_expected<'de, D, T>(_deserializer: D) -> std::result::Result<T, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    unreachable!("must never happen")
}

#[derive(thiserror::Error, Debug, serde_derive::Deserialize)]
pub enum Error {
    #[serde(deserialize_with = "never_expected")]
    #[error("HttpError: {}", _0)]
    HttpError(#[from] reqwest::Error),

    #[serde(deserialize_with = "never_expected")]
    #[error("IoError: {}", _0)]
    IoError(#[from] std::io::Error),

    #[serde(deserialize_with = "never_expected")]
    #[error("IoError: {}", _0)]
    ZipError(#[from] zip::result::ZipError),

    #[error("{doc_id}:{line_number} -> {message}")]
    ParseError {
        message: String,
        doc_id: String,
        line_number: usize,
    },
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {

    #[test]
    fn fbt() {
        if fbt_lib::main().is_some() {
            panic!("test failed")
        }
    }
}
