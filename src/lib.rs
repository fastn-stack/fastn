extern crate self as fpm;

mod commands;
mod config;
mod dependency;
mod document;
mod library;
mod snaphot;
mod style;
mod track_data;
mod utils;

pub use commands::{
    build::build, diff::diff, mark::mark, status::status, sync::sync, tracks::tracks,
};
pub(crate) use config::Config;
pub(crate) use config::Package;
pub(crate) use dependency::Dependency;
pub(crate) use document::{process_dir, process_file, Document, FileFound, StaticAsset};
pub(crate) use library::Library;
pub(crate) use snaphot::Snapshot;
pub(crate) use style::Font;
pub(crate) use track_data::Tracks;
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

pub async fn ensure_dependencies(deps: Vec<fpm::Dependency>) -> Result<()> {
    futures::future::join_all(
        deps.into_iter()
            .map(|x| tokio::spawn(async move { x.process().await }))
            .collect::<Vec<tokio::task::JoinHandle<bool>>>(),
    )
    .await;
    Ok(())
}

pub fn ignore_history() -> Option<ignore::overrides::Override> {
    let mut overrides = ignore::overrides::OverrideBuilder::new("./");
    overrides.add("!.history").unwrap();
    overrides.add("!FPM").unwrap();
    overrides.build().ok()
}

#[cfg(test)]
mod tests {

    #[test]
    fn fbt() {
        if fbt_lib::main().is_some() {
            panic!("test failed")
        }
    }
}
