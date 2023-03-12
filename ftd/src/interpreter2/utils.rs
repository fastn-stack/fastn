use itertools::Itertools;

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
    doc: &mut ftd::interpreter2::TDoc,
    line_number: usize,
) -> ftd::interpreter2::Result<ftd::interpreter2::StateWithThing<bool>> {
    let var_kind = ftd::ast::VariableKind::get_kind(key, doc.name, line_number)?;
    let kind_data = try_ok_state!(ftd::interpreter2::KindData::from_ast_kind(
        var_kind,
        &Default::default(),
        doc,
        line_number,
    )?);
    Ok(ftd::interpreter2::StateWithThing::new_thing(
        kind_data.kind.is_same_as(kind),
    ))
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
    if !s[si + 1..ei].trim().is_empty() {
        for value in s[si + 1..ei].split(',') {
            let (p1, p2) = ftd::interpreter2::utils::split(value, "=", doc_id, line_number)?;
            properties.push((p1.trim().to_string(), p2.trim().to_string()));
        }
    }

    Ok((function_name, properties))
}

pub(crate) fn get_doc_name_and_remaining(
    s: &str,
    doc_id: &str,
    line_number: usize,
) -> (String, Option<String>) {
    let mut part1 = "".to_string();
    let mut pattern_to_split_at = s.to_string();
    if let Some((p1, p2)) = s.split_once('#') {
        part1 = format!("{}#", p1);
        pattern_to_split_at = p2.to_string();
    }
    if pattern_to_split_at.contains('.') {
        let (p1, p2) =
            ftd::interpreter2::utils::split(pattern_to_split_at.as_str(), ".", doc_id, line_number)
                .unwrap();
        (format!("{}{}", part1, p1), Some(p2))
    } else {
        (s.to_string(), None)
    }
}

pub fn get_doc_name_and_thing_name_and_remaining(
    s: &str,
    doc_id: &str,
    line_number: usize,
) -> (String, String, Option<String>) {
    let (doc_name, remaining) = get_doc_name_and_remaining(s, doc_id, line_number);
    if let Some((doc_name, thing_name)) = doc_name.split_once('#') {
        (doc_name.to_string(), thing_name.to_string(), remaining)
    } else {
        (doc_id.to_string(), doc_name, remaining)
    }
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

pub fn is_argument_in_component_or_loop<'a>(
    name: &'a str,
    doc: &'a ftd::interpreter2::TDoc,
    component_definition_name_with_arguments: Option<(&'a str, &'a [String])>,
    loop_object_name_and_kind: &'a Option<String>,
) -> bool {
    if let Some((component_name, arguments)) = component_definition_name_with_arguments {
        if let Some(referenced_argument) = name
            .strip_prefix(format!("{}.", component_name).as_str())
            .or_else(|| name.strip_prefix(format!("{}#{}.", doc.name, component_name).as_str()))
        {
            let (p1, _p2) = ftd::interpreter2::utils::split_at(referenced_argument, ".");
            if arguments.iter().contains(&p1) {
                return true;
            }
        }
    }
    if let Some(loop_name) = loop_object_name_and_kind {
        let name = doc.resolve_name(name);
        if name.starts_with(format!("{}.", loop_name).as_str())
            || name.starts_with(format!("{}#{}.", doc.name, loop_name).as_str())
            || name.eq(loop_name)
            || name.eq(format!("{}#{}", doc.name, loop_name).as_str())
        {
            return true;
        }
    }

    false
}

pub fn get_mut_argument_for_reference<'a>(
    name: &str,
    doc_name: &str,
    component_definition_name_with_arguments: &'a mut Option<(
        &str,
        &mut [ftd::interpreter2::Argument],
    )>,
    line_number: usize,
) -> ftd::interpreter2::Result<Option<&'a mut ftd::interpreter2::Argument>> {
    if let Some((component_name, arguments)) = component_definition_name_with_arguments {
        if let Some(referenced_argument) = name
            .strip_prefix(format!("{}.", component_name).as_str())
            .or_else(|| name.strip_prefix(format!("{}#{}.", doc_name, component_name).as_str()))
        {
            let (p1, _) = ftd::interpreter2::utils::split_at(referenced_argument, ".");
            return if let Some(argument) = arguments.iter_mut().find(|v| v.name.eq(p1.as_str())) {
                Ok(Some(argument))
            } else {
                ftd::interpreter2::utils::e2(
                    format!("{} is not the argument in {}", p1, component_name),
                    doc_name,
                    line_number,
                )
            };
        }
    }
    Ok(None)
}

pub fn get_argument_for_reference_and_remaining(
    name: &str,
    doc: &ftd::interpreter2::TDoc,
    component_definition_name_with_arguments: &Option<(&str, &mut [ftd::interpreter2::Argument])>,
    loop_object_name_and_kind: &Option<(String, ftd::interpreter2::Argument)>,
    line_number: usize,
) -> ftd::interpreter2::Result<
    Option<(
        ftd::interpreter2::Argument,
        Option<String>,
        ftd::interpreter2::PropertyValueSource,
    )>,
> {
    if let Some((component_name, arguments)) = component_definition_name_with_arguments {
        if let Some(referenced_argument) = name
            .strip_prefix(format!("{}.", component_name).as_str())
            .or_else(|| name.strip_prefix(format!("{}#{}.", doc.name, component_name).as_str()))
        {
            let (p1, p2) = ftd::interpreter2::utils::split_at(referenced_argument, ".");
            return if let Some(argument) = arguments.iter().find(|v| v.name.eq(p1.as_str())) {
                Ok(Some((
                    argument.to_owned(),
                    p2,
                    ftd::interpreter2::PropertyValueSource::Local(component_name.to_string()),
                )))
            } else {
                ftd::interpreter2::utils::e2(
                    format!("{} is not the argument in {}", p1, component_name),
                    doc.name,
                    line_number,
                )
            };
        }
    }
    if let Some((loop_name, loop_argument)) = loop_object_name_and_kind {
        let p2 = ftd::interpreter2::utils::split_at(name, ".").1;
        let name = doc.resolve_name(name);
        if name.starts_with(format!("{}.", loop_name).as_str())
            || name.starts_with(format!("{}#{}.", doc.name, loop_name).as_str())
            || name.eq(loop_name)
            || name.eq(format!("{}#{}", doc.name, loop_name).as_str())
        {
            return Ok(Some((
                loop_argument.to_owned(),
                p2,
                ftd::interpreter2::PropertyValueSource::Loop(loop_name.to_string()),
            )));
        }
        if name
            .starts_with(format!("{}#{}", doc.name, ftd::interpreter2::FTD_LOOP_COUNTER).as_str())
        {
            return Ok(Some((
                ftd::interpreter2::Field::default(
                    ftd::interpreter2::FTD_LOOP_COUNTER,
                    ftd::interpreter2::Kind::integer()
                        .into_optional()
                        .into_kind_data(),
                ),
                None,
                ftd::interpreter2::PropertyValueSource::Loop(loop_name.to_string()),
            )));
        }
    }

    Ok(None)
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

    validate_record_value(&variable.value, doc)?;
    validate_property_value_for_mutable(&variable.value, doc)
}

pub fn validate_record_value(
    value: &ftd::interpreter2::PropertyValue,
    doc: &ftd::interpreter2::TDoc,
) -> ftd::interpreter2::Result<()> {
    if let ftd::interpreter2::PropertyValue::Value { value, .. } = value {
        if let Some(ftd::interpreter2::Value::Record { fields, .. }) = value.ref_inner() {
            validate_fields(fields.values().collect(), doc)?;
        }
    }
    return Ok(());

    fn validate_fields(
        fields: Vec<&ftd::interpreter2::PropertyValue>,
        doc: &ftd::interpreter2::TDoc,
    ) -> ftd::interpreter2::Result<()> {
        for value in fields.iter() {
            if let Some(reference_name) = value.reference_name() {
                return ftd::interpreter2::utils::e2(format!(
                    "Currently, reference `{}` to record field  is not supported. Use clone (*) instead", reference_name
                ), doc.name, value.line_number());
            }

            if let ftd::interpreter2::PropertyValue::Value { value, .. } = value {
                match value.ref_inner() {
                    Some(ftd::interpreter2::Value::Record { fields, .. }) => {
                        validate_fields(fields.values().collect(), doc)?;
                    }
                    Some(ftd::interpreter2::Value::OrType { value, .. }) => {
                        validate_fields(vec![value], doc)?;
                    }
                    Some(ftd::interpreter2::Value::List { data, .. }) => {
                        validate_fields(data.iter().collect(), doc)?;
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }
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

pub(crate) fn get_value(
    doc: &ftd::interpreter2::TDoc,
    value: &ftd::interpreter2::Value,
) -> ftd::interpreter2::Result<Option<serde_json::Value>> {
    if let ftd::interpreter2::Value::List { data, .. } = value {
        let mut list_data = vec![];
        for val in data {
            let value = match val {
                ftd::interpreter2::PropertyValue::Value { value, .. } => value,
                _ => continue, //todo
            };
            if let Some(val) = get_value(doc, value)? {
                list_data.push(val);
            }
        }
        return Ok(serde_json::to_value(&list_data).ok());
    }
    let value = value.inner();

    Ok(match value {
        None => Some(serde_json::Value::Null),
        Some(ftd::interpreter2::Value::Boolean { value }) => serde_json::to_value(value).ok(),
        Some(ftd::interpreter2::Value::Integer { value }) => serde_json::to_value(value).ok(),
        Some(ftd::interpreter2::Value::String { text: value, .. }) => {
            serde_json::to_value(value).ok()
        }
        Some(ftd::interpreter2::Value::Decimal { value, .. }) => serde_json::to_value(value).ok(),
        Some(ftd::interpreter2::Value::Record { fields, .. }) => {
            let mut value_fields = ftd::Map::new();
            for (k, v) in fields {
                if let Some(value) = get_value(doc, &v.clone().resolve(doc, v.line_number())?)? {
                    value_fields.insert(k, value);
                }
            }
            serde_json::to_value(value_fields).ok()
        }
        Some(ftd::interpreter2::Value::OrType {
            value,
            variant,
            full_variant,
            name,
            ..
        }) => {
            let value = get_value(doc, &value.clone().resolve(doc, value.line_number())?)?;
            match value {
                Some(value) if name.eq(ftd::interpreter2::FTD_LENGTH) => {
                    if let Ok(pattern) = ftd::executor::Length::set_value_from_variant(
                        variant.as_str(),
                        value.to_string().as_str(),
                        doc.name,
                        0,
                    ) {
                        serde_json::to_value(pattern).ok()
                    } else {
                        Some(value)
                    }
                }
                Some(value) if name.eq(ftd::interpreter2::FTD_FONT_SIZE) => {
                    if let Ok(pattern) = ftd::executor::FontSize::set_value_from_variant(
                        variant.as_str(),
                        value.to_string().as_str(),
                        doc.name,
                        0,
                    ) {
                        serde_json::to_value(pattern).ok()
                    } else {
                        Some(value)
                    }
                }
                Some(value)
                    if name.eq(ftd::interpreter2::FTD_RESIZING_FIXED)
                        && variant.ne(ftd::interpreter2::FTD_RESIZING_FIXED) =>
                {
                    if let Ok(pattern) = ftd::executor::Resizing::set_value_from_variant(
                        variant.as_str(),
                        full_variant.as_str(),
                        doc.name,
                        value.to_string().as_str(),
                        0,
                    ) {
                        serde_json::to_value(pattern).ok()
                    } else {
                        Some(value)
                    }
                }
                Some(value) => Some(value),
                None => None,
            }
        }
        _ => None,
    })
}

pub(crate) fn js_reference_name(s: &str) -> String {
    let mut s = s.replace("\\\\", "/").replace('\\', "/");
    if s.contains("LOOP.COUNTER") {
        s = "LOOP__COUNTER".to_string();
    }
    s
}

pub(crate) fn find_inherited_variables(
    reference_or_clone: &str,
    inherited_variables: &ftd::VecMap<(String, Vec<usize>)>,
    local_container: Option<&[usize]>,
) -> Option<String> {
    if !reference_or_clone.starts_with(ftd::interpreter2::FTD_INHERITED) {
        return None;
    }
    let values = if reference_or_clone.starts_with(ftd::interpreter2::FTD_INHERITED) {
        let reference_or_clone = reference_or_clone
            .trim_start_matches(format!("{}.", ftd::interpreter2::FTD_INHERITED).as_str());
        inherited_variables.get_value_and_rem(reference_or_clone)
    } else {
        vec![]
    };

    if local_container.is_none() {
        if let Some(((reference, _), rem)) = values.last() {
            return Some(if let Some(rem) = rem {
                format!("{}.{}", reference, rem)
            } else {
                reference.to_string()
            });
        }
    }

    if let Some(local_container) = local_container {
        for ((reference, container), rem) in values.iter() {
            if !container.is_empty()
                && container.len() == local_container.len()
                && container[container.len()] != local_container[container.len()]
            {
                continue;
            }

            for (idx, i) in container.iter().enumerate() {
                if *i != local_container[idx] {
                    break;
                }
            }

            return Some(if let Some(rem) = rem {
                format!("{}.{}", reference, rem)
            } else {
                reference.to_string()
            });
        }
    }

    if values.is_empty()
        && (reference_or_clone
            .starts_with(format!("{}.types", ftd::interpreter2::FTD_INHERITED).as_str())
            || reference_or_clone
                .starts_with(format!("{}.colors", ftd::interpreter2::FTD_INHERITED).as_str()))
    {
        return Some(format!(
            "ftd#default-{}{}",
            if reference_or_clone
                .starts_with(format!("{}.types", ftd::interpreter2::FTD_INHERITED).as_str())
            {
                "types"
            } else {
                "colors"
            },
            reference_or_clone
                .trim_start_matches(format!("{}.types", ftd::interpreter2::FTD_INHERITED).as_str())
                .trim_start_matches(
                    format!("{}.colors", ftd::interpreter2::FTD_INHERITED).as_str()
                )
        ));
    }

    None
}

pub(crate) fn insert_module_thing(
    kind: &ftd::interpreter2::KindData,
    reference: &str,
    reference_full_name: &str,
    definition_name_with_arguments: &mut Option<(&str, &mut [ftd::interpreter2::Argument])>,
    line_number: usize,
    doc_name: &str,
) -> ftd::interpreter2::Result<()> {
    let arg = get_mut_argument_for_reference(
        reference,
        doc_name,
        definition_name_with_arguments,
        line_number,
    )?
    .ok_or(ftd::interpreter2::Error::ValueNotFound {
        doc_id: doc_name.to_string(),
        line_number,
        message: format!("{} not found in component arguments.", reference,),
    })?;
    if let ftd::interpreter2::Value::Module { things, .. } = arg
        .value
        .as_mut()
        .ok_or(ftd::interpreter2::Error::ValueNotFound {
            doc_id: doc_name.to_string(),
            line_number,
            message: format!("{} not found in component arguments.", reference),
        })?
        .value_mut(doc_name, line_number)?
    {
        things.insert(reference_full_name.to_string(), kind.clone());
    }

    Ok(())
}
