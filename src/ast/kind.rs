#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct VariableKind {
    pub modifier: Option<VariableModifier>,
    pub kind: String,
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum VariableModifier {
    List,
    Optional,
}

pub const OPTIONAL: &str = "optional";
pub const LIST: &str = "list";

impl VariableModifier {
    pub(crate) fn is_optional_from_expr(expr: &str) -> bool {
        expr.eq(OPTIONAL)
    }

    pub(crate) fn is_list_from_expr(expr: &str) -> bool {
        expr.eq(LIST)
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
    },
    List {
        value: Vec<(String, VariableValue)>,
        line_number: usize,
    },
    Record {
        name: String,
        caption: Box<Option<VariableValue>>,
        headers: HeaderValues,
        body: Option<BodyValue>,
        line_number: usize,
    },
    String {
        value: String,
        line_number: usize,
    },
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct BodyValue {
    pub value: String,
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
pub struct HeaderValues(Vec<HeaderValue>);

impl HeaderValues {
    fn new(headers: Vec<HeaderValue>) -> HeaderValues {
        HeaderValues(headers)
    }

    pub fn get_by_key(&self, key: &str) -> Vec<&HeaderValue> {
        use itertools::Itertools;

        self.0.iter().filter(|v| v.key.eq(key)).collect_vec()
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct HeaderValue {
    pub key: String,
    pub value: VariableValue,
    pub line_number: usize,
}

impl HeaderValue {
    fn new(key: &str, value: VariableValue, line_number: usize) -> HeaderValue {
        HeaderValue {
            key: key.to_string(),
            value,
            line_number,
        }
    }
}

impl VariableValue {
    pub(crate) fn inner(&self) -> Option<VariableValue> {
        match self {
            VariableValue::Optional { value, .. } => value.as_ref().as_ref().map(|v| v.to_owned()),
            t => Some(t.to_owned()),
        }
    }

    pub(crate) fn string(&self, doc_id: &str) -> ftd::ast::Result<String> {
        match self {
            VariableValue::String { value, .. } => Ok(value.to_string()),
            t => ftd::ast::parse_error(
                format!("Expect Variable value string, found: `{:?}`", t),
                doc_id,
                t.line_number(),
            ),
        }
    }

    pub(crate) fn line_number(&self) -> usize {
        match self {
            VariableValue::Optional { line_number, .. }
            | VariableValue::List { line_number, .. }
            | VariableValue::Record { line_number, .. }
            | VariableValue::String { line_number, .. } => *line_number,
        }
    }

    fn is_null(&self) -> bool {
        matches!(self, VariableValue::Optional { value, .. } if value.is_none())
    }

    pub(crate) fn is_list(&self) -> bool {
        matches!(self, VariableValue::List { .. })
    }

    pub(crate) fn into_list(
        self,
        doc_name: &str,
    ) -> ftd::ast::Result<Vec<(String, VariableValue)>> {
        match self {
            VariableValue::List { value, .. } => Ok(value),
            t => ftd::ast::parse_error(
                format!("Expected list, found: `{:?}`", t),
                doc_name,
                t.line_number(),
            ),
        }
    }

    pub(crate) fn is_record(&self) -> bool {
        matches!(self, VariableValue::Record { .. })
    }

    #[allow(clippy::type_complexity)]
    pub(crate) fn get_record(
        &self,
        doc_id: &str,
    ) -> ftd::ast::Result<(
        &String,
        &Box<Option<VariableValue>>,
        &HeaderValues,
        &Option<BodyValue>,
        usize,
    )> {
        match self {
            VariableValue::Record {
                name,
                caption,
                headers,
                body,
                line_number,
            } => Ok((name, caption, headers, body, *line_number)),
            t => ftd::ast::parse_error(
                format!("Expected Record, found: `{:?}`", t),
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
                value: Box::new(Some(t)),
            },
        }
    }

    pub(crate) fn from_p1_with_modifier(
        section: &ftd::p11::Section,
        doc_id: &str,
        modifier: &Option<VariableModifier>,
    ) -> ftd::ast::Result<VariableValue> {
        let value = VariableValue::from_p1(section);
        value.into_modifier(doc_id, section.line_number, modifier)
    }

    pub(crate) fn from_header_with_modifier(
        header: &ftd::p11::Header,
        doc_id: &str,
        modifier: &Option<VariableModifier>,
    ) -> ftd::ast::Result<VariableValue> {
        let value = VariableValue::from_p1_header(header);
        value.into_modifier(doc_id, header.get_line_number(), modifier)
    }

    pub(crate) fn into_modifier(
        self,
        doc_id: &str,
        line_number: usize,
        modifier: &Option<VariableModifier>,
    ) -> ftd::ast::Result<VariableValue> {
        match modifier {
            Some(modifier) if modifier.is_list() => {
                if self.is_null() {
                    Ok(VariableValue::List {
                        value: vec![],
                        line_number: self.line_number(),
                    })
                } else if self.is_list() {
                    Ok(self)
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

    pub(crate) fn from_p1(section: &ftd::p11::Section) -> VariableValue {
        use itertools::Itertools;

        if !section.sub_sections.is_empty() {
            return VariableValue::List {
                value: section
                    .sub_sections
                    .iter()
                    .map(|v| (v.name.to_string(), VariableValue::from_p1(v)))
                    .collect_vec(),
                line_number: section.line_number,
            };
        }

        let caption = section
            .caption
            .as_ref()
            .and_then(|v| VariableValue::from_p1_header(v).inner());

        let headers = section
            .headers
            .0
            .iter()
            .filter(|v| !ftd::ast::utils::is_condition(v.get_key().as_str(), &v.get_kind()))
            .map(|header| {
                HeaderValue::new(
                    header.get_key().as_str(),
                    VariableValue::from_p1_header(header),
                    header.get_line_number(),
                )
            })
            .collect_vec();

        let body = section
            .body
            .as_ref()
            .map(|v| BodyValue::new(v.get_value().as_str(), v.line_number));

        if headers.is_empty() && !(caption.is_some() && body.is_some()) {
            return if let Some(caption) = caption {
                caption
            } else if let Some(body) = body {
                VariableValue::String {
                    value: body.value,
                    line_number: body.line_number,
                }
            } else {
                VariableValue::Optional {
                    value: Box::new(None),
                    line_number: section.line_number,
                }
            };
        }

        VariableValue::Record {
            name: section.name.to_string(),
            caption: Box::new(caption),
            headers: HeaderValues::new(headers),
            body,
            line_number: section.line_number,
        }
    }

    pub(crate) fn from_p1_header(header: &ftd::p11::Header) -> VariableValue {
        use itertools::Itertools;

        match header {
            ftd::p11::Header::KV(ftd::p11::header::KV {
                value, line_number, ..
            }) => VariableValue::from_value(value, *line_number),
            ftd::p11::Header::Section(ftd::p11::header::Section {
                section,
                line_number,
                ..
            }) => VariableValue::List {
                value: section
                    .iter()
                    .map(|v| (v.name.to_string(), VariableValue::from_p1(v)))
                    .collect_vec(),
                line_number: *line_number,
            },
        }
    }

    pub(crate) fn from_value(value: &Option<String>, line_number: usize) -> VariableValue {
        match value {
            Some(value) if value.ne(NULL) => VariableValue::String {
                value: value.to_string(),
                line_number,
            },
            _ => VariableValue::Optional {
                value: Box::new(None),
                line_number,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Condition {
    pub expression: String,
    pub line_number: usize,
}

impl Condition {
    fn new(expression: &str, line_number: usize) -> Condition {
        Condition {
            expression: expression.to_string(),
            line_number,
        }
    }

    pub(crate) fn from_headers(
        headers: &ftd::p11::Headers,
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

        let expression = condition
            .get_value(doc_id)?
            .ok_or(ftd::ast::Error::ParseError {
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
