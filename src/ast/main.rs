#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum AST {
    Import(ftd::ast::Import),
    Record(ftd::ast::Record),
    VariableDefinition(ftd::ast::VariableDefinition),
    VariableInvocation(ftd::ast::VariableInvocation),
    ComponentDefinition(ftd::ast::ComponentDefinition),
    ComponentInvocation(ftd::ast::Component),
}

impl AST {
    pub fn from_sections(
        sections: &[ftd::p11::Section],
        doc_id: &str,
    ) -> ftd::ast::Result<Vec<AST>> {
        let mut di_vec = vec![];
        for section in ignore_comments(sections) {
            di_vec.push(AST::from_section(&section, doc_id)?);
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
        } else if ftd::ast::VariableInvocation::is_variable_invocation(section) {
            AST::VariableInvocation(ftd::ast::VariableInvocation::from_p1(section, doc_id)?)
        } else if ftd::ast::ComponentDefinition::is_component_definition(section) {
            AST::ComponentDefinition(ftd::ast::ComponentDefinition::from_p1(section, doc_id)?)
        } else if ftd::ast::Component::is_component(section) {
            AST::ComponentInvocation(ftd::ast::Component::from_p1(section, doc_id)?)
        } else {
            return Err(ftd::ast::Error::ParseError {
                message: format!("Invalid AST, found: `{:?}`", section),
                doc_id: doc_id.to_string(),
                line_number: section.line_number,
            });
        })
    }

    pub fn line_number(&self) -> usize {
        match self {
            AST::Import(i) => i.line_number(),
            AST::Record(r) => r.line_number(),
            AST::VariableDefinition(v) => v.line_number(),
            AST::VariableInvocation(v) => v.line_number(),
            AST::ComponentDefinition(c) => c.line_number(),
            AST::ComponentInvocation(c) => c.line_number(),
        }
    }

    pub fn get_record(&self, doc_id: &str) -> ftd::ast::Result<&ftd::ast::Record> {
        if let ftd::ast::AST::Record(r) = self {
            return Ok(r);
        }
        ftd::ast::parse_error(
            format!("`{:?}` is not a record", self),
            doc_id,
            self.line_number(),
        )
    }

    pub fn is_record(&self) -> bool {
        matches!(self, AST::Record(_))
    }

    #[cfg(test)]
    pub(crate) fn list(self) -> Vec<Self> {
        vec![self]
    }
}

/// Filters out commented parts from the parsed document.
///
/// # Comments are ignored for
/// 1.  /-- section: caption
///
/// 2.  /section-header: value
///
/// 3.  /body
///
/// 4.  /--- subsection: caption
///
/// 5.  /sub-section-header: value
///
/// ## Note: To allow ["/content"] inside body, use ["\\/content"].
///
/// Only '/' comments are ignored here.
/// ';' comments are ignored inside the [`parser`] itself.
///
/// uses [`Section::remove_comments()`] and [`Subsection::remove_comments()`] to remove comments
/// in sections and subsections accordingly.
///
/// [`parser`]: ftd::p1::parser::parse
/// [`Section::remove_comments()`]: ftd::p1::section::Section::remove_comments
/// [`SubSection::remove_comments()`]: ftd::p1::sub_section::SubSection::remove_comments
fn ignore_comments(sections: &[ftd::p11::Section]) -> Vec<ftd::p11::Section> {
    // TODO: AST should contain the commented elements. Comments should not be ignored while creating AST.
    sections
        .iter()
        .filter_map(|s| s.remove_comments())
        .collect::<Vec<ftd::p11::Section>>()
}
