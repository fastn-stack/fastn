pub fn parse_document_to_ast(
    source: &str,
    doc_id: &ftd_tc::DocumentID,
) -> ftd_ast::Result<Vec<ftd_ast::Ast>> {
    let sections = ftd_p1::parse(source, doc_id.logical.as_str())?;
    let ast = ftd_ast::Ast::from_sections(sections.as_slice(), doc_id.logical.as_str())?;
    println!("{:?}", ast);

    Ok(ast)
}
