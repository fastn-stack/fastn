#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct VariableDefinition {
    pub kind: VariableKind,
    pub name: String,
    pub value: VariableValue,
}

impl VariableDefinition {
    fn new(name: &str, kind: VariableKind, value: VariableValue) -> VariableDefinition {
        VariableDefinition {
            kind,
            name: name.to_string(),
            value,
        }
    }

    pub fn is_variable_definition(section: &ftd::p11::Section) -> bool {
        !(ftd::ast::Import::is_import(section)
            || ftd::ast::Record::is_record(section)
            || ftd::ast::Component::is_component(section)
            || section.kind.is_none())
    }

    pub(crate) fn from_p1(
        section: &ftd::p11::Section,
        doc_id: &str,
    ) -> ftd::ast::Result<VariableDefinition> {
        if !Self::is_variable_definition(section) {
            return ftd::ast::parse_error(
                format!("Section is not variable section, found `{:?}`", section),
                doc_id,
                section.line_number,
            );
        }

        let kind = VariableKind::get_kind(
            section.kind.as_ref().unwrap().as_str(),
            doc_id,
            section.line_number,
        )?;

        let value = VariableValue::from_p1_with_modifier(section, doc_id, &kind.modifier)?;

        Ok(VariableDefinition::new(section.name.as_str(), kind, value))
    }
}

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
        if expr.len() == 2 {
            if VariableModifier::is_optional_from_expr(expr.get(0).unwrap()) {
                return Some(VariableModifier::Optional);
            } else if VariableModifier::is_list_from_expr(expr.get(1).unwrap()) {
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

    fn get_kind(kind: &str, doc_id: &str, line_number: usize) -> ftd::ast::Result<VariableKind> {
        let expr = kind.split_whitespace().collect::<Vec<&str>>();
        if expr.len() > 2 || expr.is_empty() {
            return ftd::ast::parse_error(
                format!("Invalid variable kind, found: `{}`", kind),
                doc_id,
                line_number,
            );
        }

        let modifier = VariableModifier::get_modifier(kind);
        let kind = match modifier {
            Some(VariableModifier::Optional) if expr.len() == 2 => expr.get(1).unwrap(),
            Some(VariableModifier::List) if expr.len() == 2 => expr.get(0).unwrap(),
            None => expr.get(0).unwrap(),
            _ => {
                return ftd::ast::parse_error(
                    format!("Invalid variable kind, found: `{}`", kind),
                    doc_id,
                    line_number,
                )
            }
        };

        Ok(VariableKind::new(kind, modifier))
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum VariableValue {
    Optional(Box<Option<VariableValue>>),
    List(Vec<VariableValue>),
    Regular {
        caption: Box<Option<VariableValue>>,
        headers: Vec<(String, VariableValue)>,
        body: Option<String>,
    },
    String(String),
}

impl VariableValue {
    fn inner(&self) -> Option<VariableValue> {
        match self {
            VariableValue::Optional(value) => value.as_ref().as_ref().map(|v| v.to_owned()),
            t => Some(t.to_owned()),
        }
    }

    fn is_null(&self) -> bool {
        VariableValue::Optional(Box::new(None)).eq(self)
    }

    fn is_list(&self) -> bool {
        matches!(self, VariableValue::List(_))
    }

    fn into_optional(self) -> VariableValue {
        match self {
            t @ VariableValue::Optional(_) => t,
            t => VariableValue::Optional(Box::new(Some(t))),
        }
    }

    fn from_p1_with_modifier(
        section: &ftd::p11::Section,
        doc_id: &str,
        modifier: &Option<VariableModifier>,
    ) -> ftd::ast::Result<VariableValue> {
        let value = VariableValue::from_p1(section, doc_id);
        match modifier {
            Some(modifier) if modifier.is_list() => {
                if value.is_null() {
                    Ok(VariableValue::List(vec![]))
                } else if value.is_list() {
                    Ok(value)
                } else {
                    ftd::ast::parse_error(
                        format!("Expected List found: `{:?}`", value),
                        doc_id,
                        section.line_number,
                    )
                }
            }
            Some(modifier) if modifier.is_optional() => Ok(value.into_optional()),
            _ => Ok(value),
        }
    }

    fn from_p1(section: &ftd::p11::Section, doc_id: &str) -> VariableValue {
        use itertools::Itertools;

        if !section.sub_sections.is_empty() {
            return VariableValue::List(
                section
                    .sub_sections
                    .iter()
                    .map(|v| VariableValue::from_p1(v, doc_id))
                    .collect_vec(),
            );
        }

        let caption = section
            .caption
            .as_ref()
            .and_then(|v| VariableValue::from_p1_header(v, doc_id).inner());

        let headers = section
            .headers
            .0
            .iter()
            .map(|header| {
                (
                    header.get_key(),
                    VariableValue::from_p1_header(&header, doc_id),
                )
            })
            .collect_vec();

        let body = section.body.as_ref().map(|v| v.get_value());

        if headers.is_empty() && !(caption.is_some() && body.is_some()) {
            return if let Some(caption) = caption {
                caption
            } else if let Some(body) = body {
                VariableValue::String(body)
            } else {
                VariableValue::Optional(Box::new(None))
            };
        }

        VariableValue::Regular {
            caption: Box::new(caption),
            headers,
            body,
        }
    }

    fn from_p1_header(header: &ftd::p11::Header, doc_id: &str) -> VariableValue {
        use itertools::Itertools;

        match header {
            ftd::p11::Header::KV(ftd::p11::header::KV { value, .. }) => match value {
                Some(value) if value.ne(NULL) => VariableValue::String(value.to_string()),
                _ => VariableValue::Optional(Box::new(None)),
            },
            ftd::p11::Header::Section(ftd::p11::header::Section { section, .. }) => {
                VariableValue::List(
                    section
                        .iter()
                        .map(|v| VariableValue::from_p1(v, doc_id))
                        .collect_vec(),
                )
            }
        }
    }
}

pub const NULL: &str = "NULL";
