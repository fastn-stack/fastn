pub fn process_readers(
    value: ftd::ast::VariableValue,
    kind: ftd::interpreter::Kind,
    doc: &ftd::interpreter::TDoc,
    req_config: &fastn_core::RequestConfig,
    document_id: &str,
) -> ftd::interpreter::Result<ftd::interpreter::Value> {
    use itertools::Itertools;
    // TODO: document key should be optional

    let headers = match value.get_record(doc.name) {
        Ok(val) => val.2.to_owned(),
        Err(_e) => ftd::ast::HeaderValues::new(vec![]),
    };

    let document = headers
        .get_optional_string_by_key("document", doc.name, value.line_number())?
        .unwrap_or_else(|| document_id.to_string());

    let document_name = req_config.document_name_with_default(document.as_str());

    let readers = match req_config.config.package.sitemap.as_ref() {
        Some(s) => s
            .readers(document_name.as_str(), &req_config.config.package.groups)
            .0
            .into_iter()
            .map(|g| g.to_group_compat())
            .collect_vec(),
        None => vec![],
    };

    doc.from_json(dbg!(&readers), &kind, &value)
}

pub fn process_writers(
    value: ftd::ast::VariableValue,
    kind: ftd::interpreter::Kind,
    doc: &ftd::interpreter::TDoc,
    req_config: &fastn_core::RequestConfig,
    document_id: &str,
) -> ftd::interpreter::Result<ftd::interpreter::Value> {
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

    let document_name = req_config.document_name_with_default(document.as_str());
    let writers = match req_config.config.package.sitemap.as_ref() {
        Some(s) => s
            .writers(document_name.as_str(), &req_config.config.package.groups)
            .into_iter()
            .map(|g| g.to_group_compat())
            .collect_vec(),
        None => vec![],
    };

    doc.from_json(&writers, &kind, &value)
}

pub fn document_id(
    _value: ftd::ast::VariableValue,
    _kind: ftd::interpreter::Kind,
    doc: &ftd::interpreter::TDoc,
    req_config: &fastn_core::RequestConfig,
) -> ftd::interpreter::Result<ftd::interpreter::Value> {
    let doc_id = req_config.doc_id().unwrap_or_else(|| {
        doc.name
            .to_string()
            .replace(req_config.config.package.name.as_str(), "")
    });

    let document_id = doc_id
        .split_once("/-/")
        .map(|x| x.0)
        .unwrap_or_else(|| &doc_id)
        .trim_matches('/');

    if document_id.is_empty() {
        return Ok(ftd::interpreter::Value::String {
            text: "/".to_string(),
        });
    }

    Ok(ftd::interpreter::Value::String {
        text: format!("/{}/", document_id),
    })
}

pub fn document_full_id(
    _value: ftd::ast::VariableValue,
    _kind: ftd::interpreter::Kind,
    doc: &ftd::interpreter::TDoc,
    req_config: &fastn_core::RequestConfig,
) -> ftd::interpreter::Result<ftd::interpreter::Value> {
    Ok(ftd::interpreter::Value::String {
        text: fastn_core::library2022::utils::document_full_id(req_config, doc)?,
    })
}

pub fn document_suffix(
    _value: ftd::ast::VariableValue,
    kind: ftd::interpreter::Kind,
    doc: &ftd::interpreter::TDoc,
    req_config: &fastn_core::RequestConfig,
) -> ftd::interpreter::Result<ftd::interpreter::Value> {
    let doc_id = req_config.doc_id().unwrap_or_else(|| {
        doc.name
            .to_string()
            .replace(req_config.config.package.name.as_str(), "")
    });

    let value = doc_id
        .split_once("/-/")
        .map(|(_, y)| y.trim().to_string())
        .map(|suffix| ftd::interpreter::Value::String { text: suffix });

    Ok(ftd::interpreter::Value::Optional {
        data: Box::new(value),
        kind: ftd::interpreter::KindData {
            kind,
            caption: false,
            body: false,
        },
    })
}

pub async fn document_name<'a>(
    value: ftd::ast::VariableValue,
    _kind: ftd::interpreter::Kind,
    doc: &ftd::interpreter::TDoc<'a>,
    req_config: &fastn_core::RequestConfig,
) -> ftd::interpreter::Result<ftd::interpreter::Value> {
    let doc_id = req_config.doc_id().unwrap_or_else(|| {
        doc.name
            .to_string()
            .replace(req_config.config.package.name.as_str(), "")
    });

    let file_path = req_config
        .config
        .get_file_path(&doc_id)
        .await
        .map_err(|e| ftd::ftd2021::p1::Error::ParseError {
            message: e.to_string(),
            doc_id: doc.name.to_string(),
            line_number: value.line_number(),
        })?;

    Ok(ftd::interpreter::Value::String {
        text: file_path.trim().to_string(),
    })
}
