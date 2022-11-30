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
