#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
// #[serde(tag = "ast-type", content = "c")]
pub enum Ast {
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
    ComponentInvocation(ftd_ast::ComponentInvocation),
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

impl Ast {
    pub fn from_sections(sections: &[ftd_p1::Section], doc_id: &str) -> ftd_ast::Result<Vec<Ast>> {
        let mut di_vec = vec![];
        for section in ignore_comments(sections) {
            di_vec.push(Ast::from_section(&section, doc_id)?);
        }
        Ok(di_vec)
    }

    pub fn name(&self) -> String {
        match self {
            Ast::Import(i) => i.alias.clone(),
            Ast::Record(r) => r.name.clone(),
            Ast::VariableDefinition(v) => v.name.clone(),
            Ast::VariableInvocation(v) => v.name.clone(),
            Ast::ComponentDefinition(c) => c.name.clone(),
            Ast::ComponentInvocation(c) => c.name.clone(),
            Ast::FunctionDefinition(f) => f.name.clone(),
            Ast::OrType(o) => o.name.clone(),
            Ast::WebComponentDefinition(w) => w.name.clone(),
        }
    }


    pub fn get_definition_name(&self) -> Option<String> {
        match self {
            Ast::ComponentDefinition(c) => Some(c.name.clone()),
            Ast::FunctionDefinition(f) => Some(f.name.clone()),
            Ast::VariableDefinition(v) => Some(v.name.clone()),
            Ast::Record(r) => Some(r.name.clone()),
            Ast::OrType(o) => Some(o.name.clone()),
            Ast::WebComponentDefinition(w) => Some(w.name.clone()),
            _ => None
        }
    }

    pub fn from_section(section: &ftd_p1::Section, doc_id: &str) -> ftd_ast::Result<Ast> {
        Ok(if ftd_ast::Import::is_import(section) {
            Ast::Import(ftd_ast::Import::from_p1(section, doc_id)?)
        } else if ftd_ast::Record::is_record(section) {
            Ast::Record(ftd_ast::Record::from_p1(section, doc_id)?)
        } else if ftd_ast::OrType::is_or_type(section) {
            Ast::OrType(ftd_ast::OrType::from_p1(section, doc_id)?)
        } else if ftd_ast::Function::is_function(section) {
            Ast::FunctionDefinition(ftd_ast::Function::from_p1(section, doc_id)?)
        } else if ftd_ast::VariableDefinition::is_variable_definition(section) {
            Ast::VariableDefinition(ftd_ast::VariableDefinition::from_p1(section, doc_id)?)
        } else if ftd_ast::VariableInvocation::is_variable_invocation(section) {
            Ast::VariableInvocation(ftd_ast::VariableInvocation::from_p1(section, doc_id)?)
        } else if ftd_ast::ComponentDefinition::is_component_definition(section) {
            Ast::ComponentDefinition(ftd_ast::ComponentDefinition::from_p1(section, doc_id)?)
        } else if ftd_ast::WebComponentDefinition::is_web_component_definition(section) {
            Ast::WebComponentDefinition(ftd_ast::WebComponentDefinition::from_p1(section, doc_id)?)
        } else if ftd_ast::ComponentInvocation::is_component(section) {
            Ast::ComponentInvocation(ftd_ast::ComponentInvocation::from_p1(section, doc_id)?)
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
            Ast::Import(i) => i.line_number(),
            Ast::Record(r) => r.line_number(),
            Ast::VariableDefinition(v) => v.line_number(),
            Ast::VariableInvocation(v) => v.line_number(),
            Ast::ComponentDefinition(c) => c.line_number(),
            Ast::ComponentInvocation(c) => c.line_number(),
            Ast::FunctionDefinition(f) => f.line_number(),
            Ast::OrType(o) => o.line_number(),
            Ast::WebComponentDefinition(w) => w.line_number,
        }
    }

    pub fn get_record(self, doc_id: &str) -> ftd_ast::Result<ftd_ast::Record> {
        if let ftd_ast::Ast::Record(r) = self {
            return Ok(r);
        }
        ftd_ast::parse_error(
            format!("`{:?}` is not a record", self),
            doc_id,
            self.line_number(),
        )
    }

    pub fn get_or_type(self, doc_id: &str) -> ftd_ast::Result<ftd_ast::OrType> {
        if let ftd_ast::Ast::OrType(o) = self {
            return Ok(o);
        }
        ftd_ast::parse_error(
            format!("`{:?}` is not a or-type", self),
            doc_id,
            self.line_number(),
        )
    }

    pub fn get_function(self, doc_id: &str) -> ftd_ast::Result<ftd_ast::Function> {
        if let ftd_ast::Ast::FunctionDefinition(r) = self {
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
        if let ftd_ast::Ast::VariableDefinition(v) = self {
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
        if let ftd_ast::Ast::VariableInvocation(v) = self {
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
        if let ftd_ast::Ast::ComponentDefinition(v) = self {
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
        if let ftd_ast::Ast::WebComponentDefinition(v) = self {
            return Ok(v);
        }
        ftd_ast::parse_error(
            format!("`{:?}` is not a web-component definition", self),
            doc_id,
            self.line_number(),
        )
    }

    pub fn get_component_invocation(
        self,
        doc_id: &str,
    ) -> ftd_ast::Result<ftd_ast::ComponentInvocation> {
        if let ftd_ast::Ast::ComponentInvocation(v) = self {
            return Ok(v);
        }
        ftd_ast::parse_error(
            format!("`{:?}` is not a component definition", self),
            doc_id,
            self.line_number(),
        )
    }

    pub fn is_record(&self) -> bool {
        matches!(self, Ast::Record(_))
    }

    pub fn is_or_type(&self) -> bool {
        matches!(self, Ast::OrType(_))
    }

    pub fn is_import(&self) -> bool {
        matches!(self, Ast::Import(_))
    }

    pub fn is_variable_definition(&self) -> bool {
        matches!(self, Ast::VariableDefinition(_))
    }

    pub fn is_function(&self) -> bool {
        matches!(self, Ast::FunctionDefinition(_))
    }

    pub fn is_variable_invocation(&self) -> bool {
        matches!(self, Ast::VariableInvocation(_))
    }

    pub fn is_component_definition(&self) -> bool {
        matches!(self, Ast::ComponentDefinition(_))
    }

    pub fn is_web_component_definition(&self) -> bool {
        matches!(self, Ast::WebComponentDefinition(_))
    }

    pub fn is_component(&self) -> bool {
        matches!(self, Ast::ComponentInvocation(_))
    }

    pub fn is_always_included_variable_definition(&self) -> bool {
        matches!(
            self,
            Ast::VariableDefinition(ftd_ast::VariableDefinition {
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
