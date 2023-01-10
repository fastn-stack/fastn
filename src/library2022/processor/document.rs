pub fn process_readers<'a>(
    value: ftd::ast::VariableValue,
    kind: ftd::interpreter2::Kind,
    doc: &ftd::interpreter2::TDoc<'a>,
    config: &fpm::Config,
    document_id: &str,
) -> ftd::interpreter2::Result<ftd::interpreter2::Value> {
    use itertools::Itertools;
    // TODO: document key should be optional

    let headers = match value.get_record(doc.name) {
        Ok(val) => val.2.to_owned(),
        Err(_e) => ftd::ast::HeaderValues::new(vec![]),
    };

    // sitemap document otherwise use current document
    // TODO: Possibly bug if we define different document as key in the sitemap

    dbg!(&document_id);

    let document = headers
        .get_optional_string_by_key("document", doc.name, value.line_number())?
        .unwrap_or_else(|| document_id.to_string());

    let document_name = config.document_name_with_default(document.as_str());

    let readers = match config.package.sitemap.as_ref() {
        Some(s) => s
            .readers(document_name.as_str(), &config.package.groups)
            .0
            .into_iter()
            .map(|g| g.to_group_compat())
            .collect_vec(),
        None => vec![],
    };

    doc.from_json(dbg!(&readers), &kind, value.line_number())
}

pub fn process_writers<'a>(
    value: ftd::ast::VariableValue,
    kind: ftd::interpreter2::Kind,
    doc: &ftd::interpreter2::TDoc<'a>,
    config: &fpm::Config,
    document_id: &str,
) -> ftd::interpreter2::Result<ftd::interpreter2::Value> {
    use itertools::Itertools;

    let headers = match value.get_record(doc.name) {
        Ok(val) => val.2.to_owned(),
        Err(_e) => ftd::ast::HeaderValues::new(vec![]),
    };

    // sitemap document otherwise use current document
    // TODO: Possibly bug if we define different document as key in the sitemap
    let document = headers
        .get_optional_string_by_key("document", doc.name, value.line_number())?
        .unwrap_or_else(|| document_id.to_string());

    let document_name = config.document_name_with_default(document.as_str());
    let writers = match config.package.sitemap.as_ref() {
        Some(s) => s
            .writers(document_name.as_str(), &config.package.groups)
            .into_iter()
            .map(|g| g.to_group_compat())
            .collect_vec(),
        None => vec![],
    };

    doc.from_json(&writers, &kind, value.line_number())
}
