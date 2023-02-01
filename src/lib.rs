extern crate self as fastn;

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
pub(crate) mod watcher;
#[macro_use]
mod http;
mod auth;
mod ds;
mod error;
mod i18n;
pub mod library;
mod proxy;
mod render;
pub mod sitemap;
mod snapshot;
mod sync_utils;
mod track;
mod tracker;
mod translation;
mod version;
// mod wasm;
mod library2022;
mod workspace;

pub(crate) use auto_import::AutoImport;
pub use commands::{
    abort_merge::abort_merge, add::add, build::build, clone::clone, close_cr::close_cr,
    create_cr::create_cr, create_package::create_package, diff::diff, edit::edit,
    mark_resolved::mark_resolved, mark_upto_date::mark_upto_date, merge::merge,
    resolve_conflict::resolve_conflict, revert::revert, rm::rm, serve::listen,
    start_tracking::start_tracking, status::status, sync2::sync2,
    translation_status::translation_status, update::update,
};
pub use config::{Config, FTDEdition};
pub use error::Error;
pub use file::File;
pub(crate) use file::{get_file, paths_to_files, Document, Static};
pub(crate) use font::Font;
pub use library::{FastnLibrary, Library, Library2};
pub use library2022::Library2022;
pub(crate) use package::dependency::Dependency;
pub use package::user_group;
pub(crate) use package::Package;
pub use render::render;
pub(crate) use snapshot::Snapshot;
pub(crate) use tracker::Track;
pub(crate) use translation::{TranslatedDocument, TranslationData};
pub(crate) use utils::{copy_dir_all, time, timestamp_nanosecond};
pub(crate) use version::Version;
pub use {doc::resolve_foreign_variable2, doc::resolve_import};

pub const FASTN_UI_INTERFACE: &str = "ftd-lang.github.io/fastn-ui";
pub const PACKAGE_THEME_INTERFACE: &str = "ftd-lang.github.io/theme";
pub const NUMBER_OF_CRS_TO_RESERVE: usize = 5;

pub const IMAGE_EXT: &[&str] = &["jpg", "png", "svg"];

pub fn ftd_html() -> &'static str {
    include_str!("../ftd.html")
}

fn fastn_ftd() -> &'static str {
    include_str!("../ftd/FASTN.ftd")
}

fn processor_ftd() -> &'static str {
    include_str!("../ftd/processors.ftd")
}

fn design_ftd() -> &'static str {
    include_str!("../ftd/design.ftd")
}

fn fastn_js() -> &'static str {
    if fastn::utils::is_test() {
        return "fastn_JS";
    }
    include_str!("../fastn.js")
}

fn fastn_2022_js() -> &'static str {
    if fastn::utils::is_test() {
        return "FASTN_JS";
    }
    include_str!("../fastn2022.js")
}

fn ftd_js() -> String {
    if fastn::utils::is_test() {
        return "FTD_JS".to_string();
    }
    ftd::js()
}

fn ftd_css() -> &'static str {
    if fastn::utils::is_test() {
        return "FTD_CSS";
    }
    ftd::css()
}

fn fastn_lib_ftd() -> &'static str {
    include_str!("../ftd/fastn-lib.ftd")
}

#[allow(dead_code)]
fn with_message() -> &'static str {
    include_str!("../with-message.html")
}

#[allow(dead_code)]
fn available_languages(config: &fastn::Config) -> fastn::Result<String> {
    let path = config
        .root
        .join("fastn/translation/available-languages.ftd");
    Ok(if path.is_file() {
        std::fs::read_to_string(path)?
    } else {
        include_str!("../ftd/translation/available-languages.ftd").to_string()
    })
}

fn package_info_image(
    config: &fastn::Config,
    doc: &fastn::Static,
    package: &fastn::Package,
) -> fastn::Result<String> {
    let path = config.root.join("fastn").join("image.ftd");
    Ok(if path.is_file() {
        std::fs::read_to_string(path)?
    } else {
        let body_prefix = match config.package.generate_prefix_string(false) {
            Some(bp) => bp,
            None => String::new(),
        };
        indoc::formatdoc! {"
            {body_prefix}
    
            -- import: {package_info_package}/image as pi 

            -- ftd.image-src src: {src}
            dark: {src}
    
            -- pi.image-page: {file_name}
            src: $src
        ",
        body_prefix = body_prefix,
        file_name = doc.id,
        package_info_package = config.package_info_package(),
        src = format!("-/{}/{}", package.name.as_str(), doc.id.as_str()),
        }
    })
}

fn package_info_about(config: &fastn::Config) -> fastn::Result<String> {
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

fn package_info_editor(
    config: &fastn::Config,
    file_name: &str,
    diff: fastn::Result<Option<String>>,
) -> fastn::Result<String> {
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
            $processor$: pr.fetch-file
            path: {file_name}
        ",
        body_prefix = body_prefix,
        package_info_package = config.package_info_package(),
        file_name = file_name
    };
    if let Ok(Some(diff)) = diff {
        editor_ftd = format!("{}\n\n\n-- pi.diff:\n\n{}", editor_ftd, diff);
    }
    Ok(editor_ftd)
}

fn package_info_create_cr(config: &fastn::Config) -> fastn::Result<String> {
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

fn package_info_code(
    config: &fastn::Config,
    file_name: &str,
    content: &str,
    extension: &str,
) -> fastn::Result<String> {
    let path = config.root.join("fastn").join("code.ftd");
    Ok(if path.is_file() {
        std::fs::read_to_string(path)?
    } else {
        let body_prefix = match config.package.generate_prefix_string(false) {
            Some(bp) => bp,
            None => String::new(),
        };
        if content.trim().is_empty() {
            format!(
                indoc::indoc! {"
                {body_prefix}
        
                -- import: {package_info_package}/code as pi 
        
                -- pi.code-page: {file_name}
                lang: {ext}

                "},
                body_prefix = body_prefix,
                package_info_package = config.package_info_package(),
                file_name = file_name,
                ext = extension,
            )
        } else {
            format!(
                indoc::indoc! {"
                {body_prefix}
        
                -- import: {package_info_package}/code as pi 
        
                -- pi.code-page: {file_name}
                lang: {ext}

                {content}

                "},
                body_prefix = body_prefix,
                package_info_package = config.package_info_package(),
                file_name = file_name,
                ext = extension,
                content = content,
            )
        }
    })
}

fn package_info_markdown(
    config: &fastn::Config,
    file_name: &str,
    content: &str,
) -> fastn::Result<String> {
    let path = config.root.join("fastn").join("markdown.ftd");
    Ok(if path.is_file() {
        std::fs::read_to_string(path)?
    } else if !config.ftd_edition.eq(&fastn::config::FTDEdition::FTD2021) {
        if content.trim().is_empty() {
            content.to_string()
        } else {
            format!(
                indoc::indoc! {"
                -- ftd.text:

                {content}
            "},
                content = content,
            )
        }
    } else {
        let body_prefix = match config.package.generate_prefix_string(false) {
            Some(bp) => bp,
            None => String::new(),
        };
        if content.trim().is_empty() {
            format!(
                indoc::indoc! {"
                {body_prefix}
        
                -- import: {package_info_package}/markdown as pi 
        
                -- pi.markdown-page: {file_name}

            "},
                body_prefix = body_prefix,
                package_info_package = config.package_info_package(),
                file_name = file_name,
            )
        } else {
            format!(
                indoc::indoc! {"
                {body_prefix}
        
                -- import: {package_info_package}/markdown as pi 
        
                -- pi.markdown-page: {file_name}

                {content}

            "},
                body_prefix = body_prefix,
                package_info_package = config.package_info_package(),
                content = content,
                file_name = file_name,
            )
        }
    })
}

#[allow(dead_code)]
fn original_package_status(config: &fastn::Config) -> fastn::Result<String> {
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
fn translation_package_status(config: &fastn::Config) -> fastn::Result<String> {
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
    status: &fastn::TranslatedDocument,
    config: &fastn::Config,
) -> fastn::Result<String> {
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
        .into_iter()
        .filter(|(key, val)| {
            vec!["CARGO", "VERGEN", "FASTN"]
                .iter()
                .any(|prefix| !key.is_empty() && key.starts_with(prefix) && !val.is_empty())
        })
        .fold(String::new(), |accumulator, (key, value)| {
            format!("{accumulator}\n-- string {key}: {value}")
        })
}

pub fn debug_env_vars() -> String {
    std::env::vars()
        .into_iter()
        .filter(|(key, _)| {
            vec!["CARGO", "VERGEN", "FASTN"]
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

#[cfg(test)]
mod tests {

    #[test]
    fn fbt() {
        if fbt_lib::main().is_some() {
            panic!("test failed")
        }
    }
}
