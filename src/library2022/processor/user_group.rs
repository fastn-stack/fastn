pub fn process<'a>(
    value: ftd::ast::VariableValue,
    kind: ftd::interpreter2::Kind,
    doc: &ftd::interpreter2::TDoc<'a>,
    config: &fpm::Config,
) -> ftd::interpreter2::Result<ftd::interpreter2::Value> {
    use itertools::Itertools;
    let g = config
        .package
        .groups
        .iter()
        .map(|(_, g)| g.to_group_compat())
        .collect_vec();
    doc.from_json(&g, &kind, value.line_number())
}

pub fn process_by_id<'a>(
    value: ftd::ast::VariableValue,
    kind: ftd::interpreter2::Kind,
    doc: &ftd::interpreter2::TDoc<'a>,
    config: &fpm::Config,
) -> ftd::interpreter2::Result<ftd::interpreter2::Value> {
    let headers = match value.get_record(doc.name) {
        Ok(val) => val.2.to_owned(),
        Err(e) => return Err(e.into()),
    };

    let group_id = match headers.get_optional_string_by_key("id", doc.name, value.line_number())? {
        Some(k) => k,
        None => {
            return Err(ftd::interpreter2::Error::ParseError {
                message: "`id` field is mandatory in `user-group-by-id` processor".to_string(),
                doc_id: doc.name.to_string(),
                line_number: value.line_number(),
            })
        }
    };

    let g = config
        .package
        .groups
        .get(group_id.as_str())
        .map(|g| g.to_group_compat())
        .ok_or_else(|| ftd::p1::Error::NotFound {
            key: format!("user-group: `{}` not found", group_id.as_str()),
            doc_id: doc.name.to_string(),
            line_number: value.line_number(),
        })?;
    doc.from_json(&g, &kind, value.line_number())
}
