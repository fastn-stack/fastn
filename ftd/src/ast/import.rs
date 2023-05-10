#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Import {
    pub module: String,
    pub alias: String,
    #[serde(rename = "line-number")]
    pub line_number: usize,
    pub export: Option<Export>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum Export {
    All,
    Things(Vec<String>),
}

pub const IMPORT: &str = "import";

impl Import {
    fn new(module: &str, alias: &str, line_number: usize, export: Option<Export>) -> Import {
        Import {
            module: module.to_string(),
            alias: alias.to_string(),
            line_number,
            export,
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
        let exports = Export::get_exports_from_headers(&section.headers, doc_id)?;
        match &section.caption {
            Some(ftd::p1::Header::KV(ftd::p1::header::KV {
                value: Some(value), ..
            })) => {
                let (module, alias) = ftd::ast::utils::get_import_alias(value.as_str());
                Ok(Import::new(
                    module.as_str(),
                    alias.as_str(),
                    section.line_number,
                    exports,
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

impl Export {
    fn is_export(header: &ftd::p1::Header) -> bool {
        header.get_key().eq(ftd::ast::constants::EXPORT) && header.get_kind().is_none()
    }

    pub(crate) fn get_exports_from_headers(
        headers: &ftd::p1::Headers,
        doc_id: &str,
    ) -> ftd::ast::Result<Option<Export>> {
        let mut exports = vec![];
        for header in headers.0.iter() {
            if !Self::is_export(header) {
                return ftd::ast::parse_error(
                    format!("Expected `export`, found `{:?}`", header),
                    doc_id,
                    header.get_line_number(),
                );
            }
            let value = header.get_value(doc_id)?.ok_or(ftd::ast::Error::Parse {
                message: "Expected the export thing name".to_string(),
                doc_id: doc_id.to_string(),
                line_number: header.get_line_number(),
            })?;
            if value.eq(ftd::ast::constants::EVERYTHING) {
                return Ok(Some(Export::All));
            } else {
                exports.push(value);
            }
        }
        Ok(if exports.is_empty() {
            None
        } else {
            Some(Export::Things(exports))
        })
    }
}
