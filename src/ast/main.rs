#![allow(dead_code)]

#[derive(Debug, PartialEq)]
pub enum Ast {
    Import(ftd::ast::Import),
    Record(ftd::ast::Record),
}

impl Ast {
    pub fn from_p1(sections: &[ftd::p11::Section], doc_id: &str) -> ftd::ast::Result<Vec<Ast>> {
        let mut ast_vec = vec![];
        for section in sections {
            if ftd::ast::Import::is_import(section) {
                ast_vec.push(Ast::Import(ftd::ast::Import::from_p1(section, doc_id)?));
            } else if ftd::ast::Record::is_record(section) {
                ast_vec.push(Ast::Record(ftd::ast::Record::from_p1(section, doc_id)?));
            } else {
                unimplemented!()
            }
        }
        Ok(ast_vec)
    }
}
