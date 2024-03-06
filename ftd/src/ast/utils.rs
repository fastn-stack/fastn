pub fn split_at(text: &str, at: &str) -> (String, Option<String>) {
    if let Some((p1, p2)) = text.split_once(at) {
        (p1.trim().to_string(), Some(p2.trim().to_string()))
    } else {
        (text.to_string(), None)
    }
}

pub fn get_import_alias(input: &str) -> (String, String) {
    let (module, alias) = ftd::ast::utils::split_at(input, AS);
    if let Some(alias) = alias {
        return (module, alias);
    }

    match input.rsplit_once('/') {
        Some((_, alias)) if !alias.trim().is_empty() => return (module, alias.trim().to_string()),
        _ => {}
    }

    if let Some((t, _)) = module.split_once('.') {
        return (module.to_string(), t.to_string());
    }

    (module.to_string(), module)
}

pub(crate) fn is_variable_mutable(name: &str) -> bool {
    name.starts_with(REFERENCE)
        && !name.eq(ftd::ast::utils::PROCESSOR)
        && !name.eq(ftd::ast::utils::LOOP)
}

pub(crate) fn is_condition(value: &str, kind: &Option<String>) -> bool {
    value.eq(IF) && kind.is_none()
}

pub(crate) fn get_js_and_fields_from_headers(
    headers: &ftd::p1::Headers,
    doc_id: &str,
) -> ftd::ast::Result<(Option<String>, Vec<ftd::ast::Argument>)> {
    let mut fields: Vec<ftd::ast::Argument> = Default::default();
    let mut js = None;
    for header in headers.0.iter() {
        if header.get_kind().is_none() && header.get_key().eq(ftd::ast::constants::JS) {
            js = Some(header.get_value(doc_id)?.ok_or(ftd::ast::Error::Parse {
                message: "js statement is blank".to_string(),
                doc_id: doc_id.to_string(),
                line_number: header.get_line_number(),
            })?);
            continue;
        }
        fields.push(ftd::ast::Argument::from_header(header, doc_id)?);
    }
    Ok((js, fields))
}

pub(crate) fn get_css_and_fields_from_headers(
    headers: &ftd::p1::Headers,
    doc_id: &str,
) -> ftd::ast::Result<(Option<String>, Vec<ftd::ast::Argument>)> {
    let mut fields: Vec<ftd::ast::Argument> = Default::default();
    let mut css = None;
    for header in headers.0.iter() {
        if header.get_kind().is_none() && header.get_key().eq(ftd::ast::constants::CSS) {
            css = Some(header.get_value(doc_id)?.ok_or(ftd::ast::Error::Parse {
                message: "css statement is blank".to_string(),
                doc_id: doc_id.to_string(),
                line_number: header.get_line_number(),
            })?);
            continue;
        }
        fields.push(ftd::ast::Argument::from_header(header, doc_id)?);
    }
    Ok((css, fields))
}

pub(crate) fn is_header_key(key: &str) -> bool {
    key.starts_with(HEADER_KEY_START) && key.ends_with('$')
}

pub(crate) fn get_component_id(
    headers: &ftd::p1::Headers,
    doc_id: &str,
) -> ftd::p1::Result<Option<String>> {
    match headers.0.iter().find(|header| header.get_key().eq("id")) {
        Some(id) => id.get_value(doc_id),
        None => Ok(None),
    }
}

pub const REFERENCE: &str = "$";
pub const CLONE: &str = "*$";
pub const LOOP: &str = "$loop$";
pub const AS: &str = " as ";
pub const IN: &str = " in ";
pub const IF: &str = "if";
pub const FOR: &str = "for";
pub const PROCESSOR: &str = "$processor$";
pub const HEADER_KEY_START: &str = "$header-";
