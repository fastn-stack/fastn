pub fn process<'a>(
    value: ftd::ast::VariableValue,
    kind: ftd::interpreter2::Kind,
    doc: &ftd::interpreter2::TDoc<'a>,
    _config: &fpm::Config,
) -> ftd::interpreter2::Result<ftd::interpreter2::Value> {
    let (_headers, line_number) = if let Ok(val) = value.get_record(doc.name) {
        (val.2.to_owned(), val.5.to_owned())
    } else {
        (ftd::ast::HeaderValues::new(vec![]), value.line_number())
    };

    let toc_items = fpm::library::toc::ToC::parse(value.string(doc.name)?.as_str(), doc.name)
        .map_err(|e| ftd::p1::Error::ParseError {
            message: format!("Cannot parse body: {:?}", e),
            doc_id: doc.name.to_string(),
            line_number,
        })?
        .items
        .iter()
        .map(|item| item.to_toc_item_compat())
        .collect::<Vec<fpm::library::toc::TocItemCompat>>();
    doc.from_json(&toc_items, &kind, line_number)
}
