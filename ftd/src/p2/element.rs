#[allow(clippy::too_many_arguments)]
pub fn common_from_properties(
    properties: &std::collections::BTreeMap<String, crate::Value>,
    doc: &crate::p2::TDoc,
    condition: &Option<ftd::p2::Boolean>,
    is_child: bool,
    events: &[ftd::p2::Event],
    all_locals: &mut ftd_rt::Map,
    root_name: Option<&str>,
    reference: Option<String>,
) -> crate::p1::Result<ftd_rt::Common> {
    let submit = crate::p2::utils::string_optional("submit", properties, doc.name, 0)?;
    let link = crate::p2::utils::string_optional("link", properties, doc.name, 0)?;
    if let (Some(_), Some(_)) = (&submit, &link) {
        return ftd::e2(
            "Cannot have both submit and link together",
            "common_from_properties",
            doc.name.to_string(),
            0,
        );
    }
    let gradient_color_str =
        crate::p2::utils::string_optional("gradient-colors", properties, doc.name, 0)?;

    let gradient_colors: Vec<ftd_rt::Color> = match gradient_color_str {
        Some(f) => f
            .split(',')
            .flat_map(|x| color_from(Some(x.to_string()), doc.name).ok()?)
            .collect(),
        None => vec![],
    };
    let anchor = ftd_rt::Anchor::from(
        crate::p2::utils::string_optional("anchor", properties, doc.name, 0)?,
        doc.name,
    )?;

    let inner_default = match anchor {
        Some(ref p) => match p {
            ftd_rt::Anchor::Parent => false,
            ftd_rt::Anchor::Window => true,
        },
        None => false,
    };

    let arguments = {
        //remove properties
        let mut arguments_without_properties: std::collections::BTreeMap<String, crate::Value> =
            Default::default();
        for (k, v) in properties {
            if let Some(k) = k.strip_prefix('$') {
                arguments_without_properties.insert(k.to_string(), v.to_owned());
            }
        }
        arguments_without_properties
    };

    let (cond, is_visible) = match condition {
        Some(c) => {
            let mut is_visible = true;
            if !c.eval(0, &arguments, doc)? {
                is_visible = false;
            }
            if !c.is_arg_constant() {
                (
                    Some(c.to_condition(0, all_locals, &arguments, doc.name)?),
                    is_visible,
                )
            } else {
                (None, is_visible)
            }
        }
        _ => (None, true),
    };

    Ok(ftd_rt::Common {
        conditional_attribute: Default::default(),
        locals: Default::default(),
        condition: cond,
        is_not_visible: !is_visible,
        events: ftd::p2::Event::get_events(0, events, all_locals, properties, doc, root_name)?,
        reference,
        region: ftd_rt::Region::from(
            crate::p2::utils::string_optional("region", properties, doc.name, 0)?,
            doc.name,
        )?,
        padding: crate::p2::utils::int_optional("padding", properties, doc.name, 0)?,
        padding_vertical: crate::p2::utils::int_optional(
            "padding-vertical",
            properties,
            doc.name,
            0,
        )?,
        padding_horizontal: crate::p2::utils::int_optional(
            "padding-horizontal",
            properties,
            doc.name,
            0,
        )?,
        padding_left: crate::p2::utils::int_optional("padding-left", properties, doc.name, 0)?,
        padding_right: crate::p2::utils::int_optional("padding-right", properties, doc.name, 0)?,
        padding_top: crate::p2::utils::int_optional("padding-top", properties, doc.name, 0)?,
        padding_bottom: crate::p2::utils::int_optional("padding-bottom", properties, doc.name, 0)?,
        border_top_radius: crate::p2::utils::int_optional(
            "border-top-radius",
            properties,
            doc.name,
            0,
        )?,
        border_bottom_radius: crate::p2::utils::int_optional(
            "border-bottom-radius",
            properties,
            doc.name,
            0,
        )?,
        border_left_radius: crate::p2::utils::int_optional(
            "border-left-radius",
            properties,
            doc.name,
            0,
        )?,
        border_right_radius: crate::p2::utils::int_optional(
            "border-right-radius",
            properties,
            doc.name,
            0,
        )?,
        width: ftd_rt::Length::from(
            crate::p2::utils::string_optional("width", properties, doc.name, 0)?,
            doc.name,
        )?,
        min_width: ftd_rt::Length::from(
            crate::p2::utils::string_optional("min-width", properties, doc.name, 0)?,
            doc.name,
        )?,
        max_width: ftd_rt::Length::from(
            crate::p2::utils::string_optional("max-width", properties, doc.name, 0)?,
            doc.name,
        )?,
        height: ftd_rt::Length::from(
            crate::p2::utils::string_optional("height", properties, doc.name, 0)?,
            doc.name,
        )?,
        min_height: ftd_rt::Length::from(
            crate::p2::utils::string_optional("min-height", properties, doc.name, 0)?,
            doc.name,
        )?,
        max_height: ftd_rt::Length::from(
            crate::p2::utils::string_optional("max-height", properties, doc.name, 0)?,
            doc.name,
        )?,
        color: color_from(
            crate::p2::utils::string_optional("color", properties, doc.name, 0)?,
            doc.name,
        )?,
        background_color: color_from(
            crate::p2::utils::string_optional("background-color", properties, doc.name, 0)?,
            doc.name,
        )?,
        border_color: color_from(
            crate::p2::utils::string_optional("border-color", properties, doc.name, 0)?,
            doc.name,
        )?,
        border_width: crate::p2::utils::int_with_default(
            "border-width",
            0,
            properties,
            doc.name,
            0,
        )?,
        border_radius: crate::p2::utils::int_with_default(
            "border-radius",
            0,
            properties,
            doc.name,
            0,
        )?,
        data_id: crate::p2::utils::string_optional("id", properties, doc.name, 0)?.map(|v| {
            if is_child {
                v
            } else {
                format!("{}#{}", doc.name, v)
            }
        }),
        id: None,
        overflow_x: ftd_rt::Overflow::from(
            crate::p2::utils::string_optional("overflow-x", properties, doc.name, 0)?,
            doc.name,
        )?,
        overflow_y: ftd_rt::Overflow::from(
            crate::p2::utils::string_optional("overflow-y", properties, doc.name, 0)?,
            doc.name,
        )?,
        border_top: crate::p2::utils::int_optional("border-top", properties, doc.name, 0)?,
        border_left: crate::p2::utils::int_optional("border-left", properties, doc.name, 0)?,
        border_right: crate::p2::utils::int_optional("border-right", properties, doc.name, 0)?,
        border_bottom: crate::p2::utils::int_optional("border-bottom", properties, doc.name, 0)?,
        margin_top: crate::p2::utils::int_optional("margin-top", properties, doc.name, 0)?,
        margin_bottom: crate::p2::utils::int_optional("margin-bottom", properties, doc.name, 0)?,
        margin_left: crate::p2::utils::int_optional("margin-left", properties, doc.name, 0)?,
        margin_right: crate::p2::utils::int_optional("margin-right", properties, doc.name, 0)?,
        link,
        open_in_new_tab: crate::p2::utils::bool_with_default(
            "open-in-new-tab",
            false,
            properties,
            doc.name,
            0,
        )?,
        sticky: crate::p2::utils::bool_with_default("sticky", false, properties, doc.name, 0)?,
        top: crate::p2::utils::int_optional("top", properties, doc.name, 0)?,
        bottom: crate::p2::utils::int_optional("bottom", properties, doc.name, 0)?,
        left: crate::p2::utils::int_optional("left", properties, doc.name, 0)?,
        right: crate::p2::utils::int_optional("right", properties, doc.name, 0)?,
        cursor: crate::p2::utils::string_optional("cursor", properties, doc.name, 0)?,
        submit,
        shadow_offset_x: crate::p2::utils::int_optional(
            "shadow-offset-x",
            properties,
            doc.name,
            0,
        )?,
        shadow_offset_y: crate::p2::utils::int_optional(
            "shadow-offset-y",
            properties,
            doc.name,
            0,
        )?,
        shadow_size: crate::p2::utils::int_optional("shadow-size", properties, doc.name, 0)?,
        shadow_blur: crate::p2::utils::int_optional("shadow-blur", properties, doc.name, 0)?,
        shadow_color: color_from(
            crate::p2::utils::string_optional("shadow-color", properties, doc.name, 0)?,
            doc.name,
        )?,
        gradient_direction: ftd_rt::GradientDirection::from(
            crate::p2::utils::string_optional("gradient-direction", properties, doc.name, 0)?,
            doc.name,
        )?,
        anchor,
        gradient_colors,
        background_image: crate::p2::utils::string_optional(
            "background-image",
            properties,
            doc.name,
            0,
        )?,
        background_repeat: crate::p2::utils::bool_with_default(
            "background-repeat",
            false,
            properties,
            doc.name,
            0,
        )?,
        background_parallax: crate::p2::utils::bool_with_default(
            "background-parallax",
            false,
            properties,
            doc.name,
            0,
        )?,
        scale: crate::p2::utils::decimal_optional("scale", properties, doc.name, 0)?,
        scale_x: crate::p2::utils::decimal_optional("scale-x", properties, doc.name, 0)?,
        scale_y: crate::p2::utils::decimal_optional("scale-y", properties, doc.name, 0)?,
        rotate: crate::p2::utils::int_optional("rotate", properties, doc.name, 0)?,
        move_up: crate::p2::utils::int_optional("move-up", properties, doc.name, 0)?,
        move_down: crate::p2::utils::int_optional("move-down", properties, doc.name, 0)?,
        move_left: crate::p2::utils::int_optional("move-left", properties, doc.name, 0)?,
        move_right: crate::p2::utils::int_optional("move-right", properties, doc.name, 0)?,
        position: ftd_rt::Position::from(
            match crate::p2::utils::string_optional("position", properties, doc.name, 0)? {
                None => crate::p2::utils::string_optional("align", properties, doc.name, 0)?,
                Some(v) => Some(v),
            },
            doc.name,
        )?,
        inner: crate::p2::utils::bool_with_default(
            "inner",
            inner_default,
            properties,
            doc.name,
            0,
        )?,
    })
}

fn common_arguments() -> Vec<(String, crate::p2::Kind)> {
    vec![
        (
            "padding".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "padding-vertical".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "padding-horizontal".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "padding-left".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "padding-right".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "padding-top".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "padding-bottom".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "border-top-radius".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "border-bottom-radius".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "border-left-radius".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "border-right-radius".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "width".to_string(),
            crate::p2::Kind::string().into_optional(),
        ),
        (
            "min-width".to_string(),
            crate::p2::Kind::string().into_optional(),
        ),
        (
            "max-width".to_string(),
            crate::p2::Kind::string().into_optional(),
        ),
        (
            "height".to_string(),
            crate::p2::Kind::string().into_optional(),
        ),
        (
            "min-height".to_string(),
            crate::p2::Kind::string().into_optional(),
        ),
        (
            "max-height".to_string(),
            crate::p2::Kind::string().into_optional(),
        ),
        (
            // TODO: remove this after verifying that no existing document is using this
            "explain".to_string(),
            crate::p2::Kind::boolean().into_optional(),
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
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "border-radius".to_string(),
            crate::p2::Kind::integer().into_optional(),
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
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "border-bottom".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "border-left".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "border-right".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "margin-top".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "margin-bottom".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "margin-left".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "margin-right".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "link".to_string(),
            crate::p2::Kind::string().into_optional(),
        ),
        (
            "submit".to_string(),
            crate::p2::Kind::string().into_optional(),
        ),
        (
            "open-in-new-tab".to_string(),
            crate::p2::Kind::boolean().into_optional(),
        ),
        (
            "sticky".to_string(),
            crate::p2::Kind::boolean().into_optional(),
        ),
        (
            "top".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "bottom".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "left".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "right".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "cursor".to_string(),
            crate::p2::Kind::string().into_optional(),
        ),
        (
            "anchor".to_string(),
            crate::p2::Kind::string().into_optional(),
        ),
        (
            "gradient-direction".to_string(),
            crate::p2::Kind::string().into_optional(),
        ),
        (
            "gradient-colors".to_string(),
            crate::p2::Kind::string().into_optional(),
        ),
        (
            "shadow-offset-x".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "shadow-offset-y".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "shadow-blur".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "shadow-size".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "shadow-color".to_string(),
            crate::p2::Kind::string().into_optional(),
        ),
        (
            "background-image".to_string(),
            crate::p2::Kind::string().into_optional(),
        ),
        (
            "background-repeat".to_string(),
            crate::p2::Kind::boolean().into_optional(),
        ),
        (
            "background-parallax".to_string(),
            crate::p2::Kind::boolean().into_optional(),
        ),
        (
            "scale".to_string(),
            crate::p2::Kind::decimal().into_optional(),
        ),
        (
            "scale-x".to_string(),
            crate::p2::Kind::decimal().into_optional(),
        ),
        (
            "scale-y".to_string(),
            crate::p2::Kind::decimal().into_optional(),
        ),
        (
            "rotate".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "move-up".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "move-down".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "move-left".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "move-right".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "position".to_string(),
            crate::p2::Kind::string().into_optional(),
        ),
        (
            "inner".to_string(),
            crate::p2::Kind::boolean().into_optional(),
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
    doc: &crate::p2::TDoc,
) -> crate::p1::Result<ftd_rt::Container> {
    Ok(ftd_rt::Container {
        children: Default::default(),
        external_children: Default::default(),
        open: crate::p2::utils::string_bool_optional("open", properties, doc.name, 0)?,
        spacing: crate::p2::utils::int_optional("spacing", properties, doc.name, 0)?,
        wrap: crate::p2::utils::bool_with_default("wrap", false, properties, doc.name, 0)?,
    })
}

fn container_arguments() -> Vec<(String, crate::p2::Kind)> {
    vec![
        (
            "open".to_string(),
            crate::p2::Kind::string().into_optional(),
        ),
        (
            "spacing".to_string(),
            crate::p2::Kind::integer().into_optional(),
        ),
        (
            "align".to_string(),
            crate::p2::Kind::string().into_optional(),
        ),
        (
            "wrap".to_string(),
            crate::p2::Kind::boolean().into_optional(),
        ),
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
                (
                    "crop".to_string(),
                    crate::p2::Kind::boolean().into_optional(),
                ),
            ],
            common_arguments(),
        ]
        .concat()
        .into_iter()
        .collect(),
        locals: Default::default(),
        properties: Default::default(),
        instructions: Default::default(),
        invocations: Default::default(),
        condition: None,
        events: vec![],
        line_number: 0,
    }
}

pub fn image_from_properties(
    properties_with_ref: &std::collections::BTreeMap<String, (crate::Value, Option<String>)>,
    doc: &crate::p2::TDoc,
    condition: &Option<ftd::p2::Boolean>,
    is_child: bool,
    events: &[ftd::p2::Event],
    all_locals: &mut ftd_rt::Map,
    root_name: Option<&str>,
) -> crate::p1::Result<ftd_rt::Image> {
    let (src, reference) =
        crate::p2::utils::string_and_ref(0, "src", properties_with_ref, all_locals, doc.name)?;
    let properties = &ftd::p2::utils::properties(properties_with_ref);
    Ok(ftd_rt::Image {
        src,
        description: crate::p2::utils::string_optional("description", properties, doc.name, 0)?
            .unwrap_or_else(|| "".to_string()),
        common: common_from_properties(
            properties, doc, condition, is_child, events, all_locals, root_name, reference,
        )?,
        crop: crate::p2::utils::bool_with_default("crop", false, properties, doc.name, 0)?,
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
        locals: Default::default(),
        properties: Default::default(),
        instructions: Default::default(),
        invocations: Default::default(),
        condition: None,
        events: vec![],
        line_number: 0,
    }
}

pub fn row_from_properties(
    properties_with_ref: &std::collections::BTreeMap<String, (crate::Value, Option<String>)>,
    doc: &crate::p2::TDoc,
    condition: &Option<ftd::p2::Boolean>,
    is_child: bool,
    events: &[ftd::p2::Event],
    all_locals: &mut ftd_rt::Map,
    root_name: Option<&str>,
) -> crate::p1::Result<ftd_rt::Row> {
    let properties = &ftd::p2::utils::properties(properties_with_ref);
    Ok(ftd_rt::Row {
        common: common_from_properties(
            properties, doc, condition, is_child, events, all_locals, root_name, None,
        )?,
        container: container_from_properties(properties, doc)?,
    })
}

pub fn column_function() -> crate::Component {
    crate::Component {
        line_number: 0,
        kernel: true,
        full_name: "ftd#column".to_string(),
        root: "ftd.kernel".to_string(),
        arguments: [container_arguments(), common_arguments()]
            .concat()
            .into_iter()
            .collect(),
        locals: Default::default(),
        properties: Default::default(),
        instructions: Default::default(),
        invocations: Default::default(),
        condition: None,
        events: vec![],
    }
}
pub fn column_from_properties(
    properties_with_ref: &std::collections::BTreeMap<String, (crate::Value, Option<String>)>,
    doc: &crate::p2::TDoc,
    condition: &Option<ftd::p2::Boolean>,
    is_child: bool,
    events: &[ftd::p2::Event],
    all_locals: &mut ftd_rt::Map,
    root_name: Option<&str>,
) -> crate::p1::Result<ftd_rt::Column> {
    let properties = &ftd::p2::utils::properties(properties_with_ref);
    Ok(ftd_rt::Column {
        common: common_from_properties(
            properties, doc, condition, is_child, events, all_locals, root_name, None,
        )?,
        container: container_from_properties(properties, doc)?,
    })
}

pub fn external_font_from_properties(
    properties: &std::collections::BTreeMap<String, crate::Value>,
    doc: &crate::p2::TDoc,
) -> crate::p1::Result<Option<ftd_rt::ExternalFont>> {
    let font_option = crate::p2::utils::string_optional("font", properties, doc.name, 0)?;
    let font_url_option = crate::p2::utils::string_optional("font-url", properties, doc.name, 0)?;

    match (font_option, font_url_option) {
        (Some(font), Some(font_url)) => {
            let name_opt = font.split(',').next();
            let name = match name_opt {
                Some(f) => f.to_string(),
                None => {
                    return ftd::e2(
                        "Something went wrong while parsing font vector",
                        doc.name,
                        doc.name.to_string(),
                        0,
                    )
                }
            };

            Ok(Some(ftd_rt::ExternalFont {
                url: font_url,
                display: ftd_rt::FontDisplay::from(
                    crate::p2::utils::string_optional("font-display", properties, doc.name, 0)?,
                    doc.name,
                )?,
                name,
            }))
        }
        _ => Ok(None),
    }
}

#[allow(dead_code)]
#[allow(unused_variables)]
pub fn text_render(
    tf: &ftd_rt::TextFormat,
    text: String,
    source: crate::TextSource,
    theme: String,
    doc_id: &str,
) -> crate::p1::Result<ftd_rt::Rendered> {
    Ok(match (source, tf) {
        (ftd::TextSource::Body, ftd_rt::TextFormat::Markdown) => ftd::markdown(text.as_str()),
        (_, ftd_rt::TextFormat::Markdown) => ftd::markdown_line(text.as_str()),
        (_, ftd_rt::TextFormat::Latex) => ftd::latex(text.as_str(), doc_id)?,
        (_, ftd_rt::TextFormat::Code { lang }) => {
            ftd::code_with_theme(text.as_str(), lang.as_str(), theme.as_str(), doc_id)?
        }
        (_, ftd_rt::TextFormat::Text) => ftd_rt::Rendered {
            original: text.clone(),
            rendered: text,
        },
    })
}

pub fn iframe_function() -> crate::Component {
    crate::Component {
        line_number: 0,
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
        locals: Default::default(),
        properties: Default::default(),
        instructions: Default::default(),
        invocations: Default::default(),
        condition: None,
        events: vec![],
    }
}

pub fn iframe_from_properties(
    properties_with_ref: &std::collections::BTreeMap<String, (crate::Value, Option<String>)>,
    doc: &crate::p2::TDoc,
    condition: &Option<ftd::p2::Boolean>,
    is_child: bool,
    events: &[ftd::p2::Event],
    all_locals: &mut ftd_rt::Map,
    root_name: Option<&str>,
) -> crate::p1::Result<ftd_rt::IFrame> {
    let properties = &ftd::p2::utils::properties(properties_with_ref);
    let src = match (
        crate::p2::utils::string_optional("src", properties, doc.name, 0)?,
        crate::p2::utils::string_optional("youtube", properties, doc.name, 0)?
            .and_then(|id| crate::youtube_id::from_raw(id.as_str())),
    ) {
        (Some(src), None) => src,
        (None, Some(id)) => id,
        (Some(_), Some(_)) => {
            return ftd::e2(
                "both src and youtube id provided",
                doc.name,
                doc.name.to_string(),
                0,
            )
        }
        (None, None) => {
            return ftd::e2(
                "src or youtube id is required",
                doc.name,
                doc.name.to_string(),
                0,
            )
        }
    };

    Ok(ftd_rt::IFrame {
        src,
        common: common_from_properties(
            properties, doc, condition, is_child, events, all_locals, root_name, None,
        )?,
    })
}

pub fn text_from_properties(
    properties_with_ref: &std::collections::BTreeMap<String, (crate::Value, Option<String>)>,
    doc: &crate::p2::TDoc,
    condition: &Option<ftd::p2::Boolean>,
    is_child: bool,
    events: &[ftd::p2::Event],
    all_locals: &mut ftd_rt::Map,
    root_name: Option<&str>,
) -> crate::p1::Result<ftd_rt::Text> {
    let properties = &ftd::p2::utils::properties(properties_with_ref);

    let (text, source, reference) = crate::p2::utils::string_and_source_and_ref(
        0,
        "text",
        properties_with_ref,
        all_locals,
        doc.name,
    )?;
    let font_str = crate::p2::utils::string_optional("font", properties, doc.name, 0)?;

    let font: Vec<ftd_rt::NamedFont> = match font_str {
        Some(f) => f
            .split(',')
            .flat_map(|x| ftd_rt::NamedFont::from(Some(x.to_string())))
            .collect(),
        None => vec![],
    };
    Ok(ftd_rt::Text {
        line: source != ftd::TextSource::Body,
        text: if source == ftd::TextSource::Body {
            ftd::markdown(text.as_str())
        } else {
            ftd::markdown_line(text.as_str())
        },
        common: common_from_properties(
            properties, doc, condition, is_child, events, all_locals, root_name, reference,
        )?,
        text_align: ftd_rt::TextAlign::from(
            crate::p2::utils::string_optional("text-align", properties, doc.name, 0)?,
            doc.name,
        )?,
        style: ftd_rt::Style::from(
            crate::p2::utils::string_optional("style", properties, doc.name, 0)?,
            doc.name,
        )?,
        size: crate::p2::utils::int_optional("size", properties, doc.name, 0)?,
        external_font: external_font_from_properties(properties, doc)?,
        font,
        line_height: crate::p2::utils::int_optional("line-height", properties, doc.name, 0)?,
        line_clamp: crate::p2::utils::int_optional("line-clamp", properties, doc.name, 0)?,
    })
}

pub fn text_block_from_properties(
    properties_with_ref: &std::collections::BTreeMap<String, (crate::Value, Option<String>)>,
    doc: &crate::p2::TDoc,
    condition: &Option<ftd::p2::Boolean>,
    is_child: bool,
    events: &[ftd::p2::Event],
    all_locals: &mut ftd_rt::Map,
    root_name: Option<&str>,
) -> crate::p1::Result<ftd_rt::TextBlock> {
    let properties = &ftd::p2::utils::properties(properties_with_ref);

    let (text, source, reference) = crate::p2::utils::string_and_source_and_ref(
        0,
        "text",
        properties_with_ref,
        all_locals,
        doc.name,
    )?;
    let font_str = crate::p2::utils::string_optional("font", properties, doc.name, 0)?;

    let font: Vec<ftd_rt::NamedFont> = match font_str {
        Some(f) => f
            .split(',')
            .flat_map(|x| ftd_rt::NamedFont::from(Some(x.to_string())))
            .collect(),
        None => vec![],
    };
    Ok(ftd_rt::TextBlock {
        line: source != ftd::TextSource::Body,
        text: if source == ftd::TextSource::Body {
            ftd::markdown(text.as_str())
        } else {
            ftd::markdown_line(text.as_str())
        },
        common: common_from_properties(
            properties, doc, condition, is_child, events, all_locals, root_name, reference,
        )?,
        text_align: ftd_rt::TextAlign::from(
            crate::p2::utils::string_optional("text-align", properties, doc.name, 0)?,
            doc.name,
        )?,
        style: ftd_rt::Style::from(
            crate::p2::utils::string_optional("style", properties, doc.name, 0)?,
            doc.name,
        )?,
        size: crate::p2::utils::int_optional("size", properties, doc.name, 0)?,
        external_font: external_font_from_properties(properties, doc)?,
        font,
        line_height: crate::p2::utils::int_optional("line-height", properties, doc.name, 0)?,
        line_clamp: crate::p2::utils::int_optional("line-clamp", properties, doc.name, 0)?,
    })
}

pub fn code_from_properties(
    properties_with_ref: &std::collections::BTreeMap<String, (crate::Value, Option<String>)>,
    doc: &crate::p2::TDoc,
    condition: &Option<ftd::p2::Boolean>,
    is_child: bool,
    events: &[ftd::p2::Event],
    all_locals: &mut ftd_rt::Map,
    root_name: Option<&str>,
) -> crate::p1::Result<ftd_rt::Code> {
    let properties = &ftd::p2::utils::properties(properties_with_ref);

    let (text, _, reference) = crate::p2::utils::string_and_source_and_ref(
        0,
        "text",
        properties_with_ref,
        all_locals,
        doc.name,
    )?;
    let font_str = crate::p2::utils::string_optional("font", properties, doc.name, 0)?;

    let font: Vec<ftd_rt::NamedFont> = match font_str {
        Some(f) => f
            .split(',')
            .flat_map(|x| ftd_rt::NamedFont::from(Some(x.to_string())))
            .collect(),
        None => vec![],
    };

    Ok(ftd_rt::Code {
        text: ftd::code_with_theme(
            text.as_str(),
            crate::p2::utils::string_optional("lang", properties, doc.name, 0)?
                .unwrap_or_else(|| "txt".to_string())
                .as_str(),
            crate::p2::utils::string_with_default(
                "theme",
                crate::render::DEFAULT_THEME,
                properties,
                doc.name,
                0,
            )?
            .as_str(),
            doc.name,
        )?,
        common: common_from_properties(
            properties, doc, condition, is_child, events, all_locals, root_name, reference,
        )?,
        text_align: ftd_rt::TextAlign::from(
            crate::p2::utils::string_optional("text-align", properties, doc.name, 0)?,
            doc.name,
        )?,
        style: ftd_rt::Style::from(
            crate::p2::utils::string_optional("style", properties, doc.name, 0)?,
            doc.name,
        )?,
        size: crate::p2::utils::int_optional("size", properties, doc.name, 0)?,
        external_font: external_font_from_properties(properties, doc)?,
        font,
        line_height: crate::p2::utils::int_optional("line-height", properties, doc.name, 0)?,
        line_clamp: crate::p2::utils::int_optional("line-clamp", properties, doc.name, 0)?,
    })
}

pub fn integer_from_properties(
    properties_with_ref: &std::collections::BTreeMap<String, (crate::Value, Option<String>)>,
    doc: &crate::p2::TDoc,
    condition: &Option<ftd::p2::Boolean>,
    is_child: bool,
    events: &[ftd::p2::Event],
    all_locals: &mut ftd_rt::Map,
    root_name: Option<&str>,
) -> crate::p1::Result<ftd_rt::Text> {
    let properties = &ftd::p2::utils::properties(properties_with_ref);
    let font_str = crate::p2::utils::string_optional("font", properties, doc.name, 0)?;
    let num = format_num::NumberFormat::new();
    let text = match crate::p2::utils::string_optional("format", properties, doc.name, 0)? {
        Some(f) => num.format(
            f.as_str(),
            crate::p2::utils::int("value", properties, doc.name, 0)? as f64,
        ),
        None => crate::p2::utils::int("value", properties, doc.name, 0)?.to_string(),
    };
    let reference = ftd::p2::utils::complete_reference(
        &properties_with_ref.get("value").expect("").1,
        all_locals,
    );

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
        common: common_from_properties(
            properties, doc, condition, is_child, events, all_locals, root_name, reference,
        )?,
        text_align: ftd_rt::TextAlign::from(
            crate::p2::utils::string_optional("text-align", properties, doc.name, 0)?,
            doc.name,
        )?,
        style: ftd_rt::Style::from(
            crate::p2::utils::string_optional("style", properties, doc.name, 0)?,
            doc.name,
        )?,
        size: crate::p2::utils::int_optional("size", properties, doc.name, 0)?,
        external_font: external_font_from_properties(properties, doc)?,
        font,
        line_height: crate::p2::utils::int_optional("line-height", properties, doc.name, 0)?,
        line_clamp: crate::p2::utils::int_optional("line-clamp", properties, doc.name, 0)?,
    })
}

pub fn decimal_from_properties(
    properties_with_ref: &std::collections::BTreeMap<String, (crate::Value, Option<String>)>,
    doc: &crate::p2::TDoc,
    condition: &Option<ftd::p2::Boolean>,
    is_child: bool,
    events: &[ftd::p2::Event],
    all_locals: &mut ftd_rt::Map,
    root_name: Option<&str>,
) -> crate::p1::Result<ftd_rt::Text> {
    let properties = &ftd::p2::utils::properties(properties_with_ref);
    let font_str = crate::p2::utils::string_optional("font", properties, doc.name, 0)?;
    let num = format_num::NumberFormat::new();
    let text = match crate::p2::utils::string_optional("format", properties, doc.name, 0)? {
        Some(f) => num.format(
            f.as_str(),
            crate::p2::utils::decimal("value", properties, doc.name, 0)?,
        ),
        None => crate::p2::utils::decimal("value", properties, doc.name, 0)?.to_string(),
    };

    let reference = ftd::p2::utils::complete_reference(
        &properties_with_ref.get("value").expect("").1,
        all_locals,
    );

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
        common: common_from_properties(
            properties, doc, condition, is_child, events, all_locals, root_name, reference,
        )?,
        text_align: ftd_rt::TextAlign::from(
            crate::p2::utils::string_optional("text-align", properties, doc.name, 0)?,
            doc.name,
        )?,
        style: ftd_rt::Style::from(
            crate::p2::utils::string_optional("style", properties, doc.name, 0)?,
            doc.name,
        )?,
        size: crate::p2::utils::int_optional("size", properties, doc.name, 0)?,
        external_font: external_font_from_properties(properties, doc)?,
        font,
        line_height: crate::p2::utils::int_optional("line-height", properties, doc.name, 0)?,
        line_clamp: crate::p2::utils::int_optional("line-clamp", properties, doc.name, 0)?,
    })
}

pub fn color_from(l: Option<String>, doc_id: &str) -> ftd::p1::Result<Option<ftd_rt::Color>> {
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
        Err(e) => {
            return ftd::e2(
                format!("{} is not a valid color: {:?}", v, e),
                doc_id,
                doc_id.to_string(),
                0,
            )
        }
    }
}

pub fn boolean_from_properties(
    properties_with_ref: &std::collections::BTreeMap<String, (crate::Value, Option<String>)>,
    doc: &crate::p2::TDoc,
    condition: &Option<ftd::p2::Boolean>,
    is_child: bool,
    events: &[ftd::p2::Event],
    all_locals: &mut ftd_rt::Map,
    root_name: Option<&str>,
) -> crate::p1::Result<ftd_rt::Text> {
    let properties = &ftd::p2::utils::properties(properties_with_ref);
    let font_str = crate::p2::utils::string_optional("font", properties, doc.name, 0)?;
    let value = crate::p2::utils::bool("value", properties, doc.name, 0)?;
    let text = if value {
        crate::p2::utils::string_with_default("true", "true", properties, doc.name, 0)?
    } else {
        crate::p2::utils::string_with_default("false", "false", properties, doc.name, 0)?
    };

    let reference = ftd::p2::utils::complete_reference(
        &properties_with_ref.get("value").expect("").1,
        all_locals,
    );

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
        common: common_from_properties(
            properties, doc, condition, is_child, events, all_locals, root_name, reference,
        )?,
        text_align: ftd_rt::TextAlign::from(
            crate::p2::utils::string_optional("text-align", properties, doc.name, 0)?,
            doc.name,
        )?,
        style: ftd_rt::Style::from(
            crate::p2::utils::string_optional("style", properties, doc.name, 0)?,
            doc.name,
        )?,
        size: crate::p2::utils::int_optional("size", properties, doc.name, 0)?,
        external_font: external_font_from_properties(properties, doc)?,
        font,
        line_height: crate::p2::utils::int_optional("line-height", properties, doc.name, 0)?,
        line_clamp: crate::p2::utils::int_optional("line-clamp", properties, doc.name, 0)?,
    })
}

pub fn text_function(is_text_block: bool) -> crate::Component {
    let full_name = if is_text_block {
        "ftd#text-block"
    } else {
        "ftd#text"
    };

    crate::Component {
        line_number: 0,
        kernel: true,
        root: "ftd.kernel".to_string(),
        full_name: full_name.to_string(),
        arguments: [
            vec![
                ("text".to_string(), crate::p2::Kind::caption_or_body()),
                (
                    "align".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                (
                    "style".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                (
                    "size".to_string(),
                    crate::p2::Kind::integer().into_optional(),
                ),
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
                    crate::p2::Kind::integer().into_optional(),
                ),
                (
                    "line-clamp".to_string(),
                    crate::p2::Kind::integer().into_optional(),
                ),
                (
                    "text-align".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
            ],
            common_arguments(),
        ]
        .concat()
        .into_iter()
        .collect(),
        locals: Default::default(),
        properties: Default::default(),
        instructions: Default::default(),
        invocations: Default::default(),
        condition: None,
        events: vec![],
    }
}

pub fn code_function() -> crate::Component {
    crate::Component {
        line_number: 0,
        kernel: true,
        root: "ftd.kernel".to_string(),
        full_name: "ftd#code".to_string(),
        arguments: [
            vec![
                ("text".to_string(), crate::p2::Kind::caption_or_body()),
                (
                    "align".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
                (
                    "style".to_string(),
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
                (
                    "size".to_string(),
                    crate::p2::Kind::integer().into_optional(),
                ),
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
                    crate::p2::Kind::integer().into_optional(),
                ),
                (
                    "line-clamp".to_string(),
                    crate::p2::Kind::integer().into_optional(),
                ),
                (
                    "text-align".to_string(),
                    crate::p2::Kind::string().into_optional(),
                ),
            ],
            common_arguments(),
        ]
        .concat()
        .into_iter()
        .collect(),
        locals: Default::default(),
        properties: Default::default(),
        instructions: Default::default(),
        invocations: Default::default(),
        condition: None,
        events: vec![],
    }
}

pub fn integer_function() -> crate::Component {
    crate::Component {
        line_number: 0,
        kernel: true,
        root: "ftd.kernel".to_string(),
        full_name: "ftd#integer".to_string(),
        arguments: [
            vec![
                ("value".to_string(), crate::p2::Kind::integer()),
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
                    "size".to_string(),
                    crate::p2::Kind::integer().into_optional(),
                ),
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
        locals: Default::default(),
        properties: Default::default(),
        instructions: Default::default(),
        invocations: Default::default(),
        condition: None,
        events: vec![],
    }
}

pub fn decimal_function() -> crate::Component {
    crate::Component {
        line_number: 0,
        kernel: true,
        root: "ftd.kernel".to_string(),
        full_name: "ftd#decimal".to_string(),
        arguments: [
            vec![
                ("value".to_string(), crate::p2::Kind::decimal()),
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
                    "size".to_string(),
                    crate::p2::Kind::integer().into_optional(),
                ),
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
        locals: Default::default(),
        properties: Default::default(),
        instructions: Default::default(),
        invocations: Default::default(),
        condition: None,
        events: vec![],
    }
}

pub fn scene_function() -> crate::Component {
    let arguments = {
        let mut arguments: std::collections::BTreeMap<String, ftd::p2::Kind> =
            [container_arguments(), common_arguments()]
                .concat()
                .into_iter()
                .collect();
        arguments.remove("spacing");
        arguments.remove("wrap");
        arguments
    };

    crate::Component {
        line_number: 0,
        kernel: true,
        root: "ftd.kernel".to_string(),
        full_name: "ftd#scene".to_string(),
        arguments,
        locals: Default::default(),
        properties: Default::default(),
        instructions: Default::default(),
        invocations: Default::default(),
        condition: None,
        events: vec![],
    }
}

pub fn boolean_function() -> crate::Component {
    crate::Component {
        line_number: 0,
        kernel: true,
        root: "ftd.kernel".to_string(),
        full_name: "ftd#boolean".to_string(),
        arguments: [
            vec![
                ("value".to_string(), crate::p2::Kind::boolean()),
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
                    "size".to_string(),
                    crate::p2::Kind::integer().into_optional(),
                ),
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
        locals: Default::default(),
        properties: Default::default(),
        instructions: Default::default(),
        invocations: Default::default(),
        condition: None,
        events: vec![],
    }
}

pub fn input_function() -> crate::Component {
    crate::Component {
        line_number: 0,
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
        locals: Default::default(),
        properties: Default::default(),
        instructions: Default::default(),
        invocations: Default::default(),
        condition: None,
        events: vec![],
    }
}

pub fn input_from_properties(
    properties_with_ref: &std::collections::BTreeMap<String, (crate::Value, Option<String>)>,
    doc: &crate::p2::TDoc,
    condition: &Option<ftd::p2::Boolean>,
    is_child: bool,
    events: &[ftd::p2::Event],
    all_locals: &mut ftd_rt::Map,
    root_name: Option<&str>,
) -> crate::p1::Result<ftd_rt::Input> {
    let properties = &ftd::p2::utils::properties(properties_with_ref);
    Ok(ftd_rt::Input {
        common: common_from_properties(
            properties, doc, condition, is_child, events, all_locals, root_name, None,
        )?,
        placeholder: crate::p2::utils::string_optional("placeholder", properties, doc.name, 0)?,
    })
}

pub fn scene_from_properties(
    properties_with_ref: &std::collections::BTreeMap<String, (crate::Value, Option<String>)>,
    doc: &crate::p2::TDoc,
    condition: &Option<ftd::p2::Boolean>,
    is_child: bool,
    events: &[ftd::p2::Event],
    all_locals: &mut ftd_rt::Map,
    root_name: Option<&str>,
) -> crate::p1::Result<ftd_rt::Scene> {
    let properties = &ftd::p2::utils::properties(properties_with_ref);
    Ok(ftd_rt::Scene {
        common: common_from_properties(
            properties, doc, condition, is_child, events, all_locals, root_name, None,
        )?,
        container: container_from_properties(properties, doc)?,
    })
}
