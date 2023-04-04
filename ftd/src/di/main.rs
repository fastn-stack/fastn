#![allow(dead_code)]

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum DI {
    Import(ftd::di::Import),
    Record(ftd::di::Record),
    Definition(ftd::di::Definition),
    Invocation(ftd::di::Invocation),
}

impl DI {
    pub fn from_sections(sections: &[ftd::p1::Section], doc_id: &str) -> ftd::di::Result<Vec<DI>> {
        let mut di_vec = vec![];
        for section in sections {
            di_vec.push(DI::from_section(section, doc_id)?);
        }
        Ok(di_vec)
    }

    pub fn from_section(section: &ftd::p1::Section, doc_id: &str) -> ftd::di::Result<DI> {
        Ok(if ftd::di::Import::is_import(section) {
            DI::Import(ftd::di::Import::from_p1(section, doc_id)?)
        } else if ftd::di::Record::is_record(section) {
            DI::Record(ftd::di::Record::from_p1(section, doc_id)?)
        } else if ftd::di::Definition::is_definition(section) {
            DI::Definition(ftd::di::Definition::from_p1(section, doc_id)?)
        } else if ftd::di::Invocation::is_invocation(section) {
            DI::Invocation(ftd::di::Invocation::from_p1(section, doc_id)?)
        } else {
            return Err(ftd::di::Error::ParseError {
                message: format!("Invalid DI, found: `{:?}`", section),
                doc_id: doc_id.to_string(),
                line_number: section.line_number,
            });
        })
    }

    #[cfg(test)]
    pub(crate) fn list(self) -> Vec<Self> {
        vec![self]
    }
}
