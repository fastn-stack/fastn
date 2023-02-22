#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct WebComponentDefinition {
    pub name: String,
    pub arguments: Vec<ftd::ast::Argument>,
    pub line_number: usize,
}

pub const WEB_COMPONENT: &str = "web-component";

impl WebComponentDefinition {
    fn new(
        name: &str,
        arguments: Vec<ftd::ast::Argument>,
        line_number: usize,
    ) -> WebComponentDefinition {
        WebComponentDefinition {
            name: name.to_string(),
            arguments,
            line_number,
        }
    }

    pub fn is_web_component_definition(section: &ftd::p11::Section) -> bool {
        section.kind.as_ref().map_or(false, |s| s.eq(WEB_COMPONENT))
    }

    pub fn from_p1(
        section: &ftd::p11::Section,
        doc_id: &str,
    ) -> ftd::ast::Result<WebComponentDefinition> {
        if !Self::is_web_component_definition(section) {
            return ftd::ast::parse_error(
                format!(
                    "Section is not web component definition section, found `{:?}`",
                    section
                ),
                doc_id,
                section.line_number,
            );
        }

        let arguments = ftd::ast::record::get_fields_from_headers(&section.headers, doc_id)?;

        Ok(WebComponentDefinition::new(
            section.name.as_str(),
            arguments,
            section.line_number,
        ))
    }

    pub fn line_number(&self) -> usize {
        self.line_number
    }
}
