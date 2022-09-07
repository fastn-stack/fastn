pub(crate) fn invalid_kind_error<S>(
    message: S,
    doc_id: &str,
    line_number: usize,
) -> ftd::interpreter::Error
where
    S: Into<String>,
{
    ftd::interpreter::Error::InvalidKind {
        message: message.into(),
        doc_id: doc_id.to_string(),
        line_number,
    }
}
