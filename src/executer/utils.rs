pub fn string_optional(
    name: &str,
    properties: &ftd::Map<ftd::Value>,
    doc_id: &str,
    line_number: usize,
) -> ftd::p1::Result<Option<String>> {
    match properties.get(name) {
        Some(ftd::Value::String { text: v, .. }) => Ok(Some(v.to_string())),
        Some(ftd::Value::None {
            kind: ftd::p2::Kind::String { .. },
        }) => Ok(None),
        Some(ftd::Value::None { .. }) => Ok(None),
        Some(ftd::Value::Optional {
            data,
            kind: ftd::p2::Kind::String { .. },
        }) => match data.as_ref() {
            Some(ftd::Value::String { text: v, .. }) => Ok(Some(v.to_string())),
            None => Ok(None),
            v => ftd::p2::utils::e2(
                format!("expected string, for: `{}` found: {:?}", name, v),
                doc_id,
                line_number,
            ),
        },
        Some(v) => ftd::p2::utils::e2(
            format!("expected string, for: `{}` found: {:?}", name, v),
            doc_id,
            line_number,
        ),
        None => Ok(None),
    }
}
