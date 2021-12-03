extern crate self as fpm;

mod build;
mod config;
mod dependency;
mod document;
mod library;
mod style;
mod sync;
mod toc;
mod utils;

pub use build::build;
pub(crate) use config::Config;
pub(crate) use config::Package;
pub(crate) use dependency::Dependency;
pub(crate) use document::{process_dir, Document};
pub(crate) use library::Library;
pub(crate) use style::Font;
pub use sync::sync;
pub(crate) use utils::get_timestamp_nanosecond;

pub fn fpm_ftd() -> &'static str {
    include_str!("../fpm.ftd")
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("HttpError: {}", _0)]
    HttpError(#[from] reqwest::Error),

    #[error("IoError: {}", _0)]
    IoError(#[from] std::io::Error),

    #[error("IoError: {}", _0)]
    ZipError(#[from] zip::result::ZipError),

    #[error("FTDError: {}", _0)]
    FTDError(#[from] ftd::p1::Error),

    #[error("{line_number}: -> {message}")]
    ConfigurationParseError { message: String, line_number: usize },
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
