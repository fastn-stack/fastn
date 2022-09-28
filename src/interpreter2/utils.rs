pub fn resolve_name(name: &str, doc_name: &str, aliases: &ftd::Map<String>) -> String {
    if name.contains('#') {
        return name.to_string();
    }
    match ftd::interpreter2::utils::split_module(name) {
        (Some(m), v, None) => match aliases.get(m) {
            Some(m) => format!("{}#{}", m, v),
            None => format!("{}#{}.{}", doc_name, m, v),
        },
        (Some(m), v, Some(c)) => match aliases.get(m) {
            Some(m) => format!("{}#{}.{}", m, v, c),
            None => format!("{}#{}.{}.{}", doc_name, m, v, c),
        },
        (None, v, None) => format!("{}#{}", doc_name, v),
        _ => unimplemented!(),
    }
}

pub fn split_module(id: &str) -> (Option<&str>, &str, Option<&str>) {
    match id.split_once('.') {
        Some((p1, p2)) => match p2.split_once('.') {
            Some((p21, p22)) => (Some(p1), p21, Some(p22)),
            None => (Some(p1), p2, None),
        },
        None => (None, id, None),
    }
}

pub fn e2<T, S1>(m: S1, doc_id: &str, line_number: usize) -> ftd::interpreter2::Result<T>
where
    S1: Into<String>,
{
    Err(ftd::interpreter2::Error::ParseError {
        message: m.into(),
        doc_id: doc_id.to_string(),
        line_number,
    })
}

pub(crate) fn invalid_kind_error<S>(
    message: S,
    doc_id: &str,
    line_number: usize,
) -> ftd::interpreter2::Error
where
    S: Into<String>,
{
    ftd::interpreter2::Error::InvalidKind {
        message: message.into(),
        doc_id: doc_id.to_string(),
        line_number,
    }
}

pub const REFERENCE: &str = ftd::ast::utils::REFERENCE;
