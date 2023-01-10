pub fn process<'a>(
    value: ftd::ast::VariableValue,
    kind: ftd::interpreter2::Kind,
    doc: &ftd::interpreter2::TDoc<'a>,
    config: &fpm::Config,
) -> ftd::interpreter2::Result<ftd::interpreter2::Value> {
    // TODO: URL params not yet handled
    let req = match config.request.as_ref() {
        Some(v) => v,
        None => {
            return ftd::interpreter2::utils::e2(
                "config does not contain http-request object",
                doc.name,
                value.line_number(),
            )
        }
    };
    let mut data = req.query().clone();

    let mut named_parameters = std::collections::HashMap::new();
    for (name, param_value) in config.named_parameters.iter() {
        let json_value = param_value
            .to_serde_value()
            .ok_or(ftd::p1::Error::ParseError {
                message: format!("ftd value cannot be parsed to json: name: {}", name),
                doc_id: doc.name.to_string(),
                line_number: value.line_number(),
            })?;
        named_parameters.insert(name.to_string(), json_value);
    }

    data.extend(named_parameters);

    match req.body_as_json() {
        Ok(Some(b)) => {
            data.extend(b);
        }
        Ok(None) => {}
        Err(e) => {
            return ftd::interpreter2::utils::e2(
                format!("Error while parsing request body: {:?}", e),
                doc.name,
                value.line_number(),
            )
        }
    }

    doc.from_json(&data, &kind, value.line_number())
}
