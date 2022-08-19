#![allow(clippy::derive_partial_eq_without_eq)]
#![allow(clippy::get_first)]

extern crate self as ftd;

#[cfg(test)]
#[macro_use]
pub(crate) mod test;

pub mod code;
mod component;
mod condition;
mod dnode;
mod event;
mod execute_doc;
mod html;
pub mod main;
pub mod markup;
mod or_type;
pub mod p1;
pub mod p2;
pub(crate) mod rendered;
mod rt;
mod ui;
mod value_with_default;
pub(crate) mod variable;
mod youtube_id;

pub use component::{ChildComponent, Component, Instruction};
pub use condition::Condition;
pub use event::{Action, Event};
pub use ftd::{
    ftd::p2::interpreter::{interpret, Interpreter, InterpreterState, ParsedDocument},
    value_with_default::ValueWithDefault,
};
pub use html::{anchor, color, length, overflow, Collector, Node, StyleSpec};
pub use or_type::OrType;
pub use rendered::Rendered;
pub use rt::RT;
pub use ui::{
    Anchor, AttributeType, Code, Color, ColorValue, Column, Common, ConditionalAttribute,
    ConditionalValue, Container, Element, FontDisplay, GradientDirection, Grid, IFrame, IText,
    Image, ImageSrc, Input, Length, Loading, Markup, Markups, NamedFont, Overflow, Position,
    Region, Row, Scene, Spacing, Style, Text, TextAlign, TextBlock, TextFormat, Type, Weight,
};
pub use variable::{PropertyValue, TextSource, Value, Variable, VariableFlags};

pub fn js() -> String {
    include_str!("../ftd.js").replace("if (true) { // false", "if (false) { // false")
}

pub fn css() -> &'static str {
    include_str!("../ftd.css")
}
pub fn html() -> &'static str {
    include_str!("../ftd.html")
}

// #[cfg(test)]
pub type Map<T> = std::collections::BTreeMap<String, T>;

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
    _doc_id: &str,
    _line_number: usize,
) -> ftd::p1::Result<(Option<&'a str>, &'a str, Option<&'a str>)> {
    match id.split_once('.') {
        Some((p1, p2)) => match p2.split_once('.') {
            Some((p21, p22)) => Ok((Some(p1), p21, Some(p22))),
            None => Ok((Some(p1), p2, None)),
        },
        None => Ok((None, id, None)),
    }
}

pub struct ExampleLibrary {}

impl ExampleLibrary {
    pub fn get(&self, name: &str, _doc: &ftd::p2::TDoc) -> Option<String> {
        std::fs::read_to_string(format!("./examples/{}.ftd", name)).ok()
    }

    pub fn process(
        &self,
        section: &ftd::p1::Section,
        doc: &ftd::p2::TDoc,
    ) -> ftd::p1::Result<ftd::Value> {
        ftd::unknown_processor_error(
            format!("unimplemented for section {:?} and doc {:?}", section, doc),
            doc.name.to_string(),
            section.line_number,
        )
    }

    pub fn get_with_result(&self, name: &str, doc: &ftd::p2::TDoc) -> ftd::p1::Result<String> {
        match self.get(name, doc) {
            Some(v) => Ok(v),
            None => ftd::e2(format!("library not found: {}", name), "", 0),
        }
    }
}
