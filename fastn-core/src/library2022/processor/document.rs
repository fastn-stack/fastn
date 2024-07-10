pub fn process_readers(
    _value: ftd_ast::VariableValue,
    _kind: ftd::interpreter::Kind,
    _doc: &ftd::interpreter::TDoc,
    _req_config: &fastn_core::RequestConfig,
    _document_id: &str,
) -> ftd::interpreter::Result<ftd::interpreter::Value> {
    Err(ftd::interpreter::Error::OtherError(
        "document-readers is not implemented in this version. Switch to an \
            older version."
            .into(),
    ))
}

pub fn process_writers(
    _value: ftd_ast::VariableValue,
    _kind: ftd::interpreter::Kind,
    _doc: &ftd::interpreter::TDoc,
    _req_config: &fastn_core::RequestConfig,
    _document_id: &str,
) -> ftd::interpreter::Result<ftd::interpreter::Value> {
    Err(ftd::interpreter::Error::OtherError(
        "document-writers is not implemented in this version. Switch to an \
            older version."
            .into(),
    ))
}

pub fn current_url(
    req_config: &fastn_core::RequestConfig,
) -> ftd::interpreter::Result<ftd::interpreter::Value> {
    Ok(ftd::interpreter::Value::String {
        text: req_config.url(),
    })
}

pub fn document_id(
    _value: ftd_ast::VariableValue,
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
    _value: ftd_ast::VariableValue,
    _kind: ftd::interpreter::Kind,
    doc: &ftd::interpreter::TDoc,
    req_config: &fastn_core::RequestConfig,
) -> ftd::interpreter::Result<ftd::interpreter::Value> {
    Ok(ftd::interpreter::Value::String {
        text: fastn_core::library2022::utils::document_full_id(req_config, doc)?,
    })
}

pub fn document_suffix(
    _value: ftd_ast::VariableValue,
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
    value: ftd_ast::VariableValue,
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
        .get_file_path(&doc_id, &req_config.session_id())
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
