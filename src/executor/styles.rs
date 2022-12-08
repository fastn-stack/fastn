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
        or_type_value: Option<(String, ftd::interpreter2::PropertyValue)>,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Option<Self>> {
        if let Some(value) = or_type_value {
            Ok(Some(Length::from_values(value, doc, line_number)?))
        } else {
            Ok(None)
        }
    }

    fn from_values(
        or_type_value: (String, ftd::interpreter2::PropertyValue),
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Self> {
        match or_type_value.0.as_str() {
            ftd::interpreter2::FTD_LENGTH_PERCENT => Ok(Length::Percent(
                or_type_value
                    .1
                    .clone()
                    .resolve(&doc.itdoc(), line_number)?
                    .record_fields(doc.name, line_number)?
                    .get(ftd::interpreter2::FTD_LENGTH_VALUE)
                    .unwrap()
                    .clone()
                    .resolve(&doc.itdoc(), line_number)?
                    .decimal(doc.name, line_number)?,
            )),
            ftd::interpreter2::FTD_LENGTH_PX => Ok(Length::Px(
                or_type_value
                    .1
                    .clone()
                    .resolve(&doc.itdoc(), line_number)?
                    .record_fields(doc.name, line_number)?
                    .get(ftd::interpreter2::FTD_LENGTH_VALUE)
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

    pub(crate) fn optional_length(
        properties: &[ftd::interpreter2::Property],
        arguments: &[ftd::interpreter2::Argument],
        doc: &ftd::executor::TDoc,
        line_number: usize,
        key: &str,
    ) -> ftd::executor::Result<ftd::executor::Value<Option<Length>>> {
        let or_type_value = ftd::executor::value::optional_or_type(
            key,
            properties,
            arguments,
            doc,
            line_number,
            ftd::interpreter2::FTD_LENGTH,
        )?;

        Ok(ftd::executor::Value::new(
            Length::from_optional_values(or_type_value.value, doc, line_number)?,
            or_type_value.line_number,
            or_type_value.properties,
        ))
    }

    pub(crate) fn length_with_default(
        properties: &[ftd::interpreter2::Property],
        arguments: &[ftd::interpreter2::Argument],
        doc: &ftd::executor::TDoc,
        line_number: usize,
        key: &str,
        default: Length,
    ) -> ftd::executor::Result<ftd::executor::Value<Length>> {
        let or_type_value = ftd::executor::value::optional_or_type(
            key,
            properties,
            arguments,
            doc,
            line_number,
            ftd::interpreter2::FTD_LENGTH,
        )?;

        Ok(ftd::executor::Value::new(
            Length::from_optional_values(or_type_value.value, doc, line_number)?.unwrap_or(default),
            or_type_value.line_number,
            or_type_value.properties,
        ))
    }

    pub fn to_css_string(&self) -> String {
        match self {
            Length::Px(px) => format!("{}px", px),
            Length::Percent(p) => format!("{}%", p),
        }
    }

    pub fn get_pattern_from_variant_str(
        variant: &str,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::executor::Result<&'static str> {
        match variant {
            ftd::interpreter2::FTD_LENGTH_PX | ftd::interpreter2::FTD_LENGTH_PERCENT => {
                Ok("({0}).value")
            }
            t => ftd::executor::utils::parse_error(
                format!("Unknown variant found for ftd.length: `{}`", t),
                doc_id,
                line_number,
            ),
        }
    }

    pub fn set_pattern_from_variant_str(
        variant: &str,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::executor::Result<&'static str> {
        match variant {
            ftd::interpreter2::FTD_LENGTH_PX => Ok("{0}px"),
            ftd::interpreter2::FTD_LENGTH_PERCENT => Ok("{0}%"),
            t => ftd::executor::utils::parse_error(
                format!("Unknown variant found for ftd.length: `{}`", t),
                doc_id,
                line_number,
            ),
        }
    }

    pub fn set_value_from_variant(
        variant: &str,
        value: &str,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::executor::Result<String> {
        match variant {
            ftd::interpreter2::FTD_LENGTH_PX => Ok(format!("{}px", value)),
            ftd::interpreter2::FTD_LENGTH_PERCENT => Ok(format!("{}%", value)),
            t => ftd::executor::utils::parse_error(
                format!("Unknown variant found for ftd.length: `{}`", t),
                doc_id,
                line_number,
            ),
        }
    }
}

#[derive(serde::Deserialize, Debug, PartialEq, Clone, serde::Serialize)]
pub enum Alignment {
    TopLeft,
    TopCenter,
    TopRight,
    Left,
    Center,
    Right,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

impl Default for Alignment {
    fn default() -> Alignment {
        Alignment::TopLeft
    }
}

impl Alignment {
    fn from_optional_values(
        or_type_value: Option<(String, ftd::interpreter2::PropertyValue)>,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Option<Self>> {
        if let Some(value) = or_type_value {
            Ok(Some(Alignment::from_values(value, doc, line_number)?))
        } else {
            Ok(None)
        }
    }

    fn from_values(
        or_type_value: (String, ftd::interpreter2::PropertyValue),
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Self> {
        match or_type_value.0.as_str() {
            ftd::interpreter2::FTD_ALIGN_TOP_LEFT => Ok(Alignment::TopLeft),
            ftd::interpreter2::FTD_ALIGN_TOP_CENTER => Ok(Alignment::TopCenter),
            ftd::interpreter2::FTD_ALIGN_TOP_RIGHT => Ok(Alignment::TopRight),
            ftd::interpreter2::FTD_ALIGN_LEFT => Ok(Alignment::Left),
            ftd::interpreter2::FTD_ALIGN_CENTER => Ok(Alignment::Center),
            ftd::interpreter2::FTD_ALIGN_RIGHT => Ok(Alignment::Right),
            ftd::interpreter2::FTD_ALIGN_BOTTOM_LEFT => Ok(Alignment::BottomLeft),
            ftd::interpreter2::FTD_ALIGN_BOTTOM_CENTER => Ok(Alignment::BottomCenter),
            ftd::interpreter2::FTD_ALIGN_BOTTOM_RIGHT => Ok(Alignment::BottomRight),
            t => ftd::executor::utils::parse_error(
                format!("Unknown variant `{}` for or-type `ftd.alignment`", t),
                doc.name,
                line_number,
            ),
        }
    }

    pub(crate) fn alignment_with_default(
        properties: &[ftd::interpreter2::Property],
        arguments: &[ftd::interpreter2::Argument],
        doc: &ftd::executor::TDoc,
        line_number: usize,
        key: &str,
        default: ftd::executor::Alignment,
    ) -> ftd::executor::Result<ftd::executor::Value<Alignment>> {
        let or_type_value = ftd::executor::value::optional_or_type(
            key,
            properties,
            arguments,
            doc,
            line_number,
            ftd::interpreter2::FTD_ALIGN,
        )?;

        Ok(ftd::executor::Value::new(
            Alignment::from_optional_values(or_type_value.value, doc, line_number)?
                .unwrap_or(default),
            or_type_value.line_number,
            or_type_value.properties,
        ))
    }

    #[allow(dead_code)]
    pub(crate) fn optional_alignment(
        properties: &[ftd::interpreter2::Property],
        arguments: &[ftd::interpreter2::Argument],
        doc: &ftd::executor::TDoc,
        line_number: usize,
        key: &str,
    ) -> ftd::executor::Result<ftd::executor::Value<Option<Alignment>>> {
        let or_type_value = ftd::executor::value::optional_or_type(
            key,
            properties,
            arguments,
            doc,
            line_number,
            ftd::interpreter2::FTD_ALIGN,
        )?;

        Ok(ftd::executor::Value::new(
            Alignment::from_optional_values(or_type_value.value, doc, line_number)?,
            or_type_value.line_number,
            or_type_value.properties,
        ))
    }

    pub fn to_css_justify_content(&self, is_horizontal_direction: bool) -> String {
        if is_horizontal_direction {
            match self {
                Alignment::TopLeft | Alignment::Left | Alignment::BottomLeft => "start".to_string(),
                Alignment::TopCenter | Alignment::Center | Alignment::BottomCenter => {
                    "center".to_string()
                }
                Alignment::TopRight | Alignment::Right | Alignment::BottomRight => {
                    "end".to_string()
                }
            }
        } else {
            match self {
                Alignment::TopLeft | Alignment::TopCenter | Alignment::TopRight => {
                    "start".to_string()
                }
                Alignment::Left | Alignment::Center | Alignment::Right => "center".to_string(),
                Alignment::BottomLeft | Alignment::BottomCenter | Alignment::BottomRight => {
                    "end".to_string()
                }
            }
        }
    }

    pub fn to_css_align_items(&self, is_horizontal_direction: bool) -> String {
        if is_horizontal_direction {
            match self {
                Alignment::TopLeft | Alignment::TopCenter | Alignment::TopRight => {
                    "start".to_string()
                }
                Alignment::Left | Alignment::Center | Alignment::Right => "center".to_string(),
                Alignment::BottomLeft | Alignment::BottomCenter | Alignment::BottomRight => {
                    "end".to_string()
                }
            }
        } else {
            match self {
                Alignment::TopLeft | Alignment::Left | Alignment::BottomLeft => "start".to_string(),
                Alignment::TopCenter | Alignment::Center | Alignment::BottomCenter => {
                    "center".to_string()
                }
                Alignment::TopRight | Alignment::Right | Alignment::BottomRight => {
                    "end".to_string()
                }
            }
        }
    }

    pub fn justify_content_pattern(is_horizontal_direction: bool) -> (String, bool) {
        if is_horizontal_direction {
            (
                format!(
                    indoc::indoc! {"
                        if (\"{{0}}\" == \"{top_left}\" || \"{{0}}\" == \"{left}\" || \"{{0}}\" == \"{bottom_left}\") {{
                            \"start\"
                        }} else if (\"{{0}}\" == \"{top_center}\" || \"{{0}}\" == \"{center}\" || \"{{0}}\" == \"{bottom_center}\") {{
                            \"center\"
                        }} else if (\"{{0}}\" == \"{top_right}\" || \"{{0}}\" == \"{right}\" || \"{{0}}\" == \"{bottom_right}\") {{
                            \"end\"
                        }} else {{
                            null
                        }}
                    "},
                    top_left = ftd::interpreter2::FTD_ALIGN_TOP_LEFT,
                    top_center = ftd::interpreter2::FTD_ALIGN_TOP_CENTER,
                    top_right = ftd::interpreter2::FTD_ALIGN_TOP_RIGHT,
                    left = ftd::interpreter2::FTD_ALIGN_LEFT,
                    center = ftd::interpreter2::FTD_ALIGN_CENTER,
                    right = ftd::interpreter2::FTD_ALIGN_RIGHT,
                    bottom_left = ftd::interpreter2::FTD_ALIGN_BOTTOM_LEFT,
                    bottom_center = ftd::interpreter2::FTD_ALIGN_BOTTOM_CENTER,
                    bottom_right = ftd::interpreter2::FTD_ALIGN_BOTTOM_RIGHT,
                ),
                true,
            )
        } else {
            (
                format!(
                    indoc::indoc! {"
                if (\"{{0}}\" == \"{top_left}\" || \"{{0}}\" == \"{top_center}\" || \"{{0}}\" == \"{top_right}\") {{
                    \"start\"
                }} else if (\"{{0}}\" == \"{left}\" || \"{{0}}\" == \"{center}\" || \"{{0}}\" == \"{right}\") {{
                    \"center\"
                }} else if (\"{{0}}\" == \"{bottom_left}\" || \"{{0}}\" == \"{bottom_center}\" || \"{{0}}\" == \"{bottom_right}\") {{
                    \"end\"
                }} else {{
                    null
                }}
                "},
                    top_left = ftd::interpreter2::FTD_ALIGN_TOP_LEFT,
                    top_center = ftd::interpreter2::FTD_ALIGN_TOP_CENTER,
                    top_right = ftd::interpreter2::FTD_ALIGN_TOP_RIGHT,
                    left = ftd::interpreter2::FTD_ALIGN_LEFT,
                    center = ftd::interpreter2::FTD_ALIGN_CENTER,
                    right = ftd::interpreter2::FTD_ALIGN_RIGHT,
                    bottom_left = ftd::interpreter2::FTD_ALIGN_BOTTOM_LEFT,
                    bottom_center = ftd::interpreter2::FTD_ALIGN_BOTTOM_CENTER,
                    bottom_right = ftd::interpreter2::FTD_ALIGN_BOTTOM_RIGHT,
                ),
                true,
            )
        }
    }

    pub fn align_item_pattern(is_horizontal_direction: bool) -> (String, bool) {
        if is_horizontal_direction {
            (
                format!(
                    indoc::indoc! {"
               if (\"{{0}}\" == \"{top_left}\" || \"{{0}}\" == \"{top_center}\" || \"{{0}}\" == \"{top_right}\") {{
                    \"start\"
                }} else if (\"{{0}}\" == \"{left}\" || \"{{0}}\" == \"{center}\" || \"{{0}}\" == \"{right}\") {{
                    \"center\"
                }} else if (\"{{0}}\" == \"{bottom_left}\" || \"{{0}}\" == \"{bottom_center}\" || \"{{0}}\" == \"{bottom_right}\") {{
                    \"end\"
                }} else {{
                    null
                }}
                "},
                    top_left = ftd::interpreter2::FTD_ALIGN_TOP_LEFT,
                    top_center = ftd::interpreter2::FTD_ALIGN_TOP_CENTER,
                    top_right = ftd::interpreter2::FTD_ALIGN_TOP_RIGHT,
                    left = ftd::interpreter2::FTD_ALIGN_LEFT,
                    center = ftd::interpreter2::FTD_ALIGN_CENTER,
                    right = ftd::interpreter2::FTD_ALIGN_RIGHT,
                    bottom_left = ftd::interpreter2::FTD_ALIGN_BOTTOM_LEFT,
                    bottom_center = ftd::interpreter2::FTD_ALIGN_BOTTOM_CENTER,
                    bottom_right = ftd::interpreter2::FTD_ALIGN_BOTTOM_RIGHT,
                ),
                true,
            )
        } else {
            (
                format!(
                    indoc::indoc! {"
                        if (\"{{0}}\" == \"{top_left}\" || \"{{0}}\" == \"{left}\" || \"{{0}}\" == \"{bottom_left}\") {{
                            \"start\"
                        }} else if (\"{{0}}\" == \"{top_center}\" || \"{{0}}\" == \"{center}\" || \"{{0}}\" == \"{bottom_center}\") {{
                            \"center\"
                        }} else if (\"{{0}}\" == \"{top_right}\" || \"{{0}}\" == \"{right}\" || \"{{0}}\" == \"{bottom_right}\") {{
                            \"end\"
                        }} else {{
                            null
                        }}
                    "},
                    top_left = ftd::interpreter2::FTD_ALIGN_TOP_LEFT,
                    top_center = ftd::interpreter2::FTD_ALIGN_TOP_CENTER,
                    top_right = ftd::interpreter2::FTD_ALIGN_TOP_RIGHT,
                    left = ftd::interpreter2::FTD_ALIGN_LEFT,
                    center = ftd::interpreter2::FTD_ALIGN_CENTER,
                    right = ftd::interpreter2::FTD_ALIGN_RIGHT,
                    bottom_left = ftd::interpreter2::FTD_ALIGN_BOTTOM_LEFT,
                    bottom_center = ftd::interpreter2::FTD_ALIGN_BOTTOM_CENTER,
                    bottom_right = ftd::interpreter2::FTD_ALIGN_BOTTOM_RIGHT,
                ),
                true,
            )
        }
    }
}

#[derive(serde::Deserialize, Debug, PartialEq, Clone, serde::Serialize)]
pub enum Resizing {
    HugContent,
    FillContainer,
    Fixed(ftd::executor::Length),
}

impl Default for Resizing {
    fn default() -> Resizing {
        Resizing::HugContent
    }
}

impl Resizing {
    fn from_optional_values(
        or_type_value: Option<(String, ftd::interpreter2::PropertyValue)>,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Option<Self>> {
        if let Some(value) = or_type_value {
            Ok(Some(Resizing::from_values(value, doc, line_number)?))
        } else {
            Ok(None)
        }
    }

    fn from_values(
        or_type_value: (String, ftd::interpreter2::PropertyValue),
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Self> {
        match or_type_value.0.as_str() {
            t if t.starts_with(ftd::interpreter2::FTD_RESIZING_FIXED) => {
                let value = or_type_value.1.clone().resolve(&doc.itdoc(), line_number)?;
                let (_, variant, value) = value.get_or_type(doc.name, line_number)?;
                Ok(Resizing::Fixed(Length::from_values(
                    (variant.to_owned(), value.to_owned()),
                    doc,
                    line_number,
                )?))
            }
            ftd::interpreter2::FTD_RESIZING_HUG_CONTENT => Ok(Resizing::HugContent),
            ftd::interpreter2::FTD_RESIZING_FILL_CONTAINER => Ok(Resizing::FillContainer),
            t => ftd::executor::utils::parse_error(
                format!("Unknown variant `{}` for or-type `ftd.resizing`", t),
                doc.name,
                line_number,
            ),
        }
    }

    pub(crate) fn resizing_with_default(
        properties: &[ftd::interpreter2::Property],
        arguments: &[ftd::interpreter2::Argument],
        doc: &ftd::executor::TDoc,
        line_number: usize,
        key: &str,
        default: Resizing,
    ) -> ftd::executor::Result<ftd::executor::Value<Resizing>> {
        let or_type_value = ftd::executor::value::optional_or_type(
            key,
            properties,
            arguments,
            doc,
            line_number,
            ftd::interpreter2::FTD_RESIZING,
        )?;

        Ok(ftd::executor::Value::new(
            Resizing::from_optional_values(or_type_value.value, doc, line_number)?
                .unwrap_or(default),
            or_type_value.line_number,
            or_type_value.properties,
        ))
    }

    pub fn to_css_string(&self) -> String {
        match self {
            Resizing::HugContent => "fit-content".to_string(),
            Resizing::FillContainer => "100%".to_string(),
            Resizing::Fixed(l) => l.to_css_string(),
        }
    }

    pub fn get_pattern_from_variant_str(
        variant: &str,
        full_variant: &str,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::executor::Result<(&'static str, bool)> {
        match variant {
            ftd::interpreter2::FTD_RESIZING_FIXED => {
                let remaining = full_variant
                    .trim_start_matches(format!("{}.", variant).as_str())
                    .to_string();
                let variant = format!("{}.{}", ftd::interpreter2::FTD_LENGTH, remaining);
                Ok((
                    Length::get_pattern_from_variant_str(variant.as_str(), doc_id, line_number)?,
                    true,
                ))
            }
            ftd::interpreter2::FTD_RESIZING_FILL_CONTAINER => Ok(("100%", false)),
            ftd::interpreter2::FTD_RESIZING_HUG_CONTENT => Ok(("fit-content", false)),
            t => ftd::executor::utils::parse_error(
                format!("Unknown variant found for ftd.resizing: `{}`", t),
                doc_id,
                line_number,
            ),
        }
    }

    pub fn set_pattern_from_variant_str(
        variant: &str,
        full_variant: &str,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::executor::Result<&'static str> {
        match variant {
            ftd::interpreter2::FTD_RESIZING_FIXED => {
                let remaining = full_variant
                    .trim_start_matches(format!("{}.", variant).as_str())
                    .to_string();
                let variant = format!("{}.{}", ftd::interpreter2::FTD_LENGTH, remaining);
                Length::set_pattern_from_variant_str(variant.as_str(), doc_id, line_number)
            }
            ftd::interpreter2::FTD_RESIZING_FILL_CONTAINER => Ok("100%"),
            ftd::interpreter2::FTD_RESIZING_HUG_CONTENT => Ok("fit-content"),
            t => ftd::executor::utils::parse_error(
                format!("Unknown variant found for ftd.resizing: `{}`", t),
                doc_id,
                line_number,
            ),
        }
    }

    pub fn set_value_from_variant(
        variant: &str,
        full_variant: &str,
        doc_id: &str,
        value: &str,
        line_number: usize,
    ) -> ftd::executor::Result<String> {
        match variant {
            ftd::interpreter2::FTD_RESIZING_FIXED => {
                let remaining = full_variant
                    .trim_start_matches(format!("{}.", variant).as_str())
                    .to_string();
                let variant = format!("{}.{}", ftd::interpreter2::FTD_LENGTH, remaining);
                Length::set_value_from_variant(variant.as_str(), value, doc_id, line_number)
            }
            ftd::interpreter2::FTD_RESIZING_FILL_CONTAINER => Ok("100%".to_string()),
            ftd::interpreter2::FTD_RESIZING_HUG_CONTENT => Ok("fit-content".to_string()),
            t => ftd::executor::utils::parse_error(
                format!("Unknown variant found for ftd.resizing: `{}`", t),
                doc_id,
                line_number,
            ),
        }
    }
}

#[derive(serde::Deserialize, Debug, PartialEq, Clone, serde::Serialize)]
pub enum Background {
    Solid(Color),
}

impl Background {
    fn from_optional_values(
        or_type_value: Option<(String, ftd::interpreter2::PropertyValue)>,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Option<Self>> {
        if let Some(value) = or_type_value {
            Ok(Some(Background::from_values(value, doc, line_number)?))
        } else {
            Ok(None)
        }
    }

    fn from_values(
        or_type_value: (String, ftd::interpreter2::PropertyValue),
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Self> {
        match or_type_value.0.as_str() {
            ftd::interpreter2::FTD_BACKGROUND_SOLID => Ok(Background::Solid(Color::from_value(
                or_type_value.1,
                doc,
                line_number,
            )?)),
            t => ftd::executor::utils::parse_error(
                format!("Unknown variant `{}` for or-type `ftd.length`", t),
                doc.name,
                line_number,
            ),
        }
    }

    pub(crate) fn optional_fill(
        properties: &[ftd::interpreter2::Property],
        arguments: &[ftd::interpreter2::Argument],
        doc: &ftd::executor::TDoc,
        line_number: usize,
        key: &str,
    ) -> ftd::executor::Result<ftd::executor::Value<Option<Background>>> {
        let or_type_value = ftd::executor::value::optional_or_type(
            key,
            properties,
            arguments,
            doc,
            line_number,
            ftd::interpreter2::FTD_BACKGROUND,
        )?;

        Ok(ftd::executor::Value::new(
            Background::from_optional_values(or_type_value.value, doc, line_number)?,
            or_type_value.line_number,
            or_type_value.properties,
        ))
    }

    pub fn to_css_string(&self) -> String {
        match self {
            Background::Solid(c) => c.light.value.color(),
        }
    }
}

#[derive(serde::Deserialize, Debug, Default, PartialEq, Clone, serde::Serialize)]
pub struct Color {
    pub light: ftd::executor::Value<ColorValue>,
    pub dark: ftd::executor::Value<ColorValue>,
}

impl Color {
    fn from_value(
        value: ftd::interpreter2::PropertyValue,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Color> {
        let value = value.resolve(&doc.itdoc(), line_number)?;
        let fields = match value.inner() {
            Some(ftd::interpreter2::Value::Record { name, fields })
                if name.eq(ftd::interpreter2::FTD_COLOR) =>
            {
                fields
            }
            t => {
                return ftd::executor::utils::parse_error(
                    format!(
                        "Expected value of type record `{}`, found: {:?}",
                        ftd::interpreter2::FTD_COLOR,
                        t
                    ),
                    doc.name,
                    line_number,
                )
            }
        };
        Color::from_values(fields, doc, line_number)
    }

    fn from_values(
        values: ftd::Map<ftd::interpreter2::PropertyValue>,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Color> {
        let light = {
            let value = values
                .get("light")
                .ok_or(ftd::executor::Error::ParseError {
                    message: "`light` field in ftd.image-src not found".to_string(),
                    doc_id: doc.name.to_string(),
                    line_number,
                })?;
            ftd::executor::Value::new(
                ColorValue::color_from(
                    value
                        .clone()
                        .resolve(&doc.itdoc(), line_number)?
                        .string(doc.name, line_number)?,
                    doc.name,
                    line_number,
                )?,
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
                ColorValue::color_from(
                    value
                        .clone()
                        .resolve(&doc.itdoc(), line_number)?
                        .string(doc.name, line_number)?,
                    doc.name,
                    line_number,
                )?,
                Some(line_number),
                vec![value.into_property(ftd::interpreter2::PropertySource::header("dark"))],
            )
        };

        Ok(Color { light, dark })
    }
}

#[derive(serde::Deserialize, Debug, PartialEq, Default, Clone, serde::Serialize)]
pub struct ColorValue {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub alpha: f32,
}

impl ColorValue {
    pub fn color_from(
        l: String,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::executor::Result<ColorValue> {
        use std::str::FromStr;

        let v = l.trim().to_string();

        // Remove all whitespace, not compliant, but should just be more accepting.
        let mut string = v.replace(' ', "");
        string.make_ascii_lowercase();
        if v.starts_with('#') && v.len() == 9 {
            let (_, value_string) = string.split_at(1);

            let iv = u64::from_str_radix(value_string, 16).map_err(|e| {
                ftd::executor::Error::ParseError {
                    message: e.to_string(),
                    doc_id: doc_id.to_string(),
                    line_number: 0,
                }
            })?;

            // (7thSigil) unlike original js code, NaN is impossible
            if iv > 0xffffffff {
                return ftd::executor::utils::parse_error(
                    format!("{} is not a valid color", v),
                    doc_id,
                    line_number,
                );
            }

            //Code for accepting 6-digit hexa-color code
            Ok(ColorValue {
                r: ((iv & 0xff000000) >> 24) as u8,
                g: ((iv & 0xff0000) >> 16) as u8,
                b: ((iv & 0xff00) >> 8) as u8,
                alpha: round_1p((iv & 0xff) as f32 / 255_f32) as f32,
            })
        } else {
            match css_color_parser::Color::from_str(v.as_str()) {
                Ok(v) => Ok(ColorValue {
                    r: v.r,
                    g: v.g,
                    b: v.b,
                    alpha: v.a,
                }),
                Err(e) => ftd::executor::utils::parse_error(
                    format!("{} is not a valid color: {:?}", v, e),
                    doc_id,
                    line_number,
                ),
            }
        }
    }

    pub fn color(&self) -> String {
        format!("rgba({},{},{},{})", self.r, self.g, self.b, self.alpha)
    }
}

fn round_1p(n: f32) -> f32 {
    // 1234.56
    let temp = (n * 10_f32) as u32;
    let last = (temp % 10) as f32;
    let front = n as u32;

    front as f32 + last / 10_f32
}

#[derive(serde::Deserialize, Debug, PartialEq, Clone, serde::Serialize)]
pub enum SpacingMode {
    SpaceBetween,
    SpaceEvenly,
    SpaceAround,
}

impl SpacingMode {
    fn from_optional_values(
        or_type_value: Option<(String, ftd::interpreter2::PropertyValue)>,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Option<Self>> {
        if let Some(value) = or_type_value {
            Ok(Some(SpacingMode::from_values(value, doc, line_number)?))
        } else {
            Ok(None)
        }
    }

    fn from_values(
        or_type_value: (String, ftd::interpreter2::PropertyValue),
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Self> {
        match or_type_value.0.as_str() {
            ftd::interpreter2::FTD_SPACING_MODE_SPACE_BETWEEN => Ok(SpacingMode::SpaceBetween),
            ftd::interpreter2::FTD_SPACING_MODE_SPACE_EVENLY => Ok(SpacingMode::SpaceEvenly),
            ftd::interpreter2::FTD_SPACING_MODE_SPACE_AROUND => Ok(SpacingMode::SpaceAround),
            t => ftd::executor::utils::parse_error(
                format!("Unknown variant `{}` for or-type `ftd.spacing-mode`", t),
                doc.name,
                line_number,
            ),
        }
    }

    pub(crate) fn optional_spacing_mode(
        properties: &[ftd::interpreter2::Property],
        arguments: &[ftd::interpreter2::Argument],
        doc: &ftd::executor::TDoc,
        line_number: usize,
        key: &str,
    ) -> ftd::executor::Result<ftd::executor::Value<Option<SpacingMode>>> {
        let or_type_value = ftd::executor::value::optional_or_type(
            key,
            properties,
            arguments,
            doc,
            line_number,
            ftd::interpreter2::FTD_SPACING_MODE,
        )?;

        Ok(ftd::executor::Value::new(
            SpacingMode::from_optional_values(or_type_value.value, doc, line_number)?,
            or_type_value.line_number,
            or_type_value.properties,
        ))
    }

    pub fn to_css_string(&self) -> String {
        match self {
            SpacingMode::SpaceBetween => "space-between".to_string(),
            SpacingMode::SpaceEvenly => "space-evenly".to_string(),
            SpacingMode::SpaceAround => "space-around".to_string(),
        }
    }
}
