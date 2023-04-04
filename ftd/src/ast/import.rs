#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Import {
    pub module: String,
    pub alias: String,
    pub line_number: usize,
}

pub const IMPORT: &str = "import";

impl Import {
    fn new(module: &str, alias: &str, line_number: usize) -> Import {
        Import {
            module: module.to_string(),
            alias: alias.to_string(),
            line_number,
        }
    }
    pub(crate) fn is_import(section: &ftd::p1::Section) -> bool {
        section.name.eq(IMPORT)
    }

    pub(crate) fn from_p1(section: &ftd::p1::Section, doc_id: &str) -> ftd::ast::Result<Import> {
        if !Self::is_import(section) {
            return ftd::ast::parse_error(
                format!("Section is not import section, found `{:?}`", section),
                doc_id,
                section.line_number,
            );
        }
        if !section.sub_sections.is_empty() {
            return ftd::ast::parse_error(
                format!(
                    "Subsection not expected for import statement `{:?}`",
                    section
                ),
                doc_id,
                section.line_number,
            );
        }
        match &section.caption {
            Some(ftd::p1::Header::KV(ftd::p1::header::KV {
                value: Some(value), ..
            })) => {
                let (module, alias) = ftd::ast::utils::get_import_alias(value.as_str());
                Ok(Import::new(
                    module.as_str(),
                    alias.as_str(),
                    section.line_number,
                ))
            }
            t => ftd::ast::parse_error(
                format!(
                    "Expected value in caption for import statement, found: `{:?}`",
                    t
                ),
                doc_id,
                section.line_number,
            ),
        }
    }

    pub fn line_number(&self) -> usize {
        self.line_number
    }
}
