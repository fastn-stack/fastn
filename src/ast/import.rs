#[derive(Debug, PartialEq)]
pub struct Import {
    pub module: String,
    pub alias: Option<String>,
}

pub const IMPORT: &str = "import";

impl Import {
    pub(crate) fn is_import(section: &ftd::p11::Section) -> bool {
        section.name.eq(IMPORT)
    }

    pub(crate) fn from_p1(section: &ftd::p11::Section, doc_id: &str) -> ftd::ast::Result<Import> {
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
            Some(header)
                if header.get_value(doc_id).is_ok() && header.get_value(doc_id)?.is_some() =>
            {
                let value = header.get_value(doc_id)?.unwrap();
                let (module, alias) = ftd::ast::utils::split_at(value.as_str(), "as");
                Ok(Import { module, alias })
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
}
