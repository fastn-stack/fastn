#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct VariableDefinition {
    pub name: String,
    pub kind: ftd::ast::VariableKind,
    pub mutable: bool,
    pub value: ftd::ast::VariableValue,
}

impl VariableDefinition {
    fn new(
        name: &str,
        kind: ftd::ast::VariableKind,
        mutable: bool,
        value: ftd::ast::VariableValue,
    ) -> VariableDefinition {
        VariableDefinition {
            kind,
            name: name.to_string(),
            mutable,
            value,
        }
    }

    pub fn is_variable_definition(section: &ftd::p11::Section) -> bool {
        !(ftd::ast::Import::is_import(section)
            || ftd::ast::Record::is_record(section)
            || ftd::ast::ComponentDefinition::is_component_definition(section)
            || section.kind.is_none())
    }

    pub(crate) fn from_p1(
        section: &ftd::p11::Section,
        doc_id: &str,
    ) -> ftd::ast::Result<VariableDefinition> {
        if !Self::is_variable_definition(section) {
            return ftd::ast::parse_error(
                format!(
                    "Section is not variable definition section, found `{:?}`",
                    section
                ),
                doc_id,
                section.line_number,
            );
        }

        let kind = ftd::ast::VariableKind::get_kind(
            section.kind.as_ref().unwrap().as_str(),
            doc_id,
            section.line_number,
        )?;

        let value =
            ftd::ast::VariableValue::from_p1_with_modifier(section, doc_id, &kind.modifier)?;

        Ok(VariableDefinition::new(
            section.name.trim_start_matches(ftd::ast::utils::REFERENCE),
            kind,
            ftd::ast::utils::is_variable_mutable(section.name.as_str()),
            value,
        ))
    }
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct VariableInvocation {
    pub name: String,
    pub value: ftd::ast::VariableValue,
}

impl VariableInvocation {
    fn new(name: &str, value: ftd::ast::VariableValue) -> VariableInvocation {
        VariableInvocation {
            name: name.to_string(),
            value,
        }
    }

    pub fn is_variable_invocation(section: &ftd::p11::Section) -> bool {
        section.kind.is_none() && section.name.starts_with(ftd::ast::utils::REFERENCE)
    }

    pub(crate) fn from_p1(
        section: &ftd::p11::Section,
        doc_id: &str,
    ) -> ftd::ast::Result<VariableInvocation> {
        if !Self::is_variable_invocation(section) {
            return ftd::ast::parse_error(
                format!(
                    "Section is not variable invocation section, found `{:?}`",
                    section
                ),
                doc_id,
                section.line_number,
            );
        }

        let value = ftd::ast::VariableValue::from_p1(section, doc_id);

        Ok(VariableInvocation::new(
            section.name.trim_start_matches(ftd::ast::utils::REFERENCE),
            value,
        ))
    }
}
