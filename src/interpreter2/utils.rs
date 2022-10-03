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

pub(crate) fn kind_eq(
    key: &str,
    kind: &ftd::interpreter2::Kind,
    doc: &ftd::interpreter2::TDoc,
    line_number: usize,
) -> ftd::interpreter2::Result<bool> {
    let var_kind = ftd::ast::VariableKind::get_kind(key, doc.name, line_number)?;
    let kind_data = ftd::interpreter2::KindData::from_ast_kind(var_kind, doc, line_number)?;
    Ok(kind_data.kind.eq(kind))
}

pub const CLONE: &str = "*$";
pub const REFERENCE: &str = ftd::ast::utils::REFERENCE;

pub(crate) fn get_doc_name_and_remaining(
    s: &str,
    doc_id: &str,
    line_number: usize,
) -> ftd::interpreter2::Result<(String, Option<String>)> {
    let mut part1 = "".to_string();
    let mut pattern_to_split_at = s.to_string();
    if let Some((p1, p2)) = s.split_once('#') {
        part1 = format!("{}#", p1);
        pattern_to_split_at = p2.to_string();
    }
    Ok(if pattern_to_split_at.contains('.') {
        let (p1, p2) = ftd::interpreter2::utils::split(
            pattern_to_split_at.as_str(),
            ".",
            doc_id,
            line_number,
        )?;
        (format!("{}{}", part1, p1), Some(p2))
    } else {
        (s.to_string(), None)
    })
}

pub fn split(
    name: &str,
    split_at: &str,
    doc_id: &str,
    line_number: usize,
) -> ftd::interpreter2::Result<(String, String)> {
    if !name.contains(split_at) {
        return ftd::interpreter2::utils::e2(
            format!("{} is not found in {}", split_at, name),
            doc_id,
            line_number,
        );
    }
    let mut part = name.splitn(2, split_at);
    let part_1 = part.next().unwrap().trim();
    let part_2 = part.next().unwrap().trim();
    Ok((part_1.to_string(), part_2.to_string()))
}

pub fn split_at(text: &str, at: &str) -> (String, Option<String>) {
    if let Some((p1, p2)) = text.split_once(at) {
        (p1.trim().to_string(), Some(p2.trim().to_string()))
    } else {
        (text.to_string(), None)
    }
}

pub(crate) fn get_special_variable() -> Vec<&'static str> {
    vec![
        "MOUSE-IN",
        "SIBLING-INDEX",
        "SIBLING-INDEX-0",
        "CHILDREN-COUNT",
        "CHILDREN-COUNT-MINUS-ONE",
        "PARENT",
    ]
}

pub fn get_argument_for_reference_and_remaining<'a>(
    name: &'a str,
    doc_id: &'a str,
    component_definition_name_with_arguments: Option<(&'a str, &'a [ftd::interpreter2::Argument])>,
) -> Option<(&'a ftd::interpreter2::Argument, Option<String>)> {
    if let Some((component_name, arguments)) = component_definition_name_with_arguments {
        if let Some(referenced_argument) = name
            .strip_prefix(format!("{}.", component_name).as_str())
            .or_else(|| name.strip_prefix(format!("{}#{}.", doc_id, component_name).as_str()))
        {
            let (p1, p2) = ftd::interpreter2::utils::split_at(referenced_argument, ".");
            if let Some(argument) = arguments.iter().find(|v| v.name.eq(p1.as_str())) {
                return Some((argument, p2));
            }
        }
    }
    None
}
