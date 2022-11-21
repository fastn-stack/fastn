#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct VariableDefinition {
    pub name: String,
    pub kind: ftd::ast::VariableKind,
    pub mutable: bool,
    pub value: ftd::ast::VariableValue,
    pub line_number: usize,
    pub flags: VariableFlags,
}

impl VariableDefinition {
    fn new(
        name: &str,
        kind: ftd::ast::VariableKind,
        mutable: bool,
        value: ftd::ast::VariableValue,
        line_number: usize,
        flags: VariableFlags,
    ) -> VariableDefinition {
        VariableDefinition {
            kind,
            name: name.to_string(),
            mutable,
            value,
            line_number,
            flags,
        }
    }

    pub fn is_variable_definition(section: &ftd::p11::Section) -> bool {
        !(ftd::ast::Import::is_import(section)
            || ftd::ast::Record::is_record(section)
            || ftd::ast::ComponentDefinition::is_component_definition(section)
            || section.kind.is_none()
            || ftd::ast::Function::is_function(section))
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

        let flags = ftd::ast::VariableFlags::from_headers(&section.headers, doc_id);

        Ok(VariableDefinition::new(
            section.name.trim_start_matches(ftd::ast::utils::REFERENCE),
            kind,
            ftd::ast::utils::is_variable_mutable(section.name.as_str()),
            value,
            section.line_number,
            flags,
        ))
    }

    pub fn line_number(&self) -> usize {
        self.line_number
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct VariableInvocation {
    pub name: String,
    pub value: ftd::ast::VariableValue,
    pub condition: Option<ftd::ast::Condition>,
    pub line_number: usize,
}

impl VariableInvocation {
    fn new(
        name: &str,
        value: ftd::ast::VariableValue,
        condition: Option<ftd::ast::Condition>,
        line_number: usize,
    ) -> VariableInvocation {
        VariableInvocation {
            name: name.to_string(),
            value,
            condition,
            line_number,
        }
    }

    pub fn line_number(&self) -> usize {
        self.line_number
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
        let condition = ftd::ast::Condition::from_headers(&section.headers, doc_id)?;

        Ok(VariableInvocation::new(
            section.name.trim_start_matches(ftd::ast::utils::REFERENCE),
            value,
            condition,
            section.line_number,
        ))
    }
}

#[derive(Debug, PartialEq, Clone, serde::Serialize, Default, serde::Deserialize)]
pub struct VariableFlags {
    pub always_include: Option<bool>,
}

impl VariableFlags {
    pub fn new() -> VariableFlags {
        VariableFlags {
            always_include: None,
        }
    }

    pub fn set_always_include(self) -> VariableFlags {
        let mut variable_flag = self;
        variable_flag.always_include = Some(true);
        variable_flag
    }

    pub fn from_headers(headers: &ftd::p11::Headers, doc_id: &str) -> VariableFlags {
        for header in headers.0.iter() {
            if let Ok(flag) = ftd::ast::VariableFlags::from_header(header, doc_id) {
                return flag;
            }
        }

        ftd::ast::VariableFlags::new()
    }

    pub fn from_header(header: &ftd::p11::Header, doc_id: &str) -> ftd::ast::Result<VariableFlags> {
        let kv = match header {
            ftd::p11::Header::KV(kv) => kv,
            ftd::p11::Header::Section(s) => {
                return ftd::ast::parse_error(
                    format!("Expected the boolean value for flag, found: `{:?}`", s),
                    doc_id,
                    header.get_line_number(),
                )
            }
        };

        match kv.key.as_str() {
            "$always-include$" => {
                let value = kv
                    .value
                    .as_ref()
                    .ok_or(ftd::ast::Error::ParseError {
                        message: "Value expected for `$always-include$` flag found `null`"
                            .to_string(),
                        doc_id: doc_id.to_string(),
                        line_number: kv.line_number,
                    })?
                    .parse::<bool>()?;
                if value {
                    Ok(VariableFlags::new().set_always_include())
                } else {
                    Ok(VariableFlags::new())
                }
            }
            t => {
                ftd::ast::parse_error(format!("Unknown flag found`{}`", t), doc_id, kv.line_number)
            }
        }
    }
}
