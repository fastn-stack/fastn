#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct WebComponentDefinition {
    pub name: String,
    pub arguments: Vec<ftd::ast::Argument>,
    pub js: String,
    pub line_number: usize,
}

pub const WEB_COMPONENT: &str = "web-component";

impl WebComponentDefinition {
    fn new(
        name: &str,
        arguments: Vec<ftd::ast::Argument>,
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

        let (js, arguments) =
            get_fields_from_headers(&section.headers, doc_id, section.line_number)?;

        Ok(WebComponentDefinition::new(
            section.name.as_str(),
            arguments,
            js,
            section.line_number,
        ))
    }

    pub fn line_number(&self) -> usize {
        self.line_number
    }
}

pub(crate) fn get_fields_from_headers(
    headers: &ftd::p11::Headers,
    doc_id: &str,
    line_number: usize,
) -> ftd::ast::Result<(String, Vec<ftd::ast::Argument>)> {
    let mut fields: Vec<ftd::ast::Argument> = Default::default();
    let mut js = None;
    for header in headers.0.iter() {
        if header.get_kind().is_none() && header.get_key().eq(ftd::ast::constants::JS) {
            js = Some(header.get_value(doc_id)?.ok_or(ftd::ast::Error::Parse {
                message: "js statement is blank".to_string(),
                doc_id: doc_id.to_string(),
                line_number: header.get_line_number(),
            })?);
            continue;
        }
        fields.push(ftd::ast::Argument::from_header(header, doc_id)?);
    }
    Ok((
        js.ok_or(ftd::ast::Error::Parse {
            message: "js statement not found".to_string(),
            doc_id: doc_id.to_string(),
            line_number,
        })?,
        fields,
    ))
}
