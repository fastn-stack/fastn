#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Component {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub name: String,
    pub properties: Vec<Property>,
    pub iteration: Box<Option<Loop>>,
    // pub condition: Box<Option<fastn_type::Expression>>,
    pub events: Vec<Event>,
    pub children: Vec<Component>,
    pub source: ComponentSource,
    pub line_number: usize,
}

impl fastn_type::Component {
    pub fn from_name(name: &str) -> fastn_type::Component {
        fastn_type::Component {
            id: None,
            name: name.to_string(),
            properties: vec![],
            iteration: Box::new(None),
            // condition: Box::new(None),
            events: vec![],
            children: vec![],
            source: Default::default(),
            line_number: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Loop {
    pub on: fastn_type::PropertyValue,
    pub alias: String,
    pub loop_counter_alias: Option<String>,
    pub line_number: usize,
}

#[derive(Debug, Clone, PartialEq, Default, serde::Deserialize, serde::Serialize)]
pub enum ComponentSource {
    #[default]
    Declaration,
    Variable,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Event {
    pub name: fastn_type::EventName,
    pub action: fastn_type::FunctionCall,
    pub line_number: usize,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Property {
    pub value: fastn_type::PropertyValue,
    pub source: fastn_type::PropertySource,
    // pub condition: Option<fastn_type::Expression>,
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

impl fastn_type::PropertySource {
    pub fn is_equal(&self, other: &fastn_type::PropertySource) -> bool {
        match self {
            fastn_type::PropertySource::Caption
            | fastn_type::PropertySource::Body
            | fastn_type::PropertySource::Subsection
            | fastn_type::PropertySource::Default => self.eq(other),
            fastn_type::PropertySource::Header { name, .. } => {
                matches!(other, fastn_type::PropertySource::Header {
                    name: other_name, ..
               } if other_name.eq(name))
            }
        }
    }

    pub fn is_default(&self) -> bool {
        matches!(self, fastn_type::PropertySource::Default)
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
