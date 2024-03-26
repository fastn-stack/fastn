#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct VariableKind {
    pub modifier: Option<VariableModifier>,
    pub kind: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum VariableModifier {
    List,
    Optional,
    Constant,
}

pub const OPTIONAL: &str = "optional";
pub const LIST: &str = "list";
pub const CONSTANT: &str = "constant";

impl VariableModifier {
    pub(crate) fn is_optional_from_expr(expr: &str) -> bool {
        expr.eq(OPTIONAL)
    }

    pub(crate) fn is_list_from_expr(expr: &str) -> bool {
        expr.eq(LIST)
    }

    pub(crate) fn is_constant_from_expr(expr: &str) -> bool {
        expr.eq(CONSTANT)
    }

    fn is_list(&self) -> bool {
        matches!(self, VariableModifier::List)
    }

    fn is_optional(&self) -> bool {
        matches!(self, VariableModifier::Optional)
    }

    pub(crate) fn get_modifier(expr: &str) -> Option<VariableModifier> {
        let expr = expr.split_whitespace().collect::<Vec<&str>>();
        if expr.len() >= 2 {
            if VariableModifier::is_optional_from_expr(expr.get(0).unwrap()) {
                return Some(VariableModifier::Optional);
            } else if VariableModifier::is_list_from_expr(expr.last().unwrap()) {
                return Some(VariableModifier::List);
            } else if VariableModifier::is_constant_from_expr(expr.get(0).unwrap()) {
                return Some(VariableModifier::Constant);
            }
        }
        None
    }
}

impl VariableKind {
    fn new(kind: &str, modifier: Option<VariableModifier>) -> VariableKind {
        VariableKind {
            modifier,
            kind: kind.to_string(),
        }
    }

    pub(crate) fn get_kind(
        kind: &str,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::ast::Result<VariableKind> {
        let expr = kind.split_whitespace().collect::<Vec<&str>>();
        if expr.len() > 5 || expr.is_empty() {
            return ftd::ast::parse_error(
                format!("Invalid variable kind, found: `{}`", kind),
                doc_id,
                line_number,
            );
        }

        let modifier = VariableModifier::get_modifier(kind);
        let kind = match modifier {
            Some(VariableModifier::Optional) if expr.len() >= 2 => expr[1..].join(" "),
            Some(VariableModifier::List) if expr.len() >= 2 => expr[..expr.len() - 1].join(" "),
            Some(VariableModifier::Constant) if expr.len() >= 2 => expr[1..].join(" "),
            None => expr.join(" "),
            _ => {
                return ftd::ast::parse_error(
                    format!("Invalid variable kind, found: `{}`", kind),
                    doc_id,
                    line_number,
                )
            }
        };

        Ok(VariableKind::new(kind.as_str(), modifier))
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum VariableValue {
    Optional {
        value: Box<Option<VariableValue>>,
        line_number: usize,
        condition: Option<ftd::ast::Condition>,
    },
    Constant {
        value: String,
        line_number: usize,
        source: ValueSource,
        condition: Option<ftd::ast::Condition>,
    },
    List {
        value: Vec<VariableKeyValue>,
        line_number: usize,
        condition: Option<ftd::ast::Condition>,
    },
    Record {
        name: String,
        caption: Box<Option<VariableValue>>,
        headers: HeaderValues,
        body: Option<BodyValue>,
        values: Vec<VariableKeyValue>,
        line_number: usize,
        condition: Option<ftd::ast::Condition>,
    },
    #[serde(rename = "string-value")]
    String {
        value: String,
        #[serde(rename = "line-number")]
        line_number: usize,
        source: ValueSource,
        condition: Option<ftd::ast::Condition>,
    },
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct VariableKeyValue {
    pub key: String,
    pub value: VariableValue,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum ValueSource {
    Caption,
    Body,
    #[serde(rename = "header")]
    Header {
        name: String,
        mutable: bool,
    },
    Default,
}

impl ValueSource {
    pub(crate) fn to_property_source(&self) -> ftd::ast::PropertySource {
        match self {
            ftd::ast::ValueSource::Caption => ftd::ast::PropertySource::Caption,
            ftd::ast::ValueSource::Body => ftd::ast::PropertySource::Body,
            ftd::ast::ValueSource::Header { name, mutable } => ftd::ast::PropertySource::Header {
                name: name.to_owned(),
                mutable: mutable.to_owned(),
            },
            ftd::ast::ValueSource::Default => ftd::ast::PropertySource::Caption,
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct BodyValue {
    pub value: String,
    #[serde(rename = "line-number")]
    pub line_number: usize,
}

impl BodyValue {
    fn new(value: &str, line_number: usize) -> BodyValue {
        BodyValue {
            value: value.to_string(),
            line_number,
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct HeaderValues(pub Vec<HeaderValue>);

impl HeaderValues {
    pub fn new(headers: Vec<HeaderValue>) -> HeaderValues {
        HeaderValues(headers)
    }

    pub fn get_by_key(&self, key: &str) -> Vec<&HeaderValue> {
        use itertools::Itertools;

        self.0
            .iter()
            .filter(|v| v.key.eq(key) || v.key.starts_with(format!("{}.", key).as_str()))
            .collect_vec()
    }

    pub fn optional_header_by_name(
        &self,
        name: &str,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::ast::Result<Option<&HeaderValue>> {
        let values = self
            .get_by_key(name)
            .into_iter()
            .filter(|v| v.key.eq(name))
            .collect::<Vec<_>>();
        if values.len() > 1 {
            ftd::ast::parse_error(
                format!("Multiple header found `{}`", name),
                doc_id,
                line_number,
            )
        } else if let Some(value) = values.first() {
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    pub fn get_by_key_optional(
        &self,
        key: &str,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::ast::Result<Option<&HeaderValue>> {
        let values = self.get_by_key(key);
        if values.len() > 1 {
            ftd::ast::parse_error(
                format!("Multiple header found `{}`", key),
                doc_id,
                line_number,
            )
        } else {
            Ok(values.first().copied())
        }
    }

    pub fn get_optional_string_by_key(
        &self,
        key: &str,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::ast::Result<Option<String>> {
        if let Some(header) = self.get_by_key_optional(key, doc_id, line_number)? {
            if header.value.is_null() {
                Ok(None)
            } else {
                Ok(Some(header.value.string(doc_id)?))
            }
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct HeaderValue {
    pub key: String,
    pub mutable: bool,
    pub value: VariableValue,
    #[serde(rename = "line-number")]
    pub line_number: usize,
    pub kind: Option<String>,
    pub condition: Option<String>,
}

impl HeaderValue {
    fn new(
        key: &str,
        mutable: bool,
        value: VariableValue,
        line_number: usize,
        kind: Option<String>,
        condition: Option<String>,
    ) -> HeaderValue {
        HeaderValue {
            key: key.to_string(),
            mutable,
            value,
            line_number,
            kind,
            condition,
        }
    }
}

impl VariableValue {
    pub fn inner(&self) -> Option<VariableValue> {
        match self {
            VariableValue::Optional { value, .. } => value.as_ref().as_ref().map(|v| v.to_owned()),
            t => Some(t.to_owned()),
        }
    }

    pub(crate) fn condition(&self) -> &Option<ftd::ast::Condition> {
        match self {
            ftd::ast::VariableValue::Record { condition, .. }
            | ftd::ast::VariableValue::Optional { condition, .. }
            | ftd::ast::VariableValue::Constant { condition, .. }
            | ftd::ast::VariableValue::List { condition, .. }
            | ftd::ast::VariableValue::String { condition, .. } => condition,
        }
    }

    pub(crate) fn condition_expression(&self) -> Option<String> {
        self.condition()
            .as_ref()
            .map(|condition| condition.expression.clone())
    }

    pub(crate) fn set_condition(self, condition: Option<ftd::ast::Condition>) -> Self {
        let mut variable_value = self;
        let mut_condition = match &mut variable_value {
            ftd::ast::VariableValue::Record { condition, .. }
            | ftd::ast::VariableValue::Optional { condition, .. }
            | ftd::ast::VariableValue::Constant { condition, .. }
            | ftd::ast::VariableValue::List { condition, .. }
            | ftd::ast::VariableValue::String { condition, .. } => condition,
        };
        *mut_condition = condition;
        variable_value
    }

    pub fn record_name(&self) -> Option<String> {
        let mut name = None;
        let inner_value = self.inner();
        if let Some(ftd::ast::VariableValue::Record {
            name: record_name, ..
        }) = inner_value.as_ref()
        {
            name = Some(record_name.to_owned());
        }
        name
    }

    pub fn string(&self, doc_id: &str) -> ftd::ast::Result<String> {
        match self {
            VariableValue::String { value, .. } => Ok(value.to_string()),
            VariableValue::Constant { value, .. } => Ok(value.to_string()),
            t => ftd::ast::parse_error(
                format!("Expect Variable value string, found: `{:?}`", t),
                doc_id,
                t.line_number(),
            ),
        }
    }

    pub fn is_shorthand_list(&self) -> bool {
        match self {
            VariableValue::String { value, .. } => {
                if (value.starts_with('[') && value.ends_with(']')) || value.contains(',') {
                    return true;
                }
                false
            }
            _ => false,
        }
    }

    pub fn from_string_bracket_list(
        value: &str,
        kind_name: String,
        source: ftd::ast::ValueSource,
        line_number: usize,
        condition: Option<ftd::ast::Condition>,
    ) -> ftd::ast::VariableValue {
        use itertools::Itertools;

        // Bracket list from string
        let bracket_removed_value = value
            .trim_start_matches('[')
            .trim_end_matches(']')
            .to_string();
        let raw_values = bracket_removed_value
            .split(',')
            .filter(|v| !v.is_empty())
            .map(|v| v.trim())
            .collect_vec();
        VariableValue::List {
            value: raw_values
                .iter()
                .map(|v| VariableKeyValue {
                    key: kind_name.clone(),
                    value: VariableValue::from_value(
                        &Some(v.to_string()),
                        source.clone(),
                        line_number,
                    ),
                })
                .collect_vec(),
            line_number,
            condition,
        }
    }

    pub fn caption(&self) -> Option<String> {
        match self {
            VariableValue::String { value, .. } => Some(value.to_string()),
            VariableValue::Record { caption: value, .. }
            | VariableValue::Optional { value, .. } => {
                value.as_ref().as_ref().and_then(|val| val.caption())
            }
            _ => None,
        }
    }

    pub fn line_number(&self) -> usize {
        match self {
            VariableValue::Optional { line_number, .. }
            | VariableValue::Constant { line_number, .. }
            | VariableValue::List { line_number, .. }
            | VariableValue::Record { line_number, .. }
            | VariableValue::String { line_number, .. } => *line_number,
        }
    }

    pub fn set_line_number(&mut self, new_line_number: usize) {
        match self {
            VariableValue::Optional { line_number, .. }
            | VariableValue::Constant { line_number, .. }
            | VariableValue::List { line_number, .. }
            | VariableValue::Record { line_number, .. }
            | VariableValue::String { line_number, .. } => *line_number = new_line_number,
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self, VariableValue::Optional { value, .. } if value.is_none())
    }

    pub(crate) fn is_list(&self) -> bool {
        matches!(self, VariableValue::List { .. })
    }

    pub(crate) fn into_list(
        self,
        doc_name: &str,
        kind: &ftd::interpreter::Kind,
    ) -> ftd::ast::Result<Vec<(String, VariableValue)>> {
        use itertools::Itertools;

        match self {
            VariableValue::String {
                value,
                line_number,
                source,
                condition,
            } => {
                // Bracket list from string
                let bracket_list = VariableValue::from_string_bracket_list(
                    &value,
                    kind.get_name(),
                    source,
                    line_number,
                    condition,
                );
                match bracket_list {
                    VariableValue::List { value, .. } => {
                        Ok(value.into_iter().map(|v| (v.key, v.value)).collect_vec())
                    }
                    t => ftd::ast::parse_error(
                        format!("Invalid bracket list, found: `{:?}`", t),
                        doc_name,
                        t.line_number(),
                    ),
                }
            }
            VariableValue::List { value, .. } => {
                Ok(value.into_iter().map(|v| (v.key, v.value)).collect_vec())
            }
            t => ftd::ast::parse_error(
                format!("Expected list, found: `{:?}`", t),
                doc_name,
                t.line_number(),
            ),
        }
    }

    pub fn is_record(&self) -> bool {
        matches!(self, VariableValue::Record { .. })
    }

    pub fn is_string(&self) -> bool {
        matches!(self, VariableValue::String { .. })
    }

    #[allow(clippy::type_complexity)]
    pub fn get_record(
        &self,
        doc_id: &str,
    ) -> ftd::ast::Result<(
        &String,
        &Box<Option<VariableValue>>,
        &HeaderValues,
        &Option<BodyValue>,
        &Vec<VariableKeyValue>,
        usize,
    )> {
        match self {
            VariableValue::Record {
                name,
                caption,
                headers,
                body,
                values,
                line_number,
                ..
            } => Ok((name, caption, headers, body, values, *line_number)),
            t => ftd::ast::parse_error(
                format!("Expected Record, found: `{:?}`", t),
                doc_id,
                self.line_number(),
            ),
        }
    }

    pub fn get_processor_body(&self, doc_id: &str) -> ftd::ast::Result<Option<BodyValue>> {
        match self {
            VariableValue::Record { body, .. } => Ok(body.clone()),
            VariableValue::String {
                value, line_number, ..
            } => {
                if value.is_empty() {
                    return Ok(None);
                }
                Ok(Some(BodyValue {
                    value: value.to_string(),
                    line_number: *line_number,
                }))
            }
            VariableValue::List { value, .. } => {
                let value = value
                    .first()
                    .and_then(|v| v.value.get_processor_body(doc_id).ok().flatten());
                Ok(value)
            }
            t => ftd::ast::parse_error(
                format!("Expected Body, found: `{:?}`", t),
                doc_id,
                self.line_number(),
            ),
        }
    }

    fn into_optional(self) -> VariableValue {
        match self {
            t @ VariableValue::Optional { .. } => t,
            t => VariableValue::Optional {
                line_number: t.line_number(),
                condition: t.condition().clone(),
                value: Box::new(Some(t)),
            },
        }
    }

    pub(crate) fn from_p1_with_modifier(
        section: &ftd_p1::Section,
        doc_id: &str,
        kind: &ftd::ast::VariableKind,
    ) -> ftd::ast::Result<VariableValue> {
        let value = VariableValue::from_p1(section, doc_id)?;
        value.into_modifier(doc_id, section.line_number, kind)
    }

    pub(crate) fn from_header_with_modifier(
        header: &ftd_p1::Header,
        doc_id: &str,
        kind: &ftd::ast::VariableKind,
    ) -> ftd::ast::Result<VariableValue> {
        let value = VariableValue::from_p1_header(header, doc_id)?;
        value.into_modifier(doc_id, header.get_line_number(), kind)
    }

    pub(crate) fn into_modifier(
        self,
        doc_id: &str,
        line_number: usize,
        kind: &ftd::ast::VariableKind,
    ) -> ftd::ast::Result<VariableValue> {
        match &kind.modifier {
            Some(modifier) if modifier.is_list() => {
                if self.is_null() {
                    Ok(VariableValue::List {
                        value: vec![],
                        line_number: self.line_number(),
                        condition: self.condition().clone(),
                    })
                } else if self.is_list() || self.is_record() {
                    // todo: check if `end` exists
                    Ok(self)
                } else if let VariableValue::String {
                    ref value,
                    ref condition,
                    ..
                } = self
                {
                    if value.starts_with('$') && !value.contains(',') {
                        Ok(self)
                    } else {
                        Ok(VariableValue::from_string_bracket_list(
                            value,
                            kind.kind.clone(),
                            ftd::ast::ValueSource::Default,
                            line_number,
                            condition.clone(),
                        ))
                    }
                } else {
                    ftd::ast::parse_error(
                        format!("Expected List found: `{:?}`", self),
                        doc_id,
                        line_number,
                    )
                }
            }
            Some(modifier) if modifier.is_optional() => Ok(self.into_optional()),
            _ => Ok(self),
        }
    }

    pub(crate) fn from_p1(
        section: &ftd_p1::Section,
        doc_id: &str,
    ) -> ftd::ast::Result<VariableValue> {
        let values = section
            .sub_sections
            .iter()
            .map(|v| {
                Ok(VariableKeyValue {
                    key: v.name.to_string(),
                    value: VariableValue::from_p1(v, doc_id)?,
                })
            })
            .collect::<ftd::ast::Result<Vec<VariableKeyValue>>>()?;

        let caption = match section.caption.as_ref() {
            Some(header) => VariableValue::from_p1_header(header, doc_id)?.inner(),
            None => None,
        };

        let headers = section
            .headers
            .0
            .iter()
            .filter(|v| {
                (!ftd::ast::utils::is_condition(v.get_key().as_str(), &v.get_kind())
                    && v.get_key().ne(ftd::ast::utils::PROCESSOR))
                    && ftd::ast::VariableFlags::from_header(v, doc_id).is_err()
            })
            .map(|header| {
                let key = header.get_key();
                let header_key = if ftd::ast::utils::is_variable_mutable(key.as_str())
                    && !ftd::ast::utils::is_header_key(key.as_str())
                {
                    key.trim_start_matches(ftd::ast::utils::REFERENCE)
                } else {
                    key.as_str()
                };

                Ok(HeaderValue::new(
                    header_key,
                    ftd::ast::utils::is_variable_mutable(key.as_str()),
                    VariableValue::from_p1_header(header, doc_id)?,
                    header.get_line_number(),
                    header.get_kind(),
                    header.get_condition(),
                ))
            })
            .collect::<ftd::ast::Result<Vec<HeaderValue>>>()?;

        let condition = ftd::ast::Condition::from_headers(&section.headers, doc_id)?;
        let body = section
            .body
            .as_ref()
            .map(|v| BodyValue::new(v.get_value().as_str(), v.line_number));

        if values.is_empty() && headers.is_empty() && !(caption.is_some() && body.is_some()) {
            return Ok(if let Some(caption) = caption {
                caption.set_condition(condition)
            } else if let Some(body) = body {
                VariableValue::String {
                    value: body.value,
                    line_number: body.line_number,
                    source: ftd::ast::ValueSource::Body,
                    condition,
                }
            } else {
                VariableValue::Optional {
                    value: Box::new(None),
                    line_number: section.line_number,
                    condition,
                }
            });
        }

        if !values.is_empty() && caption.is_none() && body.is_none() && headers.is_empty() {
            return Ok(VariableValue::List {
                value: values,
                line_number: section.line_number,
                condition,
            });
        }

        Ok(VariableValue::Record {
            name: section.name.to_string(),
            caption: Box::new(caption),
            headers: HeaderValues::new(headers),
            body,
            values,
            line_number: section.line_number,
            condition,
        })
    }

    pub(crate) fn from_p1_header(
        header: &ftd_p1::Header,
        doc_id: &str,
    ) -> ftd::ast::Result<VariableValue> {
        Ok(match header {
            ftd_p1::Header::KV(ftd_p1::KV {
                value, line_number, ..
            }) => VariableValue::from_value(value, ftd::ast::ValueSource::Default, *line_number),
            ftd_p1::Header::Section(ftd_p1::SectionHeader {
                section,
                line_number,
                condition,
                ..
            }) => VariableValue::List {
                value: section
                    .iter()
                    .map(|v| {
                        Ok(VariableKeyValue {
                            key: v.name.to_string(),
                            value: VariableValue::from_p1(v, doc_id)?,
                        })
                    })
                    .collect::<ftd::ast::Result<Vec<VariableKeyValue>>>()?,
                line_number: *line_number,
                condition: condition
                    .as_ref()
                    .map(|expr| ftd::ast::Condition::new(expr, *line_number)),
            },
            ftd_p1::Header::BlockRecordHeader(ftd_p1::BlockRecordHeader {
                key,
                caption,
                body,
                fields,
                line_number,
                condition,
                ..
            }) => VariableValue::Record {
                name: key.to_string(),
                caption: Box::new(caption.as_ref().map(|c| VariableValue::String {
                    value: c.to_string(),
                    line_number: *line_number,
                    source: ValueSource::Caption,
                    condition: None,
                })),
                headers: {
                    let mut headers = vec![];
                    for header in fields.iter() {
                        let key = header.get_key();
                        headers.push(HeaderValue::new(
                            key.trim_start_matches(ftd::ast::utils::REFERENCE),
                            ftd::ast::utils::is_variable_mutable(key.as_str()),
                            VariableValue::from_p1_header(header, doc_id)?,
                            header.get_line_number(),
                            header.get_kind(),
                            header.get_condition(),
                        ));
                    }
                    HeaderValues(headers)
                },
                body: body
                    .0
                    .as_ref()
                    .map(|b| BodyValue::new(b.as_str(), body.1.unwrap_or(0))),
                values: vec![],
                line_number: *line_number,
                condition: condition
                    .as_ref()
                    .map(|expr| ftd::ast::Condition::new(expr, *line_number)),
            },
        })
    }

    pub(crate) fn from_value(
        value: &Option<String>,
        source: ftd::ast::ValueSource,
        line_number: usize,
    ) -> VariableValue {
        match value {
            Some(value) if value.ne(NULL) && !value.is_empty() => VariableValue::String {
                value: value.to_string(),
                line_number,
                source,
                condition: None,
            },
            _ => VariableValue::Optional {
                value: Box::new(None),
                line_number,
                condition: None,
            },
        }
    }

    pub fn has_request_data_header(&self) -> bool {
        if let Some(ftd::ast::VariableValue::Record { headers, .. }) = self.inner() {
            for h in headers.0.iter() {
                if h.key.trim_end_matches('$').eq("processor") {
                    if let ftd::ast::VariableValue::String { ref value, .. } = h.value {
                        if value.contains("request-data") {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Condition {
    pub expression: String,
    #[serde(rename = "line-number")]
    pub line_number: usize,
}

impl Condition {
    pub fn new(expression: &str, line_number: usize) -> Condition {
        Condition {
            expression: expression.to_string(),
            line_number,
        }
    }

    pub(crate) fn from_headers(
        headers: &ftd_p1::Headers,
        doc_id: &str,
    ) -> ftd::ast::Result<Option<Condition>> {
        let condition = headers
            .0
            .iter()
            .find(|v| ftd::ast::utils::is_condition(v.get_key().as_str(), &v.get_kind()));
        let condition = if let Some(condition) = condition {
            condition
        } else {
            return Ok(None);
        };

        let expression = condition.get_value(doc_id)?.ok_or(ftd::ast::Error::Parse {
            message: "`if` condition must contain expression".to_string(),
            doc_id: doc_id.to_string(),
            line_number: condition.get_line_number(),
        })?;

        Ok(Some(Condition::new(
            expression.as_str(),
            condition.get_line_number(),
        )))
    }
}

pub const NULL: &str = "NULL";
