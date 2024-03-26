#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Function {
    pub name: String,
    pub kind: ftd::ast::VariableKind,
    pub arguments: Vec<ftd::ast::Argument>,
    pub line_number: usize,
    pub definition: FunctionDefinition,
    pub js: Option<String>,
}

pub type FunctionDefinition = ftd_p1::Body;

impl Function {
    pub(crate) fn new(
        name: &str,
        kind: ftd::ast::VariableKind,
        arguments: Vec<ftd::ast::Argument>,
        line_number: usize,
        definition: FunctionDefinition,
        js: Option<String>,
    ) -> Function {
        Function {
            name: name.to_string(),
            kind,
            arguments,
            line_number,
            definition,
            js,
        }
    }

    pub(crate) fn is_function(section: &ftd_p1::Section) -> bool {
        Function::function_name(section).is_some()
    }

    pub(crate) fn function_name(section: &ftd_p1::Section) -> Option<String> {
        if ftd::ast::Import::is_import(section)
            || ftd::ast::Record::is_record(section)
            || ftd::ast::OrType::is_or_type(section)
            || ftd::ast::ComponentDefinition::is_component_definition(section)
            || section.kind.is_none()
        {
            return None;
        }

        match (section.name.find('('), section.name.find(')')) {
            (Some(si), Some(ei)) if si < ei => Some(section.name[..si].to_string()),
            _ => None,
        }
    }

    pub(crate) fn from_p1(section: &ftd_p1::Section, doc_id: &str) -> ftd::ast::Result<Function> {
        let function_name = Self::function_name(section).ok_or(ftd::ast::Error::Parse {
            message: format!("Section is not function section, found `{:?}`", section),
            doc_id: doc_id.to_string(),
            line_number: section.line_number,
        })?;
        let kind = ftd::ast::VariableKind::get_kind(
            section.kind.as_ref().unwrap().as_str(),
            doc_id,
            section.line_number,
        )?;
        let (js, fields) =
            ftd::ast::utils::get_js_and_fields_from_headers(&section.headers, doc_id)?;
        let definition = section.body.clone().ok_or(ftd::ast::Error::Parse {
            message: format!(
                "Function definition not found for function {}",
                section.name
            ),
            doc_id: doc_id.to_string(),
            line_number: section.line_number,
        })?;
        Ok(Function::new(
            function_name.as_str(),
            kind,
            fields,
            section.line_number,
            definition,
            js,
        ))
    }

    pub fn line_number(&self) -> usize {
        self.line_number
    }
}
