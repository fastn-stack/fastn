#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Edition {
    pub edition: FTDEdition,
    pub line_number: usize,
}

pub const EDITION: &str = "edition";

impl Edition {
    fn new(edition: FTDEdition, line_number: usize) -> Edition {
        Edition {
            edition,
            line_number,
        }
    }
    pub(crate) fn is_edition(section: &ftd::p11::Section) -> bool {
        section.name.eq(EDITION)
    }

    pub(crate) fn from_p1(section: &ftd::p11::Section, doc_id: &str) -> ftd::ast::Result<Edition> {
        if !Self::is_import(section) {
            return ftd::ast::parse_error(
                format!("Section is not edition section, found `{:?}`", section),
                doc_id,
                section.line_number,
            );
        }
        if !section.sub_sections.is_empty() {
            return ftd::ast::parse_error(
                format!(
                    "Subsection not expected for edition statement `{:?}`",
                    section
                ),
                doc_id,
                section.line_number,
            );
        }
        match &section.caption {
            Some(ftd::p11::Header::KV(ftd::p11::header::KV {
                value: Some(value), ..
            })) => Ok(Edition::new(
                FTDEdition::from_string(value.as_str(), doc_id, section.line_number)?,
                section.line_number,
            )),
            t => ftd::ast::parse_error(
                format!(
                    "Expected value in caption for edition statement, found: `{:?}`",
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

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum FTDEdition {
    FTD2022,
    FTD2021,
}

impl FTDEdition {
    fn from_string(
        edition: &str,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::ast::Result<FTDEdition> {
        match edition.trim() {
            "2022" => Ok(FTDEdition::FTD2022),
            "2021" => Ok(FTDEdition::FTD2021),
            t => ftd::ast::parse_error(
                format!("Unknown edition `{}`, Help: Use 2022 or 2021", t),
                doc_id,
                line_number,
            ),
        }
    }

    pub(crate) fn is_2021(&self) -> bool {
        matches!(self, FTDEdition::FTD2021)
    }
}
