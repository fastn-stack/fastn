extern crate self as fpm;

mod build;
mod config;
mod dependency;
mod diff;
mod document;
mod library;
mod mark;
mod snaphot;
mod status;
mod style;
mod sync;
mod tracks;
mod utils;

pub use build::build;
pub(crate) use config::Config;
pub(crate) use config::Package;
pub(crate) use dependency::Dependency;
pub use diff::diff;
pub(crate) use document::{process_dir, Document, FileFound, StaticAsset};
pub(crate) use library::Library;
pub use mark::mark;
pub(crate) use snaphot::Snapshot;
pub use status::status;
pub(crate) use style::Font;
pub use sync::sync;
pub use tracks::tracks;
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
