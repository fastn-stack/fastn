extern crate self as ftd_tc;

pub struct State {
    /// These are the component invocations of the main document
    ///
    /// we process every component invocation in the main document and try to resolve them.
    /// If we find a reference to another document, we load that document and process it.
    /// We do this in a recursive manner.
    pub components: Vec<ftd_ast::ComponentInvocation>,
    pub symbols: ftd_p1::Map<ftd_ast::Ast>,
    /// any type we resolve is stored here
    pub types: ftd_p1::Map<Type>,
    /// js_buffer contains the generated JS when we resolve any type
    pub js_buffer: String,
}

// struct Sourced<T> {
//     file: String,
//     line: usize,
//     value: T,
// }

pub enum Type {
    Integer,
    MutableInteger,
}

pub fn parse_document_to_ast(source: &str, doc_id: &str) -> ftd_ast::Result<Vec<ftd_ast::Ast>> {
    let sections = ftd_p1::parse(source, doc_id)?;
    let ast = ftd_ast::Ast::from_sections(sections.as_slice(), "foo")?;
    println!("{:?}", ast);

    Ok(ast)
}
