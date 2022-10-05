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

pub(crate) fn validate_properties(
    properties: &[ftd::interpreter2::Property],
    doc_id: &str,
) -> ftd::executor::Result<()> {
    let mut found_default = None;
    for property in properties {
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
    Ok(())
}
