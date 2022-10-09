#[derive(serde::Deserialize, Clone, Debug, PartialEq, serde::Serialize)]
pub enum Element {
    Row(Row),
    Column(Column),
    Text(Text),
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

#[derive(serde::Deserialize, Debug, PartialEq, Default, Clone, serde::Serialize)]
pub struct Common {
    pub is_not_visible: bool,
    pub is_dummy: bool,
    pub padding: ftd::executor::Value<Option<i64>>,
    pub id: String,
}

pub fn default_column() -> Column {
    // TODO:
    Default::default()
}

pub fn text_from_properties(
    properties: &[ftd::interpreter2::Property],
    arguments: &[ftd::interpreter2::Argument],
    doc: &ftd::executor::TDoc,
    local_container: &[usize],
    line_number: usize,
) -> ftd::executor::Result<Text> {
    let text = ftd::executor::value::string("text", properties, arguments, doc, line_number)?
        .map(|v| ftd::executor::element::markup_inline(v.as_str()));
    let common = common_from_properties(properties, arguments, doc, local_container, line_number)?;
    Ok(Text { text, common })
}

pub fn row_from_properties(
    properties: &[ftd::interpreter2::Property],
    arguments: &[ftd::interpreter2::Argument],
    doc: &ftd::executor::TDoc,
    local_container: &[usize],
    line_number: usize,
    children: Vec<Element>,
) -> ftd::executor::Result<Row> {
    let common = common_from_properties(properties, arguments, doc, local_container, line_number)?;
    let container = container_from_properties(properties, arguments, doc, line_number, children)?;
    Ok(Row { container, common })
}

pub fn column_from_properties(
    properties: &[ftd::interpreter2::Property],
    arguments: &[ftd::interpreter2::Argument],
    doc: &ftd::executor::TDoc,
    local_container: &[usize],
    line_number: usize,
    children: Vec<Element>,
) -> ftd::executor::Result<Column> {
    let common = common_from_properties(properties, arguments, doc, local_container, line_number)?;
    let container = container_from_properties(properties, arguments, doc, line_number, children)?;
    Ok(Column { container, common })
}

pub fn common_from_properties(
    properties: &[ftd::interpreter2::Property],
    arguments: &[ftd::interpreter2::Argument],
    doc: &ftd::executor::TDoc,
    local_container: &[usize],
    line_number: usize,
) -> ftd::executor::Result<Common> {
    Ok(Common {
        is_not_visible: false,
        is_dummy: false,
        padding: ftd::executor::value::optional_i64(
            "padding",
            properties,
            arguments,
            doc,
            line_number,
        )?,
        id: ftd::executor::utils::get_string_container(local_container),
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
