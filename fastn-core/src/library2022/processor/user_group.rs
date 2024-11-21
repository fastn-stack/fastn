pub fn process(
    _value: ftd_ast::VariableValue,
    _kind: fastn_resolved::Kind,
    _doc: &ftd::interpreter::TDoc,
    _req_config: &fastn_core::RequestConfig,
) -> ftd::interpreter::Result<fastn_resolved::Value> {
    Err(ftd::interpreter::Error::OtherError(
        "user-groups is not implemented in this version. Switch to an older  \
            version."
            .into(),
    ))
}

/// processor: user-group-by-id
pub fn process_by_id(
    _value: ftd_ast::VariableValue,
    _kind: fastn_resolved::Kind,
    _doc: &ftd::interpreter::TDoc,
    _req_config: &fastn_core::RequestConfig,
) -> ftd::interpreter::Result<fastn_resolved::Value> {
    Err(ftd::interpreter::Error::OtherError(
        "user-group-by-id is not implemented in this version. Switch to an \
            older version."
            .into(),
    ))
}

/// processor: get-identities
/// This is used to get all the identities of the current document
pub async fn get_identities(
    _value: ftd_ast::VariableValue,
    _kind: fastn_resolved::Kind,
    _doc: &ftd::interpreter::TDoc<'_>,
    _req_config: &fastn_core::RequestConfig,
) -> ftd::interpreter::Result<fastn_resolved::Value> {
    Err(ftd::interpreter::Error::OtherError(
        "get-identities is not implemented in this version. Switch to an \
            older version."
            .into(),
    ))
}

// is user can_read the document or not based on defined readers in sitemap
pub async fn is_reader<'a>(
    _value: ftd_ast::VariableValue,
    _kind: fastn_resolved::Kind,
    _doc: &ftd::interpreter::TDoc<'a>,
    _req_config: &fastn_core::RequestConfig,
) -> ftd::interpreter::Result<fastn_resolved::Value> {
    Err(ftd::interpreter::Error::OtherError(
        "is-reader is not implemented in this version. Switch to an older \
            version."
            .into(),
    ))
}
