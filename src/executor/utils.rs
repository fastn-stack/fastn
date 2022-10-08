pub fn parse_error<T, S1>(m: S1, doc_id: &str, line_number: usize) -> ftd::executor::Result<T>
where
    S1: Into<String>,
{
    Err(ftd::executor::Error::ParseError {
        message: m.into(),
        doc_id: doc_id.to_string(),
        line_number,
    })
}

pub(crate) fn validate_properties_and_set_default(
    properties: &mut Vec<ftd::interpreter2::Property>,
    argument: &ftd::interpreter2::Argument,
    doc_id: &str,
    line_number: usize,
) -> ftd::executor::Result<()> {
    let mut found_default = None;
    let expected_kind = &argument.kind.kind;
    for property in properties.iter() {
        let found_kind = property.value.kind();
        if !found_kind.is_same_as(expected_kind) {
            return ftd::executor::utils::parse_error(
                format!(
                    "Expected kind is `{:?}`, found: `{:?}`",
                    expected_kind, found_kind,
                ),
                doc_id,
                property.line_number,
            );
        }

        if found_default.is_some() && property.condition.is_none() {
            return ftd::executor::utils::parse_error(
                format!(
                    "Already found default property in line number {:?}",
                    found_default
                ),
                doc_id,
                property.line_number,
            );
        }
        if property.condition.is_none() {
            found_default = Some(property.line_number);
        }
    }
    if found_default.is_none() {
        if let Some(ref default_value) = argument.value {
            properties.push(ftd::interpreter2::Property {
                value: default_value.to_owned(),
                source: Default::default(),
                condition: None,
                line_number: argument.line_number,
            });
        } else if !expected_kind.is_optional() {
            return ftd::executor::utils::parse_error(
                format!(
                    "Need value of kind: `{:?}` for `{}`",
                    expected_kind, argument.name
                ),
                doc_id,
                line_number,
            );
        }
    }
    Ok(())
}

pub(crate) fn get_string_container(local_container: &[usize]) -> String {
    local_container
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<String>>()
        .join(",")
}
