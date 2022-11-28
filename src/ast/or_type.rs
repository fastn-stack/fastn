#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct OrType {
    pub name: String,
    pub variants: Vec<ftd::ast::Record>,
    pub line_number: usize,
}

pub const ORTYPE: &str = "or-type";

impl OrType {
    fn new(name: &str, variants: Vec<ftd::ast::Record>, line_number: usize) -> OrType {
        OrType {
            name: name.to_string(),
            variants,
            line_number,
        }
    }

    pub(crate) fn is_or_type(section: &ftd::p11::Section) -> bool {
        section.kind.as_ref().map_or(false, |s| s.eq(ORTYPE))
    }

    pub(crate) fn from_p1(section: &ftd::p11::Section, doc_id: &str) -> ftd::ast::Result<OrType> {
        if !Self::is_or_type(section) {
            return ftd::ast::parse_error(
                format!("Section is not or-type section, found `{:?}`", section),
                doc_id,
                section.line_number,
            );
        }
        let mut variants = vec![];
        for section in section.sub_sections.iter() {
            variants.push(ftd::ast::Record::from_p1(section, doc_id)?);
        }

        Ok(OrType::new(
            section.name.as_str(),
            variants,
            section.line_number,
        ))
    }

    pub fn line_number(&self) -> usize {
        self.line_number
    }
}
