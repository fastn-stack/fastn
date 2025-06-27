#![allow(clippy::derive_partial_eq_without_eq, clippy::get_first)]
#![deny(unused_crate_dependencies)]

extern crate self as ftd;

pub use ftd2021::component::{ChildComponent, Component, Instruction};
pub use ftd2021::condition::Condition;
pub use ftd2021::constants::{identifier, regex};
pub use ftd2021::event::{Action, Event};
pub use ftd2021::html::{Collector, Node, StyleSpec, anchor, color, length, overflow};
pub use ftd2021::ui::{
    Anchor, AttributeType, Code, Color, ColorValue, Column, Common, ConditionalAttribute,
    ConditionalValue, Container, Element, FontDisplay, GradientDirection, Grid, IFrame, IText,
    Image, ImageSrc, Input, Length, Loading, Markup, Markups, NamedFont, Overflow, Position,
    Region, Row, Scene, Spacing, Style, Text, TextAlign, TextBlock, TextFormat, Type, Weight,
};
pub use ftd2021::value_with_default::ValueWithDefault;
pub use ftd2021::variable::{PropertyValue, TextSource, Value, Variable, VariableFlags};

pub mod executor;
pub mod ftd2021;
pub mod html;
pub mod interpreter;
pub mod js;
pub mod node;
mod parser;
pub use parser::parse_doc;
#[cfg(feature = "native-rendering")]
pub mod taffy;
pub mod test_helper;
#[cfg(feature = "native-rendering")]
mod wasm;

pub const PROCESSOR_MARKER: &str = "$processor$";

pub fn css() -> &'static str {
    // if fastn_core::utils::is_test() {
    //     return "FTD_CSS";
    // }

    include_str!("../ftd.css")
}

static THEME_CSS_DIR: include_dir::Dir<'_> =
    include_dir::include_dir!("$CARGO_MANIFEST_DIR/theme_css");

pub fn theme_css() -> ftd::Map<String> {
    let mut themes: ftd::Map<String> = Default::default();
    // let paths = ftd_p1::utils::find_all_files_matching_extension_recursively("theme_css", "css");
    for file in THEME_CSS_DIR.files() {
        let stem = file.path().file_stem().unwrap().to_str().unwrap();
        themes.insert(stem.to_string(), file.contents_utf8().unwrap().to_string());
    }
    themes
}

pub fn ftd_js_css() -> &'static str {
    include_str!("../ftd-js.css")
}

pub fn markdown_js() -> &'static str {
    fastn_js::markdown_js()
}

pub fn prism_css() -> String {
    let prism_line_highlight = include_str!("../prism/prism-line-highlight.css");
    let prism_line_numbers = include_str!("../prism/prism-line-numbers.css");
    format!("{prism_line_highlight}{prism_line_numbers}")
}

pub fn prism_js() -> String {
    let prism = include_str!("../prism/prism.js");
    let prism_line_highlight = include_str!("../prism/prism-line-highlight.js");
    let prism_line_numbers = include_str!("../prism/prism-line-numbers.js");

    // Languages supported
    // Rust, Json, Python, Markdown, SQL, Bash, JavaScript
    let prism_rust = include_str!("../prism/prism-rust.js");
    let prism_json = include_str!("../prism/prism-json.js");
    let prism_python = include_str!("../prism/prism-python.js");
    let prism_markdown = include_str!("../prism/prism-markdown.js");
    let prism_sql = include_str!("../prism/prism-sql.js");
    let prism_bash = include_str!("../prism/prism-bash.js");
    let prism_javascript = include_str!("../prism/prism-javascript.js");
    let prism_diff = include_str!("../prism/prism-diff.js");

    format!(
        "{prism}{prism_line_highlight}{prism_line_numbers}{prism_rust}{prism_json}{prism_python\
        }{prism_markdown}{prism_sql}{prism_bash}{prism_javascript}{prism_diff}"
    )
}

pub fn terminal() -> &'static str {
    include_str!("../terminal.ftd")
}
pub fn taffy() -> &'static str {
    include_str!("../taffy.ftd")
}

pub fn build_js() -> &'static str {
    include_str!("../build.js")
}

// #[cfg(test)]
pub type Map<T> = std::collections::BTreeMap<String, T>;

#[derive(serde::Deserialize, Debug, PartialEq, Default, Clone, serde::Serialize)]
pub struct VecMap<T> {
    value: Map<Vec<T>>,
}

impl<T: std::cmp::PartialEq> VecMap<T> {
    pub fn new() -> VecMap<T> {
        VecMap {
            value: Default::default(),
        }
    }

    pub fn insert(&mut self, key: String, value: T) {
        if let Some(v) = self.value.get_mut(&key) {
            v.push(value);
        } else {
            self.value.insert(key, vec![value]);
        }
    }

    pub fn unique_insert(&mut self, key: String, value: T) {
        if let Some(v) = self.value.get_mut(&key) {
            if !v.contains(&value) {
                v.push(value);
            }
        } else {
            self.value.insert(key, vec![value]);
        }
    }

    pub fn extend(&mut self, key: String, value: Vec<T>) {
        if let Some(v) = self.value.get_mut(&key) {
            v.extend(value);
        } else {
            self.value.insert(key, value);
        }
    }

    pub fn get_value(&self, key: &str) -> Vec<&T> {
        self.get_value_and_rem(key)
            .into_iter()
            .map(|(k, _)| k)
            .collect()
    }

    pub fn get_value_and_rem(&self, key: &str) -> Vec<(&T, Option<String>)> {
        let mut values = vec![];

        self.value.iter().for_each(|(k, v)| {
            if k.eq(key) {
                values.extend(
                    v.iter()
                        .map(|a| (a, None))
                        .collect::<Vec<(&T, Option<String>)>>(),
                );
            } else if let Some(rem) = key.strip_prefix(format!("{k}.").as_str()) {
                values.extend(
                    v.iter()
                        .map(|a| (a, Some(rem.to_string())))
                        .collect::<Vec<(&T, Option<String>)>>(),
                );
            } else if let Some(rem) = k.strip_prefix(format!("{key}.").as_str()) {
                values.extend(
                    v.iter()
                        .map(|a| (a, Some(rem.to_string())))
                        .collect::<Vec<(&T, Option<String>)>>(),
                );
            }
        });
        values
    }
}

// #[cfg(not(test))]
// pub type Map<T> = std::collections::HashMap<String, T>;

#[derive(serde::Deserialize, Debug, PartialEq, Clone, serde::Serialize, Default)]
pub struct Document {
    pub html: String,
    pub data: ftd::DataDependenciesMap,
    pub external_children: ExternalChildrenDependenciesMap,
    pub body_events: String,
    pub css_collector: String,
}

// Condensed form of page-heading item stored by parsed document
#[derive(Debug, Clone, serde::Serialize)]
pub struct PageHeadingItem {
    pub url: Option<String>,
    pub title: Option<String>,
    pub region: Option<ftd::Region>,
    pub number: Option<String>,
    pub children: Vec<PageHeadingItem>,
}

// Page-heading struct identical with fpm::library::toc::TocItemCompat
// to be used by page-headings processor
#[derive(Debug, Clone, serde::Serialize)]
pub struct PageHeadingItemCompat {
    pub url: Option<String>,
    pub number: Option<String>,
    pub title: Option<String>,
    pub path: Option<String>,
    #[serde(rename = "is-heading")]
    pub is_heading: bool,
    // TODO: Font icon mapping to html?
    #[serde(rename = "font-icon")]
    pub font_icon: Option<String>,
    #[serde(rename = "is-disabled")]
    pub is_disabled: bool,
    #[serde(rename = "is-active")]
    pub is_active: bool,
    #[serde(rename = "is-open")]
    pub is_open: bool,
    #[serde(rename = "img-src")]
    pub image_src: Option<String>,
    pub document: Option<String>,
    pub children: Vec<PageHeadingItemCompat>,
}

// TextSource location = (is_from_section = T/F, subsection_index if is_from_section = F else 0)
pub type TextSourceLocation = (bool, usize);
pub type TextSourceWithLocation = (ftd::TextSource, TextSourceLocation);

// ReplaceLinkBlock = (Id, TextSourceWithLocation, Line number)
// contains relevant id data associated with links along with its source
// from where those were captured and where link replacement or escaped links
// needs to be resolved
pub type ReplaceLinkBlock<T> = (T, ftd::TextSourceWithLocation, usize);

pub type DataDependenciesMap = ftd::Map<Data>;

#[derive(serde::Deserialize, Debug, PartialEq, Clone, serde::Serialize, Default)]
pub struct Data {
    pub value: serde_json::Value,
    pub dependencies: ftd::Map<serde_json::Value>,
}

pub type ExternalChildrenDependenciesMap = ftd::Map<Vec<ExternalChildrenCondition>>;

#[derive(serde::Deserialize, Debug, PartialEq, Clone, serde::Serialize, Default)]
pub struct ExternalChildrenCondition {
    pub condition: Vec<String>,
    pub set_at: String,
}

#[derive(serde::Deserialize, Debug, PartialEq, Clone, serde::Serialize)]
pub enum DependencyType {
    Style,
    Visible,
    Value,
    Variable,
}

#[derive(serde::Deserialize, Debug, PartialEq, Clone, serde::Serialize)]
pub struct Dependencies {
    pub dependency_type: DependencyType,
    pub condition: Option<serde_json::Value>,
    pub parameters: ftd::Map<ConditionalValueWithDefault>,
    pub remaining: Option<String>,
}

#[derive(serde::Deserialize, Debug, PartialEq, Clone, serde::Serialize, Default)]
pub struct ConditionalValueWithDefault {
    pub value: ConditionalValue,
    pub default: Option<ConditionalValue>,
}

pub struct ExampleLibrary {}

impl ExampleLibrary {
    pub fn dummy_global_ids_map(&self) -> std::collections::HashMap<String, String> {
        let mut global_ids: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();

        global_ids.insert("foo".to_string(), "/foo/bar/#foo".to_string());
        global_ids.insert("hello".to_string(), "/hello/there/#hello".to_string());
        global_ids.insert("some id".to_string(), "/some/id/#some-id".to_string());

        // To debug for section
        global_ids.insert("scp".to_string(), "/foo/bar/#scp".to_string());
        global_ids.insert("sh".to_string(), "/hello/there/#sh".to_string());
        global_ids.insert("sb".to_string(), "/some/id/#sb".to_string());

        // To debug for subsection
        global_ids.insert("sscp".to_string(), "/foo/bar/#sscp".to_string());
        global_ids.insert("ssh".to_string(), "/hello/there/#ssh".to_string());
        global_ids.insert("ssb".to_string(), "/some/id/#ssb".to_string());

        // More dummy instances for debugging purposes
        global_ids.insert("a".to_string(), "/some/#a".to_string());
        global_ids.insert("b".to_string(), "/some/#b".to_string());
        global_ids.insert("c".to_string(), "/some/#c".to_string());
        global_ids.insert("d".to_string(), "/some/#d".to_string());

        // to debug in case of checkboxes
        global_ids.insert("x".to_string(), "/some/#x".to_string());
        global_ids.insert("X".to_string(), "/some/#X".to_string());

        global_ids
    }

    pub fn get(&self, name: &str, _doc: &ftd2021::p2::TDoc) -> Option<String> {
        std::fs::read_to_string(format!("./ftd/examples/{name}.ftd")).ok()
    }

    /// checks if the current processor is a lazy processor
    /// or not
    ///
    /// for more details
    /// visit www.fpm.dev/glossary/#lazy-processor
    pub fn is_lazy_processor(
        section: &ftd2021::p1::Section,
        doc: &ftd2021::p2::TDoc,
    ) -> ftd2021::p1::Result<bool> {
        Ok(section
            .header
            .str(doc.name, section.line_number, "$processor$")?
            .eq("page-headings"))
    }

    pub fn process(
        &self,
        section: &ftd2021::p1::Section,
        doc: &ftd2021::p2::TDoc,
    ) -> ftd2021::p1::Result<ftd::Value> {
        ftd2021::p2::utils::unknown_processor_error(
            format!("unimplemented for section {section:?} and doc {doc:?}"),
            doc.name.to_string(),
            section.line_number,
        )
    }

    pub fn get_with_result(
        &self,
        name: &str,
        doc: &ftd2021::p2::TDoc,
    ) -> ftd2021::p1::Result<String> {
        match self.get(name, doc) {
            Some(v) => Ok(v),
            None => ftd2021::p2::utils::e2(format!("library not found: {name}"), "", 0),
        }
    }
}
