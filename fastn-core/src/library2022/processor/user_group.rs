pub fn process(
    value: ftd::ast::VariableValue,
    kind: ftd::interpreter::Kind,
    doc: &ftd::interpreter::TDoc,
    req_config: &fastn_core::RequestConfig,
) -> ftd::interpreter::Result<ftd::interpreter::Value> {
    use itertools::Itertools;
    let g = req_config
        .config
        .package
        .groups
        .values()
        .map(|g| g.to_group_compat())
        .collect_vec();
    doc.from_json(&g, &kind, &value)
}

/// processor: user-group-by-id
pub fn process_by_id(
    value: ftd::ast::VariableValue,
    kind: ftd::interpreter::Kind,
    doc: &ftd::interpreter::TDoc,
    req_config: &fastn_core::RequestConfig,
) -> ftd::interpreter::Result<ftd::interpreter::Value> {
    let headers = match value.get_record(doc.name) {
        Ok(val) => val.2.to_owned(),
        Err(e) => return Err(e.into()),
    };

    let group_id = match headers.get_optional_string_by_key("id", doc.name, value.line_number())? {
        Some(k) => k,
        None => {
            return Err(ftd::interpreter::Error::ParseError {
                message: "`id` field is mandatory in `user-group-by-id` processor".to_string(),
                doc_id: doc.name.to_string(),
                line_number: value.line_number(),
            })
        }
    };

    let g = req_config
        .config
        .package
        .groups
        .get(group_id.as_str())
        .map(|g| g.to_group_compat())
        .ok_or_else(|| ftd::ftd2021::p1::Error::NotFound {
            key: format!("user-group: `{}` not found", group_id.as_str()),
            doc_id: doc.name.to_string(),
            line_number: value.line_number(),
        })?;
    doc.from_json(&g, &kind, &value)
}

/// processor: get-identities
/// This is used to get all the identities of the current document
pub fn get_identities(
    value: ftd::ast::VariableValue,
    kind: ftd::interpreter::Kind,
    doc: &ftd::interpreter::TDoc,
    req_config: &fastn_core::RequestConfig,
) -> ftd::interpreter::Result<ftd::interpreter::Value> {
    use itertools::Itertools;

    let doc_id = fastn_core::library2022::utils::document_full_id(config, doc)?;

    let identities =
        fastn_core::user_group::get_identities(req_config.config, doc_id.as_str(), true).map_err(
            |e| ftd::ftd2021::p1::Error::ParseError {
                message: e.to_string(),
                doc_id,
                line_number: value.line_number(),
            },
        )?;

    Ok(ftd::interpreter::Value::List {
        data: identities
            .into_iter()
            .map(|i| ftd::interpreter::PropertyValue::Value {
                value: ftd::interpreter::Value::String {
                    text: i.to_string(),
                },
                is_mutable: false,
                line_number: value.line_number(),
            })
            .collect_vec(),
        kind: ftd::interpreter::KindData {
            kind,
            caption: false,
            body: false,
        },
    })
}

// is user can_read the document or not based on defined readers in sitemap
pub async fn is_reader<'a>(
    value: ftd::ast::VariableValue,
    _kind: ftd::interpreter::Kind,
    doc: &ftd::interpreter::TDoc<'a>,
    req_config: &fastn_core::RequestConfig,
) -> ftd::interpreter::Result<ftd::interpreter::Value> {
    let doc_id = fastn_core::library2022::utils::document_full_id(config, doc)?;
    let is_reader = req_config.can_read(&doc_id, false).await.map_err(|e| {
        ftd::ftd2021::p1::Error::ParseError {
            message: e.to_string(),
            doc_id,
            line_number: value.line_number(),
        }
    })?;

    Ok(ftd::interpreter::Value::Boolean { value: is_reader })
}
