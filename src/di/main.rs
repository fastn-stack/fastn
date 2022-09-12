#![allow(dead_code)]

#[derive(Debug, PartialEq)]
pub enum DI {
    Import(ftd::di::Import),
    Record(ftd::di::Record),
    Definition(ftd::di::Definition),
}

impl DI {
    pub fn from_sections(sections: &[ftd::p11::Section], doc_id: &str) -> ftd::di::Result<Vec<DI>> {
        let mut di_vec = vec![];
        for section in sections {
            di_vec.push(DI::from_section(section, doc_id)?);
        }
        Ok(di_vec)
    }

    pub fn from_section(section: &ftd::p11::Section, doc_id: &str) -> ftd::di::Result<DI> {
        Ok(if ftd::di::Import::is_import(section) {
            DI::Import(ftd::di::Import::from_p1(section, doc_id)?)
        } else if ftd::di::Record::is_record(section) {
            DI::Record(ftd::di::Record::from_p1(section, doc_id)?)
        } else if ftd::di::Definition::is_definition(section) {
            DI::Definition(ftd::di::Definition::from_p1(section, doc_id)?)
        } else {
            unimplemented!()
        })
    }

    #[cfg(test)]
    pub(crate) fn list(self) -> Vec<Self> {
        vec![self]
    }
}
