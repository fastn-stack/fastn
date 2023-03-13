#[derive(serde::Deserialize, Debug, PartialEq, Clone, serde::Serialize)]
pub enum Length {
    Px(i64),
    Percent(f64),
    Calc(String),
    Vh(f64),
    Vw(f64),
    Em(f64),
    Rem(f64),
    Responsive(Box<ResponsiveLength>),
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
    ) -> ftd::executor::Result<Option<Length>> {
        if let Some(value) = or_type_value {
            Ok(Some(Length::from_values(value, doc, line_number)?))
        } else {
            Ok(None)
        }
    }

    fn from_value(
        value: ftd::interpreter2::PropertyValue,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Length> {
        let binding = value.resolve(&doc.itdoc(), line_number)?;
        let value = binding.get_or_type(doc.name, line_number)?;
        let value = (value.1.to_owned(), value.2.to_owned());
        Length::from_values(value, doc, line_number)
    }

    fn from_values(
        or_type_value: (String, ftd::interpreter2::PropertyValue),
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Length> {
        match or_type_value.0.as_str() {
            ftd::interpreter2::FTD_LENGTH_PERCENT => Ok(Length::Percent(
                or_type_value
                    .1
                    .clone()
                    .resolve(&doc.itdoc(), line_number)?
                    .decimal(doc.name, line_number)?,
            )),
            ftd::interpreter2::FTD_LENGTH_PX => Ok(Length::Px(
                or_type_value
                    .1
                    .clone()
                    .resolve(&doc.itdoc(), line_number)?
                    .integer(doc.name, line_number)?,
            )),
            ftd::interpreter2::FTD_LENGTH_CALC => Ok(Length::Calc(
                or_type_value
                    .1
                    .clone()
                    .resolve(&doc.itdoc(), line_number)?
                    .string(doc.name, line_number)?,
            )),
            ftd::interpreter2::FTD_LENGTH_VH => Ok(Length::Vh(
                or_type_value
                    .1
                    .clone()
                    .resolve(&doc.itdoc(), line_number)?
                    .decimal(doc.name, line_number)?,
            )),
            ftd::interpreter2::FTD_LENGTH_VW => Ok(Length::Vw(
                or_type_value
                    .1
                    .clone()
                    .resolve(&doc.itdoc(), line_number)?
                    .decimal(doc.name, line_number)?,
            )),
            ftd::interpreter2::FTD_LENGTH_EM => Ok(Length::Em(
                or_type_value
                    .1
                    .clone()
                    .resolve(&doc.itdoc(), line_number)?
                    .decimal(doc.name, line_number)?,
            )),
            ftd::interpreter2::FTD_LENGTH_REM => Ok(Length::Rem(
                or_type_value
                    .1
                    .clone()
                    .resolve(&doc.itdoc(), line_number)?
                    .decimal(doc.name, line_number)?,
            )),
            ftd::interpreter2::FTD_LENGTH_RESPONSIVE => Ok(Length::Responsive(Box::new(
                ResponsiveLength::from_value(or_type_value.1.clone(), doc, line_number)?,
            ))),
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
        inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
    ) -> ftd::executor::Result<ftd::executor::Value<Option<Length>>> {
        let or_type_value = ftd::executor::value::optional_or_type(
            key,
            properties,
            arguments,
            doc,
            line_number,
            ftd::interpreter2::FTD_LENGTH,
            inherited_variables,
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
        inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
    ) -> ftd::executor::Result<ftd::executor::Value<Length>> {
        let or_type_value = ftd::executor::value::optional_or_type(
            key,
            properties,
            arguments,
            doc,
            line_number,
            ftd::interpreter2::FTD_LENGTH,
            inherited_variables,
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
            Length::Calc(calc) => format!("calc({})", calc),
            Length::Vh(vh) => format!("{}vh", vh),
            Length::Vw(vw) => format!("{}vw", vw),
            Length::Em(em) => format!("{}em", em),
            Length::Rem(rem) => format!("{}rem", rem),
            Length::Responsive(r) => r.desktop.to_css_string(),
        }
    }

    pub fn get_pattern_from_variant_str(
        variant: &str,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::executor::Result<&'static str> {
        match variant {
            ftd::interpreter2::FTD_LENGTH_PX
            | ftd::interpreter2::FTD_LENGTH_PERCENT
            | ftd::interpreter2::FTD_LENGTH_CALC
            | ftd::interpreter2::FTD_LENGTH_VH
            | ftd::interpreter2::FTD_LENGTH_VW
            | ftd::interpreter2::FTD_LENGTH_EM
            | ftd::interpreter2::FTD_LENGTH_REM => Ok("{0}"),
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
            ftd::interpreter2::FTD_LENGTH_CALC => Ok("calc({0})"),
            ftd::interpreter2::FTD_LENGTH_VH => Ok("{0}vh"),
            ftd::interpreter2::FTD_LENGTH_VW => Ok("{0}vw"),
            ftd::interpreter2::FTD_LENGTH_EM => Ok("{0}em"),
            ftd::interpreter2::FTD_LENGTH_REM => Ok("{0}rem"),
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
            ftd::interpreter2::FTD_LENGTH_CALC => Ok(format!("calc({})", value)),
            ftd::interpreter2::FTD_LENGTH_VH => Ok(format!("{}vh", value)),
            ftd::interpreter2::FTD_LENGTH_VW => Ok(format!("{}vw", value)),
            ftd::interpreter2::FTD_LENGTH_EM => Ok(format!("{}em", value)),
            ftd::interpreter2::FTD_LENGTH_REM => Ok(format!("{}rem", value)),
            t => ftd::executor::utils::parse_error(
                format!("Unknown variant found for ftd.length: `{}`", t),
                doc_id,
                line_number,
            ),
        }
    }
}

#[derive(serde::Deserialize, Debug, Default, PartialEq, Clone, serde::Serialize)]
pub struct ResponsiveLength {
    pub desktop: Length,
    pub mobile: Length,
}

impl ResponsiveLength {
    fn from_value(
        value: ftd::interpreter2::PropertyValue,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<ResponsiveLength> {
        let value = value.resolve(&doc.itdoc(), line_number)?;
        let fields = match value.inner() {
            Some(ftd::interpreter2::Value::Record { name, fields })
                if name.eq(ftd::interpreter2::FTD_RESPONSIVE_LENGTH) =>
            {
                fields
            }
            t => {
                return ftd::executor::utils::parse_error(
                    format!(
                        "Expected value of type record `{}`, found: {:?}",
                        ftd::interpreter2::FTD_RESPONSIVE_LENGTH,
                        t
                    ),
                    doc.name,
                    line_number,
                )
            }
        };
        ResponsiveLength::from_values(fields, doc, line_number)
    }

    fn from_values(
        values: ftd::Map<ftd::interpreter2::PropertyValue>,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<ResponsiveLength> {
        let desktop = {
            let value = values
                .get("desktop")
                .ok_or(ftd::executor::Error::ParseError {
                    message: "`desktop` field in ftd.responsive-type not found".to_string(),
                    doc_id: doc.name.to_string(),
                    line_number,
                })?;
            Length::from_value(value.to_owned(), doc, line_number)?
        };

        let mobile = {
            if let Some(value) = values.get("mobile") {
                Length::from_value(value.to_owned(), doc, line_number)?
            } else {
                desktop.clone()
            }
        };

        Ok(ResponsiveLength { desktop, mobile })
    }
}

#[derive(serde::Deserialize, Default, Debug, PartialEq, Clone, serde::Serialize)]
pub enum Alignment {
    #[default]
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
        inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
    ) -> ftd::executor::Result<ftd::executor::Value<Alignment>> {
        let or_type_value = ftd::executor::value::optional_or_type(
            key,
            properties,
            arguments,
            doc,
            line_number,
            ftd::interpreter2::FTD_ALIGN,
            inherited_variables,
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
        inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
    ) -> ftd::executor::Result<ftd::executor::Value<Option<Alignment>>> {
        let or_type_value = ftd::executor::value::optional_or_type(
            key,
            properties,
            arguments,
            doc,
            line_number,
            ftd::interpreter2::FTD_ALIGN,
            inherited_variables,
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

#[derive(serde::Deserialize, Default, Debug, PartialEq, Clone, serde::Serialize)]
pub enum Resizing {
    HugContent,
    FillContainer,
    #[default]
    Auto,
    Fixed(ftd::executor::Length),
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
            ftd::interpreter2::FTD_RESIZING_AUTO => Ok(Resizing::Auto),
            ftd::interpreter2::FTD_RESIZING_FILL_CONTAINER => Ok(Resizing::FillContainer),
            t => ftd::executor::utils::parse_error(
                format!("Unknown variant `{}` for or-type `ftd.resizing`", t),
                doc.name,
                line_number,
            ),
        }
    }

    pub(crate) fn optional_resizing(
        properties: &[ftd::interpreter2::Property],
        arguments: &[ftd::interpreter2::Argument],
        doc: &ftd::executor::TDoc,
        line_number: usize,
        key: &str,
        inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
    ) -> ftd::executor::Result<ftd::executor::Value<Option<Resizing>>> {
        let or_type_value = ftd::executor::value::optional_or_type(
            key,
            properties,
            arguments,
            doc,
            line_number,
            ftd::interpreter2::FTD_RESIZING,
            inherited_variables,
        )?;

        Ok(ftd::executor::Value::new(
            Resizing::from_optional_values(or_type_value.value, doc, line_number)?,
            or_type_value.line_number,
            or_type_value.properties,
        ))
    }

    pub(crate) fn resizing_with_default(
        properties: &[ftd::interpreter2::Property],
        arguments: &[ftd::interpreter2::Argument],
        doc: &ftd::executor::TDoc,
        line_number: usize,
        key: &str,
        default: Resizing,
        inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
    ) -> ftd::executor::Result<ftd::executor::Value<Resizing>> {
        let or_type_value = ftd::executor::value::optional_or_type(
            key,
            properties,
            arguments,
            doc,
            line_number,
            ftd::interpreter2::FTD_RESIZING,
            inherited_variables,
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
            Resizing::Auto => "auto".to_string(),
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
                    false,
                ))
            }
            ftd::interpreter2::FTD_RESIZING_FILL_CONTAINER => Ok(("100%", false)),
            ftd::interpreter2::FTD_RESIZING_HUG_CONTENT => Ok(("fit-content", false)),
            ftd::interpreter2::FTD_RESIZING_AUTO => Ok(("auto", false)),
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
            ftd::interpreter2::FTD_RESIZING_AUTO => Ok("auto"),
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
            ftd::interpreter2::FTD_RESIZING_AUTO => Ok("auto".to_string()),
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
        inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
    ) -> ftd::executor::Result<ftd::executor::Value<Option<Background>>> {
        let or_type_value = ftd::executor::value::optional_or_type(
            key,
            properties,
            arguments,
            doc,
            line_number,
            ftd::interpreter2::FTD_BACKGROUND,
            inherited_variables,
        )?;

        Ok(ftd::executor::Value::new(
            Background::from_optional_values(or_type_value.value, doc, line_number)?,
            or_type_value.line_number,
            or_type_value.properties,
        ))
    }

    pub fn to_css_string(&self) -> String {
        match self {
            Background::Solid(c) => c.light.value.to_css_string(),
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
                    message: "`light` field in ftd.color not found".to_string(),
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
            if let Some(value) = values.get("dark") {
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
            } else {
                light.clone()
            }
        };

        Ok(Color { light, dark })
    }

    fn from_optional_values(
        or_type_value: Option<ftd::Map<ftd::interpreter2::PropertyValue>>,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Option<Self>> {
        if let Some(value) = or_type_value {
            Ok(Some(Color::from_values(value, doc, line_number)?))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn optional_color(
        properties: &[ftd::interpreter2::Property],
        arguments: &[ftd::interpreter2::Argument],
        doc: &ftd::executor::TDoc,
        line_number: usize,
        key: &str,
        inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
    ) -> ftd::executor::Result<ftd::executor::Value<Option<Color>>> {
        let record_values = ftd::executor::value::optional_record_inherited(
            key,
            properties,
            arguments,
            doc,
            line_number,
            ftd::interpreter2::FTD_COLOR,
            inherited_variables,
        )?;

        Ok(ftd::executor::Value::new(
            Color::from_optional_values(record_values.value, doc, line_number)?,
            record_values.line_number,
            record_values.properties,
        ))
    }

    pub fn to_css_string(&self) -> String {
        self.light.value.to_css_string()
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
                alpha: round_1p((iv & 0xff) as f32 / 255_f32),
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

    pub fn to_css_string(&self) -> String {
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
pub enum Spacing {
    Fixed(Length),
    SpaceBetween,
    SpaceEvenly,
    SpaceAround,
}

impl Spacing {
    fn from_optional_values(
        or_type_value: Option<(String, ftd::interpreter2::PropertyValue)>,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Option<Self>> {
        if let Some(value) = or_type_value {
            Ok(Some(Spacing::from_values(value, doc, line_number)?))
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
            ftd::interpreter2::FTD_SPACING_SPACE_BETWEEN => Ok(Spacing::SpaceBetween),
            ftd::interpreter2::FTD_SPACING_SPACE_EVENLY => Ok(Spacing::SpaceEvenly),
            ftd::interpreter2::FTD_SPACING_SPACE_AROUND => Ok(Spacing::SpaceAround),
            ftd::interpreter2::FTD_SPACING_FIXED => Ok(Spacing::Fixed(Length::from_value(
                or_type_value.1.to_owned(),
                doc,
                line_number,
            )?)),
            t => ftd::executor::utils::parse_error(
                format!("Unknown variant `{}` for or-type `ftd.spacing`", t),
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
        inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
    ) -> ftd::executor::Result<ftd::executor::Value<Option<Spacing>>> {
        let or_type_value = ftd::executor::value::optional_or_type(
            key,
            properties,
            arguments,
            doc,
            line_number,
            ftd::interpreter2::FTD_SPACING,
            inherited_variables,
        )?;

        Ok(ftd::executor::Value::new(
            Spacing::from_optional_values(or_type_value.value, doc, line_number)?,
            or_type_value.line_number,
            or_type_value.properties,
        ))
    }

    pub fn to_css_string(&self) -> String {
        match self {
            Spacing::SpaceBetween => "space-between".to_string(),
            Spacing::SpaceEvenly => "space-evenly".to_string(),
            Spacing::SpaceAround => "space-around".to_string(),
            Spacing::Fixed(f) => f.to_css_string(),
        }
    }

    pub fn to_gap_css_string(&self) -> String {
        match self {
            Spacing::Fixed(f) => f.to_css_string(),
            _ => "0".to_string(),
        }
    }

    pub fn to_justify_content_css_string(&self) -> String {
        match self {
            Spacing::SpaceBetween => "space-between".to_string(),
            Spacing::SpaceEvenly => "space-evenly".to_string(),
            Spacing::SpaceAround => "space-around".to_string(),
            Spacing::Fixed(_) => "start".to_string(),
        }
    }

    pub fn justify_content_pattern() -> (String, bool) {
        (
            indoc::indoc! {"
                if (\"{0}\" == \"space-between\" || \"{0}\" == \"space-around\" || \"{0}\" == \"space-evenly\") {
                    \"{0}\"
                } else {
                    \"start\"
                }
            "}
            .to_string(),
            true,
        )
    }

    pub fn fixed_content_pattern() -> (String, bool) {
        (
            indoc::indoc! {"
                if (\"{0}\" != \"space-between\" && \"{0}\" != \"space_around\" && \"{0}\" != \"space-evenly\") {
                    \"{0}\"
                } else {
                    null
                }
            "}.to_string(),
            true,
        )
    }
}

#[derive(serde::Deserialize, Debug, PartialEq, Clone, serde::Serialize)]
pub enum AlignSelf {
    Start,
    Center,
    End,
}

impl AlignSelf {
    fn from_optional_values(
        or_type_value: Option<(String, ftd::interpreter2::PropertyValue)>,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Option<Self>> {
        if let Some(value) = or_type_value {
            Ok(Some(AlignSelf::from_values(value, doc, line_number)?))
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
            ftd::interpreter2::FTD_ALIGN_SELF_START => Ok(AlignSelf::Start),
            ftd::interpreter2::FTD_ALIGN_SELF_CENTER => Ok(AlignSelf::Center),
            ftd::interpreter2::FTD_ALIGN_SELF_END => Ok(AlignSelf::End),
            t => ftd::executor::utils::parse_error(
                format!("Unknown variant `{}` for or-type `ftd.align-self`", t),
                doc.name,
                line_number,
            ),
        }
    }

    pub(crate) fn optional_align_self(
        properties: &[ftd::interpreter2::Property],
        arguments: &[ftd::interpreter2::Argument],
        doc: &ftd::executor::TDoc,
        line_number: usize,
        key: &str,
        inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
    ) -> ftd::executor::Result<ftd::executor::Value<Option<AlignSelf>>> {
        let or_type_value = ftd::executor::value::optional_or_type(
            key,
            properties,
            arguments,
            doc,
            line_number,
            ftd::interpreter2::FTD_ALIGN_SELF,
            inherited_variables,
        )?;

        Ok(ftd::executor::Value::new(
            AlignSelf::from_optional_values(or_type_value.value, doc, line_number)?,
            or_type_value.line_number,
            or_type_value.properties,
        ))
    }

    pub fn to_css_string(&self) -> String {
        match self {
            AlignSelf::Start => "start".to_string(),
            AlignSelf::Center => "center".to_string(),
            AlignSelf::End => "end".to_string(),
        }
    }
}

#[derive(serde::Deserialize, Debug, PartialEq, Clone, serde::Serialize)]
pub enum Overflow {
    Scroll,
    Visible,
    Hidden,
    Auto,
}

impl Overflow {
    fn from_optional_values(
        or_type_value: Option<(String, ftd::interpreter2::PropertyValue)>,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Option<Self>> {
        if let Some(value) = or_type_value {
            Ok(Some(Overflow::from_values(value, doc, line_number)?))
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
            ftd::interpreter2::FTD_OVERFLOW_SCROLL => Ok(Overflow::Scroll),
            ftd::interpreter2::FTD_OVERFLOW_VISIBLE => Ok(Overflow::Visible),
            ftd::interpreter2::FTD_OVERFLOW_HIDDEN => Ok(Overflow::Hidden),
            ftd::interpreter2::FTD_OVERFLOW_AUTO => Ok(Overflow::Auto),
            t => ftd::executor::utils::parse_error(
                format!("Unknown variant `{}` for or-type `ftd.overflow`", t),
                doc.name,
                line_number,
            ),
        }
    }

    pub(crate) fn optional_overflow(
        properties: &[ftd::interpreter2::Property],
        arguments: &[ftd::interpreter2::Argument],
        doc: &ftd::executor::TDoc,
        line_number: usize,
        key: &str,
        inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
    ) -> ftd::executor::Result<ftd::executor::Value<Option<Overflow>>> {
        let or_type_value = ftd::executor::value::optional_or_type(
            key,
            properties,
            arguments,
            doc,
            line_number,
            ftd::interpreter2::FTD_OVERFLOW,
            inherited_variables,
        )?;

        Ok(ftd::executor::Value::new(
            Overflow::from_optional_values(or_type_value.value, doc, line_number)?,
            or_type_value.line_number,
            or_type_value.properties,
        ))
    }

    pub fn to_css_string(&self) -> String {
        match self {
            Overflow::Scroll => "scroll".to_string(),
            Overflow::Visible => "visible".to_string(),
            Overflow::Hidden => "hidden".to_string(),
            Overflow::Auto => "auto".to_string(),
        }
    }
}

#[derive(serde::Deserialize, Debug, PartialEq, Clone, serde::Serialize)]
pub enum Resize {
    Both,
    Horizontal,
    Vertical,
}

impl Resize {
    fn from_optional_values(
        or_type_value: Option<(String, ftd::interpreter2::PropertyValue)>,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Option<Self>> {
        if let Some(value) = or_type_value {
            Ok(Some(Resize::from_values(value, doc, line_number)?))
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
            ftd::interpreter2::FTD_RESIZE_HORIZONTAL => Ok(Resize::Horizontal),
            ftd::interpreter2::FTD_RESIZE_VERTICAL => Ok(Resize::Vertical),
            ftd::interpreter2::FTD_RESIZE_BOTH => Ok(Resize::Both),
            t => ftd::executor::utils::parse_error(
                format!("Unknown variant `{}` for or-type `ftd.resize`", t),
                doc.name,
                line_number,
            ),
        }
    }

    pub(crate) fn optional_resize(
        properties: &[ftd::interpreter2::Property],
        arguments: &[ftd::interpreter2::Argument],
        doc: &ftd::executor::TDoc,
        line_number: usize,
        key: &str,
        inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
    ) -> ftd::executor::Result<ftd::executor::Value<Option<Resize>>> {
        let or_type_value = ftd::executor::value::optional_or_type(
            key,
            properties,
            arguments,
            doc,
            line_number,
            ftd::interpreter2::FTD_RESIZE,
            inherited_variables,
        )?;

        Ok(ftd::executor::Value::new(
            Resize::from_optional_values(or_type_value.value, doc, line_number)?,
            or_type_value.line_number,
            or_type_value.properties,
        ))
    }

    pub fn to_css_string(&self) -> String {
        match self {
            Resize::Horizontal => "horizontal".to_string(),
            Resize::Vertical => "vertical".to_string(),
            Resize::Both => "both".to_string(),
        }
    }
}

#[derive(serde::Deserialize, Debug, PartialEq, Clone, serde::Serialize)]
pub enum TextAlign {
    Start,
    Center,
    End,
    Justify,
}

impl TextAlign {
    fn from_optional_values(
        or_type_value: Option<(String, ftd::interpreter2::PropertyValue)>,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Option<Self>> {
        if let Some(value) = or_type_value {
            Ok(Some(TextAlign::from_values(value, doc, line_number)?))
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
            ftd::interpreter2::FTD_TEXT_ALIGN_START => Ok(TextAlign::Start),
            ftd::interpreter2::FTD_TEXT_ALIGN_CENTER => Ok(TextAlign::Center),
            ftd::interpreter2::FTD_TEXT_ALIGN_END => Ok(TextAlign::End),
            ftd::interpreter2::FTD_TEXT_ALIGN_JUSTIFY => Ok(TextAlign::Justify),
            t => ftd::executor::utils::parse_error(
                format!("Unknown variant `{}` for or-type `ftd.text-align`", t),
                doc.name,
                line_number,
            ),
        }
    }

    pub(crate) fn optional_text_align(
        properties: &[ftd::interpreter2::Property],
        arguments: &[ftd::interpreter2::Argument],
        doc: &ftd::executor::TDoc,
        line_number: usize,
        key: &str,
        inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
    ) -> ftd::executor::Result<ftd::executor::Value<Option<TextAlign>>> {
        let or_type_value = ftd::executor::value::optional_or_type(
            key,
            properties,
            arguments,
            doc,
            line_number,
            ftd::interpreter2::FTD_TEXT_ALIGN,
            inherited_variables,
        )?;

        Ok(ftd::executor::Value::new(
            TextAlign::from_optional_values(or_type_value.value, doc, line_number)?,
            or_type_value.line_number,
            or_type_value.properties,
        ))
    }

    pub fn to_css_string(&self) -> String {
        match self {
            TextAlign::Start => "start".to_string(),
            TextAlign::Center => "center".to_string(),
            TextAlign::End => "end".to_string(),
            TextAlign::Justify => "justify".to_string(),
        }
    }
}

#[derive(serde::Deserialize, Debug, PartialEq, Clone, serde::Serialize)]
pub enum Cursor {
    Default,
    None,
    ContextMenu,
    Help,
    Pointer,
    Progress,
    Wait,
    Cell,
    CrossHair,
    Text,
    VerticalText,
    Alias,
    Copy,
    Move,
    NoDrop,
    NotAllowed,
    Grab,
    Grabbing,
    EResize,
    NResize,
    NeResize,
    NwResize,
    SResize,
    SeResize,
    SwResize,
    WResize,
    EwResize,
    NsResize,
    NeswResize,
    NwseResize,
    ColResize,
    RowResize,
    AllScroll,
    ZoomIn,
    ZoomOut,
}

impl Cursor {
    fn from_optional_values(
        or_type_value: Option<(String, ftd::interpreter2::PropertyValue)>,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Option<Self>> {
        if let Some(value) = or_type_value {
            Ok(Some(Cursor::from_values(value, doc, line_number)?))
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
            ftd::interpreter2::FTD_CURSOR_DEFAULT => Ok(Cursor::Default),
            ftd::interpreter2::FTD_CURSOR_NONE => Ok(Cursor::None),
            ftd::interpreter2::FTD_CURSOR_CONTEXT_MENU => Ok(Cursor::ContextMenu),
            ftd::interpreter2::FTD_CURSOR_HELP => Ok(Cursor::Help),
            ftd::interpreter2::FTD_CURSOR_POINTER => Ok(Cursor::Pointer),
            ftd::interpreter2::FTD_CURSOR_PROGRESS => Ok(Cursor::Progress),
            ftd::interpreter2::FTD_CURSOR_WAIT => Ok(Cursor::Wait),
            ftd::interpreter2::FTD_CURSOR_CELL => Ok(Cursor::Cell),
            ftd::interpreter2::FTD_CURSOR_CROSSHAIR => Ok(Cursor::CrossHair),
            ftd::interpreter2::FTD_CURSOR_TEXT => Ok(Cursor::Text),
            ftd::interpreter2::FTD_CURSOR_VERTICAL_TEXT => Ok(Cursor::VerticalText),
            ftd::interpreter2::FTD_CURSOR_ALIAS => Ok(Cursor::Alias),
            ftd::interpreter2::FTD_CURSOR_COPY => Ok(Cursor::Copy),
            ftd::interpreter2::FTD_CURSOR_MOVE => Ok(Cursor::Move),
            ftd::interpreter2::FTD_CURSOR_NO_DROP => Ok(Cursor::NoDrop),
            ftd::interpreter2::FTD_CURSOR_NOT_ALLOWED => Ok(Cursor::NotAllowed),
            ftd::interpreter2::FTD_CURSOR_GRAB => Ok(Cursor::Grab),
            ftd::interpreter2::FTD_CURSOR_GRABBING => Ok(Cursor::Grabbing),
            ftd::interpreter2::FTD_CURSOR_E_RESIZE => Ok(Cursor::EResize),
            ftd::interpreter2::FTD_CURSOR_N_RESIZE => Ok(Cursor::NResize),
            ftd::interpreter2::FTD_CURSOR_NE_RESIZE => Ok(Cursor::NeResize),
            ftd::interpreter2::FTD_CURSOR_NW_RESIZE => Ok(Cursor::NwResize),
            ftd::interpreter2::FTD_CURSOR_S_RESIZE => Ok(Cursor::SResize),
            ftd::interpreter2::FTD_CURSOR_SE_RESIZE => Ok(Cursor::SeResize),
            ftd::interpreter2::FTD_CURSOR_SW_RESIZE => Ok(Cursor::SwResize),
            ftd::interpreter2::FTD_CURSOR_W_RESIZE => Ok(Cursor::WResize),
            ftd::interpreter2::FTD_CURSOR_EW_RESIZE => Ok(Cursor::EwResize),
            ftd::interpreter2::FTD_CURSOR_NS_RESIZE => Ok(Cursor::NsResize),
            ftd::interpreter2::FTD_CURSOR_NESW_RESIZE => Ok(Cursor::NeswResize),
            ftd::interpreter2::FTD_CURSOR_NWSE_RESIZE => Ok(Cursor::NwseResize),
            ftd::interpreter2::FTD_CURSOR_COL_RESIZE => Ok(Cursor::ColResize),
            ftd::interpreter2::FTD_CURSOR_ROW_RESIZE => Ok(Cursor::RowResize),
            ftd::interpreter2::FTD_CURSOR_ALL_SCROLL => Ok(Cursor::AllScroll),
            ftd::interpreter2::FTD_CURSOR_ZOOM_IN => Ok(Cursor::ZoomIn),
            ftd::interpreter2::FTD_CURSOR_ZOOM_OUT => Ok(Cursor::ZoomOut),
            t => ftd::executor::utils::parse_error(
                format!("Unknown variant `{}` for or-type `ftd.cursor`", t),
                doc.name,
                line_number,
            ),
        }
    }

    pub(crate) fn optional_cursor(
        properties: &[ftd::interpreter2::Property],
        arguments: &[ftd::interpreter2::Argument],
        doc: &ftd::executor::TDoc,
        line_number: usize,
        key: &str,
        inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
    ) -> ftd::executor::Result<ftd::executor::Value<Option<Cursor>>> {
        let or_type_value = ftd::executor::value::optional_or_type(
            key,
            properties,
            arguments,
            doc,
            line_number,
            ftd::interpreter2::FTD_CURSOR,
            inherited_variables,
        )?;

        Ok(ftd::executor::Value::new(
            Cursor::from_optional_values(or_type_value.value, doc, line_number)?,
            or_type_value.line_number,
            or_type_value.properties,
        ))
    }

    pub fn to_css_string(&self) -> String {
        match self {
            Cursor::Default => "default".to_string(),
            Cursor::None => "none".to_string(),
            Cursor::ContextMenu => "context-menu".to_string(),
            Cursor::Help => "help".to_string(),
            Cursor::Pointer => "pointer".to_string(),
            Cursor::Progress => "progress".to_string(),
            Cursor::Wait => "wait".to_string(),
            Cursor::Cell => "cell".to_string(),
            Cursor::CrossHair => "crosshair".to_string(),
            Cursor::Text => "text".to_string(),
            Cursor::VerticalText => "vertical-text".to_string(),
            Cursor::Alias => "alias".to_string(),
            Cursor::Copy => "copy".to_string(),
            Cursor::Move => "move".to_string(),
            Cursor::NoDrop => "no-drop".to_string(),
            Cursor::NotAllowed => "not-allowed".to_string(),
            Cursor::Grab => "grab".to_string(),
            Cursor::Grabbing => "grabbing".to_string(),
            Cursor::EResize => "e-resize".to_string(),
            Cursor::NResize => "n-resize".to_string(),
            Cursor::NeResize => "ne-resize".to_string(),
            Cursor::NwResize => "nw-resize".to_string(),
            Cursor::SResize => "s-resize".to_string(),
            Cursor::SeResize => "se-resize".to_string(),
            Cursor::SwResize => "sw-resize".to_string(),
            Cursor::WResize => "w-resize".to_string(),
            Cursor::EwResize => "ew-resize".to_string(),
            Cursor::NsResize => "ns-resize".to_string(),
            Cursor::NeswResize => "nesw-resize".to_string(),
            Cursor::NwseResize => "nwse-resize".to_string(),
            Cursor::ColResize => "col-resize".to_string(),
            Cursor::RowResize => "row-resize".to_string(),
            Cursor::AllScroll => "all-scroll".to_string(),
            Cursor::ZoomIn => "zoom-in".to_string(),
            Cursor::ZoomOut => "zoom-out".to_string(),
        }
    }
}

#[derive(serde::Deserialize, Debug, PartialEq, Clone, serde::Serialize)]
pub enum FontSize {
    Px(i64),
    Em(f64),
    Rem(f64),
}

impl Default for FontSize {
    fn default() -> FontSize {
        FontSize::Px(0)
    }
}

impl FontSize {
    fn from_optional_value(
        value: ftd::interpreter2::PropertyValue,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Option<FontSize>> {
        let value = value.resolve(&doc.itdoc(), line_number)?;
        match value.inner() {
            Some(ftd::interpreter2::Value::OrType {
                name,
                variant,
                value,
                ..
            }) if name.eq(ftd::interpreter2::FTD_FONT_SIZE) => Ok(Some(FontSize::from_values(
                (variant, value.as_ref().to_owned()),
                doc,
                line_number,
            )?)),
            None => Ok(None),
            t => ftd::executor::utils::parse_error(
                format!(
                    "Expected value of font-size or-type `{}`, found: {:?}",
                    ftd::interpreter2::FTD_FONT_SIZE,
                    t
                ),
                doc.name,
                line_number,
            ),
        }
    }

    fn from_values(
        or_type_value: (String, ftd::interpreter2::PropertyValue),
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Self> {
        match or_type_value.0.as_str() {
            ftd::interpreter2::FTD_FONT_SIZE_PX => Ok(FontSize::Px(
                or_type_value
                    .1
                    .clone()
                    .resolve(&doc.itdoc(), line_number)?
                    .integer(doc.name, line_number)?,
            )),
            ftd::interpreter2::FTD_FONT_SIZE_EM => Ok(FontSize::Em(
                or_type_value
                    .1
                    .clone()
                    .resolve(&doc.itdoc(), line_number)?
                    .decimal(doc.name, line_number)?,
            )),
            ftd::interpreter2::FTD_FONT_SIZE_REM => Ok(FontSize::Rem(
                or_type_value
                    .1
                    .clone()
                    .resolve(&doc.itdoc(), line_number)?
                    .decimal(doc.name, line_number)?,
            )),
            t => ftd::executor::utils::parse_error(
                format!("Unknown variant `{}` for or-type `ftd.font-size`", t),
                doc.name,
                line_number,
            ),
        }
    }

    pub fn to_css_string(&self) -> String {
        match self {
            FontSize::Px(px) => format!("{}px", px),
            FontSize::Em(em) => format!("{}em", em),
            FontSize::Rem(rem) => format!("{}rem", rem),
        }
    }

    pub fn get_pattern_from_variant_str(
        variant: &str,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::executor::Result<&'static str> {
        match variant {
            ftd::interpreter2::FTD_FONT_SIZE_PX
            | ftd::interpreter2::FTD_FONT_SIZE_EM
            | ftd::interpreter2::FTD_FONT_SIZE_REM => Ok("{0}"),
            t => ftd::executor::utils::parse_error(
                format!("Unknown variant found for ftd.font-size: `{}`", t),
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
            ftd::interpreter2::FTD_FONT_SIZE_PX => Ok("{0}px"),
            ftd::interpreter2::FTD_FONT_SIZE_EM => Ok("{0}em"),
            ftd::interpreter2::FTD_FONT_SIZE_REM => Ok("{0}rem"),
            t => ftd::executor::utils::parse_error(
                format!("Unknown variant found for ftd.font-size: `{}`", t),
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
            ftd::interpreter2::FTD_FONT_SIZE_PX => Ok(format!("{}px", value)),
            ftd::interpreter2::FTD_FONT_SIZE_EM => Ok(format!("{}em", value)),
            ftd::interpreter2::FTD_FONT_SIZE_REM => Ok(format!("{}rem", value)),
            t => ftd::executor::utils::parse_error(
                format!("Unknown variant found for ftd.font-size: `{}`", t),
                doc_id,
                line_number,
            ),
        }
    }
}

#[derive(serde::Deserialize, Debug, Default, PartialEq, Clone, serde::Serialize)]
pub struct Type {
    pub size: Option<FontSize>,
    pub line_height: Option<FontSize>,
    pub letter_spacing: Option<FontSize>,
    pub weight: Option<i64>,
    pub font_family: Option<String>,
}

impl Type {
    fn new(
        size: Option<FontSize>,
        line_height: Option<FontSize>,
        letter_spacing: Option<FontSize>,
        weight: Option<i64>,
        font_family: Option<String>,
    ) -> Type {
        Type {
            size,
            line_height,
            letter_spacing,
            weight,
            font_family,
        }
    }

    fn from_value(
        value: ftd::interpreter2::PropertyValue,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Type> {
        let value = value.resolve(&doc.itdoc(), line_number)?;
        let fields = match value.inner() {
            Some(ftd::interpreter2::Value::Record { name, fields })
                if name.eq(ftd::interpreter2::FTD_TYPE) =>
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
        Type::from_values(fields, doc, line_number)
    }

    fn from_values(
        values: ftd::Map<ftd::interpreter2::PropertyValue>,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Type> {
        let size = {
            if let Some(value) = values.get("size") {
                FontSize::from_optional_value(value.to_owned(), doc, line_number)?
            } else {
                None
            }
        };

        let line_height = {
            if let Some(value) = values.get("line-height") {
                FontSize::from_optional_value(value.to_owned(), doc, line_number)?
            } else {
                None
            }
        };

        let letter_spacing = {
            if let Some(value) = values.get("letter-spacing") {
                FontSize::from_optional_value(value.to_owned(), doc, line_number)?
            } else {
                None
            }
        };

        let weight = {
            if let Some(value) = values.get("weight") {
                value
                    .clone()
                    .resolve(&doc.itdoc(), line_number)?
                    .optional_integer(doc.name, line_number)?
            } else {
                None
            }
        };

        let font_family = {
            if let Some(value) = values.get("font-family") {
                value
                    .clone()
                    .resolve(&doc.itdoc(), line_number)?
                    .optional_string(doc.name, line_number)?
            } else {
                None
            }
        };

        Ok(Type::new(
            size,
            line_height,
            letter_spacing,
            weight,
            font_family,
        ))
    }
}

#[derive(serde::Deserialize, Debug, Default, PartialEq, Clone, serde::Serialize)]
pub struct ResponsiveType {
    pub desktop: Type,
    pub mobile: Type,
}

impl ResponsiveType {
    fn from_values(
        values: ftd::Map<ftd::interpreter2::PropertyValue>,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<ResponsiveType> {
        let desktop = {
            let value = values
                .get("desktop")
                .ok_or(ftd::executor::Error::ParseError {
                    message: "`desktop` field in ftd.responsive-type not found".to_string(),
                    doc_id: doc.name.to_string(),
                    line_number,
                })?;
            Type::from_value(value.to_owned(), doc, line_number)?
        };

        let mobile = {
            if let Some(value) = values.get("mobile") {
                Type::from_value(value.to_owned(), doc, line_number)?
            } else {
                desktop.clone()
            }
        };

        Ok(ResponsiveType { desktop, mobile })
    }

    fn from_optional_values(
        or_type_value: Option<ftd::Map<ftd::interpreter2::PropertyValue>>,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Option<Self>> {
        if let Some(value) = or_type_value {
            Ok(Some(ResponsiveType::from_values(value, doc, line_number)?))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn optional_responsive_type(
        properties: &[ftd::interpreter2::Property],
        arguments: &[ftd::interpreter2::Argument],
        doc: &ftd::executor::TDoc,
        line_number: usize,
        key: &str,
        inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
    ) -> ftd::executor::Result<ftd::executor::Value<Option<ResponsiveType>>> {
        let record_values = ftd::executor::value::optional_record_inherited(
            key,
            properties,
            arguments,
            doc,
            line_number,
            ftd::interpreter2::FTD_RESPONSIVE_TYPE,
            inherited_variables,
        )?;

        Ok(ftd::executor::Value::new(
            ResponsiveType::from_optional_values(record_values.value, doc, line_number)?,
            record_values.line_number,
            record_values.properties,
        ))
    }

    pub fn to_css_font_size(&self) -> Option<String> {
        self.desktop.size.as_ref().map(|v| v.to_css_string())
    }

    pub fn font_size_pattern() -> (String, bool) {
        ("({0})[\"size\"]".to_string(), true)
    }

    pub fn to_css_line_height(&self) -> Option<String> {
        self.desktop.line_height.as_ref().map(|v| v.to_css_string())
    }

    pub fn line_height_pattern() -> (String, bool) {
        ("({0})[\"line-height\"]".to_string(), true)
    }

    pub fn to_css_letter_spacing(&self) -> Option<String> {
        self.desktop
            .letter_spacing
            .as_ref()
            .map(|v| v.to_css_string())
    }

    pub fn letter_spacing_pattern() -> (String, bool) {
        ("({0})[\"letter-spacing\"]".to_string(), true)
    }

    pub fn to_css_weight(&self) -> Option<String> {
        self.desktop.weight.as_ref().map(|v| v.to_string())
    }

    pub fn weight_pattern() -> (String, bool) {
        ("({0}).weight".to_string(), true)
    }

    pub fn to_css_font_family(&self) -> Option<String> {
        self.desktop.font_family.to_owned()
    }

    pub fn font_family_pattern() -> (String, bool) {
        ("({0})[\"font-family\"]".to_string(), true)
    }
}

#[derive(serde::Deserialize, Debug, PartialEq, Clone, serde::Serialize)]
pub enum Anchor {
    Window,
    Parent,
    Id(String),
}

impl Anchor {
    fn from_optional_values(
        or_type_value: Option<(String, ftd::interpreter2::PropertyValue)>,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Option<Self>> {
        if let Some(value) = or_type_value {
            Ok(Some(Anchor::from_values(value, doc, line_number)?))
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
            ftd::interpreter2::FTD_ANCHOR_WINDOW => Ok(Anchor::Window),
            ftd::interpreter2::FTD_ANCHOR_PARENT => Ok(Anchor::Parent),
            ftd::interpreter2::FTD_ANCHOR_ID => Ok(Anchor::Id(
                or_type_value
                    .1
                    .clone()
                    .resolve(&doc.itdoc(), line_number)?
                    .string(doc.name, line_number)?,
            )),
            t => ftd::executor::utils::parse_error(
                format!("Unknown variant `{}` for or-type `ftd.anchor`", t),
                doc.name,
                line_number,
            ),
        }
    }

    pub(crate) fn optional_anchor(
        properties: &[ftd::interpreter2::Property],
        arguments: &[ftd::interpreter2::Argument],
        doc: &ftd::executor::TDoc,
        line_number: usize,
        key: &str,
        inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
    ) -> ftd::executor::Result<ftd::executor::Value<Option<Anchor>>> {
        let or_type_value = ftd::executor::value::optional_or_type(
            key,
            properties,
            arguments,
            doc,
            line_number,
            ftd::interpreter2::FTD_ANCHOR,
            inherited_variables,
        )?;

        Ok(ftd::executor::Value::new(
            Anchor::from_optional_values(or_type_value.value, doc, line_number)?,
            or_type_value.line_number,
            or_type_value.properties,
        ))
    }

    pub fn to_css_string(&self) -> String {
        match self {
            Anchor::Window => "fixed".to_string(),
            Anchor::Parent => "absolute".to_string(),
            Anchor::Id(_) => "absolute".to_string(),
        }
    }
}

#[derive(serde::Deserialize, Debug, PartialEq, Clone, serde::Serialize)]
pub enum TextInputType {
    TEXT,
    EMAIL,
    PASSWORD,
    URL,
}

impl TextInputType {
    fn from_optional_values(
        or_type_value: Option<(String, ftd::interpreter2::PropertyValue)>,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Option<Self>> {
        if let Some(value) = or_type_value {
            Ok(Some(TextInputType::from_values(value, doc, line_number)?))
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
            ftd::interpreter2::FTD_TEXT_INPUT_TYPE_TEXT => Ok(TextInputType::TEXT),
            ftd::interpreter2::FTD_TEXT_INPUT_TYPE_EMAIL => Ok(TextInputType::EMAIL),
            ftd::interpreter2::FTD_TEXT_INPUT_TYPE_PASSWORD => Ok(TextInputType::PASSWORD),
            ftd::interpreter2::FTD_TEXT_INPUT_TYPE_URL => Ok(TextInputType::URL),
            t => ftd::executor::utils::parse_error(
                format!("Unknown variant `{}` for or-type `ftd.text-input-type`", t),
                doc.name,
                line_number,
            ),
        }
    }

    pub(crate) fn optional_text_input_type(
        properties: &[ftd::interpreter2::Property],
        arguments: &[ftd::interpreter2::Argument],
        doc: &ftd::executor::TDoc,
        line_number: usize,
        key: &str,
        inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
    ) -> ftd::executor::Result<ftd::executor::Value<Option<TextInputType>>> {
        let or_type_value = ftd::executor::value::optional_or_type(
            key,
            properties,
            arguments,
            doc,
            line_number,
            ftd::interpreter2::FTD_TEXT_INPUT_TYPE,
            inherited_variables,
        )?;

        Ok(ftd::executor::Value::new(
            TextInputType::from_optional_values(or_type_value.value, doc, line_number)?,
            or_type_value.line_number,
            or_type_value.properties,
        ))
    }

    pub fn to_css_string(&self) -> String {
        match self {
            TextInputType::TEXT => "text".to_string(),
            TextInputType::EMAIL => "email".to_string(),
            TextInputType::PASSWORD => "password".to_string(),
            TextInputType::URL => "url".to_string(),
        }
    }
}

#[derive(serde::Deserialize, Debug, PartialEq, Clone, serde::Serialize)]
pub enum Region {
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
}

impl Region {
    fn from_optional_values(
        or_type_value: Option<(String, ftd::interpreter2::PropertyValue)>,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Option<Self>> {
        if let Some(value) = or_type_value {
            Ok(Some(Region::from_values(value, doc, line_number)?))
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
            ftd::interpreter2::FTD_REGION_H1 => Ok(Region::H1),
            ftd::interpreter2::FTD_REGION_H2 => Ok(Region::H2),
            ftd::interpreter2::FTD_REGION_H3 => Ok(Region::H3),
            ftd::interpreter2::FTD_REGION_H4 => Ok(Region::H4),
            ftd::interpreter2::FTD_REGION_H5 => Ok(Region::H5),
            ftd::interpreter2::FTD_REGION_H6 => Ok(Region::H6),
            t => ftd::executor::utils::parse_error(
                format!("Unknown variant `{}` for or-type `ftd.region`", t),
                doc.name,
                line_number,
            ),
        }
    }

    pub(crate) fn optional_region(
        properties: &[ftd::interpreter2::Property],
        arguments: &[ftd::interpreter2::Argument],
        doc: &ftd::executor::TDoc,
        line_number: usize,
        key: &str,
        inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
    ) -> ftd::executor::Result<ftd::executor::Value<Option<Region>>> {
        let or_type_value = ftd::executor::value::optional_or_type(
            key,
            properties,
            arguments,
            doc,
            line_number,
            ftd::interpreter2::FTD_REGION,
            inherited_variables,
        )?;

        Ok(ftd::executor::Value::new(
            Region::from_optional_values(or_type_value.value, doc, line_number)?,
            or_type_value.line_number,
            or_type_value.properties,
        ))
    }

    // For now, only components with region h1 and h2 will have auto generated ids
    // if there is no user defined id
    pub fn is_heading(&self) -> bool {
        matches!(self, Region::H1 | Region::H2 | Region::H3 | Region::H4)
    }

    pub fn to_css_string(&self) -> String {
        match self {
            Region::H1 => "h1".to_string(),
            Region::H2 => "h2".to_string(),
            Region::H3 => "h3".to_string(),
            Region::H4 => "h4".to_string(),
            Region::H5 => "h5".to_string(),
            Region::H6 => "h6".to_string(),
        }
    }
}

#[derive(serde::Deserialize, Debug, PartialEq, Clone, serde::Serialize)]
pub enum WhiteSpace {
    NORMAL,
    NOWRAP,
    PRE,
    PREWRAP,
    PRELINE,
    BREAKSPACES,
}

impl WhiteSpace {
    fn from_optional_values(
        or_type_value: Option<(String, ftd::interpreter2::PropertyValue)>,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Option<Self>> {
        if let Some(value) = or_type_value {
            Ok(Some(WhiteSpace::from_values(value, doc, line_number)?))
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
            ftd::interpreter2::FTD_WHITESPACE_NORMAL => Ok(WhiteSpace::NORMAL),
            ftd::interpreter2::FTD_WHITESPACE_NOWRAP => Ok(WhiteSpace::NOWRAP),
            ftd::interpreter2::FTD_WHITESPACE_PRE => Ok(WhiteSpace::PRE),
            ftd::interpreter2::FTD_WHITESPACE_PREWRAP => Ok(WhiteSpace::PREWRAP),
            ftd::interpreter2::FTD_WHITESPACE_PRELINE => Ok(WhiteSpace::PRELINE),
            ftd::interpreter2::FTD_WHITESPACE_BREAKSPACES => Ok(WhiteSpace::BREAKSPACES),
            t => ftd::executor::utils::parse_error(
                format!("Unknown variant `{}` for or-type `ftd.whitespace`", t),
                doc.name,
                line_number,
            ),
        }
    }

    pub(crate) fn optional_whitespace(
        properties: &[ftd::interpreter2::Property],
        arguments: &[ftd::interpreter2::Argument],
        doc: &ftd::executor::TDoc,
        line_number: usize,
        key: &str,
        inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
    ) -> ftd::executor::Result<ftd::executor::Value<Option<WhiteSpace>>> {
        let or_type_value = ftd::executor::value::optional_or_type(
            key,
            properties,
            arguments,
            doc,
            line_number,
            ftd::interpreter2::FTD_WHITESPACE,
            inherited_variables,
        )?;

        Ok(ftd::executor::Value::new(
            WhiteSpace::from_optional_values(or_type_value.value, doc, line_number)?,
            or_type_value.line_number,
            or_type_value.properties,
        ))
    }

    pub fn to_css_string(&self) -> String {
        match self {
            WhiteSpace::NORMAL => "normal".to_string(),
            WhiteSpace::NOWRAP => "nowrap".to_string(),
            WhiteSpace::PRE => "pre".to_string(),
            WhiteSpace::PRELINE => "pre-line".to_string(),
            WhiteSpace::PREWRAP => "pre-wrap".to_string(),
            WhiteSpace::BREAKSPACES => "break-spaces".to_string(),
        }
    }
}

#[derive(serde::Deserialize, Debug, PartialEq, Clone, serde::Serialize)]
pub enum TextTransform {
    NONE,
    CAPITALIZE,
    UPPERCASE,
    LOWERCASE,
    INITIAL,
    INHERIT,
}

impl TextTransform {
    fn from_optional_values(
        or_type_value: Option<(String, ftd::interpreter2::PropertyValue)>,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Option<Self>> {
        if let Some(value) = or_type_value {
            Ok(Some(TextTransform::from_values(value, doc, line_number)?))
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
            ftd::interpreter2::FTD_TEXT_TRANSFORM_NONE => Ok(TextTransform::NONE),
            ftd::interpreter2::FTD_TEXT_TRANSFORM_CAPITALIZE => Ok(TextTransform::CAPITALIZE),
            ftd::interpreter2::FTD_TEXT_TRANSFORM_UPPERCASE => Ok(TextTransform::UPPERCASE),
            ftd::interpreter2::FTD_TEXT_TRANSFORM_LOWERCASE => Ok(TextTransform::LOWERCASE),
            ftd::interpreter2::FTD_TEXT_TRANSFORM_INITIAL => Ok(TextTransform::INITIAL),
            ftd::interpreter2::FTD_TEXT_TRANSFORM_INHERIT => Ok(TextTransform::INHERIT),
            t => ftd::executor::utils::parse_error(
                format!("Unknown variant `{}` for or-type `ftd.text-transform`", t),
                doc.name,
                line_number,
            ),
        }
    }

    pub(crate) fn optional_text_transform(
        properties: &[ftd::interpreter2::Property],
        arguments: &[ftd::interpreter2::Argument],
        doc: &ftd::executor::TDoc,
        line_number: usize,
        key: &str,
        inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
    ) -> ftd::executor::Result<ftd::executor::Value<Option<TextTransform>>> {
        let or_type_value = ftd::executor::value::optional_or_type(
            key,
            properties,
            arguments,
            doc,
            line_number,
            ftd::interpreter2::FTD_TEXT_TRANSFORM,
            inherited_variables,
        )?;

        Ok(ftd::executor::Value::new(
            TextTransform::from_optional_values(or_type_value.value, doc, line_number)?,
            or_type_value.line_number,
            or_type_value.properties,
        ))
    }

    pub fn to_css_string(&self) -> String {
        match self {
            TextTransform::NONE => "none".to_string(),
            TextTransform::CAPITALIZE => "capitalize".to_string(),
            TextTransform::UPPERCASE => "uppercase".to_string(),
            TextTransform::LOWERCASE => "lowercase".to_string(),
            TextTransform::INHERIT => "inherit".to_string(),
            TextTransform::INITIAL => "initial".to_string(),
        }
    }
}

#[derive(serde::Deserialize, Debug, PartialEq, Clone, serde::Serialize)]
pub enum BorderStyle {
    DOTTED,
    DASHED,
    SOLID,
    DOUBLE,
    GROOVE,
    RIDGE,
    INSET,
    OUTSET,
}

impl BorderStyle {
    fn from_optional_values(
        or_type_value: Option<(String, ftd::interpreter2::PropertyValue)>,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Option<Self>> {

        if let Some(value) = or_type_value {
            return Ok(Some(BorderStyle::from_values(value, doc, line_number)?));
        }
        Ok(None)
    }

    fn from_values(
        or_type_value: (String, ftd::interpreter2::PropertyValue),
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Self> {
        match or_type_value.0.as_str() {
            ftd::interpreter2::FTD_BORDER_STYLE_DOTTED => Ok(BorderStyle::DOTTED),
            ftd::interpreter2::FTD_BORDER_STYLE_DASHED => Ok(BorderStyle::DASHED),
            ftd::interpreter2::FTD_BORDER_STYLE_SOLID => Ok(BorderStyle::SOLID),
            ftd::interpreter2::FTD_BORDER_STYLE_GROOVE => Ok(BorderStyle::GROOVE),
            ftd::interpreter2::FTD_BORDER_STYLE_RIDGE => Ok(BorderStyle::RIDGE),
            ftd::interpreter2::FTD_BORDER_STYLE_OUTSET => Ok(BorderStyle::OUTSET),
            ftd::interpreter2::FTD_BORDER_STYLE_INSET => Ok(BorderStyle::INSET),
            ftd::interpreter2::FTD_BORDER_STYLE_DOUBLE => Ok(BorderStyle::DOUBLE),
            t => ftd::executor::utils::parse_error(
                format!("Unknown variant `{}` for or-type `ftd.border-style`", t),
                doc.name,
                line_number,
            ),
        }
    }

    pub(crate) fn optional_border_style(
        properties: &[ftd::interpreter2::Property],
        arguments: &[ftd::interpreter2::Argument],
        doc: &ftd::executor::TDoc,
        line_number: usize,
        key: &str,
        inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
    ) -> ftd::executor::Result<ftd::executor::Value<Option<BorderStyle>>> {
        let or_type_value = ftd::executor::value::optional_or_type(
            key,
            properties,
            arguments,
            doc,
            line_number,
            ftd::interpreter2::FTD_BORDER_STYLE,
            inherited_variables,
        )?;

        Ok(ftd::executor::Value::new(
            BorderStyle::from_optional_values(or_type_value.value, doc, line_number)?,
            or_type_value.line_number,
            or_type_value.properties,
        ))
    }

    pub fn to_css_string(&self) -> String {
        match self {
            BorderStyle::DOTTED => "dotted".to_string(),
            BorderStyle::DASHED => "dashed".to_string(),
            BorderStyle::SOLID => "solid".to_string(),
            BorderStyle::DOUBLE => "double".to_string(),
            BorderStyle::GROOVE => "groove".to_string(),
            BorderStyle::RIDGE => "ridge".to_string(),
            BorderStyle::INSET => "inset".to_string(),
            BorderStyle::OUTSET => "outset".to_string(),
        }
    }
}

/// https://html.spec.whatwg.org/multipage/urls-and-fetching.html#lazy-loading-attributes
#[derive(serde::Deserialize, Debug, Default, PartialEq, Clone, serde::Serialize)]
#[serde(tag = "type")]
pub enum Loading {
    #[default]
    Lazy,
    Eager,
}

impl Loading {
    fn from_optional_values(
        or_type_value: Option<(String, ftd::interpreter2::PropertyValue)>,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Option<Self>> {
        if let Some(value) = or_type_value {
            Ok(Some(Loading::from_values(value, doc, line_number)?))
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
            ftd::interpreter2::FTD_LOADING_LAZY => Ok(Loading::Lazy),
            ftd::interpreter2::FTD_LOADING_EAGER => Ok(Loading::Eager),
            t => ftd::executor::utils::parse_error(
                format!(
                    "Unknown variant `{}` for or-type `ftd.loading`. Help: use `lazy` or `eager`",
                    t
                ),
                doc.name,
                line_number,
            ),
        }
    }

    pub(crate) fn loading_with_default(
        properties: &[ftd::interpreter2::Property],
        arguments: &[ftd::interpreter2::Argument],
        doc: &ftd::executor::TDoc,
        line_number: usize,
        key: &str,
        inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
    ) -> ftd::executor::Result<ftd::executor::Value<Loading>> {
        let or_type_value = ftd::executor::value::optional_or_type(
            key,
            properties,
            arguments,
            doc,
            line_number,
            ftd::interpreter2::FTD_LOADING,
            inherited_variables,
        )?;

        Ok(ftd::executor::Value::new(
            Loading::from_optional_values(or_type_value.value, doc, line_number)?
                .unwrap_or_default(),
            or_type_value.line_number,
            or_type_value.properties,
        ))
    }

    pub fn to_css_string(&self) -> String {
        match self {
            Loading::Lazy => "lazy".to_string(),
            Loading::Eager => "eager".to_string(),
        }
    }
}

pub struct LineClamp;

impl LineClamp {
    pub(crate) fn display_pattern() -> (String, bool) {
        ("-webkit-box".to_string(), false)
    }

    pub(crate) fn overflow_pattern() -> (String, bool) {
        ("hidden".to_string(), false)
    }

    pub(crate) fn webkit_box_orient_pattern() -> (String, bool) {
        ("vertical".to_string(), false)
    }
}
