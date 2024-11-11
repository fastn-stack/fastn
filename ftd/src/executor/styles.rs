use itertools::Itertools;

#[derive(serde::Deserialize, Debug, PartialEq, Clone, serde::Serialize)]
pub enum Length {
    Px(i64),
    Percent(f64),
    Calc(String),
    Vh(f64),
    Vw(f64),
    Vmin(f64),
    Vmax(f64),
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
        or_type_value: Option<(String, fastn_type::PropertyValue)>,
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
        value: fastn_type::PropertyValue,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Length> {
        let binding = value.resolve(&doc.itdoc(), line_number)?;
        let value = binding.get_or_type(doc.name, line_number)?;
        let value = (value.1.to_owned(), value.2.to_owned());
        Length::from_values(value, doc, line_number)
    }

    fn from_optional_value(
        or_type_value: Option<fastn_type::PropertyValue>,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Option<Length>> {
        if let Some(value) = or_type_value {
            let binding = value.clone().resolve(&doc.itdoc(), line_number)?;
            if let fastn_type::Value::Optional { data, .. } = &binding {
                if data.is_none() {
                    return Ok(None);
                }
            }
            return Ok(Some(Length::from_value(value, doc, line_number)?));
        }
        Ok(None)
    }

    fn from_values(
        or_type_value: (String, fastn_type::PropertyValue),
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Length> {
        match or_type_value.0.as_str() {
            ftd::interpreter::FTD_LENGTH_PERCENT => Ok(Length::Percent(
                or_type_value
                    .1
                    .clone()
                    .resolve(&doc.itdoc(), line_number)?
                    .decimal(doc.name, line_number)?,
            )),
            ftd::interpreter::FTD_LENGTH_PX => Ok(Length::Px(
                or_type_value
                    .1
                    .clone()
                    .resolve(&doc.itdoc(), line_number)?
                    .integer(doc.name, line_number)?,
            )),
            ftd::interpreter::FTD_LENGTH_CALC => Ok(Length::Calc(
                or_type_value
                    .1
                    .clone()
                    .resolve(&doc.itdoc(), line_number)?
                    .string(doc.name, line_number)?,
            )),
            ftd::interpreter::FTD_LENGTH_VH => Ok(Length::Vh(
                or_type_value
                    .1
                    .clone()
                    .resolve(&doc.itdoc(), line_number)?
                    .decimal(doc.name, line_number)?,
            )),
            ftd::interpreter::FTD_LENGTH_VW => Ok(Length::Vw(
                or_type_value
                    .1
                    .clone()
                    .resolve(&doc.itdoc(), line_number)?
                    .decimal(doc.name, line_number)?,
            )),
            //
            ftd::interpreter::FTD_LENGTH_VMIN => Ok(Length::Vmin(
                or_type_value
                    .1
                    .clone()
                    .resolve(&doc.itdoc(), line_number)?
                    .decimal(doc.name, line_number)?,
            )),
            ftd::interpreter::FTD_LENGTH_VMAX => Ok(Length::Vmax(
                or_type_value
                    .1
                    .clone()
                    .resolve(&doc.itdoc(), line_number)?
                    .decimal(doc.name, line_number)?,
            )),
            ftd::interpreter::FTD_LENGTH_EM => Ok(Length::Em(
                or_type_value
                    .1
                    .clone()
                    .resolve(&doc.itdoc(), line_number)?
                    .decimal(doc.name, line_number)?,
            )),
            ftd::interpreter::FTD_LENGTH_REM => Ok(Length::Rem(
                or_type_value
                    .1
                    .clone()
                    .resolve(&doc.itdoc(), line_number)?
                    .decimal(doc.name, line_number)?,
            )),
            ftd::interpreter::FTD_LENGTH_RESPONSIVE => Ok(Length::Responsive(Box::new(
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
        properties: &[ftd::interpreter::Property],
        arguments: &[ftd::interpreter::Argument],
        doc: &ftd::executor::TDoc,
        line_number: usize,
        key: &str,
        inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
        component_name: &str,
    ) -> ftd::executor::Result<ftd::executor::Value<Option<Length>>> {
        let or_type_value = ftd::executor::value::optional_or_type(
            key,
            component_name,
            properties,
            arguments,
            doc,
            line_number,
            ftd::interpreter::FTD_LENGTH,
            inherited_variables,
        )?;

        Ok(ftd::executor::Value::new(
            Length::from_optional_values(or_type_value.value, doc, line_number)?,
            or_type_value.line_number,
            or_type_value.properties,
        ))
    }

    pub fn to_css_string(&self, device: &Option<ftd::executor::Device>) -> String {
        match self {
            Length::Px(px) => format!("{}px", px),
            Length::Percent(p) => format!("{}%", p),
            Length::Calc(calc) => format!("calc({})", calc),
            Length::Vh(vh) => format!("{}vh", vh),
            Length::Vw(vw) => format!("{}vw", vw),
            Length::Vmin(vmin) => format!("{}vmin", vmin),
            Length::Vmax(vmax) => format!("{}vmax", vmax),
            Length::Em(em) => format!("{}em", em),
            Length::Rem(rem) => format!("{}rem", rem),
            Length::Responsive(r) => match device {
                Some(ftd::executor::Device::Mobile) => r.mobile.to_css_string(device),
                _ => r.desktop.to_css_string(device),
            },
        }
    }

    pub fn get_pattern_from_variant_str(
        variant: &str,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::executor::Result<&'static str> {
        match variant {
            ftd::interpreter::FTD_LENGTH_PX
            | ftd::interpreter::FTD_LENGTH_PERCENT
            | ftd::interpreter::FTD_LENGTH_CALC
            | ftd::interpreter::FTD_LENGTH_VH
            | ftd::interpreter::FTD_LENGTH_VW
            | ftd::interpreter::FTD_LENGTH_VMIN
            | ftd::interpreter::FTD_LENGTH_VMAX
            | ftd::interpreter::FTD_LENGTH_EM
            | ftd::interpreter::FTD_LENGTH_REM => Ok("{0}"),
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
            ftd::interpreter::FTD_LENGTH_PX => Ok("{0}px"),
            ftd::interpreter::FTD_LENGTH_PERCENT => Ok("{0}%"),
            ftd::interpreter::FTD_LENGTH_CALC => Ok("calc({0})"),
            ftd::interpreter::FTD_LENGTH_VH => Ok("{0}vh"),
            ftd::interpreter::FTD_LENGTH_VW => Ok("{0}vw"),
            ftd::interpreter::FTD_LENGTH_VMIN => Ok("{0}vmin"),
            ftd::interpreter::FTD_LENGTH_VMAX => Ok("{0}vmax"),
            ftd::interpreter::FTD_LENGTH_EM => Ok("{0}em"),
            ftd::interpreter::FTD_LENGTH_REM => Ok("{0}rem"),
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
            ftd::interpreter::FTD_LENGTH_PX => Ok(format!("{}px", value)),
            ftd::interpreter::FTD_LENGTH_PERCENT => Ok(format!("{}%", value)),
            ftd::interpreter::FTD_LENGTH_CALC => Ok(format!("calc({})", value)),
            ftd::interpreter::FTD_LENGTH_VH => Ok(format!("{}vh", value)),
            ftd::interpreter::FTD_LENGTH_VW => Ok(format!("{}vw", value)),
            ftd::interpreter::FTD_LENGTH_VMIN => Ok(format!("{}vmin", value)),
            ftd::interpreter::FTD_LENGTH_VMAX => Ok(format!("{}vmax", value)),
            ftd::interpreter::FTD_LENGTH_EM => Ok(format!("{}em", value)),
            ftd::interpreter::FTD_LENGTH_REM => Ok(format!("{}rem", value)),
            t => ftd::executor::utils::parse_error(
                format!("Unknown variant found for ftd.length: `{}`", t),
                doc_id,
                line_number,
            ),
        }
    }
}

#[derive(serde::Deserialize, Debug, Default, PartialEq, Clone, serde::Serialize)]
pub struct LengthPair {
    pub x: Length,
    pub y: Length,
}

impl LengthPair {
    fn from_value(
        value: fastn_type::PropertyValue,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<LengthPair> {
        let value = value.resolve(&doc.itdoc(), line_number)?;
        let fields = match value.inner() {
            Some(fastn_type::Value::Record { name, fields })
                if name.eq(ftd::interpreter::FTD_LENGTH_PAIR)
                    || name.eq(ftd::interpreter::FTD_BACKGROUND_SIZE_LENGTH)
                    || name.eq(ftd::interpreter::FTD_BACKGROUND_POSITION_LENGTH) =>
            {
                fields
            }
            t => {
                return ftd::executor::utils::parse_error(
                    format!(
                        "Expected value of type record `{}`, found: {:?}",
                        ftd::interpreter::FTD_LENGTH_PAIR,
                        t
                    ),
                    doc.name,
                    line_number,
                )
            }
        };
        LengthPair::from_values(fields, doc, line_number)
    }

    fn from_values(
        values: ftd::Map<fastn_type::PropertyValue>,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<LengthPair> {
        let x = {
            let value = values.get("x").ok_or(ftd::executor::Error::ParseError {
                message: "`x` field in ftd.length-pair not found".to_string(),
                doc_id: doc.name.to_string(),
                line_number,
            })?;
            Length::from_value(value.to_owned(), doc, line_number)?
        };

        let y = {
            let value = values.get("y").ok_or(ftd::executor::Error::ParseError {
                message: "`y` field in ftd.length-pair not found".to_string(),
                doc_id: doc.name.to_string(),
                line_number,
            })?;
            Length::from_value(value.to_owned(), doc, line_number)?
        };

        Ok(LengthPair { x, y })
    }

    pub fn to_css_string(&self, device: &Option<ftd::executor::Device>) -> String {
        format!(
            "{} {}",
            self.x.to_css_string(device),
            self.y.to_css_string(device)
        )
    }
}

#[derive(serde::Deserialize, Debug, Default, PartialEq, Clone, serde::Serialize)]
pub struct ResponsiveLength {
    pub desktop: Length,
    pub mobile: Length,
}

impl ResponsiveLength {
    fn from_value(
        value: fastn_type::PropertyValue,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<ResponsiveLength> {
        let value = value.resolve(&doc.itdoc(), line_number)?;
        let fields = match value.inner() {
            Some(fastn_type::Value::Record { name, fields })
                if name.eq(ftd::interpreter::FTD_RESPONSIVE_LENGTH) =>
            {
                fields
            }
            t => {
                return ftd::executor::utils::parse_error(
                    format!(
                        "Expected value of type record `{}`, found: {:?}",
                        ftd::interpreter::FTD_RESPONSIVE_LENGTH,
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
        values: ftd::Map<fastn_type::PropertyValue>,
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

#[derive(serde::Deserialize, Debug, Default, PartialEq, Clone, serde::Serialize)]
pub struct BreakpointWidth {
    pub mobile: ftd::executor::Value<i64>,
}

impl BreakpointWidth {
    fn from_optional_values(
        or_type_value: Option<ftd::Map<fastn_type::PropertyValue>>,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Option<ftd::executor::BreakpointWidth>> {
        if let Some(value) = or_type_value {
            Ok(Some(ftd::executor::BreakpointWidth::from_values(
                value,
                doc,
                line_number,
            )?))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn optional_breakpoint_width(
        properties: &[ftd::interpreter::Property],
        arguments: &[ftd::interpreter::Argument],
        doc: &ftd::executor::TDoc,
        line_number: usize,
        key: &str,
        inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
        component_name: &str,
    ) -> ftd::executor::Result<ftd::executor::Value<Option<ftd::executor::BreakpointWidth>>> {
        let record_values = ftd::executor::value::optional_record_inherited(
            key,
            component_name,
            properties,
            arguments,
            doc,
            line_number,
            ftd::interpreter::FTD_BREAKPOINT_WIDTH_DATA,
            inherited_variables,
        )?;

        Ok(ftd::executor::Value::new(
            ftd::executor::BreakpointWidth::from_optional_values(
                record_values.value,
                doc,
                line_number,
            )?,
            record_values.line_number,
            record_values.properties,
        ))
    }

    fn from_values(
        values: ftd::Map<fastn_type::PropertyValue>,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<BreakpointWidth> {
        let get_property_value = |field_name: &str| {
            values
                .get(field_name)
                .ok_or_else(|| ftd::executor::Error::ParseError {
                    message: format!(
                        "`{}` field in {} not found",
                        field_name,
                        ftd::interpreter::FTD_BREAKPOINT_WIDTH_DATA
                    ),
                    doc_id: doc.name.to_string(),
                    line_number,
                })
        };

        let mobile = ftd::executor::Value::new(
            get_property_value("mobile")?
                .clone()
                .resolve(&doc.itdoc(), line_number)?
                .integer(doc.name, line_number)?,
            Some(line_number),
            vec![get_property_value("mobile")?
                .into_property(ftd::interpreter::PropertySource::header("mobile"))],
        );

        Ok(BreakpointWidth { mobile })
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
        or_type_value: Option<(String, fastn_type::PropertyValue)>,
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
        or_type_value: (String, fastn_type::PropertyValue),
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Self> {
        match or_type_value.0.as_str() {
            ftd::interpreter::FTD_ALIGN_TOP_LEFT => Ok(Alignment::TopLeft),
            ftd::interpreter::FTD_ALIGN_TOP_CENTER => Ok(Alignment::TopCenter),
            ftd::interpreter::FTD_ALIGN_TOP_RIGHT => Ok(Alignment::TopRight),
            ftd::interpreter::FTD_ALIGN_LEFT => Ok(Alignment::Left),
            ftd::interpreter::FTD_ALIGN_CENTER => Ok(Alignment::Center),
            ftd::interpreter::FTD_ALIGN_RIGHT => Ok(Alignment::Right),
            ftd::interpreter::FTD_ALIGN_BOTTOM_LEFT => Ok(Alignment::BottomLeft),
            ftd::interpreter::FTD_ALIGN_BOTTOM_CENTER => Ok(Alignment::BottomCenter),
            ftd::interpreter::FTD_ALIGN_BOTTOM_RIGHT => Ok(Alignment::BottomRight),
            t => ftd::executor::utils::parse_error(
                format!("Unknown variant `{}` for or-type `ftd.alignment`", t),
                doc.name,
                line_number,
            ),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn optional_alignment(
        properties: &[ftd::interpreter::Property],
        arguments: &[ftd::interpreter::Argument],
        doc: &ftd::executor::TDoc,
        line_number: usize,
        key: &str,
        inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
        component_name: &str,
    ) -> ftd::executor::Result<ftd::executor::Value<Option<Alignment>>> {
        let or_type_value = ftd::executor::value::optional_or_type(
            key,
            component_name,
            properties,
            arguments,
            doc,
            line_number,
            ftd::interpreter::FTD_ALIGN,
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
                        if ({{0}} == \"{top_left}\" || {{0}} == \"{left}\" || {{0}} == \"{bottom_left}\") {{
                            \"start\"
                        }} else if ({{0}} == \"{top_center}\" || {{0}} == \"{center}\" || {{0}} == \"{bottom_center}\") {{
                            \"center\"
                        }} else if ({{0}} == \"{top_right}\" || {{0}} == \"{right}\" || {{0}} == \"{bottom_right}\") {{
                            \"end\"
                        }} else {{
                            null
                        }}
                    "},
                    top_left = ftd::interpreter::FTD_ALIGN_TOP_LEFT,
                    top_center = ftd::interpreter::FTD_ALIGN_TOP_CENTER,
                    top_right = ftd::interpreter::FTD_ALIGN_TOP_RIGHT,
                    left = ftd::interpreter::FTD_ALIGN_LEFT,
                    center = ftd::interpreter::FTD_ALIGN_CENTER,
                    right = ftd::interpreter::FTD_ALIGN_RIGHT,
                    bottom_left = ftd::interpreter::FTD_ALIGN_BOTTOM_LEFT,
                    bottom_center = ftd::interpreter::FTD_ALIGN_BOTTOM_CENTER,
                    bottom_right = ftd::interpreter::FTD_ALIGN_BOTTOM_RIGHT,
                ),
                true,
            )
        } else {
            (
                format!(
                    indoc::indoc! {"
                if ({{0}} == \"{top_left}\" || {{0}} == \"{top_center}\" || {{0}} == \"{top_right}\") {{
                    \"start\"
                }} else if ({{0}} == \"{left}\" || {{0}} == \"{center}\" || {{0}} == \"{right}\") {{
                    \"center\"
                }} else if ({{0}} == \"{bottom_left}\" || {{0}} == \"{bottom_center}\" || {{0}} == \"{bottom_right}\") {{
                    \"end\"
                }} else {{
                    null
                }}
                "},
                    top_left = ftd::interpreter::FTD_ALIGN_TOP_LEFT,
                    top_center = ftd::interpreter::FTD_ALIGN_TOP_CENTER,
                    top_right = ftd::interpreter::FTD_ALIGN_TOP_RIGHT,
                    left = ftd::interpreter::FTD_ALIGN_LEFT,
                    center = ftd::interpreter::FTD_ALIGN_CENTER,
                    right = ftd::interpreter::FTD_ALIGN_RIGHT,
                    bottom_left = ftd::interpreter::FTD_ALIGN_BOTTOM_LEFT,
                    bottom_center = ftd::interpreter::FTD_ALIGN_BOTTOM_CENTER,
                    bottom_right = ftd::interpreter::FTD_ALIGN_BOTTOM_RIGHT,
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
               if ({{0}} == \"{top_left}\" || {{0}} == \"{top_center}\" || {{0}} == \"{top_right}\") {{
                    \"start\"
                }} else if ({{0}} == \"{left}\" || {{0}} == \"{center}\" || {{0}} == \"{right}\") {{
                    \"center\"
                }} else if ({{0}} == \"{bottom_left}\" || {{0}} == \"{bottom_center}\" || {{0}} == \"{bottom_right}\") {{
                    \"end\"
                }} else {{
                    null
                }}
                "},
                    top_left = ftd::interpreter::FTD_ALIGN_TOP_LEFT,
                    top_center = ftd::interpreter::FTD_ALIGN_TOP_CENTER,
                    top_right = ftd::interpreter::FTD_ALIGN_TOP_RIGHT,
                    left = ftd::interpreter::FTD_ALIGN_LEFT,
                    center = ftd::interpreter::FTD_ALIGN_CENTER,
                    right = ftd::interpreter::FTD_ALIGN_RIGHT,
                    bottom_left = ftd::interpreter::FTD_ALIGN_BOTTOM_LEFT,
                    bottom_center = ftd::interpreter::FTD_ALIGN_BOTTOM_CENTER,
                    bottom_right = ftd::interpreter::FTD_ALIGN_BOTTOM_RIGHT,
                ),
                true,
            )
        } else {
            (
                format!(
                    indoc::indoc! {"
                        if ({{0}} == \"{top_left}\" || {{0}} == \"{left}\" || {{0}} == \"{bottom_left}\") {{
                            \"start\"
                        }} else if ({{0}} == \"{top_center}\" || {{0}} == \"{center}\" || {{0}} == \"{bottom_center}\") {{
                            \"center\"
                        }} else if ({{0}} == \"{top_right}\" || {{0}} == \"{right}\" || {{0}} == \"{bottom_right}\") {{
                            \"end\"
                        }} else {{
                            null
                        }}
                    "},
                    top_left = ftd::interpreter::FTD_ALIGN_TOP_LEFT,
                    top_center = ftd::interpreter::FTD_ALIGN_TOP_CENTER,
                    top_right = ftd::interpreter::FTD_ALIGN_TOP_RIGHT,
                    left = ftd::interpreter::FTD_ALIGN_LEFT,
                    center = ftd::interpreter::FTD_ALIGN_CENTER,
                    right = ftd::interpreter::FTD_ALIGN_RIGHT,
                    bottom_left = ftd::interpreter::FTD_ALIGN_BOTTOM_LEFT,
                    bottom_center = ftd::interpreter::FTD_ALIGN_BOTTOM_CENTER,
                    bottom_right = ftd::interpreter::FTD_ALIGN_BOTTOM_RIGHT,
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
        or_type_value: Option<(String, fastn_type::PropertyValue)>,
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
        or_type_value: (String, fastn_type::PropertyValue),
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Self> {
        match or_type_value.0.as_str() {
            t if t.starts_with(ftd::interpreter::FTD_RESIZING_FIXED) => {
                let value = or_type_value.1.clone().resolve(&doc.itdoc(), line_number)?;
                let (_, variant, value) = value.get_or_type(doc.name, line_number)?;
                Ok(Resizing::Fixed(Length::from_values(
                    (variant.to_owned(), value.to_owned()),
                    doc,
                    line_number,
                )?))
            }
            ftd::interpreter::FTD_RESIZING_HUG_CONTENT => Ok(Resizing::HugContent),
            ftd::interpreter::FTD_RESIZING_AUTO => Ok(Resizing::Auto),
            ftd::interpreter::FTD_RESIZING_FILL_CONTAINER => Ok(Resizing::FillContainer),
            t => ftd::executor::utils::parse_error(
                format!("Unknown variant `{}` for or-type `ftd.resizing`", t),
                doc.name,
                line_number,
            ),
        }
    }

    pub(crate) fn optional_resizing(
        properties: &[ftd::interpreter::Property],
        arguments: &[ftd::interpreter::Argument],
        doc: &ftd::executor::TDoc,
        line_number: usize,
        key: &str,
        inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
        component_name: &str,
    ) -> ftd::executor::Result<ftd::executor::Value<Option<Resizing>>> {
        let or_type_value = ftd::executor::value::optional_or_type(
            key,
            component_name,
            properties,
            arguments,
            doc,
            line_number,
            ftd::interpreter::FTD_RESIZING,
            inherited_variables,
        )?;

        Ok(ftd::executor::Value::new(
            Resizing::from_optional_values(or_type_value.value, doc, line_number)?,
            or_type_value.line_number,
            or_type_value.properties,
        ))
    }

    pub fn to_css_string(&self, device: &Option<ftd::executor::Device>) -> String {
        match self {
            Resizing::HugContent => "fit-content".to_string(),
            Resizing::FillContainer => "100%".to_string(),
            Resizing::Fixed(l) => l.to_css_string(device),
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
            ftd::interpreter::FTD_RESIZING_FIXED => {
                let remaining = full_variant
                    .trim_start_matches(format!("{}.", variant).as_str())
                    .to_string();
                let variant = format!("{}.{}", ftd::interpreter::FTD_LENGTH, remaining);
                Ok((
                    Length::get_pattern_from_variant_str(variant.as_str(), doc_id, line_number)?,
                    true,
                ))
            }
            ftd::interpreter::FTD_RESIZING_FILL_CONTAINER => Ok(("100%", false)),
            ftd::interpreter::FTD_RESIZING_HUG_CONTENT => Ok(("fit-content", false)),
            ftd::interpreter::FTD_RESIZING_AUTO => Ok(("auto", false)),
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
            ftd::interpreter::FTD_RESIZING_FIXED => {
                let remaining = full_variant
                    .trim_start_matches(format!("{}.", variant).as_str())
                    .to_string();
                let variant = format!("{}.{}", ftd::interpreter::FTD_LENGTH, remaining);
                Length::set_pattern_from_variant_str(variant.as_str(), doc_id, line_number)
            }
            ftd::interpreter::FTD_RESIZING_FILL_CONTAINER => Ok("100%"),
            ftd::interpreter::FTD_RESIZING_HUG_CONTENT => Ok("fit-content"),
            ftd::interpreter::FTD_RESIZING_AUTO => Ok("auto"),
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
            ftd::interpreter::FTD_RESIZING_FIXED => {
                let remaining = full_variant
                    .trim_start_matches(format!("{}.", variant).as_str())
                    .to_string();
                let variant = format!("{}.{}", ftd::interpreter::FTD_LENGTH, remaining);
                Length::set_value_from_variant(variant.as_str(), value, doc_id, line_number)
            }
            ftd::interpreter::FTD_RESIZING_FILL_CONTAINER => Ok("100%".to_string()),
            ftd::interpreter::FTD_RESIZING_HUG_CONTENT => Ok("fit-content".to_string()),
            ftd::interpreter::FTD_RESIZING_AUTO => Ok("auto".to_string()),
            t => ftd::executor::utils::parse_error(
                format!("Unknown variant found for ftd.resizing: `{}`", t),
                doc_id,
                line_number,
            ),
        }
    }
}
#[derive(serde::Deserialize, Debug, Default, PartialEq, Clone, serde::Serialize)]
pub struct BackgroundImage {
    pub src: ftd::executor::Value<ftd::executor::ImageSrc>,
    pub repeat: ftd::executor::Value<Option<ftd::executor::BackgroundRepeat>>,
    pub size: ftd::executor::Value<Option<ftd::executor::BackgroundSize>>,
    pub position: ftd::executor::Value<Option<ftd::executor::BackgroundPosition>>,
}

impl BackgroundImage {
    fn from_value(
        value: fastn_type::PropertyValue,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<ftd::executor::BackgroundImage> {
        let value = value.resolve(&doc.itdoc(), line_number)?;
        let fields = match value.inner() {
            Some(fastn_type::Value::Record { name, fields })
                if name.eq(ftd::interpreter::FTD_BG_IMAGE) =>
            {
                fields
            }
            t => {
                return ftd::executor::utils::parse_error(
                    format!(
                        "Expected value of type record `{}`, found: {:?}",
                        ftd::interpreter::FTD_BG_IMAGE,
                        t
                    ),
                    doc.name,
                    line_number,
                )
            }
        };
        ftd::executor::BackgroundImage::from_values(fields, doc, line_number)
    }

    fn from_values(
        values: ftd::Map<fastn_type::PropertyValue>,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<ftd::executor::BackgroundImage> {
        let get_property_value = |field_name: &str| {
            values
                .get(field_name)
                .ok_or_else(|| ftd::executor::Error::ParseError {
                    message: format!("`{}` field in ftd.background-image not found", field_name),
                    doc_id: doc.name.to_string(),
                    line_number,
                })
        };

        let src = ftd::executor::Value::new(
            ftd::executor::ImageSrc::from_value(
                get_property_value("src")?.clone(),
                doc,
                line_number,
            )?,
            Some(line_number),
            vec![get_property_value("src")?
                .into_property(ftd::interpreter::PropertySource::header("src"))],
        );

        let repeat = ftd::executor::Value::new(
            ftd::executor::BackgroundRepeat::from_optional_value(
                get_property_value("repeat")?.clone(),
                doc,
                line_number,
            )?,
            Some(line_number),
            vec![get_property_value("repeat")?
                .into_property(ftd::interpreter::PropertySource::header("repeat"))],
        );

        let size = ftd::executor::Value::new(
            ftd::executor::BackgroundSize::from_optional_value(
                get_property_value("size")?.clone(),
                doc,
                line_number,
            )?,
            Some(line_number),
            vec![get_property_value("size")?
                .into_property(ftd::interpreter::PropertySource::header("size"))],
        );

        let position = ftd::executor::Value::new(
            ftd::executor::BackgroundPosition::from_optional_value(
                get_property_value("position")?.clone(),
                doc,
                line_number,
            )?,
            Some(line_number),
            vec![get_property_value("position")?
                .into_property(ftd::interpreter::PropertySource::header("position"))],
        );

        Ok(ftd::executor::BackgroundImage {
            src,
            repeat,
            size,
            position,
        })
    }

    pub fn to_image_src_css_string(&self) -> String {
        format!("url({})", self.src.value.light.value)
    }

    pub fn to_repeat_css_string(&self) -> String {
        match self.repeat.value.as_ref() {
            Some(s) => s.to_css_string(),
            None => ftd::interpreter::FTD_IGNORE_KEY.to_string(),
        }
    }

    pub fn to_size_css_string(&self, device: &Option<ftd::executor::Device>) -> String {
        match self.size.value.as_ref() {
            Some(s) => s.to_css_string(device),
            None => ftd::interpreter::FTD_IGNORE_KEY.to_string(),
        }
    }

    pub fn to_position_css_string(&self, device: &Option<ftd::executor::Device>) -> String {
        match self.position.value.as_ref() {
            Some(s) => s.to_css_string(device),
            None => ftd::interpreter::FTD_IGNORE_KEY.to_string(),
        }
    }
}

#[derive(serde::Deserialize, Debug, PartialEq, Clone, serde::Serialize)]
pub struct LinearGradientColor {
    pub color: Color,
    pub start: ftd::executor::Value<Option<Length>>,
    pub end: ftd::executor::Value<Option<Length>>,
    pub stop_position: ftd::executor::Value<Option<Length>>,
}

impl LinearGradientColor {
    fn from_vec_values(
        value: fastn_type::PropertyValue,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Vec<LinearGradientColor>> {
        let mut result = vec![];
        let value = value.resolve(&doc.itdoc(), line_number)?;
        match value.inner() {
            Some(fastn_type::Value::List { data, kind })
                if kind
                    .kind
                    .get_name()
                    .eq(ftd::interpreter::FTD_LINEAR_GRADIENT_COLOR) =>
            {
                for element in data.iter() {
                    let ln = element.line_number();
                    result.push(LinearGradientColor::from_value(
                        element.to_owned(),
                        doc,
                        ln,
                    )?)
                }
            }
            t => {
                return ftd::executor::utils::parse_error(
                    format!(
                        "Expected list value of type `{}`, found: {:?}",
                        ftd::interpreter::FTD_LINEAR_GRADIENT,
                        t
                    ),
                    doc.name,
                    line_number,
                )
            }
        };
        Ok(result)
    }

    fn from_value(
        value: fastn_type::PropertyValue,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<LinearGradientColor> {
        let value = value.resolve(&doc.itdoc(), line_number)?;
        let fields = match value.inner() {
            Some(fastn_type::Value::Record { name, fields })
                if name.eq(ftd::interpreter::FTD_LINEAR_GRADIENT_COLOR) =>
            {
                fields
            }
            t => {
                return ftd::executor::utils::parse_error(
                    format!(
                        "Expected value of type record `{}`, found: {:?}",
                        ftd::interpreter::FTD_LINEAR_GRADIENT_COLOR,
                        t
                    ),
                    doc.name,
                    line_number,
                )
            }
        };
        ftd::executor::LinearGradientColor::from_values(fields, doc, line_number)
    }

    fn from_values(
        values: ftd::Map<fastn_type::PropertyValue>,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<LinearGradientColor> {
        let get_property_value = |field_name: &str| {
            values
                .get(field_name)
                .ok_or_else(|| ftd::executor::Error::ParseError {
                    message: format!(
                        "`{}` field in ftd.linear-gradient-color not found",
                        field_name
                    ),
                    doc_id: doc.name.to_string(),
                    line_number,
                })
        };

        let color = ftd::executor::Color::from_value(
            get_property_value("color")?.clone(),
            doc,
            line_number,
        )?;

        let start = ftd::executor::Value::new(
            ftd::executor::Length::from_optional_value(
                values.get("start").cloned(),
                doc,
                line_number,
            )?,
            Some(line_number),
            vec![get_property_value("start")?
                .into_property(ftd::interpreter::PropertySource::header("start"))],
        );

        let end = ftd::executor::Value::new(
            ftd::executor::Length::from_optional_value(
                values.get("end").cloned(),
                doc,
                line_number,
            )?,
            Some(line_number),
            vec![get_property_value("end")?
                .into_property(ftd::interpreter::PropertySource::header("end"))],
        );

        let stop_position = ftd::executor::Value::new(
            ftd::executor::Length::from_optional_value(
                values.get("stop-position").cloned(),
                doc,
                line_number,
            )?,
            Some(line_number),
            vec![get_property_value("stop-position")?
                .into_property(ftd::interpreter::PropertySource::header("stop-position"))],
        );

        Ok(ftd::executor::LinearGradientColor {
            color,
            start,
            end,
            stop_position,
        })
    }

    pub fn to_css_string(&self, device: &Option<ftd::executor::Device>) -> String {
        let mut result = self.color.light.value.to_css_string();
        if let Some(start) = self.start.value.as_ref() {
            result.push_str(format!(" {}", start.to_css_string(device)).as_str());
        }
        if let Some(end) = self.end.value.as_ref() {
            result.push_str(format!(" {}", end.to_css_string(device)).as_str());
        }
        if let Some(stop) = self.stop_position.value.as_ref() {
            result.push_str(format!(", {}", stop.to_css_string(device)).as_str());
        }
        result
    }
}

#[derive(serde::Deserialize, Debug, PartialEq, Clone, serde::Serialize)]
pub enum LinearGradientDirection {
    Angle(f64),
    Turn(f64),
    Left,
    Right,
    Top,
    Bottom,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl LinearGradientDirection {
    fn from_value(
        value: fastn_type::PropertyValue,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<LinearGradientDirection> {
        let binding = value.resolve(&doc.itdoc(), line_number)?;
        let value = binding.get_or_type(doc.name, line_number)?;
        let value = (value.1.to_owned(), value.2.to_owned());
        LinearGradientDirection::from_values(value, doc, line_number)
    }

    fn from_values(
        or_type_value: (String, fastn_type::PropertyValue),
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Self> {
        match or_type_value.0.as_str() {
            ftd::interpreter::FTD_LINEAR_GRADIENT_DIRECTIONS_LEFT => {
                Ok(LinearGradientDirection::Left)
            }
            ftd::interpreter::FTD_LINEAR_GRADIENT_DIRECTIONS_RIGHT => {
                Ok(LinearGradientDirection::Right)
            }
            ftd::interpreter::FTD_LINEAR_GRADIENT_DIRECTIONS_TOP => {
                Ok(LinearGradientDirection::Top)
            }
            ftd::interpreter::FTD_LINEAR_GRADIENT_DIRECTIONS_BOTTOM => {
                Ok(LinearGradientDirection::Bottom)
            }
            ftd::interpreter::FTD_LINEAR_GRADIENT_DIRECTIONS_TOP_LEFT => {
                Ok(LinearGradientDirection::TopLeft)
            }
            ftd::interpreter::FTD_LINEAR_GRADIENT_DIRECTIONS_BOTTOM_LEFT => {
                Ok(LinearGradientDirection::BottomLeft)
            }
            ftd::interpreter::FTD_LINEAR_GRADIENT_DIRECTIONS_TOP_RIGHT => {
                Ok(LinearGradientDirection::TopRight)
            }
            ftd::interpreter::FTD_LINEAR_GRADIENT_DIRECTIONS_BOTTOM_RIGHT => {
                Ok(LinearGradientDirection::BottomRight)
            }
            ftd::interpreter::FTD_LINEAR_GRADIENT_DIRECTIONS_ANGLE => {
                Ok(LinearGradientDirection::Angle(
                    or_type_value
                        .1
                        .clone()
                        .resolve(&doc.itdoc(), line_number)?
                        .decimal(doc.name, line_number)?,
                ))
            }
            ftd::interpreter::FTD_LINEAR_GRADIENT_DIRECTIONS_TURN => {
                Ok(LinearGradientDirection::Turn(
                    or_type_value
                        .1
                        .clone()
                        .resolve(&doc.itdoc(), line_number)?
                        .decimal(doc.name, line_number)?,
                ))
            }
            t => ftd::executor::utils::parse_error(
                format!(
                    "Unknown variant `{}` for or-type `ftd.linear-gradient-directions`",
                    t
                ),
                doc.name,
                line_number,
            ),
        }
    }
    // Top, Bottom, Left, Right angles
    // 0deg, 180deg, 270deg, and 90deg
    pub fn to_css_string(&self) -> String {
        match self {
            LinearGradientDirection::Top => "0deg".to_string(),
            LinearGradientDirection::Bottom => "180deg".to_string(),
            LinearGradientDirection::Left => "270deg".to_string(),
            LinearGradientDirection::Right => "90deg".to_string(),
            LinearGradientDirection::TopLeft => "315deg".to_string(),
            LinearGradientDirection::BottomLeft => "225deg".to_string(),
            LinearGradientDirection::TopRight => "45deg".to_string(),
            LinearGradientDirection::BottomRight => "135deg".to_string(),
            LinearGradientDirection::Angle(a) => format!("{}deg", a),
            LinearGradientDirection::Turn(t) => format!("{}turn", t),
        }
    }
}

#[derive(serde::Deserialize, Debug, PartialEq, Clone, serde::Serialize)]
pub struct LinearGradient {
    pub direction: ftd::executor::Value<LinearGradientDirection>,
    pub colors: ftd::executor::Value<Vec<LinearGradientColor>>,
}

impl LinearGradient {
    fn from_value(
        value: fastn_type::PropertyValue,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<ftd::executor::LinearGradient> {
        let value = value.resolve(&doc.itdoc(), line_number)?;
        let fields = match value.inner() {
            Some(fastn_type::Value::Record { name, fields })
                if name.eq(ftd::interpreter::FTD_LINEAR_GRADIENT) =>
            {
                fields
            }
            t => {
                return ftd::executor::utils::parse_error(
                    format!(
                        "Expected value of type record `{}`, found: {:?}",
                        ftd::interpreter::FTD_LINEAR_GRADIENT,
                        t
                    ),
                    doc.name,
                    line_number,
                )
            }
        };
        ftd::executor::LinearGradient::from_values(fields, doc, line_number)
    }

    fn from_values(
        values: ftd::Map<fastn_type::PropertyValue>,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<ftd::executor::LinearGradient> {
        let get_property_value = |field_name: &str| {
            values
                .get(field_name)
                .ok_or_else(|| ftd::executor::Error::ParseError {
                    message: format!("`{}` field in ftd.linear-gradient not found", field_name),
                    doc_id: doc.name.to_string(),
                    line_number,
                })
        };

        let direction = ftd::executor::Value::new(
            ftd::executor::LinearGradientDirection::from_value(
                get_property_value("direction")?.clone(),
                doc,
                line_number,
            )?,
            Some(line_number),
            vec![get_property_value("direction")?
                .into_property(ftd::interpreter::PropertySource::header("direction"))],
        );

        let colors = ftd::executor::Value::new(
            ftd::executor::LinearGradientColor::from_vec_values(
                get_property_value("colors")?.clone(),
                doc,
                line_number,
            )?,
            Some(line_number),
            vec![get_property_value("colors")?
                .into_property(ftd::interpreter::PropertySource::header("colors"))],
        );

        Ok(ftd::executor::LinearGradient { direction, colors })
    }

    pub fn to_css_string(&self, device: &Option<ftd::executor::Device>) -> String {
        format!(
            "linear-gradient({}, {})",
            self.direction.value.to_css_string(),
            self.colors
                .value
                .iter()
                .map(|lc| lc.to_css_string(device))
                .join(", ")
        )
    }
}

#[derive(serde::Deserialize, Debug, PartialEq, Clone, serde::Serialize)]
pub enum Background {
    Solid(ftd::executor::Color),
    Image(ftd::executor::BackgroundImage),
    LinearGradient(ftd::executor::LinearGradient),
}

impl Background {
    fn from_optional_values(
        or_type_value: Option<(String, fastn_type::PropertyValue)>,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Option<Self>> {
        if let Some(value) = or_type_value {
            Ok(Some(ftd::executor::Background::from_values(
                value,
                doc,
                line_number,
            )?))
        } else {
            Ok(None)
        }
    }

    fn from_values(
        or_type_value: (String, fastn_type::PropertyValue),
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Self> {
        match or_type_value.0.as_str() {
            ftd::interpreter::FTD_BACKGROUND_SOLID => Ok(ftd::executor::Background::Solid(
                Color::from_value(or_type_value.1, doc, line_number)?,
            )),
            ftd::interpreter::FTD_BACKGROUND_IMAGE => Ok(ftd::executor::Background::Image(
                ftd::executor::BackgroundImage::from_value(or_type_value.1, doc, line_number)?,
            )),
            ftd::interpreter::FTD_BACKGROUND_LINEAR_GRADIENT => {
                Ok(ftd::executor::Background::LinearGradient(
                    ftd::executor::LinearGradient::from_value(or_type_value.1, doc, line_number)?,
                ))
            }
            t => ftd::executor::utils::parse_error(
                format!("Unknown variant `{}` for or-type `ftd.background`", t),
                doc.name,
                line_number,
            ),
        }
    }

    pub(crate) fn optional_background(
        properties: &[ftd::interpreter::Property],
        arguments: &[ftd::interpreter::Argument],
        doc: &ftd::executor::TDoc,
        line_number: usize,
        key: &str,
        inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
        component_name: &str,
    ) -> ftd::executor::Result<ftd::executor::Value<Option<ftd::executor::Background>>> {
        let or_type_value = ftd::executor::value::optional_or_type(
            key,
            component_name,
            properties,
            arguments,
            doc,
            line_number,
            ftd::interpreter::FTD_BACKGROUND,
            inherited_variables,
        )?;

        Ok(ftd::executor::Value::new(
            ftd::executor::Background::from_optional_values(or_type_value.value, doc, line_number)?,
            or_type_value.line_number,
            or_type_value.properties,
        ))
    }

    pub fn to_solid_css_string(&self) -> String {
        match self {
            ftd::executor::Background::Solid(c) => c.light.value.to_css_string(),
            ftd::executor::Background::Image(_) => ftd::interpreter::FTD_IGNORE_KEY.to_string(),
            ftd::executor::Background::LinearGradient(_) => {
                ftd::interpreter::FTD_IGNORE_KEY.to_string()
            }
        }
    }

    pub fn to_image_src_css_string(&self, device: &Option<ftd::executor::Device>) -> String {
        match self {
            ftd::executor::Background::Solid(_) => ftd::interpreter::FTD_IGNORE_KEY.to_string(),
            ftd::executor::Background::Image(i) => i.to_image_src_css_string(),
            ftd::executor::Background::LinearGradient(l) => l.to_css_string(device),
        }
    }

    pub fn to_image_repeat_css_string(&self) -> String {
        match self {
            ftd::executor::Background::Solid(_) => ftd::interpreter::FTD_IGNORE_KEY.to_string(),
            ftd::executor::Background::Image(i) => i.to_repeat_css_string(),
            ftd::executor::Background::LinearGradient(_) => {
                ftd::interpreter::FTD_IGNORE_KEY.to_string()
            }
        }
    }

    pub fn to_image_size_css_string(&self, device: &Option<ftd::executor::Device>) -> String {
        match self {
            ftd::executor::Background::Solid(_) => ftd::interpreter::FTD_IGNORE_KEY.to_string(),
            ftd::executor::Background::Image(i) => i.to_size_css_string(device),
            ftd::executor::Background::LinearGradient(_) => {
                ftd::interpreter::FTD_IGNORE_KEY.to_string()
            }
        }
    }

    pub fn to_image_position_css_string(&self, device: &Option<ftd::executor::Device>) -> String {
        match self {
            ftd::executor::Background::Solid(_) => ftd::interpreter::FTD_IGNORE_KEY.to_string(),
            ftd::executor::Background::Image(i) => i.to_position_css_string(device),
            ftd::executor::Background::LinearGradient(_) => {
                ftd::interpreter::FTD_IGNORE_KEY.to_string()
            }
        }
    }

    pub fn background_image_pattern() -> (String, bool) {
        (
            "window.ftd.dependencies.eval_background_image({0}, data)".to_string(),
            true,
        )
    }

    pub fn background_repeat_pattern() -> (String, bool) {
        (
            "window.ftd.dependencies.eval_background_repeat({0})".to_string(),
            true,
        )
    }

    pub fn background_color_pattern() -> (String, bool) {
        (
            "window.ftd.dependencies.eval_background_color({0}, data)".to_string(),
            true,
        )
    }

    pub fn background_size_pattern() -> (String, bool) {
        (
            "window.ftd.dependencies.eval_background_size({0})".to_string(),
            true,
        )
    }

    pub fn background_position_pattern() -> (String, bool) {
        (
            "window.ftd.dependencies.eval_background_position({0})".to_string(),
            true,
        )
    }
}

#[derive(serde::Deserialize, Debug, PartialEq, Clone, serde::Serialize)]
pub enum BackgroundRepeat {
    Repeat,
    RepeatX,
    RepeatY,
    NoRepeat,
    Space,
    Round,
}

impl BackgroundRepeat {
    fn from_optional_value(
        value: fastn_type::PropertyValue,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Option<ftd::executor::BackgroundRepeat>> {
        let binding = value.resolve(&doc.itdoc(), line_number)?;
        if let fastn_type::Value::Optional { data, .. } = &binding {
            if data.is_none() {
                return Ok(None);
            }
        }

        let value = binding.get_or_type(doc.name, line_number)?;
        let value = (value.1.to_owned(), value.2.to_owned());
        Ok(Some(ftd::executor::BackgroundRepeat::from_values(
            value,
            doc,
            line_number,
        )?))
    }

    fn from_values(
        or_type_value: (String, fastn_type::PropertyValue),
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<ftd::executor::BackgroundRepeat> {
        match or_type_value.0.as_str() {
            ftd::interpreter::FTD_BACKGROUND_REPEAT_BOTH_REPEAT => {
                Ok(ftd::executor::BackgroundRepeat::Repeat)
            }
            ftd::interpreter::FTD_BACKGROUND_REPEAT_X_REPEAT => {
                Ok(ftd::executor::BackgroundRepeat::RepeatX)
            }
            ftd::interpreter::FTD_BACKGROUND_REPEAT_Y_REPEAT => {
                Ok(ftd::executor::BackgroundRepeat::RepeatY)
            }
            ftd::interpreter::FTD_BACKGROUND_REPEAT_NO_REPEAT => {
                Ok(ftd::executor::BackgroundRepeat::NoRepeat)
            }
            ftd::interpreter::FTD_BACKGROUND_REPEAT_SPACE => {
                Ok(ftd::executor::BackgroundRepeat::Space)
            }
            ftd::interpreter::FTD_BACKGROUND_REPEAT_ROUND => {
                Ok(ftd::executor::BackgroundRepeat::Round)
            }
            t => ftd::executor::utils::parse_error(
                format!(
                    "Unknown variant `{}` for or-type `ftd.background-repeat`",
                    t
                ),
                doc.name,
                line_number,
            ),
        }
    }

    pub fn to_css_string(&self) -> String {
        match self {
            ftd::executor::BackgroundRepeat::Repeat => "repeat".to_string(),
            ftd::executor::BackgroundRepeat::RepeatX => "repeat-x".to_string(),
            ftd::executor::BackgroundRepeat::RepeatY => "repeat-y".to_string(),
            ftd::executor::BackgroundRepeat::NoRepeat => "no-repeat".to_string(),
            ftd::executor::BackgroundRepeat::Space => "space".to_string(),
            ftd::executor::BackgroundRepeat::Round => "round".to_string(),
        }
    }
}

#[derive(serde::Deserialize, Debug, PartialEq, Clone, serde::Serialize)]
pub enum BackgroundSize {
    Auto,
    Cover,
    Contain,
    Length(LengthPair),
}

impl BackgroundSize {
    fn from_optional_value(
        value: fastn_type::PropertyValue,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Option<ftd::executor::BackgroundSize>> {
        let binding = value.resolve(&doc.itdoc(), line_number)?;
        if let fastn_type::Value::Optional { data, .. } = &binding {
            if data.is_none() {
                return Ok(None);
            }
        }

        let value = binding.get_or_type(doc.name, line_number)?;
        let value = (value.1.to_owned(), value.2.to_owned());
        Ok(Some(ftd::executor::BackgroundSize::from_values(
            value,
            doc,
            line_number,
        )?))
    }

    fn from_values(
        or_type_value: (String, fastn_type::PropertyValue),
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<ftd::executor::BackgroundSize> {
        match or_type_value.0.as_str() {
            ftd::interpreter::FTD_BACKGROUND_SIZE_AUTO => Ok(ftd::executor::BackgroundSize::Auto),
            ftd::interpreter::FTD_BACKGROUND_SIZE_COVER => Ok(ftd::executor::BackgroundSize::Cover),
            ftd::interpreter::FTD_BACKGROUND_SIZE_CONTAIN => {
                Ok(ftd::executor::BackgroundSize::Contain)
            }
            ftd::interpreter::FTD_BACKGROUND_SIZE_LENGTH => {
                Ok(ftd::executor::BackgroundSize::Length(
                    LengthPair::from_value(or_type_value.1, doc, line_number)?,
                ))
            }
            t => ftd::executor::utils::parse_error(
                format!("Unknown variant `{}` for or-type `ftd.background-size`", t),
                doc.name,
                line_number,
            ),
        }
    }

    pub fn to_css_string(&self, device: &Option<ftd::executor::Device>) -> String {
        match self {
            ftd::executor::BackgroundSize::Auto => "auto".to_string(),
            ftd::executor::BackgroundSize::Cover => "cover".to_string(),
            ftd::executor::BackgroundSize::Contain => "contain".to_string(),
            ftd::executor::BackgroundSize::Length(l) => l.to_css_string(device),
        }
    }
}

#[derive(serde::Deserialize, Debug, PartialEq, Clone, serde::Serialize)]
pub enum BackgroundPosition {
    Left,
    Center,
    Right,
    LeftTop,
    LeftCenter,
    LeftBottom,
    CenterTop,
    CenterCenter,
    CenterBottom,
    RightTop,
    RightCenter,
    RightBottom,
    Length(LengthPair),
}

impl BackgroundPosition {
    fn from_optional_value(
        value: fastn_type::PropertyValue,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Option<ftd::executor::BackgroundPosition>> {
        let binding = value.resolve(&doc.itdoc(), line_number)?;
        if let fastn_type::Value::Optional { data, .. } = &binding {
            if data.is_none() {
                return Ok(None);
            }
        }

        let value = binding.get_or_type(doc.name, line_number)?;
        let value = (value.1.to_owned(), value.2.to_owned());
        Ok(Some(ftd::executor::BackgroundPosition::from_values(
            value,
            doc,
            line_number,
        )?))
    }

    fn from_values(
        or_type_value: (String, fastn_type::PropertyValue),
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<ftd::executor::BackgroundPosition> {
        match or_type_value.0.as_str() {
            ftd::interpreter::FTD_BACKGROUND_POSITION_LEFT => {
                Ok(ftd::executor::BackgroundPosition::Left)
            }
            ftd::interpreter::FTD_BACKGROUND_POSITION_CENTER => {
                Ok(ftd::executor::BackgroundPosition::Center)
            }
            ftd::interpreter::FTD_BACKGROUND_POSITION_RIGHT => {
                Ok(ftd::executor::BackgroundPosition::Right)
            }
            ftd::interpreter::FTD_BACKGROUND_POSITION_LEFT_TOP => {
                Ok(ftd::executor::BackgroundPosition::LeftTop)
            }
            ftd::interpreter::FTD_BACKGROUND_POSITION_LEFT_CENTER => {
                Ok(ftd::executor::BackgroundPosition::LeftCenter)
            }
            ftd::interpreter::FTD_BACKGROUND_POSITION_LEFT_BOTTOM => {
                Ok(ftd::executor::BackgroundPosition::LeftBottom)
            }
            ftd::interpreter::FTD_BACKGROUND_POSITION_CENTER_TOP => {
                Ok(ftd::executor::BackgroundPosition::CenterTop)
            }
            ftd::interpreter::FTD_BACKGROUND_POSITION_CENTER_CENTER => {
                Ok(ftd::executor::BackgroundPosition::CenterCenter)
            }
            ftd::interpreter::FTD_BACKGROUND_POSITION_CENTER_BOTTOM => {
                Ok(ftd::executor::BackgroundPosition::CenterBottom)
            }
            ftd::interpreter::FTD_BACKGROUND_POSITION_RIGHT_TOP => {
                Ok(ftd::executor::BackgroundPosition::RightTop)
            }
            ftd::interpreter::FTD_BACKGROUND_POSITION_RIGHT_CENTER => {
                Ok(ftd::executor::BackgroundPosition::RightCenter)
            }
            ftd::interpreter::FTD_BACKGROUND_POSITION_RIGHT_BOTTOM => {
                Ok(ftd::executor::BackgroundPosition::RightBottom)
            }
            ftd::interpreter::FTD_BACKGROUND_POSITION_LENGTH => {
                Ok(ftd::executor::BackgroundPosition::Length(
                    LengthPair::from_value(or_type_value.1, doc, line_number)?,
                ))
            }
            t => ftd::executor::utils::parse_error(
                format!(
                    "Unknown variant `{}` for or-type `ftd.background-position`",
                    t
                ),
                doc.name,
                line_number,
            ),
        }
    }

    pub fn to_css_string(&self, device: &Option<ftd::executor::Device>) -> String {
        match self {
            ftd::executor::BackgroundPosition::Left => "left".to_string(),
            ftd::executor::BackgroundPosition::Center => "center".to_string(),
            ftd::executor::BackgroundPosition::Right => "right".to_string(),
            ftd::executor::BackgroundPosition::LeftTop => "left top".to_string(),
            ftd::executor::BackgroundPosition::LeftCenter => "left center".to_string(),
            ftd::executor::BackgroundPosition::LeftBottom => "left bottom".to_string(),
            ftd::executor::BackgroundPosition::CenterTop => "center top".to_string(),
            ftd::executor::BackgroundPosition::CenterCenter => "center center".to_string(),
            ftd::executor::BackgroundPosition::CenterBottom => "center bottom".to_string(),
            ftd::executor::BackgroundPosition::RightTop => "right top".to_string(),
            ftd::executor::BackgroundPosition::RightCenter => "right center".to_string(),
            ftd::executor::BackgroundPosition::RightBottom => "right bottom".to_string(),
            ftd::executor::BackgroundPosition::Length(l) => l.to_css_string(device),
        }
    }
}

#[derive(serde::Deserialize, Debug, Default, PartialEq, Clone, serde::Serialize)]
pub struct Shadow {
    pub x_offset: ftd::executor::Value<Length>,
    pub y_offset: ftd::executor::Value<Length>,
    pub blur: ftd::executor::Value<Length>,
    pub spread: ftd::executor::Value<Length>,
    pub inset: ftd::executor::Value<bool>,
    pub color: ftd::executor::Value<Color>,
}

impl Shadow {
    fn from_values(
        values: ftd::Map<fastn_type::PropertyValue>,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<ftd::executor::Shadow> {
        let get_property_value = |field_name: &str| {
            values
                .get(field_name)
                .ok_or_else(|| ftd::executor::Error::ParseError {
                    message: format!("`{}` field in ftd.shadow not found", field_name),
                    doc_id: doc.name.to_string(),
                    line_number,
                })
        };

        let x_offset = ftd::executor::Value::new(
            Length::from_value(get_property_value("x-offset")?.clone(), doc, line_number)?,
            Some(line_number),
            vec![get_property_value("x-offset")?
                .into_property(ftd::interpreter::PropertySource::header("x-offset"))],
        );

        let y_offset = ftd::executor::Value::new(
            Length::from_value(get_property_value("y-offset")?.clone(), doc, line_number)?,
            Some(line_number),
            vec![get_property_value("y-offset")?
                .into_property(ftd::interpreter::PropertySource::header("y-offset"))],
        );

        let blur = ftd::executor::Value::new(
            Length::from_value(get_property_value("blur")?.clone(), doc, line_number)?,
            Some(line_number),
            vec![get_property_value("blur")?
                .into_property(ftd::interpreter::PropertySource::header("blur"))],
        );

        let spread = ftd::executor::Value::new(
            Length::from_value(get_property_value("spread")?.clone(), doc, line_number)?,
            Some(line_number),
            vec![get_property_value("spread")?
                .into_property(ftd::interpreter::PropertySource::header("spread"))],
        );

        let color = ftd::executor::Value::new(
            Color::from_value(get_property_value("color")?.clone(), doc, line_number)?,
            Some(line_number),
            vec![get_property_value("color")?
                .into_property(ftd::interpreter::PropertySource::header("color"))],
        );

        let inset = ftd::executor::Value::new(
            get_property_value("inset")?
                .clone()
                .resolve(&doc.itdoc(), line_number)?
                .bool(doc.name, line_number)?,
            Some(line_number),
            vec![get_property_value("inset")?
                .into_property(ftd::interpreter::PropertySource::header("inset"))],
        );

        Ok(ftd::executor::Shadow {
            x_offset,
            y_offset,
            blur,
            spread,
            color,
            inset,
        })
    }

    fn from_optional_values(
        or_type_value: Option<ftd::Map<fastn_type::PropertyValue>>,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Option<ftd::executor::Shadow>> {
        if let Some(value) = or_type_value {
            Ok(Some(ftd::executor::Shadow::from_values(
                value,
                doc,
                line_number,
            )?))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn optional_shadow(
        properties: &[ftd::interpreter::Property],
        arguments: &[ftd::interpreter::Argument],
        doc: &ftd::executor::TDoc,
        line_number: usize,
        key: &str,
        inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
        component_name: &str,
    ) -> ftd::executor::Result<ftd::executor::Value<Option<ftd::executor::Shadow>>> {
        let record_values = ftd::executor::value::optional_record_inherited(
            key,
            component_name,
            properties,
            arguments,
            doc,
            line_number,
            ftd::interpreter::FTD_SHADOW,
            inherited_variables,
        )?;

        Ok(ftd::executor::Value::new(
            ftd::executor::Shadow::from_optional_values(record_values.value, doc, line_number)?,
            record_values.line_number,
            record_values.properties,
        ))
    }

    pub fn to_css_string(&self, device: &Option<ftd::executor::Device>) -> String {
        let x_offset = self.x_offset.value.to_css_string(device);
        let y_offset = self.y_offset.value.to_css_string(device);
        let blur = self.blur.value.to_css_string(device);
        let spread = self.spread.value.to_css_string(device);
        let inset = match self.inset.value {
            true => "inset".to_string(),
            false => "".to_string(),
        };
        let color = self.color.value.to_css_string();

        format!(
            "{} {} {} {} {} {}",
            inset, color, x_offset, y_offset, blur, spread
        )
    }

    pub fn box_shadow_pattern() -> (String, bool) {
        (
            "window.ftd.dependencies.eval_box_shadow({0}, data)".to_string(),
            true,
        )
    }
}

#[derive(serde::Deserialize, Debug, Default, PartialEq, Clone, serde::Serialize)]
pub struct Color {
    pub light: ftd::executor::Value<ColorValue>,
    pub dark: ftd::executor::Value<ColorValue>,
}

impl Color {
    fn from_value(
        value: fastn_type::PropertyValue,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Color> {
        let value = value.resolve(&doc.itdoc(), line_number)?;
        let fields = match value.inner() {
            Some(fastn_type::Value::Record { name, fields })
                if name.eq(ftd::interpreter::FTD_COLOR) =>
            {
                fields
            }
            t => {
                return ftd::executor::utils::parse_error(
                    format!(
                        "Expected value of type record `{}`, found: {:?}",
                        ftd::interpreter::FTD_COLOR,
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
        values: ftd::Map<fastn_type::PropertyValue>,
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
                vec![value.into_property(ftd::interpreter::PropertySource::header("light"))],
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
                    vec![value.into_property(ftd::interpreter::PropertySource::header("dark"))],
                )
            } else {
                light.clone()
            }
        };

        Ok(Color { light, dark })
    }

    fn from_optional_values(
        or_type_value: Option<ftd::Map<fastn_type::PropertyValue>>,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Option<Color>> {
        if let Some(value) = or_type_value {
            Ok(Some(Color::from_values(value, doc, line_number)?))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn optional_color(
        properties: &[ftd::interpreter::Property],
        arguments: &[ftd::interpreter::Argument],
        doc: &ftd::executor::TDoc,
        line_number: usize,
        key: &str,
        inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
        component_name: &str,
    ) -> ftd::executor::Result<ftd::executor::Value<Option<Color>>> {
        let record_values = ftd::executor::value::optional_record_inherited(
            key,
            component_name,
            properties,
            arguments,
            doc,
            line_number,
            ftd::interpreter::FTD_COLOR,
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

    pub fn color_pattern() -> (String, bool) {
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
        or_type_value: Option<(String, fastn_type::PropertyValue)>,
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
        or_type_value: (String, fastn_type::PropertyValue),
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Self> {
        match or_type_value.0.as_str() {
            ftd::interpreter::FTD_SPACING_SPACE_BETWEEN => Ok(Spacing::SpaceBetween),
            ftd::interpreter::FTD_SPACING_SPACE_EVENLY => Ok(Spacing::SpaceEvenly),
            ftd::interpreter::FTD_SPACING_SPACE_AROUND => Ok(Spacing::SpaceAround),
            ftd::interpreter::FTD_SPACING_FIXED => Ok(Spacing::Fixed(Length::from_value(
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
        properties: &[ftd::interpreter::Property],
        arguments: &[ftd::interpreter::Argument],
        doc: &ftd::executor::TDoc,
        line_number: usize,
        key: &str,
        inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
        component_name: &str,
    ) -> ftd::executor::Result<ftd::executor::Value<Option<Spacing>>> {
        let or_type_value = ftd::executor::value::optional_or_type(
            key,
            component_name,
            properties,
            arguments,
            doc,
            line_number,
            ftd::interpreter::FTD_SPACING,
            inherited_variables,
        )?;

        Ok(ftd::executor::Value::new(
            Spacing::from_optional_values(or_type_value.value, doc, line_number)?,
            or_type_value.line_number,
            or_type_value.properties,
        ))
    }

    pub fn to_gap_css_string(&self, device: &Option<ftd::executor::Device>) -> String {
        match self {
            Spacing::Fixed(f) => f.to_css_string(device),
            _ => "0".to_string(),
        }
    }

    pub fn to_justify_content_css_string(&self) -> String {
        match self {
            Spacing::SpaceBetween => "space-between".to_string(),
            Spacing::SpaceEvenly => "space-evenly".to_string(),
            Spacing::SpaceAround => "space-around".to_string(),
            Spacing::Fixed(_) => "unset".to_string(),
        }
    }

    pub fn justify_content_pattern() -> (String, bool) {
        (
            indoc::indoc! {"
                if ({0} == \"space-between\" || {0} == \"space-around\" || {0} == \"space-evenly\") {
                    {0}
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
                if ({0} != \"space-between\" && {0} != \"space_around\" && {0} != \"space-evenly\") {
                    {0}
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
        or_type_value: Option<(String, fastn_type::PropertyValue)>,
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
        or_type_value: (String, fastn_type::PropertyValue),
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Self> {
        match or_type_value.0.as_str() {
            ftd::interpreter::FTD_ALIGN_SELF_START => Ok(AlignSelf::Start),
            ftd::interpreter::FTD_ALIGN_SELF_CENTER => Ok(AlignSelf::Center),
            ftd::interpreter::FTD_ALIGN_SELF_END => Ok(AlignSelf::End),
            t => ftd::executor::utils::parse_error(
                format!("Unknown variant `{}` for or-type `ftd.align-self`", t),
                doc.name,
                line_number,
            ),
        }
    }

    pub(crate) fn optional_align_self(
        properties: &[ftd::interpreter::Property],
        arguments: &[ftd::interpreter::Argument],
        doc: &ftd::executor::TDoc,
        line_number: usize,
        key: &str,
        inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
        component_name: &str,
    ) -> ftd::executor::Result<ftd::executor::Value<Option<AlignSelf>>> {
        let or_type_value = ftd::executor::value::optional_or_type(
            key,
            component_name,
            properties,
            arguments,
            doc,
            line_number,
            ftd::interpreter::FTD_ALIGN_SELF,
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
        or_type_value: Option<(String, fastn_type::PropertyValue)>,
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
        or_type_value: (String, fastn_type::PropertyValue),
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Self> {
        match or_type_value.0.as_str() {
            ftd::interpreter::FTD_OVERFLOW_SCROLL => Ok(Overflow::Scroll),
            ftd::interpreter::FTD_OVERFLOW_VISIBLE => Ok(Overflow::Visible),
            ftd::interpreter::FTD_OVERFLOW_HIDDEN => Ok(Overflow::Hidden),
            ftd::interpreter::FTD_OVERFLOW_AUTO => Ok(Overflow::Auto),
            t => ftd::executor::utils::parse_error(
                format!("Unknown variant `{}` for or-type `ftd.overflow`", t),
                doc.name,
                line_number,
            ),
        }
    }

    pub(crate) fn optional_overflow(
        properties: &[ftd::interpreter::Property],
        arguments: &[ftd::interpreter::Argument],
        doc: &ftd::executor::TDoc,
        line_number: usize,
        key: &str,
        inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
        component_name: &str,
    ) -> ftd::executor::Result<ftd::executor::Value<Option<Overflow>>> {
        let or_type_value = ftd::executor::value::optional_or_type(
            key,
            component_name,
            properties,
            arguments,
            doc,
            line_number,
            ftd::interpreter::FTD_OVERFLOW,
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
        or_type_value: Option<(String, fastn_type::PropertyValue)>,
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
        or_type_value: (String, fastn_type::PropertyValue),
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Self> {
        match or_type_value.0.as_str() {
            ftd::interpreter::FTD_RESIZE_HORIZONTAL => Ok(Resize::Horizontal),
            ftd::interpreter::FTD_RESIZE_VERTICAL => Ok(Resize::Vertical),
            ftd::interpreter::FTD_RESIZE_BOTH => Ok(Resize::Both),
            t => ftd::executor::utils::parse_error(
                format!("Unknown variant `{}` for or-type `ftd.resize`", t),
                doc.name,
                line_number,
            ),
        }
    }

    pub(crate) fn optional_resize(
        properties: &[ftd::interpreter::Property],
        arguments: &[ftd::interpreter::Argument],
        doc: &ftd::executor::TDoc,
        line_number: usize,
        key: &str,
        inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
        component_name: &str,
    ) -> ftd::executor::Result<ftd::executor::Value<Option<Resize>>> {
        let or_type_value = ftd::executor::value::optional_or_type(
            key,
            component_name,
            properties,
            arguments,
            doc,
            line_number,
            ftd::interpreter::FTD_RESIZE,
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
        or_type_value: Option<(String, fastn_type::PropertyValue)>,
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
        or_type_value: (String, fastn_type::PropertyValue),
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Self> {
        match or_type_value.0.as_str() {
            ftd::interpreter::FTD_TEXT_ALIGN_START => Ok(TextAlign::Start),
            ftd::interpreter::FTD_TEXT_ALIGN_CENTER => Ok(TextAlign::Center),
            ftd::interpreter::FTD_TEXT_ALIGN_END => Ok(TextAlign::End),
            ftd::interpreter::FTD_TEXT_ALIGN_JUSTIFY => Ok(TextAlign::Justify),
            t => ftd::executor::utils::parse_error(
                format!("Unknown variant `{}` for or-type `ftd.text-align`", t),
                doc.name,
                line_number,
            ),
        }
    }

    pub(crate) fn optional_text_align(
        properties: &[ftd::interpreter::Property],
        arguments: &[ftd::interpreter::Argument],
        doc: &ftd::executor::TDoc,
        line_number: usize,
        key: &str,
        inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
        component_name: &str,
    ) -> ftd::executor::Result<ftd::executor::Value<Option<TextAlign>>> {
        let or_type_value = ftd::executor::value::optional_or_type(
            key,
            component_name,
            properties,
            arguments,
            doc,
            line_number,
            ftd::interpreter::FTD_TEXT_ALIGN,
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
        or_type_value: Option<(String, fastn_type::PropertyValue)>,
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
        or_type_value: (String, fastn_type::PropertyValue),
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Self> {
        match or_type_value.0.as_str() {
            ftd::interpreter::FTD_CURSOR_DEFAULT => Ok(Cursor::Default),
            ftd::interpreter::FTD_CURSOR_NONE => Ok(Cursor::None),
            ftd::interpreter::FTD_CURSOR_CONTEXT_MENU => Ok(Cursor::ContextMenu),
            ftd::interpreter::FTD_CURSOR_HELP => Ok(Cursor::Help),
            ftd::interpreter::FTD_CURSOR_POINTER => Ok(Cursor::Pointer),
            ftd::interpreter::FTD_CURSOR_PROGRESS => Ok(Cursor::Progress),
            ftd::interpreter::FTD_CURSOR_WAIT => Ok(Cursor::Wait),
            ftd::interpreter::FTD_CURSOR_CELL => Ok(Cursor::Cell),
            ftd::interpreter::FTD_CURSOR_CROSSHAIR => Ok(Cursor::CrossHair),
            ftd::interpreter::FTD_CURSOR_TEXT => Ok(Cursor::Text),
            ftd::interpreter::FTD_CURSOR_VERTICAL_TEXT => Ok(Cursor::VerticalText),
            ftd::interpreter::FTD_CURSOR_ALIAS => Ok(Cursor::Alias),
            ftd::interpreter::FTD_CURSOR_COPY => Ok(Cursor::Copy),
            ftd::interpreter::FTD_CURSOR_MOVE => Ok(Cursor::Move),
            ftd::interpreter::FTD_CURSOR_NO_DROP => Ok(Cursor::NoDrop),
            ftd::interpreter::FTD_CURSOR_NOT_ALLOWED => Ok(Cursor::NotAllowed),
            ftd::interpreter::FTD_CURSOR_GRAB => Ok(Cursor::Grab),
            ftd::interpreter::FTD_CURSOR_GRABBING => Ok(Cursor::Grabbing),
            ftd::interpreter::FTD_CURSOR_E_RESIZE => Ok(Cursor::EResize),
            ftd::interpreter::FTD_CURSOR_N_RESIZE => Ok(Cursor::NResize),
            ftd::interpreter::FTD_CURSOR_NE_RESIZE => Ok(Cursor::NeResize),
            ftd::interpreter::FTD_CURSOR_NW_RESIZE => Ok(Cursor::NwResize),
            ftd::interpreter::FTD_CURSOR_S_RESIZE => Ok(Cursor::SResize),
            ftd::interpreter::FTD_CURSOR_SE_RESIZE => Ok(Cursor::SeResize),
            ftd::interpreter::FTD_CURSOR_SW_RESIZE => Ok(Cursor::SwResize),
            ftd::interpreter::FTD_CURSOR_W_RESIZE => Ok(Cursor::WResize),
            ftd::interpreter::FTD_CURSOR_EW_RESIZE => Ok(Cursor::EwResize),
            ftd::interpreter::FTD_CURSOR_NS_RESIZE => Ok(Cursor::NsResize),
            ftd::interpreter::FTD_CURSOR_NESW_RESIZE => Ok(Cursor::NeswResize),
            ftd::interpreter::FTD_CURSOR_NWSE_RESIZE => Ok(Cursor::NwseResize),
            ftd::interpreter::FTD_CURSOR_COL_RESIZE => Ok(Cursor::ColResize),
            ftd::interpreter::FTD_CURSOR_ROW_RESIZE => Ok(Cursor::RowResize),
            ftd::interpreter::FTD_CURSOR_ALL_SCROLL => Ok(Cursor::AllScroll),
            ftd::interpreter::FTD_CURSOR_ZOOM_IN => Ok(Cursor::ZoomIn),
            ftd::interpreter::FTD_CURSOR_ZOOM_OUT => Ok(Cursor::ZoomOut),
            t => ftd::executor::utils::parse_error(
                format!("Unknown variant `{}` for or-type `ftd.cursor`", t),
                doc.name,
                line_number,
            ),
        }
    }

    pub(crate) fn optional_cursor(
        properties: &[ftd::interpreter::Property],
        arguments: &[ftd::interpreter::Argument],
        doc: &ftd::executor::TDoc,
        line_number: usize,
        key: &str,
        inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
        component_name: &str,
    ) -> ftd::executor::Result<ftd::executor::Value<Option<Cursor>>> {
        let or_type_value = ftd::executor::value::optional_or_type(
            key,
            component_name,
            properties,
            arguments,
            doc,
            line_number,
            ftd::interpreter::FTD_CURSOR,
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
        value: fastn_type::PropertyValue,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Option<FontSize>> {
        let value = value.resolve(&doc.itdoc(), line_number)?;
        match value.inner() {
            Some(fastn_type::Value::OrType {
                name,
                variant,
                value,
                ..
            }) if name.eq(ftd::interpreter::FTD_FONT_SIZE) => Ok(Some(FontSize::from_values(
                (variant, value.as_ref().to_owned()),
                doc,
                line_number,
            )?)),
            None => Ok(None),
            t => ftd::executor::utils::parse_error(
                format!(
                    "Expected value of font-size or-type `{}`, found: {:?}",
                    ftd::interpreter::FTD_FONT_SIZE,
                    t
                ),
                doc.name,
                line_number,
            ),
        }
    }

    fn from_values(
        or_type_value: (String, fastn_type::PropertyValue),
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Self> {
        match or_type_value.0.as_str() {
            ftd::interpreter::FTD_FONT_SIZE_PX => Ok(FontSize::Px(
                or_type_value
                    .1
                    .clone()
                    .resolve(&doc.itdoc(), line_number)?
                    .integer(doc.name, line_number)?,
            )),
            ftd::interpreter::FTD_FONT_SIZE_EM => Ok(FontSize::Em(
                or_type_value
                    .1
                    .clone()
                    .resolve(&doc.itdoc(), line_number)?
                    .decimal(doc.name, line_number)?,
            )),
            ftd::interpreter::FTD_FONT_SIZE_REM => Ok(FontSize::Rem(
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
            ftd::interpreter::FTD_FONT_SIZE_PX
            | ftd::interpreter::FTD_FONT_SIZE_EM
            | ftd::interpreter::FTD_FONT_SIZE_REM => Ok("{0}"),
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
            ftd::interpreter::FTD_FONT_SIZE_PX => Ok("{0}px"),
            ftd::interpreter::FTD_FONT_SIZE_EM => Ok("{0}em"),
            ftd::interpreter::FTD_FONT_SIZE_REM => Ok("{0}rem"),
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
            ftd::interpreter::FTD_FONT_SIZE_PX => Ok(format!("{}px", value)),
            ftd::interpreter::FTD_FONT_SIZE_EM => Ok(format!("{}em", value)),
            ftd::interpreter::FTD_FONT_SIZE_REM => Ok(format!("{}rem", value)),
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
    pub font_family: Option<Vec<String>>,
}

impl Type {
    fn new(
        size: Option<FontSize>,
        line_height: Option<FontSize>,
        letter_spacing: Option<FontSize>,
        weight: Option<i64>,
        font_family: Option<Vec<String>>,
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
        value: fastn_type::PropertyValue,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Type> {
        let value = value.resolve(&doc.itdoc(), line_number)?;
        let fields = match value.inner() {
            Some(fastn_type::Value::Record { name, fields })
                if name.eq(ftd::interpreter::FTD_TYPE) =>
            {
                fields
            }
            t => {
                return ftd::executor::utils::parse_error(
                    format!(
                        "Expected value of type record `{}`, found: {:?}",
                        ftd::interpreter::FTD_COLOR,
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
        values: ftd::Map<fastn_type::PropertyValue>,
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
                Some(
                    value
                        .clone()
                        .resolve(&doc.itdoc(), line_number)?
                        .string_list(&doc.itdoc(), line_number)?,
                )
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
        values: ftd::Map<fastn_type::PropertyValue>,
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
        or_type_value: Option<ftd::Map<fastn_type::PropertyValue>>,
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
        properties: &[ftd::interpreter::Property],
        arguments: &[ftd::interpreter::Argument],
        doc: &ftd::executor::TDoc,
        line_number: usize,
        key: &str,
        inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
        component_name: &str,
    ) -> ftd::executor::Result<ftd::executor::Value<Option<ResponsiveType>>> {
        let record_values = ftd::executor::value::optional_record_inherited(
            key,
            component_name,
            properties,
            arguments,
            doc,
            line_number,
            ftd::interpreter::FTD_RESPONSIVE_TYPE,
            inherited_variables,
        )?;

        Ok(ftd::executor::Value::new(
            ResponsiveType::from_optional_values(record_values.value, doc, line_number)?,
            record_values.line_number,
            record_values.properties,
        ))
    }

    pub fn to_css_font_size(&self, device: &Option<ftd::executor::Device>) -> Option<String> {
        match device {
            Some(ftd::executor::Device::Mobile) => {
                self.mobile.size.as_ref().map(|v| v.to_css_string())
            }
            _ => self.desktop.size.as_ref().map(|v| v.to_css_string()),
        }
    }

    pub fn font_size_pattern() -> (String, bool) {
        ("({0})[\"size\"]".to_string(), true)
    }

    pub fn to_css_line_height(&self, device: &Option<ftd::executor::Device>) -> Option<String> {
        match device {
            Some(ftd::executor::Device::Mobile) => {
                self.mobile.line_height.as_ref().map(|v| v.to_css_string())
            }
            _ => self.desktop.line_height.as_ref().map(|v| v.to_css_string()),
        }
    }

    pub fn line_height_pattern() -> (String, bool) {
        ("({0})[\"line-height\"]".to_string(), true)
    }

    pub fn to_css_letter_spacing(&self, device: &Option<ftd::executor::Device>) -> Option<String> {
        match device {
            Some(ftd::executor::Device::Mobile) => self
                .mobile
                .letter_spacing
                .as_ref()
                .map(|v| v.to_css_string()),
            _ => self
                .desktop
                .letter_spacing
                .as_ref()
                .map(|v| v.to_css_string()),
        }
    }

    pub fn letter_spacing_pattern() -> (String, bool) {
        ("({0})[\"letter-spacing\"]".to_string(), true)
    }

    pub fn to_css_weight(&self, device: &Option<ftd::executor::Device>) -> Option<String> {
        match device {
            Some(ftd::executor::Device::Mobile) => {
                self.mobile.weight.as_ref().map(|v| v.to_string())
            }
            _ => self.desktop.weight.as_ref().map(|v| v.to_string()),
        }
    }

    pub fn weight_pattern() -> (String, bool) {
        ("({0}).weight".to_string(), true)
    }

    pub fn to_css_font_family(&self) -> Option<String> {
        if let Some(font_family) = self.desktop.font_family.as_ref() {
            return Some(font_family.join(", "));
        }
        None
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
        or_type_value: Option<(String, fastn_type::PropertyValue)>,
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
        or_type_value: (String, fastn_type::PropertyValue),
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Self> {
        match or_type_value.0.as_str() {
            ftd::interpreter::FTD_ANCHOR_WINDOW => Ok(Anchor::Window),
            ftd::interpreter::FTD_ANCHOR_PARENT => Ok(Anchor::Parent),
            ftd::interpreter::FTD_ANCHOR_ID => Ok(Anchor::Id(
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
        properties: &[ftd::interpreter::Property],
        arguments: &[ftd::interpreter::Argument],
        doc: &ftd::executor::TDoc,
        line_number: usize,
        key: &str,
        inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
        component_name: &str,
    ) -> ftd::executor::Result<ftd::executor::Value<Option<Anchor>>> {
        let or_type_value = ftd::executor::value::optional_or_type(
            key,
            component_name,
            properties,
            arguments,
            doc,
            line_number,
            ftd::interpreter::FTD_ANCHOR,
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
    DATETIME,
    DATE,
    TIME,
    MONTH,
    WEEK,
    COLOR,
    FILE,
}

impl TextInputType {
    fn from_optional_values(
        or_type_value: Option<(String, fastn_type::PropertyValue)>,
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
        or_type_value: (String, fastn_type::PropertyValue),
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Self> {
        match or_type_value.0.as_str() {
            ftd::interpreter::FTD_TEXT_INPUT_TYPE_TEXT => Ok(TextInputType::TEXT),
            ftd::interpreter::FTD_TEXT_INPUT_TYPE_EMAIL => Ok(TextInputType::EMAIL),
            ftd::interpreter::FTD_TEXT_INPUT_TYPE_PASSWORD => Ok(TextInputType::PASSWORD),
            ftd::interpreter::FTD_TEXT_INPUT_TYPE_URL => Ok(TextInputType::URL),
            ftd::interpreter::FTD_TEXT_INPUT_TYPE_DATETIME => Ok(TextInputType::DATETIME),
            ftd::interpreter::FTD_TEXT_INPUT_TYPE_DATE => Ok(TextInputType::DATE),
            ftd::interpreter::FTD_TEXT_INPUT_TYPE_TIME => Ok(TextInputType::TIME),
            ftd::interpreter::FTD_TEXT_INPUT_TYPE_MONTH => Ok(TextInputType::MONTH),
            ftd::interpreter::FTD_TEXT_INPUT_TYPE_WEEK => Ok(TextInputType::WEEK),
            ftd::interpreter::FTD_TEXT_INPUT_TYPE_COLOR => Ok(TextInputType::COLOR),
            ftd::interpreter::FTD_TEXT_INPUT_TYPE_FILE => Ok(TextInputType::FILE),
            t => ftd::executor::utils::parse_error(
                format!("Unknown variant `{}` for or-type `ftd.text-input-type`", t),
                doc.name,
                line_number,
            ),
        }
    }

    pub(crate) fn optional_text_input_type(
        properties: &[ftd::interpreter::Property],
        arguments: &[ftd::interpreter::Argument],
        doc: &ftd::executor::TDoc,
        line_number: usize,
        key: &str,
        inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
        component_name: &str,
    ) -> ftd::executor::Result<ftd::executor::Value<Option<TextInputType>>> {
        let or_type_value = ftd::executor::value::optional_or_type(
            key,
            component_name,
            properties,
            arguments,
            doc,
            line_number,
            ftd::interpreter::FTD_TEXT_INPUT_TYPE,
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
            TextInputType::DATETIME => "datetime-local".to_string(),
            TextInputType::DATE => "date".to_string(),
            TextInputType::TIME => "time".to_string(),
            TextInputType::MONTH => "month".to_string(),
            TextInputType::WEEK => "week".to_string(),
            TextInputType::COLOR => "color".to_string(),
            TextInputType::FILE => "file".to_string(),
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
        or_type_value: Option<(String, fastn_type::PropertyValue)>,
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
        or_type_value: (String, fastn_type::PropertyValue),
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Self> {
        match or_type_value.0.as_str() {
            ftd::interpreter::FTD_REGION_H1 => Ok(Region::H1),
            ftd::interpreter::FTD_REGION_H2 => Ok(Region::H2),
            ftd::interpreter::FTD_REGION_H3 => Ok(Region::H3),
            ftd::interpreter::FTD_REGION_H4 => Ok(Region::H4),
            ftd::interpreter::FTD_REGION_H5 => Ok(Region::H5),
            ftd::interpreter::FTD_REGION_H6 => Ok(Region::H6),
            t => ftd::executor::utils::parse_error(
                format!("Unknown variant `{}` for or-type `ftd.region`", t),
                doc.name,
                line_number,
            ),
        }
    }

    pub(crate) fn optional_region(
        properties: &[ftd::interpreter::Property],
        arguments: &[ftd::interpreter::Argument],
        doc: &ftd::executor::TDoc,
        line_number: usize,
        key: &str,
        inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
        component_name: &str,
    ) -> ftd::executor::Result<ftd::executor::Value<Option<Region>>> {
        let or_type_value = ftd::executor::value::optional_or_type(
            key,
            component_name,
            properties,
            arguments,
            doc,
            line_number,
            ftd::interpreter::FTD_REGION,
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
        or_type_value: Option<(String, fastn_type::PropertyValue)>,
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
        or_type_value: (String, fastn_type::PropertyValue),
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Self> {
        match or_type_value.0.as_str() {
            ftd::interpreter::FTD_WHITESPACE_NORMAL => Ok(WhiteSpace::NORMAL),
            ftd::interpreter::FTD_WHITESPACE_NOWRAP => Ok(WhiteSpace::NOWRAP),
            ftd::interpreter::FTD_WHITESPACE_PRE => Ok(WhiteSpace::PRE),
            ftd::interpreter::FTD_WHITESPACE_PREWRAP => Ok(WhiteSpace::PREWRAP),
            ftd::interpreter::FTD_WHITESPACE_PRELINE => Ok(WhiteSpace::PRELINE),
            ftd::interpreter::FTD_WHITESPACE_BREAKSPACES => Ok(WhiteSpace::BREAKSPACES),
            t => ftd::executor::utils::parse_error(
                format!("Unknown variant `{}` for or-type `ftd.whitespace`", t),
                doc.name,
                line_number,
            ),
        }
    }

    pub(crate) fn optional_whitespace(
        properties: &[ftd::interpreter::Property],
        arguments: &[ftd::interpreter::Argument],
        doc: &ftd::executor::TDoc,
        line_number: usize,
        key: &str,
        inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
        component_name: &str,
    ) -> ftd::executor::Result<ftd::executor::Value<Option<WhiteSpace>>> {
        let or_type_value = ftd::executor::value::optional_or_type(
            key,
            component_name,
            properties,
            arguments,
            doc,
            line_number,
            ftd::interpreter::FTD_WHITESPACE,
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
pub enum Display {
    Block,
    Inline,
    InlineBlock,
}

impl Display {
    fn from_optional_values(
        or_type_value: Option<(String, fastn_type::PropertyValue)>,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Option<ftd::executor::Display>> {
        if let Some(value) = or_type_value {
            Ok(Some(ftd::executor::Display::from_values(
                value,
                doc,
                line_number,
            )?))
        } else {
            Ok(None)
        }
    }

    fn from_values(
        or_type_value: (String, fastn_type::PropertyValue),
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<ftd::executor::Display> {
        match or_type_value.0.as_str() {
            ftd::interpreter::FTD_DISPLAY_BLOCK => Ok(ftd::executor::Display::Block),
            ftd::interpreter::FTD_DISPLAY_INLINE => Ok(ftd::executor::Display::Inline),
            ftd::interpreter::FTD_DISPLAY_INLINE_BLOCK => Ok(ftd::executor::Display::InlineBlock),
            t => ftd::executor::utils::parse_error(
                format!("Unknown variant `{}` for or-type `ftd.display`", t),
                doc.name,
                line_number,
            ),
        }
    }

    pub(crate) fn optional_display(
        properties: &[ftd::interpreter::Property],
        arguments: &[ftd::interpreter::Argument],
        doc: &ftd::executor::TDoc,
        line_number: usize,
        key: &str,
        inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
        component_name: &str,
    ) -> ftd::executor::Result<ftd::executor::Value<Option<ftd::executor::Display>>> {
        let or_type_value = ftd::executor::value::optional_or_type(
            key,
            component_name,
            properties,
            arguments,
            doc,
            line_number,
            ftd::interpreter::FTD_DISPLAY,
            inherited_variables,
        )?;

        Ok(ftd::executor::Value::new(
            ftd::executor::Display::from_optional_values(or_type_value.value, doc, line_number)?,
            or_type_value.line_number,
            or_type_value.properties,
        ))
    }

    pub fn to_css_str(&self) -> &str {
        match self {
            ftd::executor::Display::InlineBlock => "inline-block",
            ftd::executor::Display::Inline => "inline",
            ftd::executor::Display::Block => "block",
        }
    }
}

#[derive(serde::Deserialize, Debug, PartialEq, Clone, serde::Serialize)]
pub enum TextWeight {
    EXTRABOLD,
    BOLD,
    SEMIBOLD,
    HEAVY,
    MEDIUM,
    REGULAR,
    LIGHT,
    EXTRALIGHT,
    HAIRLINE,
}

impl TextWeight {
    pub fn to_weight_string(&self) -> String {
        match self {
            TextWeight::HEAVY => "900".to_string(),
            TextWeight::EXTRABOLD => "800".to_string(),
            TextWeight::BOLD => "700".to_string(),
            TextWeight::SEMIBOLD => "600".to_string(),
            TextWeight::MEDIUM => "500".to_string(),
            TextWeight::REGULAR => "400".to_string(),
            TextWeight::LIGHT => "300".to_string(),
            TextWeight::EXTRALIGHT => "200".to_string(),
            TextWeight::HAIRLINE => "100".to_string(),
        }
    }

    pub fn from_type_to_weight(weight_type: &str) -> String {
        match weight_type {
            "heavy" => "900".to_string(),
            "extra-bold" => "800".to_string(),
            "bold" => "700".to_string(),
            "semi-bold" => "600".to_string(),
            "medium" => "500".to_string(),
            "regular" => "400".to_string(),
            "light" => "300".to_string(),
            "extra-light" => "200".to_string(),
            "hairline" => "100".to_string(),
            _ => "none".to_string(),
        }
    }

    pub fn is_valid_weight_type(value: &str) -> bool {
        matches!(
            value,
            "hairline"
                | "extra-bold"
                | "extra-light"
                | "bold"
                | "semi-bold"
                | "light"
                | "medium"
                | "regular"
                | "heavy"
        )
    }

    pub fn is_valid_text_weight(value: &str) -> bool {
        fn is_numeric_value(s: String) -> bool {
            for c in s.chars() {
                if !c.is_numeric() {
                    return false;
                }
            }
            true
        }

        match value {
            c1 if TextWeight::is_valid_weight_type(c1) => true,
            c2 if is_numeric_value(c2.to_string()) => true,
            _ => false,
        }
    }
}

#[derive(serde::Deserialize, Debug, Default, PartialEq, Clone, serde::Serialize)]
pub struct TextStyle {
    pub underline: bool,
    pub italic: bool,
    pub strike: bool,
    pub weight: Option<TextWeight>,
}

impl TextStyle {
    fn default() -> Self {
        TextStyle {
            underline: false,
            italic: false,
            strike: false,
            weight: None,
        }
    }

    fn from_optional_values(
        or_type_value: Option<Vec<(String, fastn_type::PropertyValue)>>,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Option<Self>> {
        if let Some(value) = or_type_value {
            Ok(TextStyle::from_values(value, doc, line_number)?)
        } else {
            Ok(None)
        }
    }

    fn from_values(
        or_type_values: Vec<(String, fastn_type::PropertyValue)>,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Option<Self>> {
        fn add_in_map(style: &str, map: &mut ftd::Map<i32>) {
            if !map.contains_key(style) {
                map.insert(style.to_string(), 1);
                return;
            }
            map.insert(style.to_string(), map[style] + 1);
        }

        let mut text_style = Self::default();
        let mut booleans: ftd::Map<i32> = Default::default();
        let mut weights: ftd::Map<i32> = Default::default();

        if or_type_values.is_empty() {
            return Ok(None);
        }

        for value in or_type_values {
            match value.0.as_str() {
                ftd::interpreter::FTD_TEXT_STYLE_ITALIC => {
                    text_style.italic = true;
                    add_in_map(ftd::interpreter::FTD_TEXT_STYLE_ITALIC, &mut booleans);
                }
                ftd::interpreter::FTD_TEXT_STYLE_STRIKE => {
                    text_style.strike = true;
                    add_in_map(ftd::interpreter::FTD_TEXT_STYLE_STRIKE, &mut booleans);
                }
                ftd::interpreter::FTD_TEXT_STYLE_UNDERLINE => {
                    text_style.underline = true;
                    add_in_map(ftd::interpreter::FTD_TEXT_STYLE_UNDERLINE, &mut booleans);
                }
                ftd::interpreter::FTD_TEXT_STYLE_WEIGHT_BOLD => {
                    text_style.weight = Some(TextWeight::BOLD);
                    add_in_map(ftd::interpreter::FTD_TEXT_STYLE_WEIGHT_BOLD, &mut weights);
                }
                ftd::interpreter::FTD_TEXT_STYLE_WEIGHT_EXTRA_BOLD => {
                    text_style.weight = Some(TextWeight::EXTRABOLD);
                    add_in_map(
                        ftd::interpreter::FTD_TEXT_STYLE_WEIGHT_EXTRA_BOLD,
                        &mut weights,
                    );
                }
                ftd::interpreter::FTD_TEXT_STYLE_WEIGHT_SEMI_BOLD => {
                    text_style.weight = Some(TextWeight::SEMIBOLD);
                    add_in_map(
                        ftd::interpreter::FTD_TEXT_STYLE_WEIGHT_SEMI_BOLD,
                        &mut weights,
                    );
                }
                ftd::interpreter::FTD_TEXT_STYLE_WEIGHT_HEAVY => {
                    text_style.weight = Some(TextWeight::HEAVY);
                    add_in_map(ftd::interpreter::FTD_TEXT_STYLE_WEIGHT_HEAVY, &mut weights);
                }
                ftd::interpreter::FTD_TEXT_STYLE_WEIGHT_EXTRA_LIGHT => {
                    text_style.weight = Some(TextWeight::EXTRALIGHT);
                    add_in_map(
                        ftd::interpreter::FTD_TEXT_STYLE_WEIGHT_EXTRA_LIGHT,
                        &mut weights,
                    );
                }
                ftd::interpreter::FTD_TEXT_STYLE_WEIGHT_LIGHT => {
                    text_style.weight = Some(TextWeight::LIGHT);
                    add_in_map(ftd::interpreter::FTD_TEXT_STYLE_WEIGHT_LIGHT, &mut weights);
                }
                ftd::interpreter::FTD_TEXT_STYLE_WEIGHT_MEDIUM => {
                    text_style.weight = Some(TextWeight::MEDIUM);
                    add_in_map(ftd::interpreter::FTD_TEXT_STYLE_WEIGHT_MEDIUM, &mut weights);
                }
                ftd::interpreter::FTD_TEXT_STYLE_WEIGHT_REGULAR => {
                    text_style.weight = Some(TextWeight::REGULAR);
                    add_in_map(
                        ftd::interpreter::FTD_TEXT_STYLE_WEIGHT_REGULAR,
                        &mut weights,
                    );
                }
                ftd::interpreter::FTD_TEXT_STYLE_WEIGHT_HAIRLINE => {
                    text_style.weight = Some(TextWeight::HAIRLINE);
                    add_in_map(
                        ftd::interpreter::FTD_TEXT_STYLE_WEIGHT_HAIRLINE,
                        &mut weights,
                    );
                }
                t => {
                    return ftd::executor::utils::parse_error(
                        format!("Unknown variant `{}` for or-type `ftd.text-style`", t),
                        doc.name,
                        line_number,
                    )
                }
            };
        }

        // Check for repetition in values (underline, italic, strike)
        for (style, count) in booleans.iter() {
            if *count > 1 {
                return ftd::executor::utils::parse_error(
                    format!("\'{}\' repeated {} times", style, count),
                    doc.name,
                    line_number,
                );
            }
        }

        // Multiple font weight check
        if weights.len() > 1 {
            return ftd::executor::utils::parse_error(
                format!("Conflicting weights {:?}", weights.keys()),
                doc.name,
                line_number,
            );
        }

        // Font weight repetition check
        for (weight, count) in weights.iter() {
            if *count > 1 {
                return ftd::executor::utils::parse_error(
                    format!("\'{}\' repeated {} times ", weight, count),
                    doc.name,
                    line_number,
                );
            }
        }

        Ok(Some(text_style))
    }

    pub(crate) fn optional_text_style(
        properties: &[ftd::interpreter::Property],
        arguments: &[ftd::interpreter::Argument],
        doc: &ftd::executor::TDoc,
        line_number: usize,
        key: &str,
        inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
        component_name: &str,
    ) -> ftd::executor::Result<ftd::executor::Value<Option<TextStyle>>> {
        let or_type_value = ftd::executor::value::optional_or_type_list(
            key,
            component_name,
            properties,
            arguments,
            doc,
            line_number,
            ftd::interpreter::FTD_TEXT_STYLE,
            inherited_variables,
        )?;

        Ok(ftd::executor::Value::new(
            TextStyle::from_optional_values(Some(or_type_value.value), doc, line_number)?,
            or_type_value.line_number,
            or_type_value.properties,
        ))
    }

    pub fn font_style_string(&self) -> String {
        if self.italic {
            return "italic".to_string();
        }
        ftd::interpreter::FTD_IGNORE_KEY.to_string()
    }

    pub fn font_decoration_string(&self) -> String {
        let mut css_string: Vec<String> = vec![];
        if self.underline {
            css_string.push("underline".to_string());
        }
        if self.strike {
            css_string.push("line-through".to_string());
        }

        if css_string.is_empty() {
            return ftd::interpreter::FTD_IGNORE_KEY.to_string();
        }
        css_string.join(" ")
    }

    pub fn font_weight_string(&self) -> String {
        if let Some(weight) = self.weight.as_ref() {
            return weight.to_weight_string();
        }
        ftd::interpreter::FTD_IGNORE_KEY.to_string()
    }

    pub fn filter_for_style(values: String) -> String {
        let mut result = String::new();
        for v in values
            .trim_start_matches('\"')
            .trim_end_matches('\"')
            .split(' ')
        {
            match v {
                "italic" => result.push_str(v),
                _ => continue,
            }
            result.push(' ');
        }

        let filtered = result.trim_end();
        let res = match filtered.is_empty() {
            true => return ftd::interpreter::FTD_VALUE_UNCHANGED.to_string(),
            false => filtered.to_string(),
        };
        format!("\"{}\"", res)
    }

    pub fn filter_for_decoration(values: String) -> String {
        let mut result = String::new();
        for v in values
            .trim_start_matches('\"')
            .trim_end_matches('\"')
            .split(' ')
        {
            match v {
                "underline" => result.push_str(v),
                "strike" => result.push_str("line-through"),
                _ => continue,
            }
            result.push(' ');
        }

        let filtered = result.trim_end();
        let res = match filtered.is_empty() {
            true => return ftd::interpreter::FTD_VALUE_UNCHANGED.to_string(),
            false => filtered.to_string(),
        };
        format!("\"{}\"", res)
    }

    pub fn filter_for_weight(values: String) -> String {
        let mut result = String::new();
        for v in values
            .trim_start_matches('\"')
            .trim_end_matches('\"')
            .split(' ')
        {
            match v {
                valid if TextWeight::is_valid_text_weight(valid) => {
                    result.push_str(TextWeight::from_type_to_weight(valid).as_str())
                }
                _ => continue,
            }
        }

        let filtered = result.trim_end();
        let res = match filtered.is_empty() {
            true => return ftd::interpreter::FTD_VALUE_UNCHANGED.to_string(),
            false => filtered.to_string(),
        };
        format!("\"{}\"", res)
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
        or_type_value: Option<(String, fastn_type::PropertyValue)>,
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
        or_type_value: (String, fastn_type::PropertyValue),
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Self> {
        match or_type_value.0.as_str() {
            ftd::interpreter::FTD_TEXT_TRANSFORM_NONE => Ok(TextTransform::NONE),
            ftd::interpreter::FTD_TEXT_TRANSFORM_CAPITALIZE => Ok(TextTransform::CAPITALIZE),
            ftd::interpreter::FTD_TEXT_TRANSFORM_UPPERCASE => Ok(TextTransform::UPPERCASE),
            ftd::interpreter::FTD_TEXT_TRANSFORM_LOWERCASE => Ok(TextTransform::LOWERCASE),
            ftd::interpreter::FTD_TEXT_TRANSFORM_INITIAL => Ok(TextTransform::INITIAL),
            ftd::interpreter::FTD_TEXT_TRANSFORM_INHERIT => Ok(TextTransform::INHERIT),
            t => ftd::executor::utils::parse_error(
                format!("Unknown variant `{}` for or-type `ftd.text-transform`", t),
                doc.name,
                line_number,
            ),
        }
    }

    pub(crate) fn optional_text_transform(
        properties: &[ftd::interpreter::Property],
        arguments: &[ftd::interpreter::Argument],
        doc: &ftd::executor::TDoc,
        line_number: usize,
        key: &str,
        inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
        component_name: &str,
    ) -> ftd::executor::Result<ftd::executor::Value<Option<TextTransform>>> {
        let or_type_value = ftd::executor::value::optional_or_type(
            key,
            component_name,
            properties,
            arguments,
            doc,
            line_number,
            ftd::interpreter::FTD_TEXT_TRANSFORM,
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
        or_type_value: Option<(String, fastn_type::PropertyValue)>,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Option<Self>> {
        if let Some(value) = or_type_value {
            return Ok(Some(BorderStyle::from_values(value, doc, line_number)?));
        }
        Ok(None)
    }

    fn from_values(
        or_type_value: (String, fastn_type::PropertyValue),
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Self> {
        match or_type_value.0.as_str() {
            ftd::interpreter::FTD_BORDER_STYLE_DOTTED => Ok(BorderStyle::DOTTED),
            ftd::interpreter::FTD_BORDER_STYLE_DASHED => Ok(BorderStyle::DASHED),
            ftd::interpreter::FTD_BORDER_STYLE_SOLID => Ok(BorderStyle::SOLID),
            ftd::interpreter::FTD_BORDER_STYLE_GROOVE => Ok(BorderStyle::GROOVE),
            ftd::interpreter::FTD_BORDER_STYLE_RIDGE => Ok(BorderStyle::RIDGE),
            ftd::interpreter::FTD_BORDER_STYLE_OUTSET => Ok(BorderStyle::OUTSET),
            ftd::interpreter::FTD_BORDER_STYLE_INSET => Ok(BorderStyle::INSET),
            ftd::interpreter::FTD_BORDER_STYLE_DOUBLE => Ok(BorderStyle::DOUBLE),
            t => ftd::executor::utils::parse_error(
                format!("Unknown variant `{}` for or-type `ftd.border-style`", t),
                doc.name,
                line_number,
            ),
        }
    }

    pub(crate) fn optional_border_style(
        properties: &[ftd::interpreter::Property],
        arguments: &[ftd::interpreter::Argument],
        doc: &ftd::executor::TDoc,
        line_number: usize,
        key: &str,
        inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
        component_name: &str,
    ) -> ftd::executor::Result<ftd::executor::Value<Option<BorderStyle>>> {
        let or_type_value = ftd::executor::value::optional_or_type(
            key,
            component_name,
            properties,
            arguments,
            doc,
            line_number,
            ftd::interpreter::FTD_BORDER_STYLE,
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

#[derive(serde::Deserialize, Debug, PartialEq, Clone, serde::Serialize)]
pub enum ImageFit {
    NONE,
    FILL,
    COVER,
    CONTAIN,
    SCALEDOWN,
}

impl ImageFit {
    fn from_optional_values(
        or_type_value: Option<(String, fastn_type::PropertyValue)>,
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Option<Self>> {
        if let Some(value) = or_type_value {
            return Ok(Some(ImageFit::from_values(value, doc, line_number)?));
        }
        Ok(None)
    }

    fn from_values(
        or_type_value: (String, fastn_type::PropertyValue),
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Self> {
        match or_type_value.0.as_str() {
            ftd::interpreter::FTD_IMAGE_FIT_NONE => Ok(ImageFit::NONE),
            ftd::interpreter::FTD_IMAGE_FIT_COVER => Ok(ImageFit::COVER),
            ftd::interpreter::FTD_IMAGE_FIT_CONTAIN => Ok(ImageFit::CONTAIN),
            ftd::interpreter::FTD_IMAGE_FIT_FILL => Ok(ImageFit::FILL),
            ftd::interpreter::FTD_IMAGE_FIT_SCALE_DOWN => Ok(ImageFit::SCALEDOWN),
            t => ftd::executor::utils::parse_error(
                format!("Unknown variant `{}` for or-type `ftd.image-fit`", t),
                doc.name,
                line_number,
            ),
        }
    }

    pub(crate) fn optional_image_fit(
        properties: &[ftd::interpreter::Property],
        arguments: &[ftd::interpreter::Argument],
        doc: &ftd::executor::TDoc,
        line_number: usize,
        key: &str,
        inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
        component_name: &str,
    ) -> ftd::executor::Result<ftd::executor::Value<Option<ImageFit>>> {
        let or_type_value = ftd::executor::value::optional_or_type(
            key,
            component_name,
            properties,
            arguments,
            doc,
            line_number,
            ftd::interpreter::FTD_IMAGE_FIT,
            inherited_variables,
        )?;

        Ok(ftd::executor::Value::new(
            ImageFit::from_optional_values(or_type_value.value, doc, line_number)?,
            or_type_value.line_number,
            or_type_value.properties,
        ))
    }

    pub fn to_css_string(&self) -> String {
        match self {
            ImageFit::NONE => "none".to_string(),
            ImageFit::COVER => "cover".to_string(),
            ImageFit::CONTAIN => "contain".to_string(),
            ImageFit::FILL => "fill".to_string(),
            ImageFit::SCALEDOWN => "scale-down".to_string(),
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
        or_type_value: Option<(String, fastn_type::PropertyValue)>,
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
        or_type_value: (String, fastn_type::PropertyValue),
        doc: &ftd::executor::TDoc,
        line_number: usize,
    ) -> ftd::executor::Result<Self> {
        match or_type_value.0.as_str() {
            ftd::interpreter::FTD_LOADING_LAZY => Ok(Loading::Lazy),
            ftd::interpreter::FTD_LOADING_EAGER => Ok(Loading::Eager),
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
        properties: &[ftd::interpreter::Property],
        arguments: &[ftd::interpreter::Argument],
        doc: &ftd::executor::TDoc,
        line_number: usize,
        key: &str,
        inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
        component_name: &str,
    ) -> ftd::executor::Result<ftd::executor::Value<Loading>> {
        let or_type_value = ftd::executor::value::optional_or_type(
            key,
            component_name,
            properties,
            arguments,
            doc,
            line_number,
            ftd::interpreter::FTD_LOADING,
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
