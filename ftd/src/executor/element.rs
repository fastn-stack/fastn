#[derive(serde::Deserialize, Clone, Debug, PartialEq, serde::Serialize)]
pub enum Element {
    Row(Row),
    Column(Column),
    Container(ContainerElement),
    Document(Box<Document>),
    Text(Text),
    Integer(Text),
    Boolean(Text),
    Decimal(Text),
    Image(Image),
    Code(Code),
    Iframe(Iframe),
    TextInput(TextInput),
    RawElement(RawElement),
    IterativeElement(IterativeElement),
    CheckBox(CheckBox),
    WebComponent(WebComponent),
    Rive(Rive),
    Null { line_number: usize },
}

impl Element {
    pub(crate) fn get_common(&self) -> Option<&Common> {
        match self {
            Element::Row(r) => Some(&r.common),
            Element::Column(c) => Some(&c.common),
            Element::Container(e) => Some(&e.common),
            Element::Text(t) => Some(&t.common),
            Element::Integer(i) => Some(&i.common),
            Element::Boolean(b) => Some(&b.common),
            Element::Decimal(d) => Some(&d.common),
            Element::Image(i) => Some(&i.common),
            Element::Code(c) => Some(&c.common),
            Element::Iframe(i) => Some(&i.common),
            Element::TextInput(i) => Some(&i.common),
            Element::CheckBox(c) => Some(&c.common),
            Element::Document(_) => None,
            Element::Null { .. } => None,
            Element::RawElement(_) => None,
            Element::WebComponent(_) => None,
            Element::Rive(_) => None,
            Element::IterativeElement(i) => i.element.get_common(),
        }
    }

    pub(crate) fn get_children(&mut self) -> Option<&mut Vec<Element>> {
        match self {
            Element::Row(r) => Some(&mut r.container.children),
            Element::Column(c) => Some(&mut c.container.children),
            Element::Document(d) => Some(&mut d.children),
            Element::RawElement(r) => Some(&mut r.children),
            _ => None,
        }
    }

    pub(crate) fn is_document(&self) -> bool {
        matches!(self, Element::Document(_))
    }

    pub(crate) fn line_number(&self) -> usize {
        match self {
            Element::Row(r) => r.common.line_number,
            Element::Column(c) => c.common.line_number,
            Element::Container(e) => e.common.line_number,
            Element::Document(d) => d.line_number,
            Element::Text(t) => t.common.line_number,
            Element::Integer(i) => i.common.line_number,
            Element::Boolean(b) => b.common.line_number,
            Element::Decimal(d) => d.common.line_number,
            Element::Image(i) => i.common.line_number,
            Element::Code(c) => c.common.line_number,
            Element::Iframe(i) => i.common.line_number,
            Element::TextInput(t) => t.common.line_number,
            Element::RawElement(r) => r.line_number,
            Element::IterativeElement(i) => i.iteration.line_number,
            Element::CheckBox(c) => c.common.line_number,
            Element::WebComponent(w) => w.line_number,
            Element::Rive(r) => r.common.line_number,
            Element::Null { line_number } => *line_number,
        }
    }
}

#[derive(serde::Deserialize, Debug, Default, PartialEq, Clone, serde::Serialize)]
pub struct RawElement {
    pub name: String,
    pub properties: Vec<(String, fastn_type::Property)>,
    pub condition: Option<ftd::interpreter::Expression>,
    pub children: Vec<Element>,
    pub events: Vec<Event>,
    pub line_number: usize,
}

#[derive(serde::Deserialize, Debug, PartialEq, Clone, serde::Serialize)]
pub struct IterativeElement {
    pub element: Box<ftd::executor::Element>,
    pub iteration: fastn_type::Loop,
}

#[derive(serde::Deserialize, Debug, Default, PartialEq, Clone, serde::Serialize)]
pub struct WebComponent {
    pub name: String,
    pub properties: ftd::Map<fastn_type::PropertyValue>,
    pub device: Option<ftd::executor::Device>,
    pub line_number: usize,
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

#[derive(serde::Deserialize, Debug, Default, PartialEq, Clone, serde::Serialize)]
pub struct Rive {
    pub src: ftd::executor::Value<String>,
    pub canvas_width: ftd::executor::Value<Option<i64>>,
    pub canvas_height: ftd::executor::Value<Option<i64>>,
    pub state_machine: ftd::executor::Value<Vec<String>>,
    pub autoplay: ftd::executor::Value<bool>,
    pub artboard: ftd::executor::Value<Option<String>>,
    pub common: Common,
}

#[derive(serde::Deserialize, Debug, Default, PartialEq, Clone, serde::Serialize)]
pub struct ContainerElement {
    pub common: Common,
    pub children: Vec<ftd::executor::Element>,
    pub display: ftd::executor::Value<Option<ftd::executor::Display>>,
}

#[derive(serde::Deserialize, Debug, Default, PartialEq, Clone, serde::Serialize)]
pub struct HTMLData {
    pub title: ftd::executor::Value<Option<String>>,
    pub og_title: ftd::executor::Value<Option<String>>,
    pub twitter_title: ftd::executor::Value<Option<String>>,
    pub description: ftd::executor::Value<Option<String>>,
    pub og_description: ftd::executor::Value<Option<String>>,
    pub twitter_description: ftd::executor::Value<Option<String>>,
    pub og_image: ftd::executor::Value<Option<ftd::executor::RawImage>>,
    pub twitter_image: ftd::executor::Value<Option<ftd::executor::RawImage>>,
    pub theme_color: ftd::executor::Value<Option<ftd::executor::Color>>,
}

#[derive(serde::Deserialize, Debug, Default, PartialEq, Clone, serde::Serialize)]
pub struct Document {
    pub data: HTMLData,
    pub breakpoint_width: ftd::executor::Value<Option<ftd::executor::BreakpointWidth>>,
    pub children: Vec<Element>,
    pub line_number: usize,
}

#[derive(serde::Deserialize, Debug, PartialEq, Default, Clone, serde::Serialize)]
pub struct Text {
    pub text: ftd::executor::Value<Rendered>,
    pub text_align: ftd::executor::Value<Option<ftd::executor::TextAlign>>,
    pub text_indent: ftd::executor::Value<Option<ftd::executor::Length>>,
    pub line_clamp: ftd::executor::Value<Option<i64>>,
    pub common: Common,
    pub style: ftd::executor::Value<Option<ftd::executor::TextStyle>>,
    pub display: ftd::executor::Value<Option<ftd::executor::Display>>,
}

impl Text {
    pub(crate) fn set_auto_id(&mut self) {
        if self
            .common
            .region
            .value
            .as_ref()
            .filter(|r| r.is_heading())
            .is_some()
            && self.common.id.value.is_none()
        {
            self.common.id = ftd::executor::Value::new(
                Some(slug::slugify(self.text.value.original.as_str())),
                Some(self.common.line_number),
                vec![],
            )
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Eq, PartialEq, Debug, Default, Clone)]
pub struct Rendered {
    pub original: String,
    pub rendered: String,
}

#[derive(serde::Deserialize, Debug, PartialEq, Default, Clone, serde::Serialize)]
pub struct Image {
    pub src: ftd::executor::Value<ImageSrc>,
    pub alt: ftd::executor::Value<Option<String>>,
    pub fit: ftd::executor::Value<Option<ftd::executor::ImageFit>>,
    pub common: Common,
}

#[derive(serde::Deserialize, Debug, Default, PartialEq, Clone, serde::Serialize)]
pub struct ImageSrc {
    pub light: ftd::executor::Value<String>,
    pub dark: ftd::executor::Value<String>,
}

#[allow(dead_code)]
impl ImageSrc {
    pub(crate) fn optional_image(
        properties: &[fastn_type::Property],
        arguments: &[ftd::interpreter::Argument],
        doc: &ftd::executor::TDoc,
        line_number: usize,
        key: &str,
        inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
        component_name: &str,
    ) -> ftd::executor::Result<ftd::executor::Value<Option<ImageSrc>>> {
        let record_values = ftd::executor::value::optional_record_inherited(
            key,
            component_name,
            properties,
            arguments,
            doc,
            line_number,
            ftd::interpreter::FTD_IMAGE_SRC,
            inherited_variables,
        )?;

        Ok(ftd::executor::Value::new(
            ImageSrc::from_optional_values(record_values.value, doc, line_number)?,
            record_values.line_number,
            record_values.properties,
        ))
    }

    fn from_optional_values(
        or_type_value: Option<ftd::Map<fastn_type::PropertyValue>>,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Option<ImageSrc>> {
        if let Some(value) = or_type_value {
            Ok(Some(ImageSrc::from_values(value, doc, line_number)?))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn from_value(
        value: fastn_type::PropertyValue,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<ImageSrc> {
        use ftd::interpreter::PropertyValueExt;

        let value = value.resolve(&doc.itdoc(), line_number)?;
        let fields = match value.inner() {
            Some(fastn_type::Value::Record { name, fields })
                if name.eq(ftd::interpreter::FTD_IMAGE_SRC) =>
            {
                fields
            }
            t => {
                return ftd::executor::utils::parse_error(
                    format!(
                        "Expected value of type record `{}`, found: {:?}",
                        ftd::interpreter::FTD_IMAGE_SRC,
                        t
                    ),
                    doc.name,
                    line_number,
                )
            }
        };
        ImageSrc::from_values(fields, doc, line_number)
    }

    fn from_values(
        values: ftd::Map<fastn_type::PropertyValue>,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<ImageSrc> {
        use ftd::executor::fastn_type_functions::PropertyValueExt as _;
        use ftd::interpreter::{PropertyValueExt, ValueExt};

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
                vec![value.into_property(fastn_type::PropertySource::header("light"))],
            )
        };

        let dark = {
            if let Some(value) = values.get("dark") {
                ftd::executor::Value::new(
                    value
                        .clone()
                        .resolve(&doc.itdoc(), line_number)?
                        .string(doc.name, line_number)?,
                    Some(line_number),
                    vec![value.into_property(fastn_type::PropertySource::header("dark"))],
                )
            } else {
                light.clone()
            }
        };

        Ok(ImageSrc { light, dark })
    }

    pub fn image_pattern() -> (String, bool) {
        (
            r#"
                let c = {0};
                if (typeof c === 'object' && !!c && "light" in c) {
                    if (data["ftd#dark-mode"] && "dark" in c){ c.dark } else { c.light }
                } else {
                    c
                }
            "#
            .to_string(),
            true,
        )
    }
}

#[derive(serde::Deserialize, Debug, Default, PartialEq, Clone, serde::Serialize)]
pub struct RawImage {
    pub src: ftd::executor::Value<String>,
}

impl RawImage {
    pub(crate) fn optional_image(
        properties: &[fastn_type::Property],
        arguments: &[ftd::interpreter::Argument],
        doc: &ftd::executor::TDoc,
        line_number: usize,
        key: &str,
        inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
        component_name: &str,
    ) -> ftd::executor::Result<ftd::executor::Value<Option<RawImage>>> {
        let record_values = ftd::executor::value::optional_record_inherited(
            key,
            component_name,
            properties,
            arguments,
            doc,
            line_number,
            ftd::interpreter::FTD_RAW_IMAGE_SRC,
            inherited_variables,
        )?;

        Ok(ftd::executor::Value::new(
            RawImage::from_optional_values(record_values.value, doc, line_number)?,
            record_values.line_number,
            record_values.properties,
        ))
    }

    fn from_optional_values(
        or_type_value: Option<ftd::Map<fastn_type::PropertyValue>>,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Option<RawImage>> {
        if let Some(value) = or_type_value {
            Ok(Some(RawImage::from_values(value, doc, line_number)?))
        } else {
            Ok(None)
        }
    }

    fn from_values(
        values: ftd::Map<fastn_type::PropertyValue>,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<RawImage> {
        use ftd::executor::fastn_type_functions::PropertyValueExt as _;
        use ftd::interpreter::{PropertyValueExt, ValueExt};

        let src = {
            let value = values.get("src").ok_or(ftd::executor::Error::ParseError {
                message: "`src` field in ftd.raw-image-src not found".to_string(),
                doc_id: doc.name.to_string(),
                line_number,
            })?;
            ftd::executor::Value::new(
                value
                    .clone()
                    .resolve(&doc.itdoc(), line_number)?
                    .string(doc.name, line_number)?,
                Some(line_number),
                vec![value.into_property(fastn_type::PropertySource::header("src"))],
            )
        };

        Ok(RawImage { src })
    }

    pub fn image_pattern() -> (String, bool) {
        (
            r#"
                let c = {0};
                if (typeof c === 'object' && !!c && "src" in c) {c.src} else {c}
            "#
            .to_string(),
            true,
        )
    }
}

#[derive(serde::Deserialize, Debug, PartialEq, Default, Clone, serde::Serialize)]
pub struct Code {
    pub text: ftd::executor::Value<Rendered>,
    pub text_align: ftd::executor::Value<Option<ftd::executor::TextAlign>>,
    pub line_clamp: ftd::executor::Value<Option<i64>>,
    pub common: Common,
}

#[allow(clippy::too_many_arguments)]
pub fn code_from_properties(
    properties: &[fastn_type::Property],
    events: &[fastn_type::Event],
    arguments: &[ftd::interpreter::Argument],
    condition: &Option<ftd::interpreter::Expression>,
    doc: &mut ftd::executor::TDoc,
    local_container: &[usize],
    line_number: usize,
    inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
    device: Option<ftd::executor::Device>,
) -> ftd::executor::Result<Code> {
    // TODO: `text`, `lang` and `theme` cannot have condition

    let text = ftd::executor::value::optional_string(
        "text",
        "ftd#code",
        properties,
        arguments,
        doc,
        line_number,
    )?;
    if text.value.is_none() && condition.is_none() {
        // TODO: Check condition if `value is not null` is there
        return ftd::executor::utils::parse_error(
            "Expected string for text property",
            doc.name,
            line_number,
        );
    }

    let lang = ftd::executor::value::string_with_default(
        "lang",
        "ftd#code",
        properties,
        arguments,
        "txt",
        doc,
        line_number,
    )?;

    let theme = ftd::executor::value::string_with_default(
        "theme",
        "ftd#code",
        properties,
        arguments,
        ftd::executor::code::DEFAULT_THEME,
        doc,
        line_number,
    )?;

    let text = ftd::executor::Value::new(
        ftd::executor::element::code_with_theme(
            text.value.unwrap_or_default().as_str(),
            lang.value.as_str(),
            theme.value.as_str(),
            doc.name,
        )?,
        text.line_number,
        text.properties,
    );

    let common = common_from_properties(
        properties,
        events,
        arguments,
        condition,
        doc,
        local_container,
        line_number,
        inherited_variables,
        "ftd#code",
        device,
    )?;

    Ok(Code {
        text,
        text_align: ftd::executor::TextAlign::optional_text_align(
            properties,
            arguments,
            doc,
            line_number,
            "text-align",
            inherited_variables,
            "ftd#code",
        )?,
        common,
        line_clamp: ftd::executor::value::optional_i64(
            "line-clamp",
            "ftd#code",
            properties,
            arguments,
            doc,
            line_number,
            inherited_variables,
        )?,
    })
}

#[derive(serde::Deserialize, Debug, PartialEq, Default, Clone, serde::Serialize)]
pub struct Iframe {
    pub src: ftd::executor::Value<Option<String>>,
    pub srcdoc: ftd::executor::Value<Option<String>>,
    /// iframe can load lazily.
    pub loading: ftd::executor::Value<ftd::executor::Loading>,
    pub common: Common,
}

#[allow(clippy::too_many_arguments)]
pub fn iframe_from_properties(
    properties: &[fastn_type::Property],
    events: &[fastn_type::Event],
    arguments: &[ftd::interpreter::Argument],
    condition: &Option<ftd::interpreter::Expression>,
    doc: &mut ftd::executor::TDoc,
    local_container: &[usize],
    line_number: usize,
    inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
    device: Option<ftd::executor::Device>,
) -> ftd::executor::Result<Iframe> {
    // TODO: `youtube` should not be conditional
    let srcdoc = ftd::executor::value::optional_string(
        "srcdoc",
        "ftd#iframe",
        properties,
        arguments,
        doc,
        line_number,
    )?;

    let src = {
        let src = ftd::executor::value::optional_string(
            "src",
            "ftd#iframe",
            properties,
            arguments,
            doc,
            line_number,
        )?;

        let youtube = ftd::executor::value::optional_string(
            "youtube",
            "ftd#iframe",
            properties,
            arguments,
            doc,
            line_number,
        )?
        .map(|v| v.and_then(|v| ftd::executor::youtube_id::from_raw(v.as_str())));

        if [
            src.value.is_some(),
            youtube.value.is_some(),
            srcdoc.value.is_some(),
        ]
        .into_iter()
        .filter(|b| *b)
        .count()
            > 1
        {
            return ftd::executor::utils::parse_error(
                "Two or more than two values are provided among src, youtube and srcdoc.",
                doc.name,
                src.line_number.unwrap_or_else(|| {
                    youtube
                        .line_number
                        .unwrap_or_else(|| srcdoc.line_number.unwrap_or(line_number))
                }),
            );
        }
        if src.value.is_none() && youtube.value.is_none() && srcdoc.value.is_none() {
            return ftd::executor::utils::parse_error(
                "Either srcdoc or src or youtube id is required",
                doc.name,
                line_number,
            );
        }
        if src.value.is_some() {
            src
        } else {
            youtube
        }
    };

    let loading = ftd::executor::Loading::loading_with_default(
        properties,
        arguments,
        doc,
        line_number,
        "loading",
        inherited_variables,
        "ftd#iframe",
    )?;

    let common = common_from_properties(
        properties,
        events,
        arguments,
        condition,
        doc,
        local_container,
        line_number,
        inherited_variables,
        "ftd#iframe",
        device,
    )?;

    Ok(Iframe {
        src,
        srcdoc,
        loading,
        common,
    })
}

pub fn markup_inline(s: &str) -> Rendered {
    Rendered {
        original: s.to_string(),
        rendered: ftd::executor::markup::markup_inline(s),
    }
}

pub fn code_with_theme(
    code: &str,
    ext: &str,
    theme: &str,
    doc_id: &str,
) -> ftd::executor::Result<Rendered> {
    Ok(Rendered {
        original: code.to_string(),
        rendered: ftd::executor::code::code(
            code.replace("\n\\-- ", "\n-- ")
                .replace("\\$", "$")
                .as_str(),
            ext,
            theme,
            doc_id,
        )?,
    })
}

#[derive(serde::Deserialize, Debug, PartialEq, Default, Clone, serde::Serialize)]
pub struct Container {
    pub wrap: ftd::executor::Value<Option<bool>>,
    pub align_content: ftd::executor::Value<Option<ftd::executor::Alignment>>,
    pub spacing: ftd::executor::Value<Option<ftd::executor::Spacing>>,
    pub children: Vec<Element>,
    pub device: Option<ftd::executor::Device>,
}

pub type Event = fastn_type::Event;

#[derive(serde::Deserialize, Debug, PartialEq, Default, Clone, serde::Serialize)]
pub struct Common {
    pub id: ftd::executor::Value<Option<String>>,
    pub is_not_visible: bool,
    pub event: Vec<Event>,
    pub is_dummy: bool,
    pub z_index: ftd::executor::Value<Option<i64>>,
    pub left: ftd::executor::Value<Option<ftd::executor::Length>>,
    pub right: ftd::executor::Value<Option<ftd::executor::Length>>,
    pub top: ftd::executor::Value<Option<ftd::executor::Length>>,
    pub bottom: ftd::executor::Value<Option<ftd::executor::Length>>,
    pub anchor: ftd::executor::Value<Option<ftd::executor::Anchor>>,
    pub role: ftd::executor::Value<Option<ftd::executor::ResponsiveType>>,
    pub region: ftd::executor::Value<Option<ftd::executor::Region>>,
    pub cursor: ftd::executor::Value<Option<ftd::executor::Cursor>>,
    pub classes: ftd::executor::Value<Vec<String>>,
    pub padding: ftd::executor::Value<Option<ftd::executor::Length>>,
    pub padding_left: ftd::executor::Value<Option<ftd::executor::Length>>,
    pub padding_right: ftd::executor::Value<Option<ftd::executor::Length>>,
    pub padding_top: ftd::executor::Value<Option<ftd::executor::Length>>,
    pub padding_bottom: ftd::executor::Value<Option<ftd::executor::Length>>,
    pub padding_horizontal: ftd::executor::Value<Option<ftd::executor::Length>>,
    pub padding_vertical: ftd::executor::Value<Option<ftd::executor::Length>>,
    pub margin: ftd::executor::Value<Option<ftd::executor::Length>>,
    pub margin_left: ftd::executor::Value<Option<ftd::executor::Length>>,
    pub margin_right: ftd::executor::Value<Option<ftd::executor::Length>>,
    pub margin_top: ftd::executor::Value<Option<ftd::executor::Length>>,
    pub margin_bottom: ftd::executor::Value<Option<ftd::executor::Length>>,
    pub margin_horizontal: ftd::executor::Value<Option<ftd::executor::Length>>,
    pub margin_vertical: ftd::executor::Value<Option<ftd::executor::Length>>,
    pub border_width: ftd::executor::Value<Option<ftd::executor::Length>>,
    pub border_radius: ftd::executor::Value<Option<ftd::executor::Length>>,
    pub border_color: ftd::executor::Value<Option<ftd::executor::Color>>,
    pub border_bottom_width: ftd::executor::Value<Option<ftd::executor::Length>>,
    pub border_bottom_color: ftd::executor::Value<Option<ftd::executor::Color>>,
    pub border_top_width: ftd::executor::Value<Option<ftd::executor::Length>>,
    pub border_top_color: ftd::executor::Value<Option<ftd::executor::Color>>,
    pub border_left_width: ftd::executor::Value<Option<ftd::executor::Length>>,
    pub border_left_color: ftd::executor::Value<Option<ftd::executor::Color>>,
    pub border_right_width: ftd::executor::Value<Option<ftd::executor::Length>>,
    pub border_right_color: ftd::executor::Value<Option<ftd::executor::Color>>,
    pub border_top_left_radius: ftd::executor::Value<Option<ftd::executor::Length>>,
    pub border_top_right_radius: ftd::executor::Value<Option<ftd::executor::Length>>,
    pub border_bottom_left_radius: ftd::executor::Value<Option<ftd::executor::Length>>,
    pub border_bottom_right_radius: ftd::executor::Value<Option<ftd::executor::Length>>,
    pub width: ftd::executor::Value<Option<ftd::executor::Resizing>>,
    pub height: ftd::executor::Value<Option<ftd::executor::Resizing>>,
    pub min_width: ftd::executor::Value<Option<ftd::executor::Resizing>>,
    pub max_width: ftd::executor::Value<Option<ftd::executor::Resizing>>,
    pub min_height: ftd::executor::Value<Option<ftd::executor::Resizing>>,
    pub max_height: ftd::executor::Value<Option<ftd::executor::Resizing>>,
    pub link: ftd::executor::Value<Option<String>>,
    pub open_in_new_tab: ftd::executor::Value<Option<bool>>,
    pub background: ftd::executor::Value<Option<ftd::executor::Background>>,
    pub color: ftd::executor::Value<Option<ftd::executor::Color>>,
    pub align_self: ftd::executor::Value<Option<ftd::executor::AlignSelf>>,
    pub data_id: String,
    pub line_number: usize,
    pub condition: Option<ftd::interpreter::Expression>,
    pub overflow: ftd::executor::Value<Option<ftd::executor::Overflow>>,
    pub overflow_x: ftd::executor::Value<Option<ftd::executor::Overflow>>,
    pub overflow_y: ftd::executor::Value<Option<ftd::executor::Overflow>>,
    pub opacity: ftd::executor::Value<Option<f64>>,
    pub resize: ftd::executor::Value<Option<ftd::executor::Resize>>,
    pub white_space: ftd::executor::Value<Option<ftd::executor::WhiteSpace>>,
    pub text_transform: ftd::executor::Value<Option<ftd::executor::TextTransform>>,
    pub sticky: ftd::executor::Value<Option<bool>>,
    pub border_style: ftd::executor::Value<Option<ftd::executor::BorderStyle>>,
    pub border_style_vertical: ftd::executor::Value<Option<ftd::executor::BorderStyle>>,
    pub border_style_horizontal: ftd::executor::Value<Option<ftd::executor::BorderStyle>>,
    pub border_style_left: ftd::executor::Value<Option<ftd::executor::BorderStyle>>,
    pub border_style_right: ftd::executor::Value<Option<ftd::executor::BorderStyle>>,
    pub border_style_top: ftd::executor::Value<Option<ftd::executor::BorderStyle>>,
    pub border_style_bottom: ftd::executor::Value<Option<ftd::executor::BorderStyle>>,
    pub shadow: ftd::executor::Value<Option<ftd::executor::Shadow>>,
    pub device: Option<ftd::executor::Device>,
}

pub fn default_column() -> Column {
    ftd::executor::Column {
        container: Default::default(),
        common: ftd::executor::Common {
            width: ftd::executor::Value::new(
                Some(ftd::executor::Resizing::FillContainer),
                None,
                vec![],
            ),
            height: ftd::executor::Value::new(
                Some(ftd::executor::Resizing::FillContainer),
                None,
                vec![],
            ),
            ..Default::default()
        },
    }
}

#[allow(clippy::too_many_arguments)]
pub fn text_from_properties(
    properties: &[fastn_type::Property],
    events: &[fastn_type::Event],
    arguments: &[ftd::interpreter::Argument],
    condition: &Option<ftd::interpreter::Expression>,
    doc: &mut ftd::executor::TDoc,
    local_container: &[usize],
    is_dummy: bool,
    line_number: usize,
    inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
    device: Option<ftd::executor::Device>,
) -> ftd::executor::Result<Text> {
    let text = ftd::executor::value::dummy_optional_string(
        "text",
        "ftd#text",
        properties,
        arguments,
        doc,
        is_dummy,
        line_number,
        inherited_variables,
    )?;
    if text.value.is_none() && condition.is_none() {
        // TODO: Check condition if `value is not null` is there
        return ftd::executor::utils::parse_error(
            "Expected string for text property",
            doc.name,
            line_number,
        );
    }
    let text = text.map(|v| ftd::executor::element::markup_inline(v.unwrap_or_default().as_str()));
    let common = common_from_properties(
        properties,
        events,
        arguments,
        condition,
        doc,
        local_container,
        line_number,
        inherited_variables,
        "ftd#text",
        device,
    )?;
    Ok(Text {
        text,
        text_indent: ftd::executor::Length::optional_length(
            properties,
            arguments,
            doc,
            line_number,
            "text-indent",
            inherited_variables,
            "ftd#text",
        )?,
        text_align: ftd::executor::TextAlign::optional_text_align(
            properties,
            arguments,
            doc,
            line_number,
            "text-align",
            inherited_variables,
            "ftd#text",
        )?,
        line_clamp: ftd::executor::value::optional_i64(
            "line-clamp",
            "ftd#text",
            properties,
            arguments,
            doc,
            line_number,
            inherited_variables,
        )?,
        common,
        style: ftd::executor::TextStyle::optional_text_style(
            properties,
            arguments,
            doc,
            line_number,
            "style",
            inherited_variables,
            "ftd#text",
        )?,
        display: ftd::executor::Display::optional_display(
            properties,
            arguments,
            doc,
            line_number,
            "display",
            inherited_variables,
            "ftd#text",
        )?,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn integer_from_properties(
    properties: &[fastn_type::Property],
    events: &[fastn_type::Event],
    arguments: &[ftd::interpreter::Argument],
    condition: &Option<ftd::interpreter::Expression>,
    doc: &mut ftd::executor::TDoc,
    local_container: &[usize],
    line_number: usize,
    inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
    device: Option<ftd::executor::Device>,
) -> ftd::executor::Result<Text> {
    let value = ftd::executor::value::i64(
        "value",
        "ftd#integer",
        properties,
        arguments,
        doc,
        line_number,
    )?;
    let num = format_num::NumberFormat::new();
    let text = match ftd::executor::value::optional_string(
        "format",
        "ftd#integer",
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
        inherited_variables,
        "ftd#integer",
        device,
    )?;
    Ok(Text {
        text,
        common,
        text_indent: ftd::executor::Length::optional_length(
            properties,
            arguments,
            doc,
            line_number,
            "text-indent",
            inherited_variables,
            "ftd#integer",
        )?,
        text_align: ftd::executor::TextAlign::optional_text_align(
            properties,
            arguments,
            doc,
            line_number,
            "text-align",
            inherited_variables,
            "ftd#integer",
        )?,
        line_clamp: ftd::executor::value::optional_i64(
            "line-clamp",
            "ftd#integer",
            properties,
            arguments,
            doc,
            line_number,
            inherited_variables,
        )?,
        style: ftd::executor::TextStyle::optional_text_style(
            properties,
            arguments,
            doc,
            line_number,
            "style",
            inherited_variables,
            "ftd#integer",
        )?,
        display: ftd::executor::Display::optional_display(
            properties,
            arguments,
            doc,
            line_number,
            "display",
            inherited_variables,
            "ftd#integer",
        )?,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn decimal_from_properties(
    properties: &[fastn_type::Property],
    events: &[fastn_type::Event],
    arguments: &[ftd::interpreter::Argument],
    condition: &Option<ftd::interpreter::Expression>,
    doc: &mut ftd::executor::TDoc,
    local_container: &[usize],
    line_number: usize,
    inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
    device: Option<ftd::executor::Device>,
) -> ftd::executor::Result<Text> {
    let value = ftd::executor::value::f64(
        "value",
        "ftd#decimal",
        properties,
        arguments,
        doc,
        line_number,
    )?;
    let num = format_num::NumberFormat::new();
    let text = match ftd::executor::value::optional_string(
        "format",
        "ftd#decimal",
        properties,
        arguments,
        doc,
        line_number,
    )?
    .value
    {
        Some(f) => {
            value.map(|v| ftd::executor::element::markup_inline(num.format(f.as_str(), v).as_str()))
        }
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
        inherited_variables,
        "ftd#decimal",
        device,
    )?;
    Ok(Text {
        text,
        common,
        text_indent: ftd::executor::Length::optional_length(
            properties,
            arguments,
            doc,
            line_number,
            "text-indent",
            inherited_variables,
            "ftd#decimal",
        )?,
        text_align: ftd::executor::TextAlign::optional_text_align(
            properties,
            arguments,
            doc,
            line_number,
            "text-align",
            inherited_variables,
            "ftd#decimal",
        )?,
        line_clamp: ftd::executor::value::optional_i64(
            "line-clamp",
            "ftd#decimal",
            properties,
            arguments,
            doc,
            line_number,
            inherited_variables,
        )?,
        style: ftd::executor::TextStyle::optional_text_style(
            properties,
            arguments,
            doc,
            line_number,
            "style",
            inherited_variables,
            "ftd#decimal",
        )?,
        display: ftd::executor::Display::optional_display(
            properties,
            arguments,
            doc,
            line_number,
            "display",
            inherited_variables,
            "ftd#decimal",
        )?,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn boolean_from_properties(
    properties: &[fastn_type::Property],
    events: &[fastn_type::Event],
    arguments: &[ftd::interpreter::Argument],
    condition: &Option<ftd::interpreter::Expression>,
    doc: &mut ftd::executor::TDoc,
    local_container: &[usize],
    line_number: usize,
    inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
    device: Option<ftd::executor::Device>,
) -> ftd::executor::Result<Text> {
    let value = ftd::executor::value::bool(
        "value",
        "ftd#boolean",
        properties,
        arguments,
        doc,
        line_number,
    )?;
    let text = value.map(|v| ftd::executor::element::markup_inline(v.to_string().as_str()));
    let common = common_from_properties(
        properties,
        events,
        arguments,
        condition,
        doc,
        local_container,
        line_number,
        inherited_variables,
        "ftd#boolean",
        device,
    )?;
    Ok(Text {
        text,
        common,
        text_indent: ftd::executor::Length::optional_length(
            properties,
            arguments,
            doc,
            line_number,
            "text-indent",
            inherited_variables,
            "ftd#boolean",
        )?,
        text_align: ftd::executor::TextAlign::optional_text_align(
            properties,
            arguments,
            doc,
            line_number,
            "text-align",
            inherited_variables,
            "ftd#boolean",
        )?,
        line_clamp: ftd::executor::value::optional_i64(
            "line-clamp",
            "ftd#boolean",
            properties,
            arguments,
            doc,
            line_number,
            inherited_variables,
        )?,
        style: ftd::executor::TextStyle::optional_text_style(
            properties,
            arguments,
            doc,
            line_number,
            "style",
            inherited_variables,
            "ftd#boolean",
        )?,
        display: ftd::executor::Display::optional_display(
            properties,
            arguments,
            doc,
            line_number,
            "display",
            inherited_variables,
            "ftd#boolean",
        )?,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn image_from_properties(
    properties: &[fastn_type::Property],
    events: &[fastn_type::Event],
    arguments: &[ftd::interpreter::Argument],
    condition: &Option<ftd::interpreter::Expression>,
    doc: &mut ftd::executor::TDoc,
    local_container: &[usize],
    line_number: usize,
    inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
    device: Option<ftd::executor::Device>,
) -> ftd::executor::Result<Image> {
    let src = {
        let src = ftd::executor::value::record(
            "src",
            "ftd#image",
            properties,
            arguments,
            doc,
            line_number,
            ftd::interpreter::FTD_IMAGE_SRC,
        )?;
        ftd::executor::Value::new(
            ImageSrc::from_values(src.value, doc, line_number)?,
            Some(line_number),
            src.properties,
        )
    };

    let alt = ftd::executor::value::optional_string(
        "alt",
        "ftd#image",
        properties,
        arguments,
        doc,
        line_number,
    )?;

    let fit = ftd::executor::ImageFit::optional_image_fit(
        properties,
        arguments,
        doc,
        line_number,
        "fit",
        inherited_variables,
        "ftd#image",
    )?;

    let common = common_from_properties(
        properties,
        events,
        arguments,
        condition,
        doc,
        local_container,
        line_number,
        inherited_variables,
        "ftd#image",
        device,
    )?;
    Ok(Image {
        src,
        alt,
        fit,
        common,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn row_from_properties(
    properties: &[fastn_type::Property],
    events: &[fastn_type::Event],
    arguments: &[ftd::interpreter::Argument],
    condition: &Option<ftd::interpreter::Expression>,
    doc: &mut ftd::executor::TDoc,
    local_container: &[usize],
    line_number: usize,
    children: Vec<Element>,
    inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
    device: Option<ftd::executor::Device>,
) -> ftd::executor::Result<Row> {
    let common = common_from_properties(
        properties,
        events,
        arguments,
        condition,
        doc,
        local_container,
        line_number,
        inherited_variables,
        "ftd#row",
        device.clone(),
    )?;
    let container = container_from_properties(
        properties,
        arguments,
        doc,
        line_number,
        children,
        inherited_variables,
        "ftd#row",
        device,
    )?;
    Ok(Row { container, common })
}

#[allow(clippy::too_many_arguments)]
pub fn column_from_properties(
    properties: &[fastn_type::Property],
    events: &[fastn_type::Event],
    arguments: &[ftd::interpreter::Argument],
    condition: &Option<ftd::interpreter::Expression>,
    doc: &mut ftd::executor::TDoc,
    local_container: &[usize],
    line_number: usize,
    children: Vec<Element>,
    inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
    device: Option<ftd::executor::Device>,
) -> ftd::executor::Result<Column> {
    let common = common_from_properties(
        properties,
        events,
        arguments,
        condition,
        doc,
        local_container,
        line_number,
        inherited_variables,
        "ftd#column",
        device.clone(),
    )?;
    let container = container_from_properties(
        properties,
        arguments,
        doc,
        line_number,
        children,
        inherited_variables,
        "ftd#column",
        device,
    )?;
    Ok(Column { container, common })
}

#[allow(clippy::too_many_arguments)]
pub fn container_element_from_properties(
    properties: &[fastn_type::Property],
    events: &[fastn_type::Event],
    arguments: &[ftd::interpreter::Argument],
    condition: &Option<ftd::interpreter::Expression>,
    doc: &mut ftd::executor::TDoc,
    local_container: &[usize],
    line_number: usize,
    children: Vec<Element>,
    inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
    device: Option<ftd::executor::Device>,
) -> ftd::executor::Result<ContainerElement> {
    let common = common_from_properties(
        properties,
        events,
        arguments,
        condition,
        doc,
        local_container,
        line_number,
        inherited_variables,
        "ftd#container",
        device,
    )?;
    Ok(ContainerElement {
        common,
        children,
        display: ftd::executor::Display::optional_display(
            properties,
            arguments,
            doc,
            line_number,
            "display",
            inherited_variables,
            "ftd#container",
        )?,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn rive_from_properties(
    properties: &[fastn_type::Property],
    events: &[fastn_type::Event],
    arguments: &[ftd::interpreter::Argument],
    condition: &Option<ftd::interpreter::Expression>,
    doc: &mut ftd::executor::TDoc,
    local_container: &[usize],
    line_number: usize,
    inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
    device: Option<ftd::executor::Device>,
) -> ftd::executor::Result<Rive> {
    let component_name = "ftd#rive";
    let common = common_from_properties(
        properties,
        events,
        arguments,
        condition,
        doc,
        local_container,
        line_number,
        inherited_variables,
        component_name,
        device,
    )?;
    let rive = Rive {
        src: ftd::executor::value::string(
            "src",
            component_name,
            properties,
            arguments,
            doc,
            line_number,
        )?,
        canvas_width: ftd::executor::value::optional_i64(
            "canvas-width",
            component_name,
            properties,
            arguments,
            doc,
            line_number,
            inherited_variables,
        )?,
        canvas_height: ftd::executor::value::optional_i64(
            "canvas-height",
            component_name,
            properties,
            arguments,
            doc,
            line_number,
            inherited_variables,
        )?,
        state_machine: ftd::executor::value::string_list(
            "state-machine",
            component_name,
            properties,
            arguments,
            doc,
            line_number,
            inherited_variables,
        )?,
        autoplay: ftd::executor::value::bool(
            "autoplay",
            component_name,
            properties,
            arguments,
            doc,
            line_number,
        )?,
        artboard: ftd::executor::value::optional_string(
            "artboard",
            component_name,
            properties,
            arguments,
            doc,
            line_number,
        )?,
        common,
    };

    let id = rive
        .common
        .id
        .value
        .clone()
        .ok_or(ftd::executor::Error::ParseError {
            message: "id is required".to_string(),
            doc_id: doc.name.to_string(),
            line_number,
        })?;

    doc.rive_data.push(ftd::executor::RiveData {
        id,
        src: rive.src.value.to_string(),
        state_machine: rive.state_machine.value.clone(),
        artboard: rive.artboard.value.clone(),
        autoplay: rive.autoplay.value,
        events: events.to_vec(),
    });

    Ok(rive)
}

pub fn document_from_properties(
    properties: &[fastn_type::Property],
    arguments: &[ftd::interpreter::Argument],
    doc: &mut ftd::executor::TDoc,
    line_number: usize,
    children: Vec<Element>,
    inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
) -> ftd::executor::Result<Document> {
    Ok(Document {
        breakpoint_width: ftd::executor::BreakpointWidth::optional_breakpoint_width(
            properties,
            arguments,
            doc,
            line_number,
            "breakpoint",
            inherited_variables,
            "ftd#document",
        )?,
        data: html_data_from_properties(properties, arguments, doc, line_number, "ftd#document")?,
        children,
        line_number,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn html_data_from_properties(
    properties: &[fastn_type::Property],
    arguments: &[ftd::interpreter::Argument],
    doc: &mut ftd::executor::TDoc,
    line_number: usize,
    component_name: &str,
) -> ftd::executor::Result<HTMLData> {
    Ok(HTMLData {
        title: ftd::executor::value::optional_string(
            "title",
            component_name,
            properties,
            arguments,
            doc,
            line_number,
        )?,
        og_title: ftd::executor::value::optional_string(
            "og-title",
            component_name,
            properties,
            arguments,
            doc,
            line_number,
        )?,
        twitter_title: ftd::executor::value::optional_string(
            "twitter-title",
            component_name,
            properties,
            arguments,
            doc,
            line_number,
        )?,
        description: ftd::executor::value::optional_string(
            "description",
            component_name,
            properties,
            arguments,
            doc,
            line_number,
        )?,
        og_description: ftd::executor::value::optional_string(
            "og-description",
            component_name,
            properties,
            arguments,
            doc,
            line_number,
        )?,
        twitter_description: ftd::executor::value::optional_string(
            "twitter-description",
            component_name,
            properties,
            arguments,
            doc,
            line_number,
        )?,
        og_image: ftd::executor::RawImage::optional_image(
            properties,
            arguments,
            doc,
            line_number,
            "og-image",
            &Default::default(),
            component_name,
        )?,
        twitter_image: ftd::executor::RawImage::optional_image(
            properties,
            arguments,
            doc,
            line_number,
            "twitter-image",
            &Default::default(),
            component_name,
        )?,
        theme_color: ftd::executor::Color::optional_color(
            properties,
            arguments,
            doc,
            line_number,
            "theme-color",
            &Default::default(),
            component_name,
        )?,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn common_from_properties(
    properties: &[fastn_type::Property],
    events: &[fastn_type::Event],
    arguments: &[ftd::interpreter::Argument],
    condition: &Option<ftd::interpreter::Expression>,
    doc: &mut ftd::executor::TDoc,
    local_container: &[usize],
    line_number: usize,
    inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
    component_name: &str,
    device: Option<ftd::executor::Device>,
) -> ftd::executor::Result<Common> {
    let is_visible = if let Some(condition) = condition {
        condition.eval(&doc.itdoc())?
    } else {
        true
    };

    doc.js.extend(
        ftd::executor::value::string_list(
            "js",
            component_name,
            properties,
            arguments,
            doc,
            line_number,
            inherited_variables,
        )?
        .value,
    );

    doc.css.extend(
        ftd::executor::value::string_list(
            "css",
            component_name,
            properties,
            arguments,
            doc,
            line_number,
            inherited_variables,
        )?
        .value,
    );

    Ok(Common {
        id: ftd::executor::value::optional_string(
            "id",
            component_name,
            properties,
            arguments,
            doc,
            line_number,
        )?,
        is_not_visible: !is_visible,
        event: events.to_owned(),
        is_dummy: false,
        device,
        sticky: ftd::executor::value::optional_bool(
            "sticky",
            component_name,
            properties,
            arguments,
            doc,
            line_number,
            inherited_variables,
        )?,
        z_index: ftd::executor::value::optional_i64(
            "z-index",
            component_name,
            properties,
            arguments,
            doc,
            line_number,
            inherited_variables,
        )?,
        left: ftd::executor::Length::optional_length(
            properties,
            arguments,
            doc,
            line_number,
            "left",
            inherited_variables,
            component_name,
        )?,
        right: ftd::executor::Length::optional_length(
            properties,
            arguments,
            doc,
            line_number,
            "right",
            inherited_variables,
            component_name,
        )?,
        top: ftd::executor::Length::optional_length(
            properties,
            arguments,
            doc,
            line_number,
            "top",
            inherited_variables,
            component_name,
        )?,
        bottom: ftd::executor::Length::optional_length(
            properties,
            arguments,
            doc,
            line_number,
            "bottom",
            inherited_variables,
            component_name,
        )?,
        anchor: ftd::executor::Anchor::optional_anchor(
            properties,
            arguments,
            doc,
            line_number,
            "anchor",
            inherited_variables,
            component_name,
        )?,
        role: ftd::executor::ResponsiveType::optional_responsive_type(
            properties,
            arguments,
            doc,
            line_number,
            "role",
            inherited_variables,
            component_name,
        )?,
        region: ftd::executor::Region::optional_region(
            properties,
            arguments,
            doc,
            line_number,
            "region",
            inherited_variables,
            component_name,
        )?,
        cursor: ftd::executor::Cursor::optional_cursor(
            properties,
            arguments,
            doc,
            line_number,
            "cursor",
            inherited_variables,
            component_name,
        )?,
        text_transform: ftd::executor::TextTransform::optional_text_transform(
            properties,
            arguments,
            doc,
            line_number,
            "text-transform",
            inherited_variables,
            component_name,
        )?,
        border_style: ftd::executor::BorderStyle::optional_border_style(
            properties,
            arguments,
            doc,
            line_number,
            "border-style",
            inherited_variables,
            component_name,
        )?,
        border_style_horizontal: ftd::executor::BorderStyle::optional_border_style(
            properties,
            arguments,
            doc,
            line_number,
            "border-style-horizontal",
            inherited_variables,
            component_name,
        )?,
        border_style_vertical: ftd::executor::BorderStyle::optional_border_style(
            properties,
            arguments,
            doc,
            line_number,
            "border-style-vertical",
            inherited_variables,
            component_name,
        )?,
        border_style_top: ftd::executor::BorderStyle::optional_border_style(
            properties,
            arguments,
            doc,
            line_number,
            "border-style-top",
            inherited_variables,
            component_name,
        )?,
        border_style_bottom: ftd::executor::BorderStyle::optional_border_style(
            properties,
            arguments,
            doc,
            line_number,
            "border-style-bottom",
            inherited_variables,
            component_name,
        )?,
        border_style_left: ftd::executor::BorderStyle::optional_border_style(
            properties,
            arguments,
            doc,
            line_number,
            "border-style-left",
            inherited_variables,
            component_name,
        )?,
        border_style_right: ftd::executor::BorderStyle::optional_border_style(
            properties,
            arguments,
            doc,
            line_number,
            "border-style-right",
            inherited_variables,
            component_name,
        )?,
        classes: ftd::executor::value::string_list(
            "classes",
            component_name,
            properties,
            arguments,
            doc,
            line_number,
            inherited_variables,
        )?,
        padding: ftd::executor::Length::optional_length(
            properties,
            arguments,
            doc,
            line_number,
            "padding",
            inherited_variables,
            component_name,
        )?,
        padding_left: ftd::executor::Length::optional_length(
            properties,
            arguments,
            doc,
            line_number,
            "padding-left",
            inherited_variables,
            component_name,
        )?,
        padding_right: ftd::executor::Length::optional_length(
            properties,
            arguments,
            doc,
            line_number,
            "padding-right",
            inherited_variables,
            component_name,
        )?,
        padding_top: ftd::executor::Length::optional_length(
            properties,
            arguments,
            doc,
            line_number,
            "padding-top",
            inherited_variables,
            component_name,
        )?,
        padding_bottom: ftd::executor::Length::optional_length(
            properties,
            arguments,
            doc,
            line_number,
            "padding-bottom",
            inherited_variables,
            component_name,
        )?,
        padding_horizontal: ftd::executor::Length::optional_length(
            properties,
            arguments,
            doc,
            line_number,
            "padding-horizontal",
            inherited_variables,
            component_name,
        )?,
        padding_vertical: ftd::executor::Length::optional_length(
            properties,
            arguments,
            doc,
            line_number,
            "padding-vertical",
            inherited_variables,
            component_name,
        )?,
        margin: ftd::executor::Length::optional_length(
            properties,
            arguments,
            doc,
            line_number,
            "margin",
            inherited_variables,
            component_name,
        )?,
        margin_left: ftd::executor::Length::optional_length(
            properties,
            arguments,
            doc,
            line_number,
            "margin-left",
            inherited_variables,
            component_name,
        )?,
        margin_right: ftd::executor::Length::optional_length(
            properties,
            arguments,
            doc,
            line_number,
            "margin-right",
            inherited_variables,
            component_name,
        )?,
        margin_top: ftd::executor::Length::optional_length(
            properties,
            arguments,
            doc,
            line_number,
            "margin-top",
            inherited_variables,
            component_name,
        )?,
        margin_bottom: ftd::executor::Length::optional_length(
            properties,
            arguments,
            doc,
            line_number,
            "margin-bottom",
            inherited_variables,
            component_name,
        )?,
        margin_horizontal: ftd::executor::Length::optional_length(
            properties,
            arguments,
            doc,
            line_number,
            "margin-horizontal",
            inherited_variables,
            component_name,
        )?,
        margin_vertical: ftd::executor::Length::optional_length(
            properties,
            arguments,
            doc,
            line_number,
            "margin-vertical",
            inherited_variables,
            component_name,
        )?,
        border_width: ftd::executor::Length::optional_length(
            properties,
            arguments,
            doc,
            line_number,
            "border-width",
            inherited_variables,
            component_name,
        )?,
        border_radius: ftd::executor::Length::optional_length(
            properties,
            arguments,
            doc,
            line_number,
            "border-radius",
            inherited_variables,
            component_name,
        )?,
        border_color: ftd::executor::Color::optional_color(
            properties,
            arguments,
            doc,
            line_number,
            "border-color",
            inherited_variables,
            component_name,
        )?,
        border_bottom_width: ftd::executor::Length::optional_length(
            properties,
            arguments,
            doc,
            line_number,
            "border-bottom-width",
            inherited_variables,
            component_name,
        )?,
        border_bottom_color: ftd::executor::Color::optional_color(
            properties,
            arguments,
            doc,
            line_number,
            "border-bottom-color",
            inherited_variables,
            component_name,
        )?,
        border_top_width: ftd::executor::Length::optional_length(
            properties,
            arguments,
            doc,
            line_number,
            "border-top-width",
            inherited_variables,
            component_name,
        )?,
        border_top_color: ftd::executor::Color::optional_color(
            properties,
            arguments,
            doc,
            line_number,
            "border-top-color",
            inherited_variables,
            component_name,
        )?,
        border_left_width: ftd::executor::Length::optional_length(
            properties,
            arguments,
            doc,
            line_number,
            "border-left-width",
            inherited_variables,
            component_name,
        )?,
        border_left_color: ftd::executor::Color::optional_color(
            properties,
            arguments,
            doc,
            line_number,
            "border-left-color",
            inherited_variables,
            component_name,
        )?,
        border_right_width: ftd::executor::Length::optional_length(
            properties,
            arguments,
            doc,
            line_number,
            "border-right-width",
            inherited_variables,
            component_name,
        )?,
        border_right_color: ftd::executor::Color::optional_color(
            properties,
            arguments,
            doc,
            line_number,
            "border-right-color",
            inherited_variables,
            component_name,
        )?,
        border_top_left_radius: ftd::executor::Length::optional_length(
            properties,
            arguments,
            doc,
            line_number,
            "border-top-left-radius",
            inherited_variables,
            component_name,
        )?,
        border_top_right_radius: ftd::executor::Length::optional_length(
            properties,
            arguments,
            doc,
            line_number,
            "border-top-right-radius",
            inherited_variables,
            component_name,
        )?,
        border_bottom_left_radius: ftd::executor::Length::optional_length(
            properties,
            arguments,
            doc,
            line_number,
            "border-bottom-left-radius",
            inherited_variables,
            component_name,
        )?,
        border_bottom_right_radius: ftd::executor::Length::optional_length(
            properties,
            arguments,
            doc,
            line_number,
            "border-bottom-right-radius",
            inherited_variables,
            component_name,
        )?,
        width: ftd::executor::Resizing::optional_resizing(
            properties,
            arguments,
            doc,
            line_number,
            "width",
            inherited_variables,
            component_name,
        )?,
        height: ftd::executor::Resizing::optional_resizing(
            properties,
            arguments,
            doc,
            line_number,
            "height",
            inherited_variables,
            component_name,
        )?,
        min_width: ftd::executor::Resizing::optional_resizing(
            properties,
            arguments,
            doc,
            line_number,
            "min-width",
            inherited_variables,
            component_name,
        )?,
        max_width: ftd::executor::Resizing::optional_resizing(
            properties,
            arguments,
            doc,
            line_number,
            "max-width",
            inherited_variables,
            component_name,
        )?,
        min_height: ftd::executor::Resizing::optional_resizing(
            properties,
            arguments,
            doc,
            line_number,
            "min-height",
            inherited_variables,
            component_name,
        )?,
        max_height: ftd::executor::Resizing::optional_resizing(
            properties,
            arguments,
            doc,
            line_number,
            "max-height",
            inherited_variables,
            component_name,
        )?,
        link: ftd::executor::value::optional_string(
            "link",
            component_name,
            properties,
            arguments,
            doc,
            line_number,
        )?,
        open_in_new_tab: ftd::executor::value::optional_bool(
            "open-in-new-tab",
            component_name,
            properties,
            arguments,
            doc,
            line_number,
            inherited_variables,
        )?,
        condition: condition.to_owned(),
        data_id: ftd::executor::utils::get_string_container(local_container),
        line_number,
        background: ftd::executor::Background::optional_background(
            properties,
            arguments,
            doc,
            line_number,
            "background",
            inherited_variables,
            component_name,
        )?,
        color: ftd::executor::Color::optional_color(
            properties,
            arguments,
            doc,
            line_number,
            "color",
            inherited_variables,
            component_name,
        )?,
        align_self: ftd::executor::AlignSelf::optional_align_self(
            properties,
            arguments,
            doc,
            line_number,
            "align-self",
            inherited_variables,
            component_name,
        )?,
        overflow: ftd::executor::Overflow::optional_overflow(
            properties,
            arguments,
            doc,
            line_number,
            "overflow",
            inherited_variables,
            component_name,
        )?,
        overflow_x: ftd::executor::Overflow::optional_overflow(
            properties,
            arguments,
            doc,
            line_number,
            "overflow-x",
            inherited_variables,
            component_name,
        )?,
        overflow_y: ftd::executor::Overflow::optional_overflow(
            properties,
            arguments,
            doc,
            line_number,
            "overflow-y",
            inherited_variables,
            component_name,
        )?,
        opacity: ftd::executor::value::optional_f64(
            "opacity",
            component_name,
            properties,
            arguments,
            doc,
            line_number,
        )?,
        resize: ftd::executor::Resize::optional_resize(
            properties,
            arguments,
            doc,
            line_number,
            "resize",
            inherited_variables,
            component_name,
        )?,
        white_space: ftd::executor::WhiteSpace::optional_whitespace(
            properties,
            arguments,
            doc,
            line_number,
            "white-space",
            inherited_variables,
            component_name,
        )?,
        shadow: ftd::executor::Shadow::optional_shadow(
            properties,
            arguments,
            doc,
            line_number,
            "shadow",
            inherited_variables,
            component_name,
        )?,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn container_from_properties(
    properties: &[fastn_type::Property],
    arguments: &[ftd::interpreter::Argument],
    doc: &ftd::executor::TDoc,
    line_number: usize,
    children: Vec<Element>,
    inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
    component_name: &str,
    device: Option<ftd::executor::Device>,
) -> ftd::executor::Result<Container> {
    Ok(Container {
        wrap: ftd::executor::value::optional_bool(
            "wrap",
            component_name,
            properties,
            arguments,
            doc,
            line_number,
            inherited_variables,
        )?,
        align_content: ftd::executor::Alignment::optional_alignment(
            properties,
            arguments,
            doc,
            line_number,
            "align-content",
            inherited_variables,
            component_name,
        )?,
        spacing: ftd::executor::Spacing::optional_spacing_mode(
            properties,
            arguments,
            doc,
            line_number,
            "spacing",
            inherited_variables,
            component_name,
        )?,
        children,
        device,
    })
}

#[derive(serde::Deserialize, Debug, Default, PartialEq, Clone, serde::Serialize)]
pub struct TextInput {
    pub placeholder: ftd::executor::Value<Option<String>>,
    pub value: ftd::executor::Value<Option<String>>,
    pub multiline: ftd::executor::Value<bool>,
    pub default_value: ftd::executor::Value<Option<String>>,
    pub type_: ftd::executor::Value<Option<ftd::executor::TextInputType>>,
    pub enabled: ftd::executor::Value<Option<bool>>,
    pub common: Common,
}

impl TextInput {
    pub fn enabled_pattern() -> (String, bool) {
        (
            format!(
                indoc::indoc! {"
                    if ({{0}}) {{
                        \"{remove_key}\"
                    }} else {{
                        \"\"
                    }}
                "},
                remove_key = ftd::interpreter::FTD_REMOVE_KEY,
            ),
            true,
        )
    }
}

#[allow(clippy::too_many_arguments)]
pub fn text_input_from_properties(
    properties: &[fastn_type::Property],
    events: &[fastn_type::Event],
    arguments: &[ftd::interpreter::Argument],
    condition: &Option<ftd::interpreter::Expression>,
    doc: &mut ftd::executor::TDoc,
    local_container: &[usize],
    line_number: usize,
    inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
    device: Option<ftd::executor::Device>,
) -> ftd::executor::Result<TextInput> {
    // TODO: `youtube` should not be conditional
    let placeholder = ftd::executor::value::optional_string(
        "placeholder",
        "ftd#text-input",
        properties,
        arguments,
        doc,
        line_number,
    )?;

    let value = ftd::executor::value::optional_string(
        "value",
        "ftd#text-input",
        properties,
        arguments,
        doc,
        line_number,
    )?;

    let multiline = ftd::executor::value::bool_with_default(
        "multiline",
        "ftd#text-input",
        properties,
        arguments,
        false,
        doc,
        line_number,
    )?;

    let enabled = ftd::executor::value::optional_bool(
        "enabled",
        "ftd#text-input",
        properties,
        arguments,
        doc,
        line_number,
        inherited_variables,
    )?;

    let default_value = ftd::executor::value::optional_string(
        "default-value",
        "ftd#text-input",
        properties,
        arguments,
        doc,
        line_number,
    )?;

    let type_ = ftd::executor::TextInputType::optional_text_input_type(
        properties,
        arguments,
        doc,
        line_number,
        "type",
        inherited_variables,
        "ftd#text-input",
    )?;

    let common = common_from_properties(
        properties,
        events,
        arguments,
        condition,
        doc,
        local_container,
        line_number,
        inherited_variables,
        "ftd#text-input",
        device,
    )?;

    Ok(TextInput {
        placeholder,
        value,
        multiline,
        default_value,
        common,
        type_,
        enabled,
    })
}

#[derive(serde::Deserialize, Debug, Default, PartialEq, Clone, serde::Serialize)]
pub struct CheckBox {
    pub checked: ftd::executor::Value<Option<bool>>,
    pub enabled: ftd::executor::Value<Option<bool>>,
    pub common: Common,
}

impl CheckBox {
    pub fn checked_pattern() -> (String, bool) {
        (
            format!(
                indoc::indoc! {"
                    if ({{0}}) {{
                        \"\"
                    }} else {{
                        \"{remove_key}\"
                    }}
                "},
                remove_key = ftd::interpreter::FTD_REMOVE_KEY,
            ),
            true,
        )
    }

    pub fn enabled_pattern() -> (String, bool) {
        (
            format!(
                indoc::indoc! {"
                    if ({{0}}) {{
                        \"{remove_key}\"
                    }} else {{
                        \"\"
                    }}
                "},
                remove_key = ftd::interpreter::FTD_REMOVE_KEY,
            ),
            true,
        )
    }
}

#[allow(clippy::too_many_arguments)]
pub fn checkbox_from_properties(
    properties: &[fastn_type::Property],
    events: &[fastn_type::Event],
    arguments: &[ftd::interpreter::Argument],
    condition: &Option<ftd::interpreter::Expression>,
    doc: &mut ftd::executor::TDoc,
    local_container: &[usize],
    line_number: usize,
    inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
    device: Option<ftd::executor::Device>,
) -> ftd::executor::Result<CheckBox> {
    let checked = ftd::executor::value::optional_bool(
        "checked",
        "ftd#checkbox",
        properties,
        arguments,
        doc,
        line_number,
        inherited_variables,
    )?;

    let enabled = ftd::executor::value::optional_bool(
        "enabled",
        "ftd#checkbox",
        properties,
        arguments,
        doc,
        line_number,
        inherited_variables,
    )?;

    let common = common_from_properties(
        properties,
        events,
        arguments,
        condition,
        doc,
        local_container,
        line_number,
        inherited_variables,
        "ftd#checkbox",
        device,
    )?;

    Ok(CheckBox {
        checked,
        enabled,
        common,
    })
}
