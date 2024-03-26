pub fn process(
    value: ftd::ast::VariableValue,
    kind: ftd::interpreter::Kind,
    doc: &ftd::interpreter::TDoc,
) -> ftd::interpreter::Result<ftd::interpreter::Value> {
    let (body, line_number) = if let Ok(body) = value.get_processor_body(doc.name) {
        let line_number = body
            .as_ref()
            .map(|b| b.line_number)
            .unwrap_or(value.line_number());
        (body, line_number)
    } else {
        (None, value.line_number())
    };

    let toc_items = fastn_core::library::toc::ToC::parse(
        body.map(|v| v.value).unwrap_or_default().as_str(),
        doc.name,
    )
    .map_err(|e| ftd::ftd2021::p1::Error::ParseError {
        message: format!("Cannot parse body: {:?}", e),
        doc_id: doc.name.to_string(),
        line_number,
    })?
    .items
    .iter()
    .map(|item| item.to_toc_item_compat())
    .collect::<Vec<fastn_core::library::toc::TocItemCompat>>();
    doc.from_json(&toc_items, &kind, &value)
}
