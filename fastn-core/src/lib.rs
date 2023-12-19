extern crate self as fastn_core;

#[macro_use]
pub mod utils;
mod apis;
mod auto_import;
mod cache;
pub mod commands;
mod config;
mod controller;
mod cr;
mod doc;
mod file;
mod font;
mod history;
mod package;
pub mod tutor;
pub(crate) mod watcher;
#[macro_use]
mod http;
mod auth;
mod ds;
mod error;
mod i18n;
pub mod library;
mod proxy;
pub mod sitemap;
mod snapshot;
mod sync_utils;
mod track;
mod tracker;
mod translation;
mod version;
// mod wasm;
pub(crate) mod catch_panic;
pub(crate) mod google_sheets;
mod library2022;
mod workspace;

pub(crate) use auto_import::AutoImport;
pub use commands::{
    abort_merge::abort_merge, add::add, build::build, check::post_build_check, clone::clone,
    close_cr::close_cr, create_cr::create_cr, create_package::create_package, diff::diff,
    edit::edit, mark_resolved::mark_resolved, mark_upto_date::mark_upto_date, merge::merge,
    query::query, resolve_conflict::resolve_conflict, revert::revert, rm::rm, serve::listen,
    start_tracking::start_tracking, status::status, sync2::sync2, test::test,
    translation_status::translation_status, update::update,
};
pub use config::{Config, FTDEdition, RequestConfig};
pub use error::Error;
pub use file::File;
pub(crate) use file::{get_file, paths_to_files, Document, Static};
pub(crate) use font::Font;
pub use library::{FastnLibrary, Library, Library2};
pub use library2022::Library2022;
pub(crate) use package::dependency::Dependency;
pub use package::user_group;
pub(crate) use package::Package;
pub(crate) use snapshot::Snapshot;
pub(crate) use tracker::Track;
pub(crate) use translation::{TranslatedDocument, TranslationData};
pub(crate) use utils::{copy_dir_all, timestamp_nanosecond};
pub(crate) use version::Version;
pub use {doc::resolve_foreign_variable2, doc::resolve_import};

pub const FASTN_UI_INTERFACE: &str = "fastn-stack.github.io/fastn-ui";
pub const PACKAGE_THEME_INTERFACE: &str = "ftd-lang.github.io/theme";
pub const NUMBER_OF_CRS_TO_RESERVE: usize = 5;

pub const IMAGE_EXT: &[&str] = &["jpg", "png", "svg"];

pub const VIDEO_EXT: &[&str] = &["mp4", "ogg", "webm"];

pub fn ftd_html() -> &'static str {
    include_str!("../ftd_2022.html")
}

fn processor_ftd() -> &'static str {
    include_str!("../ftd/processors.ftd")
}

fn design_ftd() -> &'static str {
    include_str!("../ftd/design.ftd")
}

fn fastn_2022_js() -> &'static str {
    if fastn_core::utils::is_test() {
        return "FASTN_JS";
    }
    include_str!("../fastn2022.js")
}

fn fastn_lib_ftd() -> &'static str {
    include_str!("../ftd/fastn-lib.ftd")
}

fn package_info_about(config: &fastn_core::Config) -> fastn_core::Result<String> {
    let path = config.root.join("fastn").join("cr.ftd");
    Ok(if path.is_file() {
        std::fs::read_to_string(path)?
    } else {
        let body_prefix = match config.package.generate_prefix_string(false) {
            Some(bp) => bp,
            None => String::new(),
        };
        indoc::formatdoc! {"
            {body_prefix}
    
            -- import: {package_info_package}/cr

            -- cr.description:
        ",
        body_prefix = body_prefix,
        package_info_package = config.package_info_package(),
        }
    })
}

fn package_editor_source(
    config: &fastn_core::Config,
    file_name: &str,
) -> fastn_core::Result<String> {
    let body_prefix = match config.package.generate_prefix_string(false) {
        Some(bp) => bp,
        None => String::new(),
    };
    let editor_ftd = indoc::formatdoc! {"
            {body_prefix}
    
            -- import: {package_info_package}/e as pi
            -- import: fastn/processors as pr

            
            -- pi.editor:
            $asts: $asts
            path: {file_name}

            -- pr.ast list $asts:
            {processor_marker}: pr.query
            file: {file_name}
        ",
        body_prefix = body_prefix,
        package_info_package = config.package.name,
        file_name = file_name,
        processor_marker = ftd::PROCESSOR_MARKER,
    };

    Ok(editor_ftd)
}

fn package_info_editor(
    config: &fastn_core::Config,
    file_name: &str,
    diff: fastn_core::Result<Option<String>>,
) -> fastn_core::Result<String> {
    let body_prefix = match config.package.generate_prefix_string(false) {
        Some(bp) => bp,
        None => String::new(),
    };
    let mut editor_ftd = indoc::formatdoc! {"
            {body_prefix}
    
            -- import: {package_info_package}/editor as pi
            -- import: fastn/processors as pr

            
            -- pi.editor:
            source: $source
            path: {file_name}

            -- string source:
            {processor_marker}: pr.fetch-file
            path: {file_name}
        ",
        body_prefix = body_prefix,
        package_info_package = config.package_info_package(),
        file_name = file_name,
        processor_marker = ftd::PROCESSOR_MARKER,
    };
    if let Ok(Some(diff)) = diff {
        editor_ftd = format!("{}\n\n\n-- pi.diff:\n\n{}", editor_ftd, diff);
    }
    Ok(editor_ftd)
}

fn package_info_create_cr(config: &fastn_core::Config) -> fastn_core::Result<String> {
    let body_prefix = match config.package.generate_prefix_string(false) {
        Some(bp) => bp,
        None => String::new(),
    };
    Ok(indoc::formatdoc! {"
            {body_prefix}
    
            -- import: {package_info_package}/create-cr as pi

            -- pi.create-cr:
        ",
        body_prefix = body_prefix,
        package_info_package = config.package_info_package(),
    })
}

#[allow(dead_code)]
fn original_package_status(config: &fastn_core::Config) -> fastn_core::Result<String> {
    let path = config
        .root
        .join("fastn")
        .join("translation")
        .join("original-status.ftd");
    Ok(if path.is_file() {
        std::fs::read_to_string(path)?
    } else {
        let body_prefix = match config.package.generate_prefix_string(false) {
            Some(bp) => bp,
            None => String::new(),
        };
        format!(
            "{}\n\n-- import: {}/original-status as pi\n\n-- pi.original-status-page:",
            body_prefix,
            config.package_info_package()
        )
    })
}

#[allow(dead_code)]
fn translation_package_status(config: &fastn_core::Config) -> fastn_core::Result<String> {
    let path = config
        .root
        .join("fastn")
        .join("translation")
        .join("translation-status.ftd");
    Ok(if path.is_file() {
        std::fs::read_to_string(path)?
    } else {
        let body_prefix = match config.package.generate_prefix_string(false) {
            Some(bp) => bp,
            None => String::new(),
        };
        format!(
            "{}\n\n-- import: {}/translation-status as pi\n\n-- pi.translation-status-page:",
            body_prefix,
            config.package_info_package()
        )
    })
}

fn get_messages(
    status: &fastn_core::TranslatedDocument,
    config: &fastn_core::Config,
) -> fastn_core::Result<String> {
    Ok(match status {
        TranslatedDocument::Missing { .. } => {
            let path = config.root.join("fastn/translation/missing.ftd");
            if path.is_file() {
                std::fs::read_to_string(path)?
            } else {
                include_str!("../ftd/translation/missing.ftd").to_string()
            }
        }
        TranslatedDocument::NeverMarked { .. } => {
            let path = config.root.join("fastn/translation/never-marked.ftd");
            if path.is_file() {
                std::fs::read_to_string(path)?
            } else {
                include_str!("../ftd/translation/never-marked.ftd").to_string()
            }
        }
        TranslatedDocument::Outdated { .. } => {
            let path = config.root.join("fastn/translation/out-of-date.ftd");
            if path.is_file() {
                std::fs::read_to_string(path)?
            } else {
                include_str!("../ftd/translation/out-of-date.ftd").to_string()
            }
        }
        TranslatedDocument::UptoDate { .. } => {
            let path = config.root.join("fastn/translation/upto-date.ftd");
            if path.is_file() {
                std::fs::read_to_string(path)?
            } else {
                include_str!("../ftd/translation/upto-date.ftd").to_string()
            }
        }
    })
}

pub fn get_env_ftd_file() -> String {
    std::env::vars()
        .filter(|(key, val)| {
            ["CARGO", "VERGEN", "FASTN"]
                .iter()
                .any(|prefix| !key.is_empty() && key.starts_with(prefix) && !val.is_empty())
        })
        .fold(String::new(), |accumulator, (key, value)| {
            format!("{accumulator}\n-- string {key}: {value}")
        })
}

pub fn debug_env_vars() -> String {
    std::env::vars()
        .filter(|(key, _)| {
            ["CARGO", "VERGEN", "FASTN"]
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

pub type Result<T> = std::result::Result<T, Error>;

pub(crate) fn usage_error<T>(message: String) -> Result<T> {
    Err(Error::UsageError { message })
}

pub(crate) fn generic_error<T>(message: String) -> Result<T> {
    Error::generic_err(message)
}

pub(crate) fn assert_error<T>(message: String) -> Result<T> {
    Err(Error::AssertError { message })
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
