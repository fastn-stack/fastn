#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Import {
    pub module: String,
    pub alias: Option<String>,
}

pub const IMPORT: &str = "import";
pub const AS: &str = "as";

impl Import {
    pub(crate) fn is_import(section: &ftd_p1::Section) -> bool {
        section.name.eq(IMPORT)
    }

    pub(crate) fn from_p1(
        section: &ftd_p1::Section,
        doc_id: &str,
    ) -> ftd::ftd2021::di::Result<Import> {
        if !Self::is_import(section) {
            return ftd::ftd2021::di::parse_error(
                format!("Section is not import section, found `{section:?}`"),
                doc_id,
                section.line_number,
            );
        }
        if !section.sub_sections.is_empty() {
            return ftd::ftd2021::di::parse_error(
                format!("SubSection not expected for import statement `{section:?}`"),
                doc_id,
                section.line_number,
            );
        }
        match &section.caption {
            Some(ftd_p1::Header::KV(ftd_p1::KV {
                value: Some(value), ..
            })) => {
                let (module, alias) = ftd::ftd2021::di::utils::split_at(value.as_str(), AS);
                Ok(Import { module, alias })
            }
            t => ftd::ftd2021::di::parse_error(
                format!("Expected value in caption for import statement, found: `{t:?}`"),
                doc_id,
                section.line_number,
            ),
        }
    }
}
