#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ComponentInvocation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub name: String,
    pub properties: Vec<Property>,
    pub iteration: Box<Option<Loop>>,
    pub condition: Box<Option<fastn_resolved::Expression>>,
    pub events: Vec<Event>,
    pub children: Vec<ComponentInvocation>,
    pub source: ComponentSource,
    pub line_number: usize,
}

impl fastn_resolved::ComponentInvocation {
    pub fn from_name(name: &str) -> fastn_resolved::ComponentInvocation {
        fastn_resolved::ComponentInvocation {
            id: None,
            name: name.to_string(),
            properties: vec![],
            iteration: Box::new(None),
            condition: Box::new(None),
            events: vec![],
            children: vec![],
            source: Default::default(),
            line_number: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Loop {
    pub on: fastn_resolved::PropertyValue,
    pub alias: String,
    pub loop_counter_alias: Option<String>,
    pub line_number: usize,
}

impl Loop {
    pub fn new(
        on: fastn_resolved::PropertyValue,
        alias: &str,
        loop_counter_alias: Option<String>,
        line_number: usize,
    ) -> fastn_resolved::Loop {
        fastn_resolved::Loop {
            on,
            alias: alias.to_string(),
            line_number,
            loop_counter_alias,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, serde::Deserialize, serde::Serialize)]
pub enum ComponentSource {
    #[default]
    Declaration,
    Variable,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Event {
    pub name: fastn_resolved::EventName,
    pub action: fastn_resolved::FunctionCall,
    pub line_number: usize,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Property {
    pub value: fastn_resolved::PropertyValue,
    pub source: fastn_resolved::PropertySource,
    pub condition: Option<fastn_resolved::Expression>,
    pub line_number: usize,
}

#[derive(Debug, Clone, PartialEq, Default, serde::Deserialize, serde::Serialize)]
pub enum PropertySource {
    #[default]
    Caption,
    Body,
    Header {
        name: String,
        mutable: bool,
    },
    Subsection,
    Default,
}

impl fastn_resolved::PropertySource {
    pub fn is_equal(&self, other: &fastn_resolved::PropertySource) -> bool {
        match self {
            fastn_resolved::PropertySource::Caption
            | fastn_resolved::PropertySource::Body
            | fastn_resolved::PropertySource::Subsection
            | fastn_resolved::PropertySource::Default => self.eq(other),
            fastn_resolved::PropertySource::Header { name, .. } => {
                matches!(other, fastn_resolved::PropertySource::Header {
                    name: other_name, ..
               } if other_name.eq(name))
            }
        }
    }

    pub fn is_default(&self) -> bool {
        matches!(self, fastn_resolved::PropertySource::Default)
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum EventName {
    Click,
    MouseEnter,
    MouseLeave,
    ClickOutside,
    GlobalKey(Vec<String>),
    GlobalKeySeq(Vec<String>),
    Input,
    Change,
    Blur,
    Focus,
    RivePlay(String),
    RiveStateChange(String),
    RivePause(String),
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ComponentDefinition {
    pub name: String,
    pub arguments: Vec<Argument>,
    pub definition: fastn_resolved::ComponentInvocation,
    pub css: Option<fastn_resolved::PropertyValue>,
    pub line_number: usize,
}

impl fastn_resolved::ComponentDefinition {
    pub fn new(
        name: &str,
        arguments: Vec<fastn_resolved::Argument>,
        definition: fastn_resolved::ComponentInvocation,
        css: Option<fastn_resolved::PropertyValue>,
        line_number: usize,
    ) -> fastn_resolved::ComponentDefinition {
        fastn_resolved::ComponentDefinition {
            name: name.to_string(),
            arguments,
            definition,
            css,
            line_number,
        }
    }
}

pub type Argument = fastn_resolved::Field;
