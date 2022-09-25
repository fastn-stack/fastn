pub fn split_at(text: &str, at: &str) -> (String, Option<String>) {
    if let Some((p1, p2)) = text.split_once(at) {
        (p1.trim().to_string(), Some(p2.trim().to_string()))
    } else {
        (text.to_string(), None)
    }
}

pub(crate) fn is_variable_mutable(name: &str) -> bool {
    name.starts_with(REFERENCE)
}

pub(crate) fn is_condition(value: &str, kind: &Option<String>) -> bool {
    value.eq(IF) && kind.is_none()
}

pub const REFERENCE: &str = "$";
pub const LOOP: &str = "$loop$";
pub const AS: &str = " as ";
pub const IF: &str = "if";
