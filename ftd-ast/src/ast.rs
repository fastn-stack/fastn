#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
// #[serde(tag = "ast-type", content = "c")]
pub enum AST {
    #[serde(rename = "import")]
    Import(ftd_ast::Import),
    #[serde(rename = "record")]
    Record(ftd_ast::Record),
    #[serde(rename = "or-type")]
    OrType(ftd_ast::OrType),
    VariableDefinition(ftd_ast::VariableDefinition),
    VariableInvocation(ftd_ast::VariableInvocation),
    ComponentDefinition(ftd_ast::ComponentDefinition),
    #[serde(rename = "component-invocation")]
    ComponentInvocation(ftd_ast::Component),
    FunctionDefinition(ftd_ast::Function),
    WebComponentDefinition(ftd_ast::WebComponentDefinition),
}

// -- foo:

// -- component foo:
// -- ftd.text: hello
// -- end: foo

// -- integer x(a,b):
// a + b

// ->

// -- ftd.text: hello

impl AST {
    pub fn from_sections(sections: &[ftd_p1::Section], doc_id: &str) -> ftd_ast::Result<Vec<AST>> {
        let mut di_vec = vec![];
        for section in ignore_comments(sections) {
            di_vec.push(AST::from_section(&section, doc_id)?);
        }
        Ok(di_vec)
    }

    pub fn name(&self) -> String {
        match self {
            AST::Import(i) => i.alias.clone(),
            AST::Record(r) => r.name.clone(),
            AST::VariableDefinition(v) => v.name.clone(),
            AST::VariableInvocation(v) => v.name.clone(),
            AST::ComponentDefinition(c) => c.name.clone(),
            AST::ComponentInvocation(c) => c.name.clone(),
            AST::FunctionDefinition(f) => f.name.clone(),
            AST::OrType(o) => o.name.clone(),
            AST::WebComponentDefinition(w) => w.name.clone(),
        }
    }

    pub fn from_section(section: &ftd_p1::Section, doc_id: &str) -> ftd_ast::Result<AST> {
        Ok(if ftd_ast::Import::is_import(section) {
            AST::Import(ftd_ast::Import::from_p1(section, doc_id)?)
        } else if ftd_ast::Record::is_record(section) {
            AST::Record(ftd_ast::Record::from_p1(section, doc_id)?)
        } else if ftd_ast::OrType::is_or_type(section) {
            AST::OrType(ftd_ast::OrType::from_p1(section, doc_id)?)
        } else if ftd_ast::Function::is_function(section) {
            AST::FunctionDefinition(ftd_ast::Function::from_p1(section, doc_id)?)
        } else if ftd_ast::VariableDefinition::is_variable_definition(section) {
            AST::VariableDefinition(ftd_ast::VariableDefinition::from_p1(section, doc_id)?)
        } else if ftd_ast::VariableInvocation::is_variable_invocation(section) {
            AST::VariableInvocation(ftd_ast::VariableInvocation::from_p1(section, doc_id)?)
        } else if ftd_ast::ComponentDefinition::is_component_definition(section) {
            AST::ComponentDefinition(ftd_ast::ComponentDefinition::from_p1(section, doc_id)?)
        } else if ftd_ast::WebComponentDefinition::is_web_component_definition(section) {
            AST::WebComponentDefinition(ftd_ast::WebComponentDefinition::from_p1(section, doc_id)?)
        } else if ftd_ast::Component::is_component(section) {
            AST::ComponentInvocation(ftd_ast::Component::from_p1(section, doc_id)?)
        } else {
            return Err(ftd_ast::Error::Parse {
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
            AST::FunctionDefinition(f) => f.line_number(),
            AST::OrType(o) => o.line_number(),
            AST::WebComponentDefinition(w) => w.line_number,
        }
    }

    pub fn get_record(self, doc_id: &str) -> ftd_ast::Result<ftd_ast::Record> {
        if let ftd_ast::AST::Record(r) = self {
            return Ok(r);
        }
        ftd_ast::parse_error(
            format!("`{:?}` is not a record", self),
            doc_id,
            self.line_number(),
        )
    }

    pub fn get_or_type(self, doc_id: &str) -> ftd_ast::Result<ftd_ast::OrType> {
        if let ftd_ast::AST::OrType(o) = self {
            return Ok(o);
        }
        ftd_ast::parse_error(
            format!("`{:?}` is not a or-type", self),
            doc_id,
            self.line_number(),
        )
    }

    pub fn get_function(self, doc_id: &str) -> ftd_ast::Result<ftd_ast::Function> {
        if let ftd_ast::AST::FunctionDefinition(r) = self {
            return Ok(r);
        }
        ftd_ast::parse_error(
            format!("`{:?}` is not a function", self),
            doc_id,
            self.line_number(),
        )
    }

    pub fn get_variable_definition(
        self,
        doc_id: &str,
    ) -> ftd_ast::Result<ftd_ast::VariableDefinition> {
        if let ftd_ast::AST::VariableDefinition(v) = self {
            return Ok(v);
        }
        ftd_ast::parse_error(
            format!("`{:?}` is not a variable definition", self),
            doc_id,
            self.line_number(),
        )
    }

    pub fn get_variable_invocation(
        self,
        doc_id: &str,
    ) -> ftd_ast::Result<ftd_ast::VariableInvocation> {
        if let ftd_ast::AST::VariableInvocation(v) = self {
            return Ok(v);
        }
        ftd_ast::parse_error(
            format!("`{:?}` is not a variable definition", self),
            doc_id,
            self.line_number(),
        )
    }

    pub fn get_component_definition(
        self,
        doc_id: &str,
    ) -> ftd_ast::Result<ftd_ast::ComponentDefinition> {
        if let ftd_ast::AST::ComponentDefinition(v) = self {
            return Ok(v);
        }
        ftd_ast::parse_error(
            format!("`{:?}` is not a component definition", self),
            doc_id,
            self.line_number(),
        )
    }

    pub fn get_web_component_definition(
        self,
        doc_id: &str,
    ) -> ftd_ast::Result<ftd_ast::WebComponentDefinition> {
        if let ftd_ast::AST::WebComponentDefinition(v) = self {
            return Ok(v);
        }
        ftd_ast::parse_error(
            format!("`{:?}` is not a web-component definition", self),
            doc_id,
            self.line_number(),
        )
    }

    pub fn get_component_invocation(self, doc_id: &str) -> ftd_ast::Result<ftd_ast::Component> {
        if let ftd_ast::AST::ComponentInvocation(v) = self {
            return Ok(v);
        }
        ftd_ast::parse_error(
            format!("`{:?}` is not a component definition", self),
            doc_id,
            self.line_number(),
        )
    }

    pub fn is_record(&self) -> bool {
        matches!(self, AST::Record(_))
    }

    pub fn is_or_type(&self) -> bool {
        matches!(self, AST::OrType(_))
    }

    pub fn is_import(&self) -> bool {
        matches!(self, AST::Import(_))
    }

    pub fn is_variable_definition(&self) -> bool {
        matches!(self, AST::VariableDefinition(_))
    }

    pub fn is_function(&self) -> bool {
        matches!(self, AST::FunctionDefinition(_))
    }

    pub fn is_variable_invocation(&self) -> bool {
        matches!(self, AST::VariableInvocation(_))
    }

    pub fn is_component_definition(&self) -> bool {
        matches!(self, AST::ComponentDefinition(_))
    }

    pub fn is_web_component_definition(&self) -> bool {
        matches!(self, AST::WebComponentDefinition(_))
    }

    pub fn is_component(&self) -> bool {
        matches!(self, AST::ComponentInvocation(_))
    }

    pub fn is_always_included_variable_definition(&self) -> bool {
        matches!(
            self,
            AST::VariableDefinition(ftd_ast::VariableDefinition {
                flags: ftd_ast::VariableFlags {
                    always_include: Some(true)
                },
                ..
            })
        )
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
/// uses [`Section::remove_comments()`] and [`SubSection::remove_comments()`] to remove comments
/// in sections and sub_sections accordingly.
///
/// [`parser`]: ftd_p1::parser::parse
/// [`Section::remove_comments()`]: ftd_p1::section::Section::remove_comments
/// [`SubSection::remove_comments()`]: ftd_p1::sub_section::SubSection::remove_comments
fn ignore_comments(sections: &[ftd_p1::Section]) -> Vec<ftd_p1::Section> {
    // TODO: AST should contain the commented elements. Comments should not be ignored while creating AST.
    sections
        .iter()
        .filter_map(|s| s.remove_comments())
        .collect::<Vec<ftd_p1::Section>>()
}
