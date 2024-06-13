#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct VariableDefinition {
    pub name: String,
    pub kind: ftd_ast::VariableKind,
    pub mutable: bool,
    pub value: ftd_ast::VariableValue,
    pub processor: Option<String>,
    pub flags: VariableFlags,
    pub line_number: usize,
}

impl VariableDefinition {
    fn new(
        name: &str,
        kind: ftd_ast::VariableKind,
        mutable: bool,
        value: ftd_ast::VariableValue,
        processor: Option<String>,
        flags: VariableFlags,
        line_number: usize,
    ) -> VariableDefinition {
        VariableDefinition {
            kind,
            name: name.to_string(),
            mutable,
            value,
            processor,
            flags,
            line_number,
        }
    }

    pub fn is_variable_definition(section: &ftd_p1::Section) -> bool {
        !(ftd_ast::Import::is_import(section)
            || ftd_ast::Record::is_record(section)
            || ftd_ast::OrType::is_or_type(section)
            || ftd_ast::ComponentDefinition::is_component_definition(section)
            || section.kind.is_none()
            || ftd_ast::Function::is_function(section)
            || ftd_ast::WebComponentDefinition::is_web_component_definition(section))
    }

    pub(crate) fn from_p1(
        section: &ftd_p1::Section,
        doc_id: &str,
    ) -> ftd_ast::Result<VariableDefinition> {
        if !Self::is_variable_definition(section) {
            return ftd_ast::parse_error(
                format!(
                    "Section is not variable definition section, found `{:?}`",
                    section
                ),
                doc_id,
                section.line_number,
            );
        }

        let kind = ftd_ast::VariableKind::get_kind(
            section.kind.as_ref().unwrap().as_str(),
            doc_id,
            section.line_number,
        )?;
        let processor = Processor::from_headers(&section.headers, doc_id)?;
        let value = ftd_ast::VariableValue::from_p1_with_modifier(
            section,
            doc_id,
            &kind,
            processor.is_some(),
        )?;

        let flags = ftd_ast::VariableFlags::from_headers(&section.headers, doc_id);

        Ok(VariableDefinition::new(
            section.name.trim_start_matches(ftd_ast::utils::REFERENCE),
            kind,
            ftd_ast::utils::is_variable_mutable(section.name.as_str()),
            value,
            processor,
            flags,
            section.line_number,
        ))
    }

    pub fn line_number(&self) -> usize {
        self.line_number
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct VariableInvocation {
    pub name: String,
    pub value: ftd_ast::VariableValue,
    pub condition: Option<ftd_ast::Condition>,
    pub processor: Option<String>,
    pub line_number: usize,
}

impl VariableInvocation {
    fn new(
        name: &str,
        value: ftd_ast::VariableValue,
        condition: Option<ftd_ast::Condition>,
        processor: Option<String>,
        line_number: usize,
    ) -> VariableInvocation {
        VariableInvocation {
            name: name.to_string(),
            value,
            condition,
            processor,
            line_number,
        }
    }

    pub fn line_number(&self) -> usize {
        self.line_number
    }

    pub fn is_variable_invocation(section: &ftd_p1::Section) -> bool {
        section.kind.is_none() && section.name.starts_with(ftd_ast::utils::REFERENCE)
    }

    pub(crate) fn from_p1(
        section: &ftd_p1::Section,
        doc_id: &str,
    ) -> ftd_ast::Result<VariableInvocation> {
        if !Self::is_variable_invocation(section) {
            return ftd_ast::parse_error(
                format!(
                    "Section is not variable invocation section, found `{:?}`",
                    section
                ),
                doc_id,
                section.line_number,
            );
        }

        let value = ftd_ast::VariableValue::from_p1(section, doc_id)?;
        let condition = value.condition().clone();
        let processor = Processor::from_headers(&section.headers, doc_id)?;

        Ok(VariableInvocation::new(
            section.name.trim_start_matches(ftd_ast::utils::REFERENCE),
            // Removing condition because it's redundant here.
            value.set_condition(None),
            condition,
            processor,
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

    pub fn from_headers(headers: &ftd_p1::Headers, doc_id: &str) -> VariableFlags {
        for header in headers.0.iter() {
            if let Ok(flag) = ftd_ast::VariableFlags::from_header(header, doc_id) {
                return flag;
            }
        }

        ftd_ast::VariableFlags::new()
    }

    pub fn from_header(header: &ftd_p1::Header, doc_id: &str) -> ftd_ast::Result<VariableFlags> {
        let kv = match header {
            ftd_p1::Header::KV(kv) => kv,
            ftd_p1::Header::Section(s) => {
                return ftd_ast::parse_error(
                    format!("Expected the boolean value for flag, found: `{:?}`", s),
                    doc_id,
                    header.get_line_number(),
                )
            }
            ftd_p1::Header::BlockRecordHeader(b) => {
                return ftd_ast::parse_error(
                    format!("Expected the boolean value for flag, found: `{:?}`", b),
                    doc_id,
                    header.get_line_number(),
                )
            }
        };

        match kv.key.as_str() {
            ftd_ast::constants::ALWAYS_INCLUDE => {
                let value = kv
                    .value
                    .as_ref()
                    .ok_or(ftd_ast::Error::Parse {
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
            t => ftd_ast::parse_error(format!("Unknown flag found`{}`", t), doc_id, kv.line_number),
        }
    }
}

struct Processor;

impl Processor {
    fn from_headers(headers: &ftd_p1::Headers, doc_id: &str) -> ftd_ast::Result<Option<String>> {
        let processor_header = headers
            .0
            .iter()
            .find(|v| v.get_key().eq(ftd_ast::utils::PROCESSOR));
        let processor_header = if let Some(processor_header) = processor_header {
            processor_header
        } else {
            return Ok(None);
        };

        let processor_statement =
            processor_header
                .get_value(doc_id)?
                .ok_or(ftd_ast::Error::Parse {
                    message: "Processor statement is blank".to_string(),
                    doc_id: doc_id.to_string(),
                    line_number: processor_header.get_line_number(),
                })?;

        Ok(Some(processor_statement))
    }
}
