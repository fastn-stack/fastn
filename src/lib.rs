extern crate self as fpm;

#[macro_use]
extern crate lazy_static;

#[macro_use]
pub(crate) mod utils;

// Temp comment
mod auto_import;
mod commands;
mod config;
mod dependency;
mod document;
mod font;
mod i18n;
mod library;
mod snapshot;
mod tracker;
mod translation;

pub(crate) use auto_import::AutoImport;
pub(crate) use commands::build::process_file;
pub use commands::{
    build::build, diff::diff, mark_upto_date::mark_upto_date, start_tracking::start_tracking,
    status::status, stop_tracking::stop_tracking, sync::sync,
    translation_status::translation_status, update::update,
};
pub use config::Config;
pub(crate) use config::Package;
pub(crate) use dependency::Dependency;
pub(crate) use document::{get_documents, get_file, paths_to_files, Document, File, Static};
pub(crate) use font::Font;
pub(crate) use library::{FPMLibrary, Library};
pub(crate) use snapshot::Snapshot;
pub(crate) use tracker::Track;
pub(crate) use translation::{TranslatedDocument, TranslationData};
pub(crate) use utils::{copy_dir_all, get_timestamp_nanosecond};

pub const PACKAGE_INFO_INTERFACE: &str = "fifthtry.github.io/package-info";

fn ftd_html() -> &'static str {
    include_str!("../ftd.html")
}

fn fpm_ftd() -> &'static str {
    include_str!("../ftd/fpm.ftd")
}

fn fpm_js() -> &'static str {
    if fpm::utils::is_test() {
        return "FPM_JS";
    }
    include_str!("../fpm.js")
}

fn ftd_js() -> &'static str {
    if fpm::utils::is_test() {
        return "FTD_JS";
    }
    ftd::js()
}

fn ftd_css() -> &'static str {
    if fpm::utils::is_test() {
        return "FTD_CSS";
    }
    ftd::css()
}

fn fpm_lib_ftd() -> &'static str {
    include_str!("../ftd/fpm-lib.ftd")
}

fn with_fallback() -> &'static str {
    include_str!("../with-fallback.html")
}

fn with_message() -> &'static str {
    include_str!("../with-message.html")
}

fn available_languages(config: &fpm::Config) -> fpm::Result<String> {
    let path = config.root.join("FPM/translation/available-languages.ftd");
    Ok(if path.is_file() {
        std::fs::read_to_string(path)?
    } else {
        include_str!("../ftd/translation/available-languages.ftd").to_string()
    })
}

fn original_package_status(config: &fpm::Config) -> fpm::Result<String> {
    let path = config
        .root
        .join("FPM")
        .join("translation")
        .join("original-status.ftd");
    Ok(if path.is_file() {
        std::fs::read_to_string(path)?
    } else {
        let package_info_package = match config
            .package
            .get_dependency_for_interface(fpm::PACKAGE_INFO_INTERFACE)
        {
            Some(dep) => dep.package.name.as_str(),
            None => fpm::PACKAGE_INFO_INTERFACE,
        };
        let body_prefix = match config.package.generate_prefix_string(false) {
            Some(bp) => bp,
            None => String::new(),
        };
        format!(
            "{}\n\n-- import: {}/original-status as pi\n\n-- pi.original-status-page:",
            body_prefix, package_info_package
        )
    })
}

fn translation_package_status(config: &fpm::Config) -> fpm::Result<String> {
    let path = config
        .root
        .join("FPM")
        .join("translation")
        .join("translation-status.ftd");
    Ok(if path.is_file() {
        std::fs::read_to_string(path)?
    } else {
        let package_info_package = match config
            .package
            .get_dependency_for_interface(fpm::PACKAGE_INFO_INTERFACE)
        {
            Some(dep) => dep.package.name.as_str(),
            None => fpm::PACKAGE_INFO_INTERFACE,
        };
        let body_prefix = match config.package.generate_prefix_string(false) {
            Some(bp) => bp,
            None => String::new(),
        };
        format!(
            "{}\n\n-- import: {}/translation-status as pi\n\n-- pi.translation-status-page:",
            body_prefix, package_info_package
        )
    })
}

fn get_messages(status: &fpm::TranslatedDocument, config: &fpm::Config) -> fpm::Result<String> {
    Ok(match status {
        TranslatedDocument::Missing { .. } => {
            let path = config.root.join("FPM/translation/missing.ftd");
            if path.is_file() {
                std::fs::read_to_string(path)?
            } else {
                include_str!("../ftd/translation/missing.ftd").to_string()
            }
        }
        TranslatedDocument::NeverMarked { .. } => {
            let path = config.root.join("FPM/translation/never-marked.ftd");
            if path.is_file() {
                std::fs::read_to_string(path)?
            } else {
                include_str!("../ftd/translation/never-marked.ftd").to_string()
            }
        }
        TranslatedDocument::Outdated { .. } => {
            let path = config.root.join("FPM/translation/out-of-date.ftd");
            if path.is_file() {
                std::fs::read_to_string(path)?
            } else {
                include_str!("../ftd/translation/out-of-date.ftd").to_string()
            }
        }
        TranslatedDocument::UptoDate { .. } => {
            let path = config.root.join("FPM/translation/upto-date.ftd");
            if path.is_file() {
                std::fs::read_to_string(path)?
            } else {
                include_str!("../ftd/translation/upto-date.ftd").to_string()
            }
        }
    })
}

pub fn debug_env_vars() -> String {
    std::env::vars()
        .into_iter()
        .filter(|(key, _)| {
            vec!["CARGO", "VERGEN", "FPM"]
                .iter()
                .any(|prefix| key.starts_with(prefix))
        })
        .fold(String::new(), |consolidated_res, (key, value)| {
            format!("{}\n{}: {}", consolidated_res, key, value)
        })
}

// fn default_markdown() -> &'static str {
//     include_str!("../ftd/markdown.ftd")
// }

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

    #[error("PackageError: {message}")]
    PackageError { message: String },

    #[error("UsageError: {message}")]
    UsageError { message: String },

    #[error("IgnoreError: {}", _0)]
    IgnoreError(#[from] ignore::Error),

    #[error("FromPathBufError: {}", _0)]
    FromPathBufError(#[from] camino::FromPathBufError),
}

pub type Result<T> = std::result::Result<T, Error>;

pub(crate) fn usage_error<T>(message: String) -> Result<T> {
    Err(Error::UsageError { message })
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
