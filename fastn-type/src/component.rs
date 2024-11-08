#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Component {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub name: String,
    pub properties: Vec<Property>,
    pub iteration: Box<Option<fastn_type::Loop>>,
    pub condition: Box<Option<fastn_type::Expression>>,
    pub events: Vec<fastn_type::Event>,
    pub children: Vec<fastn_type::Component>,
    pub source: fastn_type::ComponentSource,
    pub line_number: usize,
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
pub struct Property {
    pub value: fastn_type::PropertyValue,
    pub source: fastn_type::PropertySource,
    pub condition: Option<fastn_type::Expression>,
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

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Event {
    pub name: fastn_type::EventName,
    pub action: fastn_type::FunctionCall,
    pub line_number: usize,
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
