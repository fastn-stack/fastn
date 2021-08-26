pub fn common_from_properties(
    properties: &std::collections::BTreeMap<String, crate::Value>,
    doc: &crate::p2::TDoc,
    condition: &Option<ftd::p2::Boolean>,
) -> crate::p1::Result<ftd_rt::Common> {
    Ok(ftd_rt::Common {
        condition: match condition {
            Some(c) if !c.is_constant() => Some(c.to_condition()),
            _ => None,
        },
        region: ftd_rt::Region::from(crate::p2::utils::string_optional("region", properties)?)?,
        padding: crate::p2::utils::int_optional("padding", properties)?,
        padding_left: crate::p2::utils::int_optional("padding-left", properties)?,
        padding_right: crate::p2::utils::int_optional("padding-right", properties)?,
        padding_top: crate::p2::utils::int_optional("padding-top", properties)?,
        padding_bottom: crate::p2::utils::int_optional("padding-bottom", properties)?,
        border_top_radius: crate::p2::utils::int_optional("border-top-radius", properties)?,
        border_bottom_radius: crate::p2::utils::int_optional("border-bottom-radius", properties)?,
        width: ftd_rt::Length::from(crate::p2::utils::string_optional("width", properties)?)?,
        height: ftd_rt::Length::from(crate::p2::utils::string_optional("height", properties)?)?,
        explain: crate::p2::utils::bool_with_default("explain", false, properties)?,
        color: color_from(crate::p2::utils::string_optional("color", properties)?)?,
        background_color: color_from(crate::p2::utils::string_optional(
            "background-color",
            properties,
        )?)?,
        border_color: color_from(crate::p2::utils::string_optional(
            "border-color",
            properties,
        )?)?,
        border_width: crate::p2::utils::int_with_default("border-width", 0, properties)?,
        border_radius: crate::p2::utils::int_with_default("border-radius", 0, properties)?,
        id: crate::p2::utils::string_optional("id", properties)?
            .map(|v| format!("{}#{}", doc.name, v)),
        overflow_x: ftd_rt::Overflow::from(crate::p2::utils::string_optional(
            "overflow-x",
            properties,
        )?)?,
        overflow_y: ftd_rt::Overflow::from(crate::p2::utils::string_optional(
            "overflow-y",
            properties,
        )?)?,
        border_top: crate::p2::utils::int_optional("border-top", properties)?,
        border_left: crate::p2::utils::int_optional("border-left", properties)?,
        border_right: crate::p2::utils::int_optional("border-right", properties)?,
        border_bottom: crate::p2::utils::int_optional("border-bottom", properties)?,
        margin_top: crate::p2::utils::int_optional("margin-top", properties)?,
        margin_bottom: crate::p2::utils::int_optional("margin-bottom", properties)?,
        margin_left: crate::p2::utils::int_optional("margin-left", properties)?,
        margin_right: crate::p2::utils::int_optional("margin-right", properties)?,
        link: crate::p2::utils::string_optional("link", properties)?,
        open_in_new_tab: crate::p2::utils::bool_with_default("open-in-new-tab", false, properties)?,
    })
}

fn common_arguments() -> Vec<(String, crate::p2::Kind)> {
    vec![
        (
            "padding".to_string(),
            crate::p2::Kind::Integer.into_optional(),
        ),
        (
            "padding-left".to_string(),
            crate::p2::Kind::Integer.into_optional(),
        ),
        (
            "padding-right".to_string(),
            crate::p2::Kind::Integer.into_optional(),
        ),
        (
            "padding-top".to_string(),
            crate::p2::Kind::Integer.into_optional(),
        ),
        (
            "padding-bottom".to_string(),
            crate::p2::Kind::Integer.into_optional(),
        ),
        (
            "border-top-radius".to_string(),
            crate::p2::Kind::Integer.into_optional(),
        ),
        (
            "border-bottom-radius".to_string(),
            crate::p2::Kind::Integer.into_optional(),
        ),
        (
            "width".to_string(),
            crate::p2::Kind::string().into_optional(),
        ),
        (
            "height".to_string(),
            crate::p2::Kind::string().into_optional(),
        ),
        (
            "explain".to_string(),
            crate::p2::Kind::Boolean.into_optional(),
        ),
        (
            "region".to_string(),
            crate::p2::Kind::string().into_optional(),
        ),
        (
            "color".to_string(),
            crate::p2::Kind::string().into_optional(),
        ),
        (
            "background-color".to_string(),
            crate::p2::Kind::string().into_optional(),
        ),
        (
            "border-color".to_string(),
            crate::p2::Kind::string().into_optional(),
        ),
        (
            "border-width".to_string(),
            crate::p2::Kind::Integer.into_optional(),
        ),
        (
            "border-radius".to_string(),
            crate::p2::Kind::Integer.into_optional(),
        ),
        ("id".to_string(), crate::p2::Kind::string().into_optional()),
        (
            "overflow-x".to_string(),
            crate::p2::Kind::string().into_optional(),
        ),
        (
            "overflow-y".to_string(),
            crate::p2::Kind::string().into_optional(),
        ),
        (
            "border-top".to_string(),
            crate::p2::Kind::Integer.into_optional(),
        ),
        (
            "border-bottom".to_string(),
            crate::p2::Kind::Integer.into_optional(),
        ),
        (
            "border-left".to_string(),
            crate::p2::Kind::Integer.into_optional(),
        ),
        (
            "border-right".to_string(),
            crate::p2::Kind::Integer.into_optional(),
        ),
        (
            "margin-top".to_string(),
            crate::p2::Kind::Integer.into_optional(),
        ),
        (
            "margin-bottom".to_string(),
            crate::p2::Kind::Integer.into_optional(),
        ),
        (
            "margin-left".to_string(),
            crate::p2::Kind::Integer.into_optional(),
        ),
        (
            "margin-right".to_string(),
            crate::p2::Kind::Integer.into_optional(),
        ),
        (
            "link".to_string(),
            crate::p2::Kind::string().into_optional(),
        ),
        (
            "open-in-new-tab".to_string(),
            crate::p2::Kind::Boolean.into_optional(),
        ),
    ]
}

pub fn null() -> ftd::Component {
    ftd::Component {
        kernel: true,
        full_name: "ftd#null".to_string(),
        root: "ftd.kernel".to_string(),
        ..Default::default()
    }
}

pub fn container_from_properties(
    properties: &std::collections::BTreeMap<String, crate::Value>,
    _doc: &crate::p2::TDoc,
) -> crate::p1::Result<ftd_rt::Container> {
    Ok(ftd_rt::Container {
        children: Default::default(),
        open: crate::p2::utils::bool_optional("open", properties)?,
        spacing: crate::p2::utils::int_optional("spacing", properties)?,
        align: ftd_rt::Align::from(crate::p2::utils::string_optional("align", properties)?)?,
        wrap: crate::p2::utils::bool_with_default("wrap", false, properties)?,
    })
}

fn container_arguments() -> Vec<(String, crate::p2::Kind)> {
    vec![
        ("open".to_string(), crate::p2::Kind::Boolean.into_optional()),
        (
            "spacing".to_string(),
            crate::p2::Kind::Integer.into_optional(),
        ),
        (
            "align".to_string(),
            crate::p2::Kind::string().into_optional(),
        ),
        ("wrap".to_string(), crate::p2::Kind::Boolean.into_optional()),
    ]
}

pub fn image_function() -> crate::Component {
    crate::Component {
        kernel: true,
        full_name: "ftd#image".to_string(),
        root: "ftd.kernel".to_string(),
        arguments: [
            vec![
                ("src".to_string(), crate::p2::Kind::string()),
                (
                    "description".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                (
                    "align".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
            ],
            common_arguments(),
        ]
        .concat()
        .into_iter()
        .collect(),
        properties: Default::default(),
        instructions: Default::default(),
        invocations: Default::default(),
    }
}

pub fn image_from_properties(
    properties: &std::collections::BTreeMap<String, crate::Value>,
    doc: &crate::p2::TDoc,
    condition: &Option<ftd::p2::Boolean>,
) -> crate::p1::Result<ftd_rt::Image> {
    Ok(ftd_rt::Image {
        src: crate::p2::utils::string("src", properties)?,
        description: crate::p2::utils::string_optional("description", properties)?
            .unwrap_or_else(|| "".to_string()),
        common: common_from_properties(properties, doc, condition)?,
        align: ftd_rt::Align::from(crate::p2::utils::string_optional("align", properties)?)?,
    })
}

pub fn row_function() -> crate::Component {
    crate::Component {
        kernel: true,
        full_name: "ftd#row".to_string(),
        root: "ftd.kernel".to_string(),
        arguments: [container_arguments(), common_arguments()]
            .concat()
            .into_iter()
            .collect(),
        properties: Default::default(),
        instructions: Default::default(),
        invocations: Default::default(),
    }
}

pub fn row_from_properties(
    properties: &std::collections::BTreeMap<String, crate::Value>,
    doc: &crate::p2::TDoc,
    condition: &Option<ftd::p2::Boolean>,
) -> crate::p1::Result<ftd_rt::Row> {
    Ok(ftd_rt::Row {
        common: common_from_properties(properties, doc, condition)?,
        container: container_from_properties(properties, doc)?,
    })
}

pub fn column_function() -> crate::Component {
    crate::Component {
        kernel: true,
        full_name: "ftd#column".to_string(),
        root: "ftd.kernel".to_string(),
        arguments: [container_arguments(), common_arguments()]
            .concat()
            .into_iter()
            .collect(),
        properties: Default::default(),
        instructions: Default::default(),
        invocations: Default::default(),
    }
}
pub fn column_from_properties(
    properties: &std::collections::BTreeMap<String, crate::Value>,
    doc: &crate::p2::TDoc,
    condition: &Option<ftd::p2::Boolean>,
) -> crate::p1::Result<ftd_rt::Column> {
    Ok(ftd_rt::Column {
        common: common_from_properties(properties, doc, condition)?,
        container: container_from_properties(properties, doc)?,
    })
}

pub fn external_font_from_properties(
    properties: &std::collections::BTreeMap<String, crate::Value>,
    _doc: &crate::p2::TDoc,
) -> crate::p1::Result<Option<ftd_rt::ExternalFont>> {
    let font_option = crate::p2::utils::string_optional("font", properties)?;
    let font_url_option = crate::p2::utils::string_optional("font-url", properties)?;

    match (font_option, font_url_option) {
        (Some(font), Some(font_url)) => {
            let name_opt = font.split(',').next();
            let name = match name_opt {
                Some(f) => f.to_string(),
                None => return crate::e("Something went wrong while parsing font vector"),
            };

            Ok(Some(ftd_rt::ExternalFont {
                url: font_url,
                display: ftd_rt::FontDisplay::from(crate::p2::utils::string_optional(
                    "font-display",
                    properties,
                )?)?,
                name,
            }))
        }
        _ => Ok(None),
    }
}

#[allow(unused_variables)]
pub fn text_render(
    tf: &ftd_rt::TextFormat,
    text: String,
    source: crate::TextSource,
    theme: String,
) -> crate::p1::Result<ftd_rt::Rendered> {
    Ok(match (source, tf) {
        (ftd::TextSource::Body, ftd_rt::TextFormat::Markdown) => ftd::markdown(text.as_str()),
        (_, ftd_rt::TextFormat::Markdown) => ftd::markdown_line(text.as_str()),
        (_, ftd_rt::TextFormat::Latex) => ftd::latex(text.as_str())?,
        (_, ftd_rt::TextFormat::Code { lang }) => {
            ftd::code_with_theme(text.as_str(), lang.as_str(), theme.as_str())?
        }
    })
}

pub fn iframe_function() -> crate::Component {
    crate::Component {
        kernel: true,
        root: "ftd.kernel".to_string(),
        full_name: "ftd#iframe".to_string(),
        arguments: [
            vec![
                ("src".to_string(), crate::p2::Kind::string().into_optional()),
                (
                    "youtube".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
            ],
            common_arguments(),
        ]
        .concat()
        .into_iter()
        .collect(),
        properties: Default::default(),
        instructions: Default::default(),
        invocations: Default::default(),
    }
}

pub fn iframe_from_properties(
    properties: &std::collections::BTreeMap<String, crate::Value>,
    doc: &crate::p2::TDoc,
    condition: &Option<ftd::p2::Boolean>,
) -> crate::p1::Result<ftd_rt::IFrame> {
    let src = match (
        crate::p2::utils::string_optional("src", properties)?,
        crate::p2::utils::string_optional("youtube", properties)?
            .and_then(|id| crate::youtube_id::from_raw(id.as_str())),
    ) {
        (Some(src), None) => src,
        (None, Some(id)) => id,
        (Some(_), Some(_)) => return crate::e("both src and youtube id provided"),
        (None, None) => return crate::e("src or youtube id is required"),
    };

    Ok(ftd_rt::IFrame {
        src,
        common: common_from_properties(properties, doc, condition)?,
    })
}

pub fn text_from_properties(
    properties: &std::collections::BTreeMap<String, crate::Value>,
    doc: &crate::p2::TDoc,
    condition: &Option<ftd::p2::Boolean>,
) -> crate::p1::Result<ftd_rt::Text> {
    let format = ftd_rt::TextFormat::from(
        crate::p2::utils::string_optional("format", properties)?,
        crate::p2::utils::string_optional("lang", properties)?,
    )?;
    let (text, source) = crate::p2::utils::string_and_source("text", properties)?;
    let font_str = crate::p2::utils::string_optional("font", properties)?;

    let font: Vec<ftd_rt::NamedFont> = match font_str {
        Some(f) => f
            .split(',')
            .flat_map(|x| ftd_rt::NamedFont::from(Some(x.to_string())))
            .collect(),
        None => vec![],
    };
    Ok(ftd_rt::Text {
        line: source != ftd::TextSource::Body,
        text: text_render(
            &format,
            text,
            source,
            crate::p2::utils::string_with_default(
                "theme",
                crate::render::DEFAULT_THEME,
                properties,
            )?,
        )?,
        common: common_from_properties(properties, doc, condition)?,
        align: ftd_rt::TextAlign::from(crate::p2::utils::string_optional("align", properties)?)?,
        style: ftd_rt::Style::from(crate::p2::utils::string_optional("style", properties)?)?,
        format,
        size: crate::p2::utils::int_optional("size", properties)?,
        external_font: external_font_from_properties(properties, doc)?,
        font,
        line_height: crate::p2::utils::string_optional("line-height", properties)?,
    })
}

pub fn integer_from_properties(
    properties: &std::collections::BTreeMap<String, crate::Value>,
    doc: &crate::p2::TDoc,
    condition: &Option<ftd::p2::Boolean>,
) -> crate::p1::Result<ftd_rt::Text> {
    let font_str = crate::p2::utils::string_optional("font", properties)?;
    let num = format_num::NumberFormat::new();
    let text = match crate::p2::utils::string_optional("format", properties)? {
        Some(f) => num.format(
            f.as_str(),
            crate::p2::utils::int("value", properties)? as f64,
        ),
        None => crate::p2::utils::int("value", properties)?.to_string(),
    };

    let font: Vec<ftd_rt::NamedFont> = match font_str {
        Some(f) => f
            .split(',')
            .flat_map(|x| ftd_rt::NamedFont::from(Some(x.to_string())))
            .collect(),
        None => vec![],
    };

    Ok(ftd_rt::Text {
        text: crate::markdown_line(text.as_str()),
        line: false,
        common: common_from_properties(properties, doc, condition)?,
        align: ftd_rt::TextAlign::from(crate::p2::utils::string_optional("align", properties)?)?,
        style: ftd_rt::Style::from(crate::p2::utils::string_optional("style", properties)?)?,
        format: Default::default(),
        size: crate::p2::utils::int_optional("size", properties)?,
        external_font: external_font_from_properties(properties, doc)?,
        font,
        line_height: crate::p2::utils::string_optional("line-height", properties)?,
    })
}

pub fn decimal_from_properties(
    properties: &std::collections::BTreeMap<String, crate::Value>,
    doc: &crate::p2::TDoc,
    condition: &Option<ftd::p2::Boolean>,
) -> crate::p1::Result<ftd_rt::Text> {
    let font_str = crate::p2::utils::string_optional("font", properties)?;
    let num = format_num::NumberFormat::new();
    let text = match crate::p2::utils::string_optional("format", properties)? {
        Some(f) => num.format(f.as_str(), crate::p2::utils::decimal("value", properties)?),
        None => crate::p2::utils::decimal("value", properties)?.to_string(),
    };

    let font: Vec<ftd_rt::NamedFont> = match font_str {
        Some(f) => f
            .split(',')
            .flat_map(|x| ftd_rt::NamedFont::from(Some(x.to_string())))
            .collect(),
        None => vec![],
    };
    Ok(ftd_rt::Text {
        text: crate::markdown_line(text.as_str()),
        line: false,
        common: common_from_properties(properties, doc, condition)?,
        align: ftd_rt::TextAlign::from(crate::p2::utils::string_optional("align", properties)?)?,
        style: ftd_rt::Style::from(crate::p2::utils::string_optional("style", properties)?)?,
        format: Default::default(),
        size: crate::p2::utils::int_optional("size", properties)?,
        external_font: external_font_from_properties(properties, doc)?,
        font,
        line_height: crate::p2::utils::string_optional("line-height", properties)?,
    })
}

pub fn color_from(l: Option<String>) -> ftd::p1::Result<Option<ftd_rt::Color>> {
    use std::str::FromStr;

    let v = match l {
        Some(v) => v,
        None => return Ok(None),
    };

    match css_color_parser::Color::from_str(v.as_str()) {
        Ok(v) => Ok(Some(ftd_rt::Color {
            r: v.r,
            g: v.g,
            b: v.b,
            alpha: v.a,
        })),
        Err(e) => return crate::e(format!("{} is not a valid color: {:?}", v, e)),
    }
}

pub fn boolean_from_properties(
    properties: &std::collections::BTreeMap<String, crate::Value>,
    doc: &crate::p2::TDoc,
    condition: &Option<ftd::p2::Boolean>,
) -> crate::p1::Result<ftd_rt::Text> {
    let font_str = crate::p2::utils::string_optional("font", properties)?;
    let value = crate::p2::utils::bool("value", properties)?;
    let text = if value {
        crate::p2::utils::string_with_default("true", "true", properties)?
    } else {
        crate::p2::utils::string_with_default("false", "false", properties)?
    };
    let font: Vec<ftd_rt::NamedFont> = match font_str {
        Some(f) => f
            .split(',')
            .flat_map(|x| ftd_rt::NamedFont::from(Some(x.to_string())))
            .collect(),
        None => vec![],
    };

    Ok(ftd_rt::Text {
        text: crate::markdown_line(text.as_str()),
        line: false,
        common: common_from_properties(properties, doc, condition)?,
        align: ftd_rt::TextAlign::from(crate::p2::utils::string_optional("align", properties)?)?,
        style: ftd_rt::Style::from(crate::p2::utils::string_optional("style", properties)?)?,
        format: Default::default(),
        size: crate::p2::utils::int_optional("size", properties)?,
        external_font: external_font_from_properties(properties, doc)?,
        font,
        line_height: crate::p2::utils::string_optional("line-height", properties)?,
    })
}

pub fn text_function() -> crate::Component {
    crate::Component {
        kernel: true,
        root: "ftd.kernel".to_string(),
        full_name: "ftd#text".to_string(),
        arguments: [
            vec![
                (
                    "text".to_string(),
                    crate::p2::Kind::String {
                        caption: true,
                        body: true,
                    },
                ),
                (
                    "align".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                (
                    "style".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                (
                    "format".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                (
                    "lang".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                (
                    "theme".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                ("size".to_string(), crate::p2::Kind::Integer.into_optional()),
                (
                    "font-url".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                (
                    "font".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                (
                    "font-display".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                (
                    "line-height".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
            ],
            common_arguments(),
        ]
        .concat()
        .into_iter()
        .collect(),
        properties: Default::default(),
        instructions: Default::default(),
        invocations: Default::default(),
    }
}

pub fn integer_function() -> crate::Component {
    crate::Component {
        kernel: true,
        root: "ftd.kernel".to_string(),
        full_name: "ftd#integer".to_string(),
        arguments: [
            vec![
                ("value".to_string(), crate::p2::Kind::Integer),
                (
                    "align".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                (
                    "style".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                (
                    "format".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                ("size".to_string(), crate::p2::Kind::Integer.into_optional()),
                (
                    "font-url".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                (
                    "font".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                (
                    "font-display".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                (
                    "line-height".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
            ],
            common_arguments(),
        ]
        .concat()
        .into_iter()
        .collect(),
        properties: Default::default(),
        instructions: Default::default(),
        invocations: Default::default(),
    }
}

pub fn decimal_function() -> crate::Component {
    crate::Component {
        kernel: true,
        root: "ftd.kernel".to_string(),
        full_name: "ftd#decimal".to_string(),
        arguments: [
            vec![
                ("value".to_string(), crate::p2::Kind::Decimal),
                (
                    "align".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                (
                    "style".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                (
                    "format".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                ("size".to_string(), crate::p2::Kind::Integer.into_optional()),
                (
                    "font-url".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                (
                    "font".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                (
                    "font-display".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                (
                    "line-height".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
            ],
            common_arguments(),
        ]
        .concat()
        .into_iter()
        .collect(),
        properties: Default::default(),
        instructions: Default::default(),
        invocations: Default::default(),
    }
}

pub fn boolean_function() -> crate::Component {
    crate::Component {
        kernel: true,
        root: "ftd.kernel".to_string(),
        full_name: "ftd#boolean".to_string(),
        arguments: [
            vec![
                ("value".to_string(), crate::p2::Kind::Boolean),
                (
                    "align".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                (
                    "style".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                (
                    "format".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                ("size".to_string(), crate::p2::Kind::Integer.into_optional()),
                (
                    "font-url".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                (
                    "font".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                (
                    "font-display".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                (
                    "line-height".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                (
                    "true".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                (
                    "false".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
            ],
            common_arguments(),
        ]
        .concat()
        .into_iter()
        .collect(),
        properties: Default::default(),
        instructions: Default::default(),
        invocations: Default::default(),
    }
}

pub fn input_function() -> crate::Component {
    crate::Component {
        kernel: true,
        root: "ftd.kernel".to_string(),
        full_name: "ftd#input".to_string(),
        arguments: [
            vec![(
                "placeholder".to_string(),
                crate::p2::Kind::string().into_optional(),
            )],
            common_arguments(),
        ]
        .concat()
        .into_iter()
        .collect(),
        properties: Default::default(),
        instructions: Default::default(),
        invocations: Default::default(),
    }
}

pub fn input_from_properties(
    properties: &std::collections::BTreeMap<String, crate::Value>,
    doc: &crate::p2::TDoc,
    condition: &Option<ftd::p2::Boolean>,
) -> crate::p1::Result<ftd_rt::Input> {
    Ok(ftd_rt::Input {
        common: common_from_properties(properties, doc, condition)?,
        placeholder: crate::p2::utils::string_optional("placeholder", properties)?,
    })
}
