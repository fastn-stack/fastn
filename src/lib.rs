extern crate self as fpm;

#[macro_use]
pub mod utils;

// Temp comment
mod apis;
mod auto_import;
mod cache;
mod commands;
mod config;
mod controller;
mod cr;
mod doc;
mod file;
mod font;
mod history;
mod package;
#[macro_use]
mod http;
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
mod wasm;
mod workspace;

pub(crate) use auto_import::AutoImport;
pub use commands::{
    abort_merge::abort_merge, add::add, build::build, clone::clone, close_cr::close_cr,
    create_cr::create_cr, create_package::create_package, diff::diff, edit::edit,
    mark_resolved::mark_resolved, mark_upto_date::mark_upto_date, merge::merge,
    resolve_conflict::resolve_conflict, revert::revert, rm::rm, serve::listen,
    start_tracking::start_tracking, status::status, stop_tracking::stop_tracking, sync2::sync2,
    sync_status::sync_status, translation_status::translation_status, update::update,
};
pub use config::Config;
pub use error::Error;
pub use file::File;
pub(crate) use file::{get_file, paths_to_files, Document, Static};
pub(crate) use font::Font;
pub use library::{FPMLibrary, Library, Library2};
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

pub const FPM_UI_INTERFACE: &str = "ftd-lang.github.io/fpm-ui";
pub const PACKAGE_THEME_INTERFACE: &str = "ftd-lang.github.io/theme";
pub const NUMBER_OF_CRS_TO_RESERVE: usize = 5;

pub const IMAGE_EXT: &[&str] = &["jpg", "png", "svg"];

pub fn ftd_html() -> &'static str {
    include_str!("../ftd.html")
}

fn fpm_ftd() -> &'static str {
    include_str!("../ftd/fpm.ftd")
}

fn design_ftd() -> &'static str {
    include_str!("../ftd/design.ftd")
}

fn fpm_js() -> &'static str {
    if fpm::utils::is_test() {
        return "FPM_JS";
    }
    include_str!("../fpm.js")
}

fn ftd_js() -> String {
    if fpm::utils::is_test() {
        return "FTD_JS".to_string();
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

#[allow(dead_code)]
fn with_message() -> &'static str {
    include_str!("../with-message.html")
}

#[allow(dead_code)]
fn available_languages(config: &fpm::Config) -> fpm::Result<String> {
    let path = config.root.join("FPM/translation/available-languages.ftd");
    Ok(if path.is_file() {
        std::fs::read_to_string(path)?
    } else {
        include_str!("../ftd/translation/available-languages.ftd").to_string()
    })
}

fn package_info_image(
    config: &fpm::Config,
    doc: &fpm::Static,
    package: &fpm::Package,
) -> fpm::Result<String> {
    let path = config.root.join("FPM").join("image.ftd");
    Ok(if path.is_file() {
        std::fs::read_to_string(path)?
    } else {
        let package_info_package = match config
            .package
            .get_dependency_for_interface(fpm::FPM_UI_INTERFACE)
            .or_else(|| {
                config
                    .package
                    .get_dependency_for_interface(fpm::PACKAGE_THEME_INTERFACE)
            }) {
            Some(dep) => dep.package.name.as_str(),
            None => fpm::FPM_UI_INTERFACE,
        };
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
        package_info_package = package_info_package,
        src = format!("-/{}/{}", package.name.as_str(), doc.id.as_str()),
        }
    })
}

fn package_info_about(config: &fpm::Config) -> fpm::Result<String> {
    let path = config.root.join("FPM").join("cr.ftd");
    Ok(if path.is_file() {
        std::fs::read_to_string(path)?
    } else {
        let package_info_package = match config
            .package
            .get_dependency_for_interface(fpm::FPM_UI_INTERFACE)
            .or_else(|| {
                config
                    .package
                    .get_dependency_for_interface(fpm::PACKAGE_THEME_INTERFACE)
            }) {
            Some(dep) => dep.package.name.as_str(),
            None => fpm::FPM_UI_INTERFACE,
        };
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
        package_info_package = package_info_package,
        }
    })
}

fn package_info_editor(
    config: &fpm::Config,
    file_name: &str,
    diff: fpm::Result<Option<String>>,
) -> fpm::Result<String> {
    let package_info_package = match config
        .package
        .get_dependency_for_interface(fpm::FPM_UI_INTERFACE)
        .or_else(|| {
            config
                .package
                .get_dependency_for_interface(fpm::PACKAGE_THEME_INTERFACE)
        }) {
        Some(dep) => dep.package.name.as_str(),
        None => fpm::FPM_UI_INTERFACE,
    };
    let body_prefix = match config.package.generate_prefix_string(false) {
        Some(bp) => bp,
        None => String::new(),
    };
    let mut editor_ftd = indoc::formatdoc! {"
            {body_prefix}
    
            -- import: {package_info_package}/editor as pi

            -- pi.editor:

            -- pi.source:
            $processor$: fetch-file
            path: {file_name}

            -- pi.path: {file_name}
        ",
        body_prefix = body_prefix,
        package_info_package = package_info_package,
        file_name = file_name
    };
    if let Ok(Some(diff)) = diff {
        editor_ftd = format!("{}\n\n\n-- pi.diff:\n\n{}", editor_ftd, diff);
    }
    Ok(editor_ftd)
}

fn package_info_create_cr(config: &fpm::Config) -> fpm::Result<String> {
    let package_info_package = match config
        .package
        .get_dependency_for_interface(fpm::FPM_UI_INTERFACE)
        .or_else(|| {
            config
                .package
                .get_dependency_for_interface(fpm::PACKAGE_THEME_INTERFACE)
        }) {
        Some(dep) => dep.package.name.as_str(),
        None => fpm::FPM_UI_INTERFACE,
    };
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
        package_info_package = package_info_package,
    })
}

fn package_info_code(
    config: &fpm::Config,
    file_name: &str,
    content: &str,
    extension: &str,
) -> fpm::Result<String> {
    let path = config.root.join("FPM").join("code.ftd");
    Ok(if path.is_file() {
        std::fs::read_to_string(path)?
    } else {
        let package_info_package = match config
            .package
            .get_dependency_for_interface(fpm::FPM_UI_INTERFACE)
            .or_else(|| {
                config
                    .package
                    .get_dependency_for_interface(fpm::PACKAGE_THEME_INTERFACE)
            }) {
            Some(dep) => dep.package.name.as_str(),
            None => fpm::FPM_UI_INTERFACE,
        };
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
                package_info_package = package_info_package,
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
                package_info_package = package_info_package,
                file_name = file_name,
                ext = extension,
                content = content,
            )
        }
    })
}

fn package_info_markdown(
    config: &fpm::Config,
    file_name: &str,
    content: &str,
) -> fpm::Result<String> {
    let path = config.root.join("FPM").join("markdown.ftd");
    Ok(if path.is_file() {
        std::fs::read_to_string(path)?
    } else {
        let package_info_package = match config
            .package
            .get_dependency_for_interface(fpm::FPM_UI_INTERFACE)
            .or_else(|| {
                config
                    .package
                    .get_dependency_for_interface(fpm::PACKAGE_THEME_INTERFACE)
            }) {
            Some(dep) => dep.package.name.as_str(),
            None => fpm::FPM_UI_INTERFACE,
        };
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
                package_info_package = package_info_package,
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
                package_info_package = package_info_package,
                content = content,
                file_name = file_name,
            )
        }
    })
}

#[allow(dead_code)]
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
            .get_dependency_for_interface(fpm::FPM_UI_INTERFACE)
            .or_else(|| {
                config
                    .package
                    .get_dependency_for_interface(fpm::PACKAGE_THEME_INTERFACE)
            }) {
            Some(dep) => dep.package.name.as_str(),
            None => fpm::FPM_UI_INTERFACE,
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

#[allow(dead_code)]
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
            .get_dependency_for_interface(fpm::FPM_UI_INTERFACE)
            .or_else(|| {
                config
                    .package
                    .get_dependency_for_interface(fpm::PACKAGE_THEME_INTERFACE)
            }) {
            Some(dep) => dep.package.name.as_str(),
            None => fpm::FPM_UI_INTERFACE,
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

pub fn get_env_ftd_file() -> String {
    std::env::vars()
        .into_iter()
        .filter(|(key, val)| {
            vec!["CARGO", "VERGEN", "FPM"]
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
