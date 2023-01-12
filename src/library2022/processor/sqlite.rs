pub async fn process<'a>(
    value: ftd::ast::VariableValue,
    _kind: ftd::interpreter2::Kind,
    doc: &ftd::interpreter2::TDoc<'a>,
    config: &fpm::Config,
) -> ftd::interpreter2::Result<ftd::interpreter2::Value> {
    Ok(ftd::interpreter2::Value::Boolean { value: false })
}
