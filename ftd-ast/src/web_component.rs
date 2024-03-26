#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct WebComponentDefinition {
    pub name: String,
    pub arguments: Vec<ftd_ast::Argument>,
    pub js: String,
    pub line_number: usize,
}

pub const WEB_COMPONENT: &str = "web-component";

impl WebComponentDefinition {
    fn new(
        name: &str,
        arguments: Vec<ftd_ast::Argument>,
        js: String,
        line_number: usize,
    ) -> WebComponentDefinition {
        WebComponentDefinition {
            name: name.to_string(),
            arguments,
            js,
            line_number,
        }
    }

    pub fn is_web_component_definition(section: &ftd_p1::Section) -> bool {
        section.kind.as_ref().map_or(false, |s| s.eq(WEB_COMPONENT))
    }

    pub fn from_p1(
        section: &ftd_p1::Section,
        doc_id: &str,
    ) -> ftd_ast::Result<WebComponentDefinition> {
        if !Self::is_web_component_definition(section) {
            return ftd_ast::parse_error(
                format!(
                    "Section is not web component definition section, found `{:?}`",
                    section
                ),
                doc_id,
                section.line_number,
            );
        }

        let (js, arguments) =
            ftd_ast::utils::get_js_and_fields_from_headers(&section.headers, doc_id)?;

        Ok(WebComponentDefinition::new(
            section.name.as_str(),
            arguments,
            js.ok_or(ftd_ast::Error::Parse {
                message: "js statement not found".to_string(),
                doc_id: doc_id.to_string(),
                line_number: section.line_number,
            })?,
            section.line_number,
        ))
    }

    pub fn line_number(&self) -> usize {
        self.line_number
    }
}
