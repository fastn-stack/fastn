#[allow(clippy::too_many_arguments)]
pub fn common_from_properties(
    unresolved_properties: &ftd::Map<ftd::ftd2021::component::Property>,
    doc: &ftd::ftd2021::p2::TDoc,
    condition: &Option<ftd::ftd2021::p2::Boolean>,
    is_child: bool,
    events: &[ftd::ftd2021::p2::Event],
    reference: Option<String>,
) -> ftd::ftd2021::p1::Result<ftd::Common> {
    let properties = &ftd::ftd2021::component::resolve_properties(0, unresolved_properties, doc)?;
    let submit = ftd::ftd2021::p2::utils::string_optional("submit", properties, doc.name, 0)?;
    let link = ftd::ftd2021::p2::utils::string_optional("link", properties, doc.name, 0)?;
    if let (Some(_), Some(_)) = (&submit, &link) {
        return ftd::ftd2021::p2::utils::e2(
            "Cannot have both submit and link together",
            doc.name,
            0,
        );
    }
    let gradient_color_str =
        ftd::ftd2021::p2::utils::string_optional("gradient-colors", properties, doc.name, 0)?;

    let gradient_colors: Vec<ftd::ColorValue> = match gradient_color_str {
        Some(f) => f
            .split(',')
            .flat_map(|x| color_from(Some(x.to_string()), doc.name).ok()?)
            .collect(),
        None => vec![],
    };

    let anchor = ftd::Anchor::from(
        ftd::ftd2021::p2::utils::string_optional("anchor", properties, doc.name, 0)?,
        doc.name,
    )?;

    let (position, inner) = {
        let mut position = None;
        let mut inner = match anchor {
            Some(ref p) => match p {
                ftd::Anchor::Parent => false,
                ftd::Anchor::Window => true,
            },
            None => false,
        };
        let position_inner =
            match ftd::ftd2021::p2::utils::string_optional("position", properties, doc.name, 0)? {
                None => ftd::ftd2021::p2::utils::string_optional("align", properties, doc.name, 0)?,
                Some(v) => Some(v),
            };
        if let Some(position_inner) = position_inner {
            if let Some(p) = position_inner.strip_prefix("inner ") {
                position = ftd::Position::from(Some(p.to_string()), doc.name)?;
                inner = true;
            } else {
                position = ftd::Position::from(Some(position_inner), doc.name)?;
            }
        }
        (position, inner)
    };

    let (cond, is_visible) = match condition {
        Some(c) => {
            let mut is_visible = true;
            if !c.eval(0, doc)? {
                is_visible = false;
            }
            if !c.is_arg_constant() {
                (Some(c.to_condition(0, doc)?), is_visible)
            } else {
                (None, is_visible)
            }
        }
        _ => (None, true),
    };

    Ok(ftd::Common {
        title: ftd::ftd2021::p2::utils::string_optional("title", properties, doc.name, 0)?,
        conditional_attribute: Default::default(),
        condition: cond,
        is_not_visible: !is_visible,
        is_dummy: false,
        events: ftd::ftd2021::p2::Event::get_events(0, events, doc)?,
        reference,
        region: ftd::Region::from(
            ftd::ftd2021::p2::utils::string_optional("region", properties, doc.name, 0)?,
            doc.name,
        )?,
        padding: ftd::ftd2021::p2::utils::int_optional("padding", properties, doc.name, 0)?,
        padding_vertical: ftd::ftd2021::p2::utils::int_optional(
            "padding-vertical",
            properties,
            doc.name,
            0,
        )?,
        padding_horizontal: ftd::ftd2021::p2::utils::int_optional(
            "padding-horizontal",
            properties,
            doc.name,
            0,
        )?,
        classes: ftd::ftd2021::p2::utils::string_optional("classes", properties, doc.name, 0)?,
        padding_left: ftd::ftd2021::p2::utils::int_optional(
            "padding-left",
            properties,
            doc.name,
            0,
        )?,
        padding_right: ftd::ftd2021::p2::utils::int_optional(
            "padding-right",
            properties,
            doc.name,
            0,
        )?,
        padding_top: ftd::ftd2021::p2::utils::int_optional("padding-top", properties, doc.name, 0)?,
        padding_bottom: ftd::ftd2021::p2::utils::int_optional(
            "padding-bottom",
            properties,
            doc.name,
            0,
        )?,
        border_top_radius: ftd::ftd2021::p2::utils::int_optional(
            "border-top-radius",
            properties,
            doc.name,
            0,
        )?,
        border_bottom_radius: ftd::ftd2021::p2::utils::int_optional(
            "border-bottom-radius",
            properties,
            doc.name,
            0,
        )?,
        border_left_radius: ftd::ftd2021::p2::utils::int_optional(
            "border-left-radius",
            properties,
            doc.name,
            0,
        )?,
        border_right_radius: ftd::ftd2021::p2::utils::int_optional(
            "border-right-radius",
            properties,
            doc.name,
            0,
        )?,
        width: ftd::Length::from(
            ftd::ftd2021::p2::utils::string_optional("width", properties, doc.name, 0)?,
            doc.name,
        )?,
        min_width: ftd::Length::from(
            ftd::ftd2021::p2::utils::string_optional("min-width", properties, doc.name, 0)?,
            doc.name,
        )?,
        max_width: ftd::Length::from(
            ftd::ftd2021::p2::utils::string_optional("max-width", properties, doc.name, 0)?,
            doc.name,
        )?,
        height: ftd::Length::from(
            ftd::ftd2021::p2::utils::string_optional("height", properties, doc.name, 0)?,
            doc.name,
        )?,
        min_height: ftd::Length::from(
            ftd::ftd2021::p2::utils::string_optional("min-height", properties, doc.name, 0)?,
            doc.name,
        )?,
        max_height: ftd::Length::from(
            ftd::ftd2021::p2::utils::string_optional("max-height", properties, doc.name, 0)?,
            doc.name,
        )?,
        color: ftd::Color::from(
            ftd::ftd2021::p2::utils::record_optional_with_ref(
                "color",
                unresolved_properties,
                doc,
                0,
            )?,
            doc,
            0,
        )?,
        background_color: ftd::Color::from(
            ftd::ftd2021::p2::utils::record_optional_with_ref(
                "background-color",
                unresolved_properties,
                doc,
                0,
            )?,
            doc,
            0,
        )?,
        border_color: ftd::Color::from(
            ftd::ftd2021::p2::utils::record_optional_with_ref(
                "border-color",
                unresolved_properties,
                doc,
                0,
            )?,
            doc,
            0,
        )?,
        border_width: ftd::ftd2021::p2::utils::int_with_default(
            "border-width",
            0,
            properties,
            doc.name,
            0,
        )?,
        border_radius: ftd::ftd2021::p2::utils::int_with_default(
            "border-radius",
            0,
            properties,
            doc.name,
            0,
        )?,
        data_id: ftd::ftd2021::p2::utils::string_optional("id", properties, doc.name, 0)?.map(
            |v| {
                if is_child {
                    v
                } else {
                    format!("{}#{}", doc.name, v)
                }
            },
        ),
        id: None,
        overflow_x: ftd::Overflow::from(
            ftd::ftd2021::p2::utils::string_optional("overflow-x", properties, doc.name, 0)?,
            doc.name,
        )?,
        overflow_y: ftd::Overflow::from(
            ftd::ftd2021::p2::utils::string_optional("overflow-y", properties, doc.name, 0)?,
            doc.name,
        )?,
        border_top: ftd::ftd2021::p2::utils::int_optional("border-top", properties, doc.name, 0)?,
        border_left: ftd::ftd2021::p2::utils::int_optional("border-left", properties, doc.name, 0)?,
        border_right: ftd::ftd2021::p2::utils::int_optional(
            "border-right",
            properties,
            doc.name,
            0,
        )?,
        border_bottom: ftd::ftd2021::p2::utils::int_optional(
            "border-bottom",
            properties,
            doc.name,
            0,
        )?,
        border_top_color: ftd::Color::from(
            ftd::ftd2021::p2::utils::record_optional_with_ref(
                "border-top-color",
                unresolved_properties,
                doc,
                0,
            )?,
            doc,
            0,
        )?,
        border_left_color: ftd::Color::from(
            ftd::ftd2021::p2::utils::record_optional_with_ref(
                "border-left-color",
                unresolved_properties,
                doc,
                0,
            )?,
            doc,
            0,
        )?,
        border_right_color: ftd::Color::from(
            ftd::ftd2021::p2::utils::record_optional_with_ref(
                "border-right-color",
                unresolved_properties,
                doc,
                0,
            )?,
            doc,
            0,
        )?,
        border_bottom_color: ftd::Color::from(
            ftd::ftd2021::p2::utils::record_optional_with_ref(
                "border-bottom-color",
                unresolved_properties,
                doc,
                0,
            )?,
            doc,
            0,
        )?,
        margin_top: ftd::ftd2021::p2::utils::int_optional("margin-top", properties, doc.name, 0)?,
        margin_bottom: ftd::ftd2021::p2::utils::int_optional(
            "margin-bottom",
            properties,
            doc.name,
            0,
        )?,
        margin_left: ftd::ftd2021::p2::utils::int_optional("margin-left", properties, doc.name, 0)?,
        margin_right: ftd::ftd2021::p2::utils::int_optional(
            "margin-right",
            properties,
            doc.name,
            0,
        )?,
        link,
        open_in_new_tab: ftd::ftd2021::p2::utils::bool_with_default(
            "open-in-new-tab",
            false,
            properties,
            doc.name,
            0,
        )?,
        sticky: ftd::ftd2021::p2::utils::bool_with_default(
            "sticky", false, properties, doc.name, 0,
        )?,
        top: ftd::ftd2021::p2::utils::int_optional("top", properties, doc.name, 0)?,
        bottom: ftd::ftd2021::p2::utils::int_optional("bottom", properties, doc.name, 0)?,
        left: ftd::ftd2021::p2::utils::int_optional("left", properties, doc.name, 0)?,
        right: ftd::ftd2021::p2::utils::int_optional("right", properties, doc.name, 0)?,
        cursor: ftd::ftd2021::p2::utils::string_optional("cursor", properties, doc.name, 0)?,
        submit,
        shadow_offset_x: ftd::ftd2021::p2::utils::int_optional(
            "shadow-offset-x",
            properties,
            doc.name,
            0,
        )?,
        shadow_offset_y: ftd::ftd2021::p2::utils::int_optional(
            "shadow-offset-y",
            properties,
            doc.name,
            0,
        )?,
        shadow_size: ftd::ftd2021::p2::utils::int_optional("shadow-size", properties, doc.name, 0)?,
        shadow_blur: ftd::ftd2021::p2::utils::int_optional("shadow-blur", properties, doc.name, 0)?,
        shadow_color: ftd::Color::from(
            ftd::ftd2021::p2::utils::record_optional_with_ref(
                "shadow-color",
                unresolved_properties,
                doc,
                0,
            )?,
            doc,
            0,
        )?,
        gradient_direction: ftd::GradientDirection::from(
            ftd::ftd2021::p2::utils::string_optional(
                "gradient-direction",
                properties,
                doc.name,
                0,
            )?,
            doc.name,
        )?,
        anchor,
        gradient_colors,
        background_image: {
            let (src, reference) = ftd::ftd2021::p2::utils::record_optional_with_ref(
                "background-image",
                unresolved_properties,
                doc,
                0,
            )?;
            src.map_or(Ok(None), |r| {
                ftd::ImageSrc::from(&r, doc, 0, reference).map(Some)
            })?
        },
        background_repeat: ftd::ftd2021::p2::utils::bool_with_default(
            "background-repeat",
            false,
            properties,
            doc.name,
            0,
        )?,
        background_parallax: ftd::ftd2021::p2::utils::bool_with_default(
            "background-parallax",
            false,
            properties,
            doc.name,
            0,
        )?,
        scale: ftd::ftd2021::p2::utils::decimal_optional("scale", properties, doc.name, 0)?,
        scale_x: ftd::ftd2021::p2::utils::decimal_optional("scale-x", properties, doc.name, 0)?,
        scale_y: ftd::ftd2021::p2::utils::decimal_optional("scale-y", properties, doc.name, 0)?,
        rotate: ftd::ftd2021::p2::utils::int_optional("rotate", properties, doc.name, 0)?,
        move_up: ftd::ftd2021::p2::utils::int_optional("move-up", properties, doc.name, 0)?,
        move_down: ftd::ftd2021::p2::utils::int_optional("move-down", properties, doc.name, 0)?,
        move_left: ftd::ftd2021::p2::utils::int_optional("move-left", properties, doc.name, 0)?,
        move_right: ftd::ftd2021::p2::utils::int_optional("move-right", properties, doc.name, 0)?,
        position,
        inner,
        z_index: ftd::ftd2021::p2::utils::int_optional("z-index", properties, doc.name, 0)?,
        slot: ftd::ftd2021::p2::utils::string_optional("slot", properties, doc.name, 0)?,
        grid_column: ftd::ftd2021::p2::utils::string_optional(
            "grid-column",
            properties,
            doc.name,
            0,
        )?,
        grid_row: ftd::ftd2021::p2::utils::string_optional("grid-row", properties, doc.name, 0)?,
        white_space: ftd::ftd2021::p2::utils::string_optional(
            "white-space",
            properties,
            doc.name,
            0,
        )?,
        border_style: ftd::ftd2021::p2::utils::string_optional(
            "border-style",
            properties,
            doc.name,
            0,
        )?,
        text_transform: ftd::ftd2021::p2::utils::string_optional(
            "text-transform",
            properties,
            doc.name,
            0,
        )?,
        heading_number: ftd::ftd2021::p2::utils::string_list_optional(
            "heading-number",
            properties,
            doc,
            0,
        )?,
    })
}

fn common_arguments() -> Vec<(String, ftd::ftd2021::p2::Kind)> {
    vec![
        (
            "classes".to_string(),
            ftd::ftd2021::p2::Kind::string().into_optional(),
        ),
        (
            "padding".to_string(),
            ftd::ftd2021::p2::Kind::integer().into_optional(),
        ),
        (
            "padding-vertical".to_string(),
            ftd::ftd2021::p2::Kind::integer().into_optional(),
        ),
        (
            "padding-horizontal".to_string(),
            ftd::ftd2021::p2::Kind::integer().into_optional(),
        ),
        (
            "padding-left".to_string(),
            ftd::ftd2021::p2::Kind::integer().into_optional(),
        ),
        (
            "padding-right".to_string(),
            ftd::ftd2021::p2::Kind::integer().into_optional(),
        ),
        (
            "padding-top".to_string(),
            ftd::ftd2021::p2::Kind::integer().into_optional(),
        ),
        (
            "padding-bottom".to_string(),
            ftd::ftd2021::p2::Kind::integer().into_optional(),
        ),
        (
            "border-top-radius".to_string(),
            ftd::ftd2021::p2::Kind::integer().into_optional(),
        ),
        (
            "border-bottom-radius".to_string(),
            ftd::ftd2021::p2::Kind::integer().into_optional(),
        ),
        (
            "border-left-radius".to_string(),
            ftd::ftd2021::p2::Kind::integer().into_optional(),
        ),
        (
            "border-right-radius".to_string(),
            ftd::ftd2021::p2::Kind::integer().into_optional(),
        ),
        (
            "width".to_string(),
            ftd::ftd2021::p2::Kind::string().into_optional(),
        ),
        (
            "min-width".to_string(),
            ftd::ftd2021::p2::Kind::string().into_optional(),
        ),
        (
            "max-width".to_string(),
            ftd::ftd2021::p2::Kind::string().into_optional(),
        ),
        (
            "height".to_string(),
            ftd::ftd2021::p2::Kind::string().into_optional(),
        ),
        (
            "min-height".to_string(),
            ftd::ftd2021::p2::Kind::string().into_optional(),
        ),
        (
            "max-height".to_string(),
            ftd::ftd2021::p2::Kind::string().into_optional(),
        ),
        (
            // TODO: remove this after verifying that no existing document is using this
            "explain".to_string(),
            ftd::ftd2021::p2::Kind::boolean().into_optional(),
        ),
        (
            "region".to_string(),
            ftd::ftd2021::p2::Kind::string().into_optional(),
        ),
        (
            "color".to_string(),
            ftd::ftd2021::p2::Kind::Record {
                name: "ftd#color".to_string(),
                default: None,
                is_reference: false,
            }
            .into_optional(),
        ),
        (
            "background-color".to_string(),
            ftd::ftd2021::p2::Kind::Record {
                name: "ftd#color".to_string(),
                default: None,
                is_reference: false,
            }
            .into_optional(),
        ),
        (
            "border-color".to_string(),
            ftd::ftd2021::p2::Kind::Record {
                name: "ftd#color".to_string(),
                default: None,
                is_reference: false,
            }
            .into_optional(),
        ),
        (
            "border-width".to_string(),
            ftd::ftd2021::p2::Kind::integer().into_optional(),
        ),
        (
            "border-radius".to_string(),
            ftd::ftd2021::p2::Kind::integer().into_optional(),
        ),
        (
            "id".to_string(),
            ftd::ftd2021::p2::Kind::string().into_optional(),
        ),
        (
            "overflow-x".to_string(),
            ftd::ftd2021::p2::Kind::string().into_optional(),
        ),
        (
            "overflow-y".to_string(),
            ftd::ftd2021::p2::Kind::string().into_optional(),
        ),
        (
            "border-top".to_string(),
            ftd::ftd2021::p2::Kind::integer().into_optional(),
        ),
        (
            "border-bottom".to_string(),
            ftd::ftd2021::p2::Kind::integer().into_optional(),
        ),
        (
            "border-left".to_string(),
            ftd::ftd2021::p2::Kind::integer().into_optional(),
        ),
        (
            "border-right".to_string(),
            ftd::ftd2021::p2::Kind::integer().into_optional(),
        ),
        (
            "border-top-color".to_string(),
            ftd::ftd2021::p2::Kind::record("ftd#color").into_optional(),
        ),
        (
            "border-left-color".to_string(),
            ftd::ftd2021::p2::Kind::record("ftd#color").into_optional(),
        ),
        (
            "border-right-color".to_string(),
            ftd::ftd2021::p2::Kind::record("ftd#color").into_optional(),
        ),
        (
            "border-bottom-color".to_string(),
            ftd::ftd2021::p2::Kind::record("ftd#color").into_optional(),
        ),
        (
            "margin-top".to_string(),
            ftd::ftd2021::p2::Kind::integer().into_optional(),
        ),
        (
            "margin-bottom".to_string(),
            ftd::ftd2021::p2::Kind::integer().into_optional(),
        ),
        (
            "margin-left".to_string(),
            ftd::ftd2021::p2::Kind::integer().into_optional(),
        ),
        (
            "margin-right".to_string(),
            ftd::ftd2021::p2::Kind::integer().into_optional(),
        ),
        (
            "link".to_string(),
            ftd::ftd2021::p2::Kind::string().into_optional(),
        ),
        (
            "submit".to_string(),
            ftd::ftd2021::p2::Kind::string().into_optional(),
        ),
        (
            "open-in-new-tab".to_string(),
            ftd::ftd2021::p2::Kind::boolean().into_optional(),
        ),
        (
            "sticky".to_string(),
            ftd::ftd2021::p2::Kind::boolean().into_optional(),
        ),
        (
            "top".to_string(),
            ftd::ftd2021::p2::Kind::integer().into_optional(),
        ),
        (
            "bottom".to_string(),
            ftd::ftd2021::p2::Kind::integer().into_optional(),
        ),
        (
            "left".to_string(),
            ftd::ftd2021::p2::Kind::integer().into_optional(),
        ),
        (
            "right".to_string(),
            ftd::ftd2021::p2::Kind::integer().into_optional(),
        ),
        (
            "cursor".to_string(),
            ftd::ftd2021::p2::Kind::string().into_optional(),
        ),
        (
            "anchor".to_string(),
            ftd::ftd2021::p2::Kind::string().into_optional(),
        ),
        (
            "gradient-direction".to_string(),
            ftd::ftd2021::p2::Kind::string().into_optional(),
        ),
        (
            "gradient-colors".to_string(),
            ftd::ftd2021::p2::Kind::string().into_optional(),
        ),
        (
            "shadow-offset-x".to_string(),
            ftd::ftd2021::p2::Kind::integer().into_optional(),
        ),
        (
            "shadow-offset-y".to_string(),
            ftd::ftd2021::p2::Kind::integer().into_optional(),
        ),
        (
            "shadow-blur".to_string(),
            ftd::ftd2021::p2::Kind::integer().into_optional(),
        ),
        (
            "shadow-size".to_string(),
            ftd::ftd2021::p2::Kind::integer().into_optional(),
        ),
        (
            "shadow-color".to_string(),
            ftd::ftd2021::p2::Kind::record("ftd#color").into_optional(),
        ),
        (
            "background-image".to_string(),
            ftd::ftd2021::p2::Kind::record("ftd#image-src").into_optional(),
        ),
        (
            "background-repeat".to_string(),
            ftd::ftd2021::p2::Kind::boolean().into_optional(),
        ),
        (
            "background-parallax".to_string(),
            ftd::ftd2021::p2::Kind::boolean().into_optional(),
        ),
        (
            "scale".to_string(),
            ftd::ftd2021::p2::Kind::decimal().into_optional(),
        ),
        (
            "scale-x".to_string(),
            ftd::ftd2021::p2::Kind::decimal().into_optional(),
        ),
        (
            "scale-y".to_string(),
            ftd::ftd2021::p2::Kind::decimal().into_optional(),
        ),
        (
            "rotate".to_string(),
            ftd::ftd2021::p2::Kind::integer().into_optional(),
        ),
        (
            "move-up".to_string(),
            ftd::ftd2021::p2::Kind::integer().into_optional(),
        ),
        (
            "move-down".to_string(),
            ftd::ftd2021::p2::Kind::integer().into_optional(),
        ),
        (
            "move-left".to_string(),
            ftd::ftd2021::p2::Kind::integer().into_optional(),
        ),
        (
            "move-right".to_string(),
            ftd::ftd2021::p2::Kind::integer().into_optional(),
        ),
        (
            "position".to_string(),
            ftd::ftd2021::p2::Kind::string().into_optional(),
        ),
        (
            "z-index".to_string(),
            ftd::ftd2021::p2::Kind::integer().into_optional(),
        ),
        (
            "slot".to_string(),
            ftd::ftd2021::p2::Kind::string().into_optional(),
        ),
        (
            "white-space".to_string(),
            ftd::ftd2021::p2::Kind::string().into_optional(),
        ),
        (
            "border-style".to_string(),
            ftd::ftd2021::p2::Kind::string().into_optional(),
        ),
        (
            "text-transform".to_string(),
            ftd::ftd2021::p2::Kind::string().into_optional(),
        ),
        (
            "heading-number".to_string(),
            ftd::ftd2021::p2::Kind::list(ftd::ftd2021::p2::Kind::string()).into_optional(),
        ),
        /*(
            "grid-column".to_string(),
            ftd::p2::Kind::string().into_optional(),
        ),
        (
            "grid-row".to_string(),
            ftd::p2::Kind::string().into_optional(),
        ),*/
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
    properties: &ftd::Map<ftd::Value>,
    doc: &ftd::ftd2021::p2::TDoc,
) -> ftd::ftd2021::p1::Result<ftd::Container> {
    Ok(ftd::Container {
        children: Default::default(),
        external_children: Default::default(),
        open: ftd::ftd2021::p2::utils::bool_optional("open", properties, doc.name, 0)?,
        append_at: ftd::ftd2021::p2::utils::string_optional("append-at", properties, doc.name, 0)?,
        wrap: ftd::ftd2021::p2::utils::bool_with_default("wrap", false, properties, doc.name, 0)?,
    })
}

fn container_arguments() -> Vec<(String, ftd::ftd2021::p2::Kind)> {
    vec![
        (
            "open".to_string(),
            ftd::ftd2021::p2::Kind::boolean().into_optional(),
        ),
        (
            "append-at".to_string(),
            ftd::ftd2021::p2::Kind::string().into_optional(),
        ),
        (
            "align".to_string(),
            ftd::ftd2021::p2::Kind::string().into_optional(),
        ),
        (
            "wrap".to_string(),
            ftd::ftd2021::p2::Kind::boolean().into_optional(),
        ),
    ]
}

pub fn image_function() -> ftd::Component {
    ftd::Component {
        kernel: true,
        full_name: "ftd#image".to_string(),
        root: "ftd.kernel".to_string(),
        arguments: [
            vec![
                (
                    "src".to_string(),
                    ftd::ftd2021::p2::Kind::record("ftd#image-src"),
                ),
                (
                    "description".to_string(),
                    ftd::ftd2021::p2::Kind::string().into_optional(),
                ),
                (
                    "title".to_string(),
                    ftd::ftd2021::p2::Kind::string().into_optional(),
                ),
                (
                    "align".to_string(),
                    ftd::ftd2021::p2::Kind::string().into_optional(),
                ),
                (
                    "crop".to_string(),
                    ftd::ftd2021::p2::Kind::boolean().into_optional(),
                ),
                (
                    "loading".to_string(),
                    ftd::ftd2021::p2::Kind::string().into_optional(),
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
    unresolved_properties: &ftd::Map<ftd::ftd2021::component::Property>,
    doc: &ftd::ftd2021::p2::TDoc,
    condition: &Option<ftd::ftd2021::p2::Boolean>,
    is_child: bool,
    events: &[ftd::ftd2021::p2::Event],
) -> ftd::ftd2021::p1::Result<ftd::Image> {
    let (src, reference) =
        ftd::ftd2021::p2::utils::record_and_ref(0, "src", unresolved_properties, doc, condition)?;
    let src_record = ftd::ImageSrc::from(&src, doc, 0, reference.clone())?;
    let properties = &ftd::ftd2021::component::resolve_properties(0, unresolved_properties, doc)?;
    Ok(ftd::Image {
        src: src_record,
        description: ftd::ftd2021::p2::utils::string_optional(
            "description",
            properties,
            doc.name,
            0,
        )?,
        common: Box::new(common_from_properties(
            unresolved_properties,
            doc,
            condition,
            is_child,
            events,
            reference,
        )?),
        loading: ftd::Loading::from(
            ftd::ftd2021::p2::utils::string_with_default(
                "loading", "lazy", properties, doc.name, 0,
            )?
            .as_str(),
            doc.name,
        )?,
        crop: ftd::ftd2021::p2::utils::bool_with_default("crop", false, properties, doc.name, 0)?,
    })
}

pub fn row_function() -> ftd::Component {
    ftd::Component {
        kernel: true,
        full_name: "ftd#row".to_string(),
        root: "ftd.kernel".to_string(),
        arguments: [
            container_arguments(),
            common_arguments(),
            vec![(
                "spacing".to_string(),
                ftd::ftd2021::p2::Kind::string().into_optional(),
            )],
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

pub fn row_from_properties(
    unresolved_properties: &ftd::Map<ftd::ftd2021::component::Property>,
    doc: &ftd::ftd2021::p2::TDoc,
    condition: &Option<ftd::ftd2021::p2::Boolean>,
    is_child: bool,
    events: &[ftd::ftd2021::p2::Event],
) -> ftd::ftd2021::p1::Result<ftd::Row> {
    let properties = &ftd::ftd2021::component::resolve_properties(0, unresolved_properties, doc)?;
    Ok(ftd::Row {
        common: Box::new(common_from_properties(
            unresolved_properties,
            doc,
            condition,
            is_child,
            events,
            None,
        )?),
        container: container_from_properties(properties, doc)?,
        spacing: ftd::Spacing::from(ftd::ftd2021::p2::utils::string_optional(
            "spacing", properties, doc.name, 0,
        )?)?,
    })
}

pub fn column_function() -> ftd::Component {
    ftd::Component {
        line_number: 0,
        kernel: true,
        full_name: "ftd#column".to_string(),
        root: "ftd.kernel".to_string(),
        arguments: [
            container_arguments(),
            common_arguments(),
            vec![(
                "spacing".to_string(),
                ftd::ftd2021::p2::Kind::string().into_optional(),
            )],
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

pub fn column_from_properties(
    unresolved_properties: &ftd::Map<ftd::ftd2021::component::Property>,
    doc: &ftd::ftd2021::p2::TDoc,
    condition: &Option<ftd::ftd2021::p2::Boolean>,
    is_child: bool,
    events: &[ftd::ftd2021::p2::Event],
) -> ftd::ftd2021::p1::Result<ftd::Column> {
    let properties = &ftd::ftd2021::component::resolve_properties(0, unresolved_properties, doc)?;
    Ok(ftd::Column {
        common: Box::new(common_from_properties(
            unresolved_properties,
            doc,
            condition,
            is_child,
            events,
            None,
        )?),
        container: container_from_properties(properties, doc)?,
        spacing: ftd::Spacing::from(ftd::ftd2021::p2::utils::string_optional(
            "spacing", properties, doc.name, 0,
        )?)?,
    })
}

#[allow(dead_code)]
#[allow(unused_variables)]
pub fn text_render(
    tf: &ftd::TextFormat,
    text: String,
    source: ftd::TextSource,
    theme: String,
    doc_id: &str,
) -> ftd::ftd2021::p1::Result<ftd::ftd2021::Rendered> {
    Ok(match (source, tf) {
        (ftd::TextSource::Body, ftd::TextFormat::Markdown) => {
            ftd::ftd2021::rendered::markup(text.as_str())
        }
        (_, ftd::TextFormat::Markdown) => ftd::ftd2021::rendered::markup_line(text.as_str()),
        (_, ftd::TextFormat::Code { lang }) => ftd::ftd2021::rendered::code_with_theme(
            text.as_str(),
            lang.as_str(),
            theme.as_str(),
            doc_id,
        )?,
        (_, ftd::TextFormat::Text) => ftd::ftd2021::Rendered {
            original: text.clone(),
            rendered: text,
        },
    })
}

pub fn iframe_function() -> ftd::Component {
    ftd::Component {
        line_number: 0,
        kernel: true,
        root: "ftd.kernel".to_string(),
        full_name: "ftd#iframe".to_string(),
        arguments: [
            vec![
                (
                    "src".to_string(),
                    ftd::ftd2021::p2::Kind::string().into_optional(),
                ),
                (
                    "youtube".to_string(),
                    ftd::ftd2021::p2::Kind::string().into_optional(),
                ),
                (
                    "loading".to_string(),
                    ftd::ftd2021::p2::Kind::string().into_optional(),
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
    unresolved_properties: &ftd::Map<ftd::ftd2021::component::Property>,
    doc: &ftd::ftd2021::p2::TDoc,
    condition: &Option<ftd::ftd2021::p2::Boolean>,
    is_child: bool,
    events: &[ftd::ftd2021::p2::Event],
) -> ftd::ftd2021::p1::Result<ftd::IFrame> {
    let src = match (
        ftd::ftd2021::p2::utils::string_optional_with_ref("src", unresolved_properties, doc, 0)?,
        {
            let (youtube, reference) = ftd::ftd2021::p2::utils::string_optional_with_ref(
                "youtube",
                unresolved_properties,
                doc,
                0,
            )?;
            (
                youtube.and_then(|id| ftd::ftd2021::youtube_id::from_raw(id.as_str())),
                reference,
            )
        },
    ) {
        ((Some(src), _), (None, _)) => src,
        ((None, _), (Some(id), _)) => id,
        ((Some(_), _), (Some(_), _)) => {
            return ftd::ftd2021::p2::utils::e2("both src and youtube id provided", doc.name, 0);
        }
        ((None, Some(reference)), (None, None)) | ((None, None), (None, Some(reference))) => {
            if let Some(ftd::ftd2021::p2::Boolean::IsNotNull { value }) = condition {
                match value {
                    ftd::PropertyValue::Reference { name, .. }
                    | ftd::PropertyValue::Variable { name, .. }
                        if name.eq(&reference) =>
                    {
                        "".to_string()
                    }
                    _ => {
                        return ftd::ftd2021::p2::utils::e2(
                            "src or youtube id is required",
                            doc.name,
                            0,
                        );
                    }
                }
            } else {
                return ftd::ftd2021::p2::utils::e2("src or youtube id is required", doc.name, 0);
            }
        }
        (_, _) => return ftd::ftd2021::p2::utils::e2("src or youtube id is required", doc.name, 0),
    };

    let properties = &ftd::ftd2021::component::resolve_properties(0, unresolved_properties, doc)?;

    Ok(ftd::IFrame {
        src,
        loading: ftd::Loading::from(
            ftd::ftd2021::p2::utils::string_with_default(
                "loading", "lazy", properties, doc.name, 0,
            )?
            .as_str(),
            doc.name,
        )?,
        common: Box::new(common_from_properties(
            unresolved_properties,
            doc,
            condition,
            is_child,
            events,
            None,
        )?),
    })
}

pub fn text_block_from_properties(
    unresolved_properties: &ftd::Map<ftd::ftd2021::component::Property>,
    doc: &ftd::ftd2021::p2::TDoc,
    condition: &Option<ftd::ftd2021::p2::Boolean>,
    is_child: bool,
    events: &[ftd::ftd2021::p2::Event],
) -> ftd::ftd2021::p1::Result<ftd::TextBlock> {
    let (text, source, reference) = ftd::ftd2021::p2::utils::string_and_source_and_ref(
        0,
        "text",
        unresolved_properties,
        doc,
        condition,
    )?;
    let properties = &ftd::ftd2021::component::resolve_properties(0, unresolved_properties, doc)?;
    let font_str = ftd::ftd2021::p2::utils::string_optional("role", properties, doc.name, 0)?;

    let font: Vec<ftd::NamedFont> = match font_str {
        Some(f) => f
            .split(',')
            .flat_map(|x| ftd::NamedFont::from(Some(x.to_string())))
            .collect(),
        None => vec![],
    };
    Ok(ftd::TextBlock {
        line: source != ftd::TextSource::Body,
        text: if source == ftd::TextSource::Body {
            ftd::ftd2021::rendered::markup(text.as_str())
        } else {
            ftd::ftd2021::rendered::markup_line(text.as_str())
        },
        common: Box::new(common_from_properties(
            unresolved_properties,
            doc,
            condition,
            is_child,
            events,
            reference,
        )?),
        text_align: ftd::TextAlign::from(
            ftd::ftd2021::p2::utils::string_optional("text-align", properties, doc.name, 0)?,
            doc.name,
        )?,
        style: ftd::Style::from(
            ftd::ftd2021::p2::utils::string_optional("style", properties, doc.name, 0)?,
            doc.name,
        )?,
        size: ftd::ftd2021::p2::utils::int_optional("size", properties, doc.name, 0)?,
        font,
        line_height: ftd::ftd2021::p2::utils::int_optional("line-height", properties, doc.name, 0)?,
        line_clamp: ftd::ftd2021::p2::utils::int_optional("line-clamp", properties, doc.name, 0)?,
        text_indent: ftd::Length::from(
            ftd::ftd2021::p2::utils::string_optional("text-indent", properties, doc.name, 0)?,
            doc.name,
        )?,
    })
}

pub fn code_from_properties(
    unresolved_properties: &ftd::Map<ftd::ftd2021::component::Property>,
    doc: &ftd::ftd2021::p2::TDoc,
    condition: &Option<ftd::ftd2021::p2::Boolean>,
    is_child: bool,
    events: &[ftd::ftd2021::p2::Event],
) -> ftd::ftd2021::p1::Result<ftd::Code> {
    let (text, _, reference) = ftd::ftd2021::p2::utils::string_and_source_and_ref(
        0,
        "text",
        unresolved_properties,
        doc,
        condition,
    )?;
    let properties = &ftd::ftd2021::component::resolve_properties(0, unresolved_properties, doc)?;
    let font_str = ftd::ftd2021::p2::utils::record_optional("role", properties, doc.name, 0)?;
    let mut font_reference = None;
    if font_str.is_some() {
        font_reference = ftd::ftd2021::p2::utils::record_and_ref(
            0,
            "role",
            unresolved_properties,
            doc,
            condition,
        )?
        .1;
    }
    let font = font_str.map_or(Ok(None), |v| {
        ftd::Type::from(&v, doc, 0, font_reference).map(Some)
    })?;

    Ok(ftd::Code {
        text: ftd::ftd2021::rendered::code_with_theme(
            text.as_str(),
            ftd::ftd2021::p2::utils::string_optional("lang", properties, doc.name, 0)?
                .unwrap_or_else(|| "txt".to_string())
                .as_str(),
            ftd::ftd2021::p2::utils::string_with_default(
                "theme",
                ftd::ftd2021::code::DEFAULT_THEME,
                properties,
                doc.name,
                0,
            )?
            .as_str(),
            doc.name,
        )?,
        common: Box::new(common_from_properties(
            unresolved_properties,
            doc,
            condition,
            is_child,
            events,
            reference,
        )?),
        text_align: ftd::TextAlign::from(
            ftd::ftd2021::p2::utils::string_optional("text-align", properties, doc.name, 0)?,
            doc.name,
        )?,
        style: ftd::Style::from(
            ftd::ftd2021::p2::utils::string_optional("style", properties, doc.name, 0)?,
            doc.name,
        )?,
        font,
        line_clamp: ftd::ftd2021::p2::utils::int_optional("line-clamp", properties, doc.name, 0)?,
        text_indent: ftd::Length::from(
            ftd::ftd2021::p2::utils::string_optional("text-indent", properties, doc.name, 0)?,
            doc.name,
        )?,
    })
}

pub fn integer_from_properties(
    unresolved_properties: &ftd::Map<ftd::ftd2021::component::Property>,
    doc: &ftd::ftd2021::p2::TDoc,
    condition: &Option<ftd::ftd2021::p2::Boolean>,
    is_child: bool,
    events: &[ftd::ftd2021::p2::Event],
) -> ftd::ftd2021::p1::Result<ftd::Text> {
    let reference = ftd::ftd2021::p2::utils::integer_and_ref(
        0,
        "value",
        unresolved_properties,
        doc,
        condition,
    )?
    .1;
    let properties = &ftd::ftd2021::component::resolve_properties(0, unresolved_properties, doc)?;
    let num = format_num::NumberFormat::new();
    let text = match ftd::ftd2021::p2::utils::string_optional("format", properties, doc.name, 0)? {
        Some(f) => num.format(
            f.as_str(),
            ftd::ftd2021::p2::utils::int("value", properties, doc.name, 0)? as f64,
        ),
        None => ftd::ftd2021::p2::utils::int("value", properties, doc.name, 0)?.to_string(),
    };

    let font_str = ftd::ftd2021::p2::utils::record_optional("role", properties, doc.name, 0)?;
    let mut font_reference = None;
    if font_str.is_some() {
        font_reference = ftd::ftd2021::p2::utils::record_and_ref(
            0,
            "role",
            unresolved_properties,
            doc,
            condition,
        )?
        .1;
    }
    let font = font_str.map_or(Ok(None), |v| {
        ftd::Type::from(&v, doc, 0, font_reference).map(Some)
    })?;

    Ok(ftd::Text {
        text: ftd::ftd2021::rendered::markup_line(text.as_str()),
        line: false,
        common: Box::new(common_from_properties(
            unresolved_properties,
            doc,
            condition,
            is_child,
            events,
            reference,
        )?),
        text_align: ftd::TextAlign::from(
            ftd::ftd2021::p2::utils::string_optional("text-align", properties, doc.name, 0)?,
            doc.name,
        )?,
        style: ftd::Style::from(
            ftd::ftd2021::p2::utils::string_optional("style", properties, doc.name, 0)?,
            doc.name,
        )?,
        font,
        line_clamp: ftd::ftd2021::p2::utils::int_optional("line-clamp", properties, doc.name, 0)?,
        text_indent: ftd::Length::from(
            ftd::ftd2021::p2::utils::string_optional("text-indent", properties, doc.name, 0)?,
            doc.name,
        )?,
    })
}

pub fn decimal_from_properties(
    unresolved_properties: &ftd::Map<ftd::ftd2021::component::Property>,
    doc: &ftd::ftd2021::p2::TDoc,
    condition: &Option<ftd::ftd2021::p2::Boolean>,
    is_child: bool,
    events: &[ftd::ftd2021::p2::Event],
) -> ftd::ftd2021::p1::Result<ftd::Text> {
    let reference = ftd::ftd2021::p2::utils::decimal_and_ref(
        0,
        "value",
        unresolved_properties,
        doc,
        condition,
    )?
    .1;
    let properties = &ftd::ftd2021::component::resolve_properties(0, unresolved_properties, doc)?;
    let num = format_num::NumberFormat::new();
    let text = match ftd::ftd2021::p2::utils::string_optional("format", properties, doc.name, 0)? {
        Some(f) => num.format(
            f.as_str(),
            ftd::ftd2021::p2::utils::decimal("value", properties, doc.name, 0)?,
        ),
        None => ftd::ftd2021::p2::utils::decimal("value", properties, doc.name, 0)?.to_string(),
    };

    let font_str = ftd::ftd2021::p2::utils::record_optional("role", properties, doc.name, 0)?;
    let mut font_reference = None;
    if font_str.is_some() {
        font_reference = ftd::ftd2021::p2::utils::record_and_ref(
            0,
            "role",
            unresolved_properties,
            doc,
            condition,
        )?
        .1;
    }
    let font = font_str.map_or(Ok(None), |v| {
        ftd::Type::from(&v, doc, 0, font_reference).map(Some)
    })?;
    Ok(ftd::Text {
        text: ftd::ftd2021::rendered::markup_line(text.as_str()),
        line: false,
        common: Box::new(common_from_properties(
            unresolved_properties,
            doc,
            condition,
            is_child,
            events,
            reference,
        )?),
        text_align: ftd::TextAlign::from(
            ftd::ftd2021::p2::utils::string_optional("text-align", properties, doc.name, 0)?,
            doc.name,
        )?,
        style: ftd::Style::from(
            ftd::ftd2021::p2::utils::string_optional("style", properties, doc.name, 0)?,
            doc.name,
        )?,
        font,
        line_clamp: ftd::ftd2021::p2::utils::int_optional("line-clamp", properties, doc.name, 0)?,
        text_indent: ftd::Length::from(
            ftd::ftd2021::p2::utils::string_optional("text-indent", properties, doc.name, 0)?,
            doc.name,
        )?,
    })
}

pub fn color_from(
    l: Option<String>,
    doc_id: &str,
) -> ftd::ftd2021::p1::Result<Option<ftd::ColorValue>> {
    use std::str::FromStr;

    let v = match l {
        Some(v) => v,
        None => return Ok(None),
    };

    let v = v.trim().to_string();

    // Remove all whitespace, not compliant, but should just be more accepting.
    let mut string = v.replace(' ', "");
    string.make_ascii_lowercase();
    if v.starts_with('#') && v.len() == 9 {
        let (_, value_string) = string.split_at(1);

        let iv = u64::from_str_radix(value_string, 16).map_err(|e| {
            ftd::ftd2021::p1::Error::ParseError {
                message: e.to_string(),
                doc_id: doc_id.to_string(),
                line_number: 0,
            }
        })?;

        // (7thSigil) unlike original js code, NaN is impossible
        if iv > 0xffffffff {
            return ftd::ftd2021::p2::utils::e2(format!("{v} is not a valid color"), doc_id, 0);
        }

        //Code for accepting 6-digit hexa-color code
        Ok(Some(ftd::ColorValue {
            r: ((iv & 0xff000000) >> 24) as u8,
            g: ((iv & 0xff0000) >> 16) as u8,
            b: ((iv & 0xff00) >> 8) as u8,
            alpha: round_1p((iv & 0xff) as f32 / 255_f32),
        }))
    } else {
        match css_color_parser::Color::from_str(v.as_str()) {
            Ok(v) => Ok(Some(ftd::ColorValue {
                r: v.r,
                g: v.g,
                b: v.b,
                alpha: v.a,
            })),
            Err(e) => {
                ftd::ftd2021::p2::utils::e2(format!("{v} is not a valid color: {e:?}"), doc_id, 0)
            }
        }
    }
}
fn round_1p(n: f32) -> f32 {
    // 1234.56
    let temp = (n * 10_f32) as u32;
    let last = (temp % 10) as f32;
    let front = n as u32;

    front as f32 + last / 10_f32
}

pub fn boolean_from_properties(
    unresolved_properties: &ftd::Map<ftd::ftd2021::component::Property>,
    doc: &ftd::ftd2021::p2::TDoc,
    condition: &Option<ftd::ftd2021::p2::Boolean>,
    is_child: bool,
    events: &[ftd::ftd2021::p2::Event],
) -> ftd::ftd2021::p1::Result<ftd::Text> {
    let reference = ftd::ftd2021::p2::utils::boolean_and_ref(
        0,
        "value",
        unresolved_properties,
        doc,
        condition,
    )?
    .1;
    let properties = &ftd::ftd2021::component::resolve_properties(0, unresolved_properties, doc)?;
    let value = ftd::ftd2021::p2::utils::bool_("value", properties, doc.name, 0)?;
    let text = if value {
        ftd::ftd2021::p2::utils::string_with_default("true", "true", properties, doc.name, 0)?
    } else {
        ftd::ftd2021::p2::utils::string_with_default("false", "false", properties, doc.name, 0)?
    };

    let font_str = ftd::ftd2021::p2::utils::record_optional("role", properties, doc.name, 0)?;
    let mut font_reference = None;
    if font_str.is_some() {
        font_reference = ftd::ftd2021::p2::utils::record_and_ref(
            0,
            "role",
            unresolved_properties,
            doc,
            condition,
        )?
        .1;
    }
    let font = font_str.map_or(Ok(None), |v| {
        ftd::Type::from(&v, doc, 0, font_reference).map(Some)
    })?;

    Ok(ftd::Text {
        text: ftd::ftd2021::rendered::markup_line(text.as_str()),
        line: false,
        common: Box::new(common_from_properties(
            unresolved_properties,
            doc,
            condition,
            is_child,
            events,
            reference,
        )?),
        text_align: ftd::TextAlign::from(
            ftd::ftd2021::p2::utils::string_optional("text-align", properties, doc.name, 0)?,
            doc.name,
        )?,
        style: ftd::Style::from(
            ftd::ftd2021::p2::utils::string_optional("style", properties, doc.name, 0)?,
            doc.name,
        )?,
        font,
        line_clamp: ftd::ftd2021::p2::utils::int_optional("line-clamp", properties, doc.name, 0)?,
        text_indent: ftd::Length::from(
            ftd::ftd2021::p2::utils::string_optional("text-indent", properties, doc.name, 0)?,
            doc.name,
        )?,
    })
}

pub fn text_function() -> ftd::Component {
    ftd::Component {
        line_number: 0,
        kernel: true,
        root: "ftd.kernel".to_string(),
        full_name: "ftd#text-block".to_string(),
        arguments: [
            vec![
                (
                    "text".to_string(),
                    ftd::ftd2021::p2::Kind::caption_or_body(),
                ),
                (
                    "align".to_string(),
                    ftd::ftd2021::p2::Kind::string().into_optional(),
                ),
                (
                    "style".to_string(),
                    ftd::ftd2021::p2::Kind::string().into_optional(),
                ),
                (
                    "role".to_string(),
                    ftd::ftd2021::p2::Kind::record("ftd#type").into_optional(),
                ),
                (
                    "line-clamp".to_string(),
                    ftd::ftd2021::p2::Kind::integer().into_optional(),
                ),
                (
                    "text-indent".to_string(),
                    ftd::ftd2021::p2::Kind::integer().into_optional(),
                ),
                (
                    "text-align".to_string(),
                    ftd::ftd2021::p2::Kind::string().into_optional(),
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

pub fn code_function() -> ftd::Component {
    ftd::Component {
        line_number: 0,
        kernel: true,
        root: "ftd.kernel".to_string(),
        full_name: "ftd#code".to_string(),
        arguments: [
            vec![
                (
                    "text".to_string(),
                    ftd::ftd2021::p2::Kind::caption_or_body(),
                ),
                (
                    "align".to_string(),
                    ftd::ftd2021::p2::Kind::string().into_optional(),
                ),
                (
                    "style".to_string(),
                    ftd::ftd2021::p2::Kind::string().into_optional(),
                ),
                (
                    "lang".to_string(),
                    ftd::ftd2021::p2::Kind::string().into_optional(),
                ),
                (
                    "theme".to_string(),
                    ftd::ftd2021::p2::Kind::string().into_optional(),
                ),
                (
                    "role".to_string(),
                    ftd::ftd2021::p2::Kind::record("ftd#type").into_optional(),
                ),
                (
                    "line-clamp".to_string(),
                    ftd::ftd2021::p2::Kind::integer().into_optional(),
                ),
                (
                    "text-indent".to_string(),
                    ftd::ftd2021::p2::Kind::string().into_optional(),
                ),
                (
                    "text-align".to_string(),
                    ftd::ftd2021::p2::Kind::string().into_optional(),
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

pub fn integer_function() -> ftd::Component {
    ftd::Component {
        line_number: 0,
        kernel: true,
        root: "ftd.kernel".to_string(),
        full_name: "ftd#integer".to_string(),
        arguments: [
            vec![
                ("value".to_string(), ftd::ftd2021::p2::Kind::integer()),
                (
                    "align".to_string(),
                    ftd::ftd2021::p2::Kind::string().into_optional(),
                ),
                (
                    "style".to_string(),
                    ftd::ftd2021::p2::Kind::string().into_optional(),
                ),
                (
                    "format".to_string(),
                    ftd::ftd2021::p2::Kind::string().into_optional(),
                ),
                (
                    "role".to_string(),
                    ftd::ftd2021::p2::Kind::record("ftd#type").into_optional(),
                ),
                (
                    "text-align".to_string(),
                    ftd::ftd2021::p2::Kind::string().into_optional(),
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

pub fn decimal_function() -> ftd::Component {
    ftd::Component {
        line_number: 0,
        kernel: true,
        root: "ftd.kernel".to_string(),
        full_name: "ftd#decimal".to_string(),
        arguments: [
            vec![
                ("value".to_string(), ftd::ftd2021::p2::Kind::decimal()),
                (
                    "align".to_string(),
                    ftd::ftd2021::p2::Kind::string().into_optional(),
                ),
                (
                    "style".to_string(),
                    ftd::ftd2021::p2::Kind::string().into_optional(),
                ),
                (
                    "format".to_string(),
                    ftd::ftd2021::p2::Kind::string().into_optional(),
                ),
                (
                    "role".to_string(),
                    ftd::ftd2021::p2::Kind::record("ftd#type").into_optional(),
                ),
                (
                    "text-align".to_string(),
                    ftd::ftd2021::p2::Kind::string().into_optional(),
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

pub fn scene_function() -> ftd::Component {
    let arguments = {
        let mut arguments: ftd::Map<ftd::ftd2021::p2::Kind> = [
            container_arguments(),
            common_arguments(),
            vec![(
                "spacing".to_string(),
                ftd::ftd2021::p2::Kind::string().into_optional(),
            )],
        ]
        .concat()
        .into_iter()
        .collect();
        arguments.remove("spacing");
        arguments.remove("wrap");
        arguments
    };

    ftd::Component {
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

pub fn markup_function() -> ftd::Component {
    ftd::Component {
        line_number: 0,
        kernel: true,
        root: "ftd.kernel".to_string(),
        full_name: "ftd#text".to_string(),
        arguments: [
            vec![
                (
                    "text".to_string(),
                    ftd::ftd2021::p2::Kind::caption_or_body(),
                ),
                (
                    "align".to_string(),
                    ftd::ftd2021::p2::Kind::string().into_optional(),
                ),
                (
                    "style".to_string(),
                    ftd::ftd2021::p2::Kind::string().into_optional(),
                ),
                (
                    "role".to_string(),
                    ftd::ftd2021::p2::Kind::record("ftd#type").into_optional(),
                ),
                (
                    "line-clamp".to_string(),
                    ftd::ftd2021::p2::Kind::integer().into_optional(),
                ),
                (
                    "text-indent".to_string(),
                    ftd::ftd2021::p2::Kind::string().into_optional(),
                ),
                (
                    "text-align".to_string(),
                    ftd::ftd2021::p2::Kind::string().into_optional(),
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

pub fn grid_function() -> ftd::Component {
    let arguments: ftd::Map<ftd::ftd2021::p2::Kind> = [
        container_arguments(),
        common_arguments(),
        vec![
            ("slots".to_string(), ftd::ftd2021::p2::Kind::string()),
            (
                "slot-widths".to_string(),
                ftd::ftd2021::p2::Kind::string().into_optional(),
            ),
            (
                "slot-heights".to_string(),
                ftd::ftd2021::p2::Kind::string().into_optional(),
            ),
            (
                "spacing".to_string(),
                ftd::ftd2021::p2::Kind::integer().into_optional(),
            ),
            (
                "inline".to_string(),
                ftd::ftd2021::p2::Kind::boolean().into_optional(),
            ),
        ],
    ]
    .concat()
    .into_iter()
    .collect();

    ftd::Component {
        line_number: 0,
        kernel: true,
        root: "ftd.kernel".to_string(),
        full_name: "ftd#grid".to_string(),
        arguments,
        locals: Default::default(),
        properties: Default::default(),
        instructions: Default::default(),
        invocations: Default::default(),
        condition: None,
        events: vec![],
    }
}

pub fn boolean_function() -> ftd::Component {
    ftd::Component {
        line_number: 0,
        kernel: true,
        root: "ftd.kernel".to_string(),
        full_name: "ftd#boolean".to_string(),
        arguments: [
            vec![
                ("value".to_string(), ftd::ftd2021::p2::Kind::boolean()),
                (
                    "align".to_string(),
                    ftd::ftd2021::p2::Kind::string().into_optional(),
                ),
                (
                    "style".to_string(),
                    ftd::ftd2021::p2::Kind::string().into_optional(),
                ),
                (
                    "format".to_string(),
                    ftd::ftd2021::p2::Kind::string().into_optional(),
                ),
                (
                    "role".to_string(),
                    ftd::ftd2021::p2::Kind::record("ftd#type").into_optional(),
                ),
                (
                    "true".to_string(),
                    ftd::ftd2021::p2::Kind::string().into_optional(),
                ),
                (
                    "false".to_string(),
                    ftd::ftd2021::p2::Kind::string().into_optional(),
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

pub fn input_function() -> ftd::Component {
    ftd::Component {
        line_number: 0,
        kernel: true,
        root: "ftd.kernel".to_string(),
        full_name: "ftd#input".to_string(),
        arguments: [
            vec![
                (
                    "placeholder".to_string(),
                    ftd::ftd2021::p2::Kind::string().into_optional(),
                ),
                (
                    "value".to_string(),
                    ftd::ftd2021::p2::Kind::string().into_optional(),
                ),
                (
                    "default-value".to_string(),
                    ftd::ftd2021::p2::Kind::string().into_optional(),
                ),
                (
                    "multiline".to_string(),
                    ftd::ftd2021::p2::Kind::boolean().set_default(Some("false".to_string())),
                ),
                (
                    "role".to_string(),
                    ftd::ftd2021::p2::Kind::record("ftd#type").into_optional(),
                ),
                (
                    "type".to_string(),
                    ftd::ftd2021::p2::Kind::string().into_optional(),
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

pub fn input_from_properties(
    unresolved_properties: &ftd::Map<ftd::ftd2021::component::Property>,
    doc: &ftd::ftd2021::p2::TDoc,
    condition: &Option<ftd::ftd2021::p2::Boolean>,
    is_child: bool,
    events: &[ftd::ftd2021::p2::Event],
) -> ftd::ftd2021::p1::Result<ftd::Input> {
    let reference = ftd::ftd2021::p2::utils::string_and_source_and_ref(
        0,
        "value",
        unresolved_properties,
        doc,
        condition,
    )
    .map(|v| v.2)
    .unwrap_or(None);

    let properties = &ftd::ftd2021::component::resolve_properties(0, unresolved_properties, doc)?;
    let font_str = ftd::ftd2021::p2::utils::record_optional("role", properties, doc.name, 0)?;
    let mut font_reference = None;
    if font_str.is_some() {
        font_reference = ftd::ftd2021::p2::utils::record_and_ref(
            0,
            "role",
            unresolved_properties,
            doc,
            condition,
        )?
        .1;
    }
    let font = font_str.map_or(Ok(None), |v| {
        ftd::Type::from(&v, doc, 0, font_reference).map(Some)
    })?;

    Ok(ftd::Input {
        common: Box::new(common_from_properties(
            unresolved_properties,
            doc,
            condition,
            is_child,
            events,
            reference,
        )?),
        placeholder: ftd::ftd2021::p2::utils::string_optional(
            "placeholder",
            properties,
            doc.name,
            0,
        )?,
        multiline: ftd::ftd2021::p2::utils::bool_("multiline", properties, doc.name, 0)?,
        type_: ftd::ftd2021::p2::utils::string_optional("type", properties, doc.name, 0)?,
        value: ftd::ftd2021::p2::utils::string_optional("value", properties, doc.name, 0)?,
        default_value: ftd::ftd2021::p2::utils::string_optional(
            "default-value",
            properties,
            doc.name,
            0,
        )?,
        font,
    })
}

pub fn scene_from_properties(
    unresolved_properties: &ftd::Map<ftd::ftd2021::component::Property>,
    doc: &ftd::ftd2021::p2::TDoc,
    condition: &Option<ftd::ftd2021::p2::Boolean>,
    is_child: bool,
    events: &[ftd::ftd2021::p2::Event],
) -> ftd::ftd2021::p1::Result<ftd::Scene> {
    let properties = &ftd::ftd2021::component::resolve_properties(0, unresolved_properties, doc)?;
    Ok(ftd::Scene {
        common: Box::new(common_from_properties(
            unresolved_properties,
            doc,
            condition,
            is_child,
            events,
            None,
        )?),
        container: container_from_properties(properties, doc)?,
        spacing: ftd::Spacing::from(ftd::ftd2021::p2::utils::string_optional(
            "spacing", properties, doc.name, 0,
        )?)?,
    })
}

pub fn grid_from_properties(
    unresolved_properties: &ftd::Map<ftd::ftd2021::component::Property>,
    doc: &ftd::ftd2021::p2::TDoc,
    condition: &Option<ftd::ftd2021::p2::Boolean>,
    is_child: bool,
    events: &[ftd::ftd2021::p2::Event],
) -> ftd::ftd2021::p1::Result<ftd::Grid> {
    let properties = &ftd::ftd2021::component::resolve_properties(0, unresolved_properties, doc)?;
    Ok(ftd::Grid {
        slots: match ftd::ftd2021::p2::utils::string_optional("slots", properties, doc.name, 0)? {
            Some(val) => val,
            None => return ftd::ftd2021::p2::utils::e2("expected slots", doc.name, 0),
        },
        slot_widths: ftd::ftd2021::p2::utils::string_optional(
            "slot-widths",
            properties,
            doc.name,
            0,
        )?,
        slot_heights: ftd::ftd2021::p2::utils::string_optional(
            "slot-heights",
            properties,
            doc.name,
            0,
        )?,
        spacing: ftd::ftd2021::p2::utils::int_optional("spacing", properties, doc.name, 0)?,
        spacing_vertical: ftd::ftd2021::p2::utils::int_optional(
            "spacing-vertical",
            properties,
            doc.name,
            0,
        )?,
        spacing_horizontal: ftd::ftd2021::p2::utils::int_optional(
            "spacing-horizontal",
            properties,
            doc.name,
            0,
        )?,
        common: Box::new(common_from_properties(
            unresolved_properties,
            doc,
            condition,
            is_child,
            events,
            None,
        )?),
        container: container_from_properties(properties, doc)?,
        inline: ftd::ftd2021::p2::utils::bool_with_default(
            "inline", false, properties, doc.name, 0,
        )?,
        auto_flow: ftd::ftd2021::p2::utils::string_optional("auto-flow", properties, doc.name, 0)?,
    })
}

pub fn markup_from_properties(
    unresolved_properties: &ftd::Map<ftd::ftd2021::component::Property>,
    doc: &ftd::ftd2021::p2::TDoc,
    condition: &Option<ftd::ftd2021::p2::Boolean>,
    is_child: bool,
    events: &[ftd::ftd2021::p2::Event],
) -> ftd::ftd2021::p1::Result<ftd::Markups> {
    let (value, source, reference) = ftd::ftd2021::p2::utils::string_and_source_and_ref(
        0,
        "text",
        unresolved_properties,
        doc,
        condition,
    )?;
    let properties = &ftd::ftd2021::component::resolve_properties(0, unresolved_properties, doc)?;
    let font_str = ftd::ftd2021::p2::utils::record_optional("role", properties, doc.name, 0)?;
    let mut font_reference = None;
    if font_str.is_some() {
        font_reference = ftd::ftd2021::p2::utils::record_and_ref(
            0,
            "role",
            unresolved_properties,
            doc,
            condition,
        )?
        .1;
    }
    let font = font_str.map_or(Ok(None), |v| {
        ftd::Type::from(&v, doc, 0, font_reference).map(Some)
    })?;

    Ok(ftd::Markups {
        text: ftd::ftd2021::rendered::markup_line(value.as_str()),
        common: Box::new(common_from_properties(
            unresolved_properties,
            doc,
            condition,
            is_child,
            events,
            reference,
        )?),
        children: vec![],
        line: source != ftd::TextSource::Body,
        text_align: ftd::TextAlign::from(
            ftd::ftd2021::p2::utils::string_optional("text-align", properties, doc.name, 0)?,
            doc.name,
        )?,
        style: ftd::Style::from(
            ftd::ftd2021::p2::utils::string_optional("style", properties, doc.name, 0)?,
            doc.name,
        )?,
        font,
        line_clamp: ftd::ftd2021::p2::utils::int_optional("line-clamp", properties, doc.name, 0)?,
        text_indent: ftd::Length::from(
            ftd::ftd2021::p2::utils::string_optional("text-indent", properties, doc.name, 0)?,
            doc.name,
        )?,
    })
}
