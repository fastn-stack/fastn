pub fn process<'a>(
    value: ftd::ast::VariableValue,
    kind: ftd::interpreter2::Kind,
    doc: &ftd::interpreter2::TDoc<'a>,
    _config: &fastn_core::Config,
) -> ftd::interpreter2::Result<ftd::interpreter2::Value> {
    let (body, line_number) = if let Ok(val) = value.get_record(doc.name) {
        (val.3.to_owned(), val.5.to_owned())
    } else {
        (None, value.line_number())
    };

    let toc_items = fastn_core::library::toc::ToC::parse(
        body.map(|v| v.value).unwrap_or_default().as_str(),
        doc.name,
    )
    .map_err(|e| ftd::p1::Error::ParseError {
        message: format!("Cannot parse body: {:?}", e),
        doc_id: doc.name.to_string(),
        line_number,
    })?
    .items
    .iter()
    .map(|item| item.to_toc_item_compat())
    .collect::<Vec<fastn_core::library::toc::TocItemCompat>>();
    doc.from_json(&toc_items, &kind, line_number)
}
