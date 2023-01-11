pub(crate) fn remove_value_comment(value: &mut Option<String>) {
    if let Some(v) = value {
        if v.starts_with('/') {
            *value = None;
            return;
        }

        if v.starts_with(r"\/") {
            *v = v.trim_start_matches('\\').to_string();
        }
    }
}

pub const CAPTION: &str = "$caption$";
pub const INLINE_IF: &str = " if ";
pub const IF: &str = "if";

pub fn parse_error<T, S1>(m: S1, doc_id: &str, line_number: usize) -> ftd::p11::Result<T>
where
    S1: Into<String>,
{
    Err(ftd::p11::Error::ParseError {
        message: m.into(),
        doc_id: doc_id.to_string(),
        line_number,
    })
}

pub(crate) fn i32_to_usize(i: i32) -> usize {
    if i < 0 {
        0
    } else {
        i as usize
    }
}
