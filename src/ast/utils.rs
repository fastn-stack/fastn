pub fn split_at(text: &str, at: &str) -> (String, Option<String>) {
    if let Some((p1, p2)) = text.split_once(at) {
        (p1.trim().to_string(), Some(p2.trim().to_string()))
    } else {
        (text.to_string(), None)
    }
}

pub(crate) fn get_import_alias(input: &str) -> (String, String) {
    let (module, alias) = ftd::ast::utils::split_at(input, AS);
    if let Some(alias) = alias {
        return (module, alias);
    }

    match input.rsplit_once('/') {
        Some((_, alias)) if alias.trim().is_empty() => return (module, alias.trim().to_string()),
        _ => {}
    }

    if let Some((t, _)) = module.split_once('.') {
        return (module.to_string(), t.to_string());
    }

    (module.to_string(), module)
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
pub const PROCESSOR: &str = "$processor$";
