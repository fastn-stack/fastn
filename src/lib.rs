extern crate self as ftd;

#[cfg(test)]
#[macro_use]
pub(crate) mod test;

mod component;
mod condition;
mod dnode;
mod event;
mod execute_doc;
mod html;
pub mod main;
mod or_type;
pub mod p1;
pub mod p2;
pub mod render;
mod rt;
mod ui;
mod value_with_default;
pub(crate) mod variable;
mod youtube_id;

pub use component::{ChildComponent, Component, Instruction};
pub use condition::Condition;
pub use event::{Action, Event};
pub use ftd::value_with_default::ValueWithDefault;
pub use html::{anchor, color, length, overflow, Node};
pub use or_type::OrType;
pub use rt::RT;
pub use ui::{
    Anchor, AttributeType, Code, Color, Column, Common, ConditionalAttribute, ConditionalValue,
    Container, Element, ExternalFont, FontDisplay, GradientDirection, Grid, IFrame, IText, Image,
    Input, Length, Markup, Markups, NamedFont, Overflow, Position, Region, Row, Scene, Spacing,
    Style, Text, TextAlign, TextBlock, TextFormat, Weight,
};
pub use variable::{PropertyValue, TextSource, Value, Variable, VariableFlags};

pub fn js() -> &'static str {
    include_str!("../ftd.js")
}
pub fn css() -> &'static str {
    include_str!("../ftd.css")
}
pub fn html() -> &'static str {
    include_str!("../ftd.html")
}

pub type Map = std::collections::BTreeMap<String, String>;

#[derive(serde::Serialize, serde::Deserialize, Eq, PartialEq, Debug, Default, Clone)]
pub struct Rendered {
    pub original: String,
    pub rendered: String,
}

#[derive(serde::Deserialize, Debug, PartialEq, Clone, serde::Serialize, Default)]
pub struct Document {
    pub html: String,
    pub data: ftd::DataDependenciesMap,
    pub external_children: ExternalChildrenDependenciesMap,
}

pub fn get_name<'a, 'b>(prefix: &'a str, s: &'b str, doc_id: &str) -> ftd::p1::Result<&'b str> {
    match s.split_once(' ') {
        Some((p1, p2)) => {
            if p1 != prefix {
                return ftd::e2(format!("must start with {}", prefix), doc_id, 0);
                // TODO
            }
            Ok(p2)
        }
        None => ftd::e2(
            format!("{} does not contain space (prefix={})", s, prefix),
            doc_id,
            0, // TODO
        ),
    }
}

pub type DataDependenciesMap = std::collections::BTreeMap<String, Data>;

#[derive(serde::Deserialize, Debug, PartialEq, Clone, serde::Serialize, Default)]
pub struct Data {
    pub value: String,
    pub dependencies: Map,
}

pub type ExternalChildrenDependenciesMap =
    std::collections::BTreeMap<String, Vec<ExternalChildrenCondition>>;

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
    pub condition: Option<String>,
    pub parameters: std::collections::BTreeMap<String, ConditionalValueWithDefault>,
}

#[derive(serde::Deserialize, Debug, PartialEq, Clone, serde::Serialize)]
pub struct ConditionalValueWithDefault {
    pub value: ConditionalValue,
    pub default: Option<ConditionalValue>,
}

impl From<&str> for Rendered {
    fn from(item: &str) -> Self {
        Rendered {
            original: item.to_string(),
            rendered: item.to_string(),
        }
    }
}

pub fn rst(s: &str) -> ftd::Rendered {
    // TODO: use pandoc to render
    ftd::Rendered {
        original: s.to_string(),
        rendered: s.to_string(),
    }
}

pub fn markdown(s: &str) -> ftd::Rendered {
    ftd::Rendered {
        original: s.to_string(),
        rendered: render::render(s, true, false),
    }
}

pub fn markdown_extra(s: &str, auto_links: bool, hard_breaks: bool) -> ftd::Rendered {
    ftd::Rendered {
        original: s.to_string(),
        rendered: render::render(s, auto_links, hard_breaks),
    }
}

pub fn code(code: &str, ext: &str, doc_id: &str) -> ftd::Rendered {
    code_with_theme(code, ext, ftd::render::DEFAULT_THEME, doc_id).unwrap()
}

pub fn code_with_theme(
    code: &str,
    ext: &str,
    theme: &str,
    doc_id: &str,
) -> ftd::p1::Result<ftd::Rendered> {
    Ok(ftd::Rendered {
        original: code.to_string(),
        rendered: render::code_with_theme(
            code.replace("\n\\-- ", "\n-- ").as_str(),
            ext,
            theme,
            doc_id,
        )?,
    })
}

pub fn markdown_line(s: &str) -> ftd::Rendered {
    ftd::Rendered {
        original: s.to_string(),
        rendered: render::inline(s),
    }
}

pub fn markup_line(s: &str) -> ftd::Rendered {
    ftd::Rendered {
        original: s.to_string(),
        rendered: render::markup_inline(s),
    }
}

pub fn e2<T, S1>(m: S1, doc_id: &str, line_number: usize) -> ftd::p1::Result<T>
where
    S1: Into<String>,
{
    Err(ftd::p1::Error::ParseError {
        message: m.into(),
        doc_id: doc_id.to_string(),
        line_number,
    })
}

pub fn unknown_processor_error<T, S>(m: S, doc_id: String, line_number: usize) -> ftd::p1::Result<T>
where
    S: Into<String>,
{
    Err(ftd::p1::Error::ParseError {
        message: m.into(),
        doc_id,
        line_number,
    })
}

pub fn split_module<'a>(
    id: &'a str,
    doc_id: &str,
    line_number: usize,
) -> ftd::p1::Result<(Option<&'a str>, &'a str, Option<&'a str>)> {
    if id.chars().filter(|v| *v == '.').count() > 2 {
        return ftd::e2(
            format!("id contains more than two dots: {}", id),
            doc_id,
            line_number,
        );
    }

    match id.split_once('.') {
        Some((p1, p2)) => match p2.split_once(".") {
            Some((p21, p22)) => Ok((Some(p1), p21, Some(p22))),
            None => Ok((Some(p1), p2, None)),
        },
        None => Ok((None, id, None)),
    }
}

pub struct ExampleLibrary {}

#[cfg(feature = "async")]
use async_trait::async_trait;

#[cfg(feature = "async")]
#[async_trait]
impl ftd::p2::Library for ExampleLibrary {
    async fn get(&self, name: &str) -> Option<String> {
        std::fs::read_to_string(format!("./examples/{}.ftd", name)).ok()
    }
}

#[cfg(not(feature = "async"))]
impl ftd::p2::Library for ExampleLibrary {
    fn get(&self, name: &str, _doc: &ftd::p2::TDoc) -> Option<String> {
        std::fs::read_to_string(format!("./examples/{}.ftd", name)).ok()
    }
}
