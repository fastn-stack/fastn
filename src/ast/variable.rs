#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct VariableDefinition {
    pub kind: VariableKind,
    pub name: String,
}

impl VariableDefinition {
    fn new(name: &str, kind: VariableKind) -> VariableDefinition {
        VariableDefinition {
            kind,
            name: name.to_string(),
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

        Ok(VariableDefinition::new(section.name.as_str(), kind))
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
    pub(crate) fn is_optional(expr: &str) -> bool {
        expr.eq(OPTIONAL)
    }

    pub(crate) fn is_list(expr: &str) -> bool {
        expr.eq(LIST)
    }

    pub(crate) fn get_modifier(expr: &str) -> Option<VariableModifier> {
        let expr = expr.split_whitespace().collect::<Vec<&str>>();
        if expr.len() == 2 {
            if VariableModifier::is_optional(expr.get(0).unwrap()) {
                return Some(VariableModifier::Optional);
            } else if VariableModifier::is_list(expr.get(1).unwrap()) {
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
