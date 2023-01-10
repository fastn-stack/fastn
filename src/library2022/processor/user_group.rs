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

/// processor: user-group-by-id
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

/// processor: get-identities
/// This is used to get all the identities of the current document
pub fn get_identities<'a>(
    value: ftd::ast::VariableValue,
    _kind: ftd::interpreter2::Kind,
    doc: &ftd::interpreter2::TDoc<'a>,
    config: &fpm::Config,
) -> ftd::interpreter2::Result<ftd::interpreter2::Value> {
    use itertools::Itertools;

    let doc_id = fpm::library2022::utils::document_full_id(config, doc)?;

    let identities =
        fpm::user_group::get_identities(config, doc_id.as_str(), true).map_err(|e| {
            ftd::p1::Error::ParseError {
                message: e.to_string(),
                doc_id,
                line_number: value.line_number(),
            }
        })?;

    Ok(ftd::interpreter2::Value::List {
        data: identities
            .into_iter()
            .map(|i| ftd::interpreter2::PropertyValue::Value {
                value: ftd::interpreter2::Value::String {
                    text: i.to_string(),
                },
                is_mutable: false,
                line_number: value.line_number(),
            })
            .collect_vec(),
        kind: ftd::interpreter2::KindData {
            kind: ftd::interpreter2::Kind::String,
            caption: false,
            body: false,
        },
    })
}
