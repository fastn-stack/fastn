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

    pub fn pattern_from_variant_str(
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
}

#[derive(serde::Deserialize, Debug, PartialEq, Clone, serde::Serialize)]
pub enum Alignment {
    TopLeft,
    TopCenter,
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
            ftd::interpreter2::FTD_ALIGNMENT_TOP_LEFT => Ok(Alignment::TopLeft),
            ftd::interpreter2::FTD_ALIGNMENT_TOP_CENTER => Ok(Alignment::TopCenter),
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
            ftd::interpreter2::FTD_ALIGNMENT,
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
            ftd::interpreter2::FTD_ALIGNMENT,
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
                Alignment::TopLeft => "start".to_string(),
                Alignment::TopCenter => "center".to_string(),
            }
        } else {
            match self {
                Alignment::TopLeft => "start".to_string(),
                Alignment::TopCenter => "start".to_string(),
            }
        }
    }

    pub fn to_css_align_items(&self, is_horizontal_direction: bool) -> String {
        if is_horizontal_direction {
            match self {
                Alignment::TopLeft => "start".to_string(),
                Alignment::TopCenter => "start".to_string(),
            }
        } else {
            match self {
                Alignment::TopLeft => "start".to_string(),
                Alignment::TopCenter => "center".to_string(),
            }
        }
    }

    pub fn justify_content_pattern(is_horizontal_direction: bool) -> (String, bool) {
        if is_horizontal_direction {
            (
                format!(
                    indoc::indoc! {"
                if (\"{{0}}\" == \"{top_left}\") {{\"start\"}} else if (\"{{0}}\" == \"{top_center}\") {{\"center\"}} else {{null}}
                "},
                    top_left = ftd::interpreter2::FTD_ALIGNMENT_TOP_LEFT,
                    top_center = ftd::interpreter2::FTD_ALIGNMENT_TOP_CENTER,
                ),
                true,
            )
        } else {
            (
                format!(
                    indoc::indoc! {"
                if (\"{{0}}\" == \"{top_left}\") {{\"start\"}} else if (\"{{0}}\" == \"{top_center}\") {{\"start\"}} else {{null}}
                "},
                    top_left = ftd::interpreter2::FTD_ALIGNMENT_TOP_LEFT,
                    top_center = ftd::interpreter2::FTD_ALIGNMENT_TOP_CENTER,
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
                if (\"{{0}}\" == \"{top_left}\") {{\"start\"}} else if (\"{{0}}\" == \"{top_center}\") {{\"start\"}} else {{null}}
                "},
                    top_left = ftd::interpreter2::FTD_ALIGNMENT_TOP_LEFT,
                    top_center = ftd::interpreter2::FTD_ALIGNMENT_TOP_CENTER,
                ),
                true,
            )
        } else {
            (
                format!(
                    indoc::indoc! {"
                if (\"{{0}}\" == \"{top_left}\") {{\"start\"}} else if (\"{{0}}\" == \"{top_center}\") {{\"center\"}} else {{null}}
                "},
                    top_left = ftd::interpreter2::FTD_ALIGNMENT_TOP_LEFT,
                    top_center = ftd::interpreter2::FTD_ALIGNMENT_TOP_CENTER,
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
                dbg!("from_values", &t, &or_type_value);
                let value = or_type_value.1.clone().resolve(&doc.itdoc(), line_number)?;
                let (_, variant, value) = value.get_or_type(doc.name, line_number)?;
                dbg!(&variant, &value);
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

    pub fn pattern_from_variant_str(
        variant: &str,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::executor::Result<&'static str> {
        match variant {
            ftd::interpreter2::FTD_RESIZING_FIXED => Ok("{0}px"),
            ftd::interpreter2::FTD_LENGTH_PERCENT => Ok("{0}%"),
            t => ftd::executor::utils::parse_error(
                format!("Unknown variant found for ftd.length: `{}`", t),
                doc_id,
                line_number,
            ),
        }
    }
}
