pub async fn process(
    value: ftd_ast::VariableValue,
    kind: fastn_resolved::Kind,
    doc: &ftd::interpreter::TDoc<'_>,
    req_config: &mut fastn_core::RequestConfig,
) -> ftd::interpreter::Result<fastn_resolved::Value> {
    // req_config.current_language() is the two letter language code for current request
    // should be deserialized into `optional string`.
    doc.from_json(&req_config.current_language(), &kind, &value)
}
