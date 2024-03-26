#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Import {
    pub module: String,
    pub alias: String,
    #[serde(rename = "line-number")]
    pub line_number: usize,
    pub exports: Option<Export>,
    pub exposing: Option<Exposing>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum Export {
    All,
    Things(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum Exposing {
    All,
    Things(Vec<String>),
}

pub const IMPORT: &str = "import";

impl Import {
    fn new(
        module: &str,
        alias: &str,
        line_number: usize,
        exports: Option<Export>,
        exposing: Option<Exposing>,
    ) -> Import {
        Import {
            module: module.to_string(),
            alias: alias.to_string(),
            line_number,
            exports,
            exposing,
        }
    }
    pub(crate) fn is_import(section: &ftd_p1::Section) -> bool {
        section.name.eq(IMPORT)
    }

    pub(crate) fn from_p1(section: &ftd_p1::Section, doc_id: &str) -> ftd::ast::Result<Import> {
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
                    "SubSection not expected for import statement `{:?}`",
                    section
                ),
                doc_id,
                section.line_number,
            );
        }
        let exports = Export::get_exports_from_headers(&section.headers, doc_id)?;
        let exposing = Exposing::get_exposing_from_headers(&section.headers, doc_id)?;
        match &section.caption {
            Some(ftd_p1::Header::KV(ftd_p1::KV {
                value: Some(value), ..
            })) => {
                let (module, alias) = ftd::ast::utils::get_import_alias(value.as_str());
                Ok(Import::new(
                    module.as_str(),
                    alias.as_str(),
                    section.line_number,
                    exports,
                    exposing,
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
    fn is_export(header: &ftd_p1::Header) -> bool {
        header.get_key().eq(ftd::ast::constants::EXPORT) && header.get_kind().is_none()
    }

    pub(crate) fn get_exports_from_headers(
        headers: &ftd_p1::Headers,
        doc_id: &str,
    ) -> ftd::ast::Result<Option<Export>> {
        let mut exports = vec![];
        for header in headers.0.iter() {
            if !Self::is_export(header) {
                if !Exposing::is_exposing(header) {
                    return ftd::ast::parse_error(
                        format!("Expected `export` or `exposing`, found `{:?}`", header),
                        doc_id,
                        header.get_line_number(),
                    );
                }
                continue;
            }
            let value = header.get_value(doc_id)?.ok_or(ftd::ast::Error::Parse {
                message: "Expected the export thing name".to_string(),
                doc_id: doc_id.to_string(),
                line_number: header.get_line_number(),
            })?;
            if value.eq(ftd::ast::constants::EVERYTHING) {
                return Ok(Some(Export::All));
            } else {
                exports.extend(value.split(',').map(|v| v.trim().to_string()));
            }
        }
        Ok(if exports.is_empty() {
            None
        } else {
            Some(Export::Things(exports))
        })
    }
}

impl Exposing {
    fn is_exposing(header: &ftd_p1::Header) -> bool {
        header.get_key().eq(ftd::ast::constants::EXPOSING) && header.get_kind().is_none()
    }

    pub(crate) fn get_exposing_from_headers(
        headers: &ftd_p1::Headers,
        doc_id: &str,
    ) -> ftd::ast::Result<Option<Exposing>> {
        let mut exposing = vec![];
        for header in headers.0.iter() {
            if !Self::is_exposing(header) {
                if !Export::is_export(header) {
                    return ftd::ast::parse_error(
                        format!("Expected `export` or `exposing`, found `{:?}`", header),
                        doc_id,
                        header.get_line_number(),
                    );
                }
                continue;
            }
            let value = header.get_value(doc_id)?.ok_or(ftd::ast::Error::Parse {
                message: "Expected the exposing thing name".to_string(),
                doc_id: doc_id.to_string(),
                line_number: header.get_line_number(),
            })?;
            if value.eq(ftd::ast::constants::EVERYTHING) {
                return Ok(Some(Exposing::All));
            } else {
                exposing.extend(value.split(',').map(|v| v.trim().to_string()));
            }
        }
        Ok(if exposing.is_empty() {
            None
        } else {
            Some(Exposing::Things(exposing))
        })
    }
}
