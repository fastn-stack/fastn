use crate::interpreter2::PropertyValue;
use crate::Map;

#[derive(serde::Deserialize, Clone, Debug, PartialEq, serde::Serialize)]
pub enum Element {
    Row(Row),
    Column(Column),
    Text(Text),
    Integer(Text),
    Boolean(Text),
    Image(Image),
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

#[derive(serde::Deserialize, Debug, PartialEq, Default, Clone, serde::Serialize)]
pub struct Image {
    pub src: ftd::executor::Value<ImageSrc>,
    pub common: Common,
}

#[derive(serde::Deserialize, Debug, Default, PartialEq, Clone, serde::Serialize)]
pub struct ImageSrc {
    pub light: ftd::executor::Value<String>,
    pub dark: ftd::executor::Value<String>,
}

impl ImageSrc {
    fn from_values(
        values: ftd::Map<ftd::interpreter2::PropertyValue>,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<ImageSrc> {
        let light = {
            let value = values
                .get("light")
                .ok_or(ftd::executor::Error::ParseError {
                    message: "`light` field in ftd.image-src not found".to_string(),
                    doc_id: doc.name.to_string(),
                    line_number,
                })?;
            ftd::executor::Value::new(
                value
                    .clone()
                    .resolve(&doc.itdoc(), line_number)?
                    .string(doc.name, line_number)?,
                Some(line_number),
                vec![value.into_property(ftd::interpreter2::PropertySource::header("light"))],
            )
        };

        let dark = {
            let value = values.get("dark").ok_or(ftd::executor::Error::ParseError {
                message: "`dark` field in ftd.image-src not found".to_string(),
                doc_id: doc.name.to_string(),
                line_number,
            })?;
            ftd::executor::Value::new(
                value
                    .clone()
                    .resolve(&doc.itdoc(), line_number)?
                    .string(doc.name, line_number)?,
                Some(line_number),
                vec![value.into_property(ftd::interpreter2::PropertySource::header("dark"))],
            )
        };

        Ok(ImageSrc { light, dark })
    }
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
    pub padding: ftd::executor::Value<Option<Length>>,
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

pub fn image_from_properties(
    properties: &[ftd::interpreter2::Property],
    events: &[ftd::interpreter2::Event],
    arguments: &[ftd::interpreter2::Argument],
    condition: &Option<ftd::interpreter2::Expression>,
    doc: &ftd::executor::TDoc,
    local_container: &[usize],
    line_number: usize,
) -> ftd::executor::Result<Image> {
    let src = {
        let src = ftd::executor::value::record(
            "src",
            properties,
            arguments,
            doc,
            line_number,
            "ftd#image-src",
        )?;
        ftd::executor::Value::new(
            ImageSrc::from_values(src.value, doc, line_number)?,
            Some(line_number),
            src.properties,
        )
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
    Ok(Image { src, common })
}

#[allow(clippy::too_many_arguments)]
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

#[allow(clippy::too_many_arguments)]
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
        padding: Length::optional_length(properties, arguments, doc, line_number)?,
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

#[derive(serde::Deserialize, Debug, PartialEq, Clone, serde::Serialize)]
pub enum Length {
    Px(i64),
    Percent(f64),
}

impl Default for Length {
    fn default() -> Length {
        Length::Px(0)
    }
}

impl Length {
    fn from_optional_values(
        value: Option<(String, Map<PropertyValue>)>,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Option<Self>> {
        if let Some(value) = value {
            Ok(Some(Length::from_values(value, doc, line_number)?))
        } else {
            Ok(None)
        }
    }

    fn from_values(
        value: (String, Map<PropertyValue>),
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Self> {
        match value.0.as_str() {
            "ftd#length.percent" => Ok(Length::Percent(
                value
                    .1
                    .get("value")
                    .unwrap()
                    .clone()
                    .resolve(&doc.itdoc(), line_number)?
                    .decimal(doc.name, line_number)?,
            )),
            "ftd#length.px" => Ok(Length::Px(
                value
                    .1
                    .get("value")
                    .unwrap()
                    .clone()
                    .resolve(&doc.itdoc(), line_number)?
                    .integer(doc.name, line_number)?,
            )),
            t => ftd::executor::utils::parse_error(
                format!("Unknown variant `{}` for or-type `ftd.length`", t),
                doc.name,
                line_number,
            ),
        }
    }

    fn optional_length(
        properties: &[ftd::interpreter2::Property],
        arguments: &[ftd::interpreter2::Argument],
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<ftd::executor::Value<Option<Length>>> {
        let or_type_value = ftd::executor::value::optional_or_type(
            "padding",
            properties,
            arguments,
            doc,
            line_number,
            "ftd#length",
        )?;

        Ok(ftd::executor::Value::new(
            Length::from_optional_values(or_type_value.value, doc, line_number)?,
            or_type_value.line_number,
            or_type_value.properties,
        ))
    }

    pub fn to_string(&self) -> String {
        match self {
            Length::Px(px) => format!("{}px", px),
            Length::Percent(p) => format!("{}%", p),
        }
    }
}
