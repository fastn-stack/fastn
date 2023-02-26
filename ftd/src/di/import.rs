#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Import {
    pub module: String,
    pub alias: Option<String>,
}

pub const IMPORT: &str = "import";
pub const AS: &str = "as";

impl Import {
    pub(crate) fn is_import(section: &ftd::p11::Section) -> bool {
        section.name.eq(IMPORT)
    }

    pub(crate) fn from_p1(section: &ftd::p11::Section, doc_id: &str) -> ftd::di::Result<Import> {
        if !Self::is_import(section) {
            return ftd::di::parse_error(
                format!("Section is not import section, found `{:?}`", section),
                doc_id,
                section.line_number,
            );
        }
        if !section.sub_sections.is_empty() {
            return ftd::di::parse_error(
                format!(
                    "Subsection not expected for import statement `{:?}`",
                    section
                ),
                doc_id,
                section.line_number,
            );
        }
        match &section.caption {
            Some(ftd::p11::Header::KV(ftd::p11::header::KV {
                value: Some(value), ..
            })) => {
                let (module, alias) = ftd::di::utils::split_at(value.as_str(), AS);
                Ok(Import { module, alias })
            }
            t => ftd::di::parse_error(
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
