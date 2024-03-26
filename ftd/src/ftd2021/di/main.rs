#![allow(dead_code)]

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum DI {
    Import(ftd::ftd2021::di::Import),
    Record(ftd::ftd2021::di::Record),
    Definition(ftd::ftd2021::di::Definition),
    Invocation(ftd::ftd2021::di::Invocation),
}

impl DI {
    pub fn from_sections(
        sections: &[ftd_p1::Section],
        doc_id: &str,
    ) -> ftd::ftd2021::di::Result<Vec<DI>> {
        let mut di_vec = vec![];
        for section in sections {
            di_vec.push(DI::from_section(section, doc_id)?);
        }
        Ok(di_vec)
    }

    pub fn from_section(section: &ftd_p1::Section, doc_id: &str) -> ftd::ftd2021::di::Result<DI> {
        Ok(if ftd::ftd2021::di::Import::is_import(section) {
            DI::Import(ftd::ftd2021::di::Import::from_p1(section, doc_id)?)
        } else if ftd::ftd2021::di::Record::is_record(section) {
            DI::Record(ftd::ftd2021::di::Record::from_p1(section, doc_id)?)
        } else if ftd::ftd2021::di::Definition::is_definition(section) {
            DI::Definition(ftd::ftd2021::di::Definition::from_p1(section, doc_id)?)
        } else if ftd::ftd2021::di::Invocation::is_invocation(section) {
            DI::Invocation(ftd::ftd2021::di::Invocation::from_p1(section, doc_id)?)
        } else {
            return Err(ftd::ftd2021::di::Error::ParseError {
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
