#![allow(dead_code)]

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum AST {
    Import(ftd::ast::Import),
    Record(ftd::ast::Record),
    VariableDefinition(ftd::ast::VariableDefinition),
}

impl AST {
    pub fn from_sections(
        sections: &[ftd::p11::Section],
        doc_id: &str,
    ) -> ftd::ast::Result<Vec<AST>> {
        let mut di_vec = vec![];
        for section in sections {
            di_vec.push(AST::from_section(section, doc_id)?);
        }
        Ok(di_vec)
    }

    pub fn from_section(section: &ftd::p11::Section, doc_id: &str) -> ftd::ast::Result<AST> {
        Ok(if ftd::ast::Import::is_import(section) {
            AST::Import(ftd::ast::Import::from_p1(section, doc_id)?)
        } else if ftd::ast::Record::is_record(section) {
            AST::Record(ftd::ast::Record::from_p1(section, doc_id)?)
        } else if ftd::ast::VariableDefinition::is_variable_definition(section) {
            AST::VariableDefinition(ftd::ast::VariableDefinition::from_p1(section, doc_id)?)
        } else {
            return Err(ftd::ast::Error::ParseError {
                message: format!("Invalid AST, found: `{:?}`", section),
                doc_id: doc_id.to_string(),
                line_number: section.line_number,
            });
        })
    }

    #[cfg(test)]
    pub(crate) fn list(self) -> Vec<Self> {
        vec![self]
    }
}
