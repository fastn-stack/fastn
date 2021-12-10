extern crate self as fpm;

mod commands;
mod config;
mod dependency;
mod document;
mod font;
mod library;
mod snapshot;
mod tracker;
mod utils;

pub use commands::{
    build::build, diff::diff, mark_upto_date::mark_upto_date, start_tracking::start_tracking,
    status::status, sync::sync,
};
pub(crate) use config::Config;
pub(crate) use config::Package;
pub(crate) use dependency::Dependency;
pub(crate) use document::{process_dir, process_file, Document, File, StaticAsset};
pub(crate) use font::Font;
pub(crate) use library::Library;
pub(crate) use snapshot::Snapshot;
pub(crate) use tracker::Track;
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

    #[error("ConfigurationError: {message}")]
    ConfigurationError { message: String },
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
