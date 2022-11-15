#[derive(serde::Deserialize, Clone, Debug, PartialEq, serde::Serialize)]
pub enum Element {
    Row(Row),
    Column(Column),
    Text(Text),
    Integer(Text),
    Boolean(Text),
    Null,
}

#[derive(serde::Deserialize, Debug, Default, PartialEq, Clone, serde::Serialize)]
pub struct Row {
    pub container: Container,
    pub common: Common,
}

#[derive(serde::Deserialize, Debug, Default, PartialEq, Clone, serde::Serialize)]
pub struct Column {
    pub container: Container,
    pub common: Common,
}

#[derive(serde::Deserialize, Debug, PartialEq, Default, Clone, serde::Serialize)]
pub struct Text {
    pub text: ftd::executor::Value<Rendered>,
    pub common: Common,
}

#[derive(serde::Serialize, serde::Deserialize, Eq, PartialEq, Debug, Default, Clone)]
pub struct Rendered {
    pub original: String,
    pub rendered: String,
}

pub fn markup_inline(s: &str) -> Rendered {
    Rendered {
        original: s.to_string(),
        rendered: ftd::executor::markup::markup_inline(s),
    }
}

#[derive(serde::Deserialize, Debug, PartialEq, Default, Clone, serde::Serialize)]
pub struct Container {
    pub children: Vec<Element>,
}

pub type Event = ftd::interpreter2::Event;

#[derive(serde::Deserialize, Debug, PartialEq, Default, Clone, serde::Serialize)]
pub struct Common {
    pub is_not_visible: bool,
    pub event: Vec<Event>,
    pub is_dummy: bool,
    pub padding: ftd::executor::Value<Option<i64>>,
    pub data_id: String,
    pub line_number: usize,
    pub condition: Option<ftd::interpreter2::Expression>,
}

pub fn default_column() -> Column {
    // TODO:
    Default::default()
}

pub fn text_from_properties(
    properties: &[ftd::interpreter2::Property],
    events: &[ftd::interpreter2::Event],
    arguments: &[ftd::interpreter2::Argument],
    condition: &Option<ftd::interpreter2::Expression>,
    doc: &ftd::executor::TDoc,
    local_container: &[usize],
    line_number: usize,
) -> ftd::executor::Result<Text> {
    let text = ftd::executor::value::string("text", properties, arguments, doc, line_number)?
        .map(|v| ftd::executor::element::markup_inline(v.as_str()));
    let common = common_from_properties(
        properties,
        events,
        arguments,
        condition,
        doc,
        local_container,
        line_number,
    )?;
    Ok(Text { text, common })
}

pub fn integer_from_properties(
    properties: &[ftd::interpreter2::Property],
    events: &[ftd::interpreter2::Event],
    arguments: &[ftd::interpreter2::Argument],
    condition: &Option<ftd::interpreter2::Expression>,
    doc: &ftd::executor::TDoc,
    local_container: &[usize],
    line_number: usize,
) -> ftd::executor::Result<Text> {
    let value = ftd::executor::value::i64("value", properties, arguments, doc, line_number)?;
    let num = format_num::NumberFormat::new();
    let text = match ftd::executor::value::optional_string(
        "format",
        properties,
        arguments,
        doc,
        line_number,
    )?
    .value
    {
        Some(f) => value.map(|v| {
            ftd::executor::element::markup_inline(num.format(f.as_str(), v as f64).as_str())
        }),
        None => value.map(|v| ftd::executor::element::markup_inline(v.to_string().as_str())),
    };
    let common = common_from_properties(
        properties,
        events,
        arguments,
        condition,
        doc,
        local_container,
        line_number,
    )?;
    Ok(Text { text, common })
}

pub fn boolean_from_properties(
    properties: &[ftd::interpreter2::Property],
    events: &[ftd::interpreter2::Event],
    arguments: &[ftd::interpreter2::Argument],
    condition: &Option<ftd::interpreter2::Expression>,
    doc: &ftd::executor::TDoc,
    local_container: &[usize],
    line_number: usize,
) -> ftd::executor::Result<Text> {
    let value = ftd::executor::value::bool("value", properties, arguments, doc, line_number)?;
    let text = value.map(|v| ftd::executor::element::markup_inline(v.to_string().as_str()));
    let common = common_from_properties(
        properties,
        events,
        arguments,
        condition,
        doc,
        local_container,
        line_number,
    )?;
    Ok(Text { text, common })
}

pub fn row_from_properties(
    properties: &[ftd::interpreter2::Property],
    events: &[ftd::interpreter2::Event],
    arguments: &[ftd::interpreter2::Argument],
    condition: &Option<ftd::interpreter2::Expression>,
    doc: &ftd::executor::TDoc,
    local_container: &[usize],
    line_number: usize,
    children: Vec<Element>,
) -> ftd::executor::Result<Row> {
    let common = common_from_properties(
        properties,
        events,
        arguments,
        condition,
        doc,
        local_container,
        line_number,
    )?;
    let container = container_from_properties(properties, arguments, doc, line_number, children)?;
    Ok(Row { container, common })
}

pub fn column_from_properties(
    properties: &[ftd::interpreter2::Property],
    events: &[ftd::interpreter2::Event],
    arguments: &[ftd::interpreter2::Argument],
    condition: &Option<ftd::interpreter2::Expression>,
    doc: &ftd::executor::TDoc,
    local_container: &[usize],
    line_number: usize,
    children: Vec<Element>,
) -> ftd::executor::Result<Column> {
    let common = common_from_properties(
        properties,
        events,
        arguments,
        condition,
        doc,
        local_container,
        line_number,
    )?;
    let container = container_from_properties(properties, arguments, doc, line_number, children)?;
    Ok(Column { container, common })
}

pub fn common_from_properties(
    properties: &[ftd::interpreter2::Property],
    events: &[ftd::interpreter2::Event],
    arguments: &[ftd::interpreter2::Argument],
    condition: &Option<ftd::interpreter2::Expression>,
    doc: &ftd::executor::TDoc,
    local_container: &[usize],
    line_number: usize,
) -> ftd::executor::Result<Common> {
    let is_visible = if let Some(condition) = condition {
        condition.eval(&doc.itdoc())?
    } else {
        true
    };

    Ok(Common {
        is_not_visible: !is_visible,
        event: events.to_owned(),
        is_dummy: false,
        padding: ftd::executor::value::optional_i64(
            "padding",
            properties,
            arguments,
            doc,
            line_number,
        )?,
        condition: condition.to_owned(),
        data_id: ftd::executor::utils::get_string_container(local_container),
        line_number,
    })
}

pub fn container_from_properties(
    _properties: &[ftd::interpreter2::Property],
    _arguments: &[ftd::interpreter2::Argument],
    _doc: &ftd::executor::TDoc,
    _line_number: usize,
    children: Vec<Element>,
) -> ftd::executor::Result<Container> {
    Ok(Container { children })
}
