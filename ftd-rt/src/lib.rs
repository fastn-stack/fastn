extern crate self as ftd_rt;

mod condition;
mod dnode;
mod event;
mod html;
mod ui;
#[cfg(feature = "wasm")]
mod wasm;

pub use condition::Condition;
pub use event::{Action, Event};
pub use html::Node;
pub use ui::{
    Align, Color, Column, Common, Container, Element, ExternalFont, FontDisplay, GradientDirection,
    IFrame, Image, Input, Length, NamedFont, Overflow, Region, Row, Style, Text, TextAlign,
    TextFormat, Weight,
};

#[derive(serde::Serialize, serde::Deserialize, Eq, PartialEq, Debug, Default, Clone)]
pub struct Rendered {
    pub original: String,
    pub rendered: String,
}

impl From<&str> for Rendered {
    fn from(item: &str) -> Self {
        Rendered {
            original: item.to_string(),
            rendered: item.to_string(),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("invalid input: {message}")]
    InvalidInput { message: String, context: String },
}

pub type Map = std::collections::BTreeMap<String, String>;
type Result<T> = std::result::Result<T, Error>;

#[derive(serde::Deserialize)]
#[cfg_attr(
    not(feature = "wasm"),
    derive(serde::Serialize, PartialEq, Debug, Default, Clone)
)]
pub struct Document {
    pub html: String,
    pub data: ftd_rt::DataDependenciesMap,
    pub external_children: ExternalChildrenDependenciesMap,
}

pub fn e<T, S>(m: S) -> Result<T>
where
    S: Into<String>,
{
    Err(Error::InvalidInput {
        message: m.into(),
        context: "".to_string(),
    })
}

pub fn get_name<'a, 'b>(prefix: &'a str, s: &'b str) -> ftd_rt::Result<&'b str> {
    match s.split_once(' ') {
        Some((p1, p2)) => {
            if p1 != prefix {
                return ftd_rt::e(format!("must start with {}", prefix));
            }
            Ok(p2)
        }
        None => ftd_rt::e(format!("{} does not contain space (prefix={})", s, prefix)),
    }
}

pub type DataDependenciesMap = std::collections::BTreeMap<String, Data>;

#[derive(serde::Deserialize)]
#[cfg_attr(
    not(feature = "wasm"),
    derive(serde::Serialize, PartialEq, Debug, Default, Clone)
)]
pub struct Data {
    pub value: String,
    pub dependencies: ftd_rt::Map,
}

pub type ExternalChildrenDependenciesMap =
    std::collections::BTreeMap<String, Vec<ExternalChildrenCondition>>;

#[derive(serde::Deserialize)]
#[cfg_attr(
    not(feature = "wasm"),
    derive(serde::Serialize, PartialEq, Debug, Default, Clone)
)]
pub struct ExternalChildrenCondition {
    pub condition: Vec<String>,
    pub set_at: String,
}

#[cfg(test)]
mod test {
    #[test]
    fn get_name() {
        assert_eq!(super::get_name("fn", "fn foo").unwrap(), "foo")
    }
}
