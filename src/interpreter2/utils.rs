pub fn resolve_name(name: &str, doc_name: &str, aliases: &ftd::Map<String>) -> String {
    let name = name
        .trim_start_matches(ftd::interpreter2::utils::CLONE)
        .trim_start_matches(ftd::interpreter2::utils::REFERENCE)
        .to_string();

    if name.contains('#') {
        return name;
    }
    match ftd::interpreter2::utils::split_module(name.as_str()) {
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
    let kind_data = ftd::interpreter2::KindData::from_ast_kind(
        var_kind,
        &Default::default(),
        doc,
        line_number,
    )?;
    Ok(kind_data.kind.is_same_as(kind))
}

pub const CLONE: &str = "*$";
pub const REFERENCE: &str = ftd::ast::utils::REFERENCE;

pub(crate) fn get_function_name(
    s: &str,
    doc_id: &str,
    line_number: usize,
) -> ftd::interpreter2::Result<String> {
    Ok(get_function_name_and_properties(s, doc_id, line_number)?.0)
}

pub(crate) fn get_function_name_and_properties(
    s: &str,
    doc_id: &str,
    line_number: usize,
) -> ftd::interpreter2::Result<(String, Vec<(String, String)>)> {
    let (si, ei) = match (s.find('('), s.find(')')) {
        (Some(si), Some(ei)) if si < ei => (si, ei),
        _ => {
            return ftd::interpreter2::utils::e2(
                format!("{} is not a function", s),
                doc_id,
                line_number,
            )
        }
    };
    let function_name = s[..si].to_string();
    let mut properties = vec![];
    for value in s[si + 1..ei].split(',') {
        let (p1, p2) = ftd::interpreter2::utils::split(value, "=", doc_id, line_number)?;
        properties.push((p1.trim().to_string(), p2.trim().to_string()));
    }

    Ok((function_name, properties))
}

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
    loop_object_name_and_kind: &'a Option<(String, ftd::interpreter2::Argument)>,
) -> Option<(
    &'a ftd::interpreter2::Argument,
    Option<String>,
    ftd::interpreter2::PropertyValueSource,
)> {
    if let Some((component_name, arguments)) = component_definition_name_with_arguments {
        if let Some(referenced_argument) = name
            .strip_prefix(format!("{}.", component_name).as_str())
            .or_else(|| name.strip_prefix(format!("{}#{}.", doc_id, component_name).as_str()))
        {
            let (p1, p2) = ftd::interpreter2::utils::split_at(referenced_argument, ".");
            if let Some(argument) = arguments.iter().find(|v| v.name.eq(p1.as_str())) {
                return Some((
                    argument,
                    p2,
                    ftd::interpreter2::PropertyValueSource::Local(component_name.to_string()),
                ));
            }
        }
    }
    if let Some((loop_name, loop_argument)) = loop_object_name_and_kind {
        if name.starts_with(format!("{}.", loop_name).as_str())
            || name.starts_with(format!("{}#{}.", doc_id, loop_name).as_str())
            || name.eq(loop_name)
            || name.eq(format!("{}#{}", doc_id, loop_name).as_str())
        {
            let p2 = ftd::interpreter2::utils::split_at(name, ".").1;
            return Some((
                loop_argument,
                p2,
                ftd::interpreter2::PropertyValueSource::Loop(loop_name.to_string()),
            ));
        }
    }

    None
}

pub fn validate_variable(
    variable: &ftd::interpreter2::Variable,
    doc: &ftd::interpreter2::TDoc,
) -> ftd::interpreter2::Result<()> {
    if !variable.mutable {
        return Ok(());
    }
    if !variable.conditional_value.is_empty() {
        return ftd::interpreter2::utils::e2(
            format!(
                "conditional properties are not supported for mutable argument `{}`",
                variable.name,
            ),
            doc.name,
            variable.line_number,
        );
    }

    validate_property_value_for_mutable(&variable.value, doc)
}

pub fn validate_property_value_for_mutable(
    value: &ftd::interpreter2::PropertyValue,
    doc: &ftd::interpreter2::TDoc,
) -> ftd::interpreter2::Result<()> {
    if let Some(name) = value.reference_name() {
        if let Ok(ref_variable) = doc.get_variable(name, value.line_number()) {
            if !ref_variable.mutable {
                return ftd::interpreter2::utils::e2(
                    format!(
                        "Cannot pass immutable reference `{}` to mutable",
                        ref_variable.name
                    ),
                    doc.name,
                    value.line_number(),
                );
            }
        }
    } else if let Some(function_call) = value.get_function() {
        validate_function_call(function_call, doc)?;
    }

    return Ok(());

    fn validate_function_call(
        function_call: &ftd::interpreter2::FunctionCall,
        doc: &ftd::interpreter2::TDoc,
    ) -> ftd::interpreter2::Result<()> {
        for (key, value) in function_call.values.iter() {
            if let Some(ref_name) = value.reference_name() {
                return ftd::interpreter2::utils::e2(
                    format!(
                        "Cannot pass reference `{}`:`{}` to mutable: Hint: Use *${} instead.",
                        key, ref_name, ref_name
                    ),
                    doc.name,
                    value.line_number(),
                );
            } else if let Some(function_call) = value.get_function() {
                validate_function_call(function_call, doc)?;
            }
        }

        Ok(())
    }
}
