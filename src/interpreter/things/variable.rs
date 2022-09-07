#![allow(dead_code)]

#[derive(Debug, PartialEq, Clone)]
pub struct Variable {
    pub name: String,
    pub value: ftd::interpreter::PropertyValue,
    pub conditions: Vec<ConditionalValue>,
    pub flags: VariableFlags,
    pub source: TextSource,
}

impl Variable {
    pub(crate) fn from_p1_section(
        s: &ftd::p11::Section,
        doc_id: &str,
    ) -> ftd::interpreter::Result<Variable> {
        let value = ftd::interpreter::PropertyValue::from_p1_section(s, doc_id)?;
        if !s.headers.find("if").is_empty() {
            return Err(ftd::interpreter::Error::ParseError {
                message: format!(
                    "`if` can't be present in variable declaration for section: `{}`",
                    s.name
                ),
                doc_id: doc_id.to_string(),
                line_number: s.line_number,
            });
        }
        let flags = Variable::get_flags(s, doc_id)?;
        Ok(Variable {
            name: s.name.to_string(),
            value,
            conditions: vec![],
            flags,
            source: TextSource::Header,
        })
    }

    pub(crate) fn get_flags(
        s: &ftd::p11::Section,
        doc_id: &str,
    ) -> ftd::p11::Result<VariableFlags> {
        let header = match ftd::interpreter::PropertyValue::for_header_with_kind(
            s,
            doc_id,
            ALWAYS_INCLUDE,
            &ftd::interpreter::KindData::boolean(),
        ) {
            Ok(val) => val,
            _ => return Ok(VariableFlags::default()),
        };

        match header {
            ftd::interpreter::PropertyValue::Value {
                value: ftd::interpreter::Value::Boolean { value },
            } => Ok(VariableFlags {
                always_include: Some(value),
            }),
            ftd::interpreter::PropertyValue::Reference { .. } => unimplemented!(),
            t => {
                return Err(ftd::p11::Error::ParseError {
                    message: format!("Expected boolean found: {:?}", t),
                    doc_id: doc_id.to_string(),
                    line_number: s.line_number,
                })
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ConditionalValue {
    pub expression: ftd::interpreter::Boolean,
    pub value: ftd::interpreter::PropertyValue,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct VariableFlags {
    pub always_include: Option<bool>,
}

pub const ALWAYS_INCLUDE: &str = "$always-include$";

#[derive(Debug, PartialEq, Clone)]
pub enum TextSource {
    Header,
    Caption,
    Body,
    Default,
}
