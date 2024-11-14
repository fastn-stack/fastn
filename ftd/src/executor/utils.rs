pub fn parse_error<T, S1>(m: S1, doc_id: &str, line_number: usize) -> ftd::executor::Result<T>
where
    S1: Into<String>,
{
    Err(ftd::executor::Error::ParseError {
        message: m.into(),
        doc_id: doc_id.to_string(),
        line_number,
    })
}

pub(crate) fn get_string_container(local_container: &[usize]) -> String {
    local_container
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<String>>()
        .join(",")
}

pub(crate) fn create_dummy_instruction_for_loop_element(
    instruction: &fastn_type::ComponentInvocation,
    doc: &mut ftd::executor::TDoc,
    inherited_variables: &mut ftd::VecMap<(String, Vec<usize>)>,
    local_container: &[usize],
) -> ftd::executor::Result<fastn_type::ComponentInvocation> {
    let mut instruction = instruction.clone();
    /*let reference_replace_pattern = fastn_type::PropertyValueSource::Loop(alias.to_string())
        .get_reference_name(alias, &doc.itdoc());
    let replace_with = format!("{}.INDEX", reference_name);
    let map =
        std::iter::IntoIterator::into_iter([(reference_replace_pattern, replace_with)]).collect();*/

    update_local_variable_references_in_component(
        &mut instruction,
        &Default::default(),
        inherited_variables,
        &Default::default(),
        local_container,
        doc,
    );
    Ok(instruction)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn update_instruction_for_loop_element(
    instruction: &fastn_type::ComponentInvocation,
    doc: &mut ftd::executor::TDoc,
    index_in_loop: usize,
    alias: &str,
    reference_name: &str,
    inherited_variables: &mut ftd::VecMap<(String, Vec<usize>)>,
    local_container: &[usize],
    doc_name: &str,
) -> ftd::executor::Result<fastn_type::ComponentInvocation> {
    use ftd::interpreter::PropertyValueSourceExt;

    let mut instruction = instruction.clone();
    let reference_replace_pattern = fastn_type::PropertyValueSource::Loop(alias.to_string())
        .get_reference_name(alias, &doc.itdoc());
    let replace_with = format!("{}.{}", reference_name, index_in_loop);
    let map =
        std::iter::IntoIterator::into_iter([(reference_replace_pattern, replace_with)]).collect();
    let replace_property_value = std::iter::IntoIterator::into_iter([(
        doc.itdoc()
            .resolve_name(format!("{}#{}", doc_name, ftd::interpreter::FTD_LOOP_COUNTER).as_str()),
        fastn_type::Value::Integer {
            value: index_in_loop as i64,
        }
        .into_property_value(false, instruction.line_number),
    )])
    .collect();

    update_local_variable_references_in_component(
        &mut instruction,
        &map,
        inherited_variables,
        &replace_property_value,
        local_container,
        doc,
    );

    Ok(instruction)
}

pub(crate) fn update_condition_in_component(
    component: &mut fastn_type::ComponentInvocation,
    outer_condition: fastn_type::Expression,
) {
    if let Some(condition) = component.condition.as_mut() {
        let references = {
            let mut reference = outer_condition.references;
            reference.extend(condition.references.to_owned());
            reference
        };
        let new_condition = fastn_type::Expression {
            expression: fastn_type::evalexpr::ExprNode::new(
                fastn_type::evalexpr::Operator::RootNode,
            )
            .add_children(vec![fastn_type::evalexpr::ExprNode::new(
                fastn_type::evalexpr::Operator::And,
            )
            .add_children(vec![
                outer_condition.expression,
                condition.expression.to_owned(),
            ])]),
            references,
            line_number: 0,
        };
        *condition = new_condition;
        return;
    }
    component.condition = Box::new(Some(outer_condition));
}

pub(crate) fn update_events_in_component(
    component: &mut fastn_type::ComponentInvocation,
    outer_event: Vec<fastn_type::Event>,
) {
    component.events.extend(outer_event);
}

pub(crate) fn insert_local_variables(
    component_name: &str,
    inherited_variables: &mut ftd::VecMap<(String, Vec<usize>)>,
    local_variable_map: &ftd::Map<String>,
    local_container: &[usize],
) {
    for (k, v) in local_variable_map {
        let key = k.trim_start_matches(format!("{}.", component_name).as_str());
        inherited_variables.insert(key.to_string(), (v.to_string(), local_container.to_vec()));
    }
}

pub(crate) fn update_inherited_reference_in_instruction(
    component_definition: &mut fastn_type::ComponentInvocation,
    inherited_variables: &mut ftd::VecMap<(String, Vec<usize>)>,
    local_container: &[usize],
    doc: &mut ftd::executor::TDoc,
) {
    update_local_variable_references_in_component(
        component_definition,
        &Default::default(),
        inherited_variables,
        &Default::default(),
        local_container,
        doc,
    );
}

pub(crate) fn update_local_variable_references_in_component(
    component: &mut fastn_type::ComponentInvocation,
    local_variable_map: &ftd::Map<String>,
    inherited_variables: &mut ftd::VecMap<(String, Vec<usize>)>,
    replace_property_value: &ftd::Map<fastn_type::PropertyValue>,
    local_container: &[usize],
    doc: &mut ftd::executor::TDoc,
) {
    update_local_variable_references_in_component_(
        component,
        local_variable_map,
        inherited_variables,
        replace_property_value,
        local_container,
        doc,
        false,
    )
}

pub(crate) fn update_local_variable_references_in_component_(
    component: &mut fastn_type::ComponentInvocation,
    local_variable_map: &ftd::Map<String>,
    inherited_variables: &mut ftd::VecMap<(String, Vec<usize>)>,
    replace_property_value: &ftd::Map<fastn_type::PropertyValue>,
    local_container: &[usize],
    doc: &mut ftd::executor::TDoc,
    is_children: bool,
) {
    use ftd::executor::fastn_type_functions::ComponentExt;

    if component.is_variable() {
        let mut component_name = fastn_type::PropertyValue::Reference {
            name: component.name.to_string(),
            kind: fastn_type::Kind::ui().into_kind_data(),
            source: fastn_type::PropertyValueSource::Global,
            is_mutable: false,
            line_number: 0,
        };
        update_local_variable_reference_in_property_value(
            &mut component_name,
            local_variable_map,
            inherited_variables,
            replace_property_value,
            local_container,
            doc,
            is_children,
        );

        component.name = component_name
            .reference_name()
            .map(ToString::to_string)
            .unwrap_or(component.name.to_string());
    }

    for property in component.properties.iter_mut() {
        update_local_variable_reference_in_property(
            property,
            local_variable_map,
            inherited_variables,
            replace_property_value,
            local_container,
            doc,
            is_children,
        );
    }

    for events in component.events.iter_mut() {
        for action in events.action.values.values_mut() {
            update_local_variable_reference_in_property_value(
                action,
                local_variable_map,
                inherited_variables,
                replace_property_value,
                local_container,
                doc,
                is_children,
            );
        }
    }

    if let Some(condition) = component.condition.as_mut() {
        update_local_variable_reference_in_condition(
            condition,
            local_variable_map,
            inherited_variables,
            replace_property_value,
            local_container,
            doc,
            is_children,
        );
    }

    if let Some(fastn_type::Loop { on, .. }) = component.iteration.as_mut() {
        update_local_variable_reference_in_property_value(
            on,
            local_variable_map,
            inherited_variables,
            replace_property_value,
            local_container,
            doc,
            is_children,
        );
    }

    for child in component.children.iter_mut() {
        update_local_variable_references_in_component_(
            child,
            local_variable_map,
            inherited_variables,
            &Default::default(),
            local_container,
            doc,
            is_children,
        );
    }
}

fn update_local_variable_reference_in_property(
    property: &mut fastn_type::Property,
    local_variable: &ftd::Map<String>,
    inherited_variables: &mut ftd::VecMap<(String, Vec<usize>)>,
    replace_property_value: &ftd::Map<fastn_type::PropertyValue>,
    local_container: &[usize],
    doc: &mut ftd::executor::TDoc,
    is_children: bool,
) {
    update_local_variable_reference_in_property_value(
        &mut property.value,
        local_variable,
        inherited_variables,
        replace_property_value,
        local_container,
        doc,
        is_children,
    );
    if let Some(ref mut condition) = property.condition {
        update_local_variable_reference_in_condition(
            condition,
            local_variable,
            inherited_variables,
            replace_property_value,
            local_container,
            doc,
            is_children,
        );
    }
}

fn update_local_variable_reference_in_condition(
    condition: &mut fastn_type::Expression,
    local_variable: &ftd::Map<String>,
    inherited_variables: &mut ftd::VecMap<(String, Vec<usize>)>,
    replace_property_value: &ftd::Map<fastn_type::PropertyValue>,
    local_container: &[usize],
    doc: &mut ftd::executor::TDoc,
    is_children: bool,
) {
    for reference in condition.references.values_mut() {
        update_local_variable_reference_in_property_value(
            reference,
            local_variable,
            inherited_variables,
            replace_property_value,
            local_container,
            doc,
            is_children,
        );
    }
}

fn update_local_variable_reference_in_property_value(
    property_value: &mut fastn_type::PropertyValue,
    local_variable: &ftd::Map<String>,
    inherited_variables: &mut ftd::VecMap<(String, Vec<usize>)>,
    replace_property_value: &ftd::Map<fastn_type::PropertyValue>,
    local_container: &[usize],
    doc: &mut ftd::executor::TDoc,
    is_children: bool, //Using children
) {
    let reference_or_clone = match property_value {
        fastn_type::PropertyValue::Reference { name, .. }
        | fastn_type::PropertyValue::Clone { name, .. } => name.to_string(),
        fastn_type::PropertyValue::FunctionCall(function_call) => {
            for property_value in function_call.values.values_mut() {
                update_local_variable_reference_in_property_value(
                    property_value,
                    local_variable,
                    inherited_variables,
                    replace_property_value,
                    local_container,
                    doc,
                    is_children,
                );
            }
            return;
        }
        fastn_type::PropertyValue::Value { value, .. } => {
            let is_children = is_children || value.kind().inner_list().is_subsection_ui();
            return match value {
                fastn_type::Value::List { data, .. } => {
                    for d in data.iter_mut() {
                        update_local_variable_reference_in_property_value(
                            d,
                            local_variable,
                            inherited_variables,
                            replace_property_value,
                            local_container,
                            doc,
                            is_children,
                        );
                    }
                }
                fastn_type::Value::Record { fields, .. }
                | fastn_type::Value::Object { values: fields } => {
                    for d in fields.values_mut() {
                        update_local_variable_reference_in_property_value(
                            d,
                            local_variable,
                            inherited_variables,
                            replace_property_value,
                            local_container,
                            doc,
                            is_children,
                        );
                    }
                }
                fastn_type::Value::UI {
                    component, name, ..
                } => {
                    if let Some(local_variable) = local_variable.iter().find_map(|(k, v)| {
                        if name.starts_with(format!("{}.", k).as_str()) || k.eq(name) {
                            Some(name.replace(k, v))
                        } else {
                            None
                        }
                    }) {
                        *name = local_variable;
                    }
                    update_local_variable_references_in_component_(
                        component,
                        local_variable,
                        inherited_variables,
                        &Default::default(),
                        local_container,
                        doc,
                        is_children,
                    )
                }
                fastn_type::Value::OrType { value, .. } => {
                    update_local_variable_reference_in_property_value(
                        value,
                        local_variable,
                        inherited_variables,
                        replace_property_value,
                        local_container,
                        doc,
                        is_children,
                    );
                }
                _ => {}
            };
        }
    };

    if let Some(local_variable) = local_variable.iter().find_map(|(k, v)| {
        if reference_or_clone.starts_with(format!("{}.", k).as_str()) || reference_or_clone.eq(k) {
            Some(reference_or_clone.replace(k, v))
        } else {
            None
        }
    }) {
        property_value.set_reference_or_clone(local_variable.as_str());
    }

    if let Some(replace_with) = replace_property_value.get(reference_or_clone.as_str()) {
        replace_with.clone_into(property_value);
    }

    if !is_children {
        update_inherited_reference_in_property_value(
            property_value,
            reference_or_clone.as_str(),
            inherited_variables,
            local_container,
            doc,
        )
    }
}

fn update_inherited_reference_in_property_value(
    property_value: &mut fastn_type::PropertyValue,
    reference_or_clone: &str,
    inherited_variables: &mut ftd::VecMap<(String, Vec<usize>)>,
    local_container: &[usize],
    doc: &mut ftd::executor::TDoc,
) {
    use ftd::interpreter::PropertyValueExt;

    let values = if reference_or_clone.starts_with(ftd::interpreter::FTD_INHERITED) {
        let reference_or_clone = reference_or_clone
            .trim_start_matches(format!("{}.", ftd::interpreter::FTD_INHERITED).as_str());
        inherited_variables.get_value_and_rem(reference_or_clone)
    } else {
        return;
    };

    let mut is_reference_updated = false;

    for ((reference, container), rem) in values.iter().rev() {
        if container.len() > local_container.len() {
            continue;
        }
        let mut found = true;

        if !container.is_empty()
            && container.len() == local_container.len()
            && container[container.len() - 1] != local_container[container.len() - 1]
        {
            continue;
        }

        for (idx, i) in container.iter().enumerate() {
            if *i != local_container[idx] {
                found = false;
                break;
            }
        }
        if found {
            is_reference_updated = true;
            let reference_name = if let Some(rem) = rem {
                format!("{}.{}", reference, rem)
            } else {
                reference.to_string()
            };

            if let Ok(ftd::interpreter::StateWithThing::Thing(property)) =
                fastn_type::PropertyValue::from_ast_value(
                    ftd_ast::VariableValue::String {
                        // TODO: ftd#default-colors, ftd#default-types
                        value: format!("${}", reference_name),
                        line_number: 0,
                        source: ftd_ast::ValueSource::Default,
                        condition: None,
                    },
                    &mut doc.itdoc(),
                    property_value.is_mutable(),
                    Some(&property_value.kind().into_kind_data()),
                )
            {
                *property_value = property;
            } else {
                property_value.set_reference_or_clone(reference_name.as_str());
            }

            property_value.set_reference_or_clone(
                if let Some(rem) = rem {
                    format!("{}.{}", reference, rem)
                } else {
                    reference.to_string()
                }
                .as_str(),
            );
            break;
        }
    }

    if !is_reference_updated
        && (reference_or_clone
            .starts_with(format!("{}.types", ftd::interpreter::FTD_INHERITED).as_str())
            || reference_or_clone
                .starts_with(format!("{}.colors", ftd::interpreter::FTD_INHERITED).as_str()))
    {
        if let Ok(ftd::interpreter::StateWithThing::Thing(property)) =
            fastn_type::PropertyValue::from_ast_value(
                ftd_ast::VariableValue::String {
                    // TODO: ftd#default-colors, ftd#default-types
                    value: {
                        format!(
                            "$ftd#default-{}{}",
                            if reference_or_clone.starts_with(
                                format!("{}.types", ftd::interpreter::FTD_INHERITED).as_str()
                            ) {
                                "types"
                            } else {
                                "colors"
                            },
                            reference_or_clone
                                .trim_start_matches(
                                    format!("{}.types", ftd::interpreter::FTD_INHERITED).as_str()
                                )
                                .trim_start_matches(
                                    format!("{}.colors", ftd::interpreter::FTD_INHERITED).as_str()
                                )
                        )
                    },
                    line_number: 0,
                    source: ftd_ast::ValueSource::Default,
                    condition: None,
                },
                &mut doc.itdoc(),
                property_value.is_mutable(),
                Some(&property_value.kind().into_kind_data()),
            )
        {
            *property_value = property;
        } else {
            property_value.set_reference_or_clone(
                format!("ftd#{}", reference_or_clone.trim_start_matches("ftd.")).as_str(),
            );
        }
    }
}

pub fn found_parent_containers(containers: &[&(String, Vec<usize>)], container: &[usize]) -> bool {
    for (_, item_container) in containers.iter().rev() {
        if item_container.len() > container.len() {
            continue;
        }
        let mut found = true;
        for (idx, i) in item_container.iter().enumerate() {
            if *i != container[idx] {
                found = false;
                break;
            }
        }
        if found {
            return true;
        }
    }
    false
}

pub(crate) fn replace_last_occurrence(s: &str, old_word: &str, new_word: &str) -> String {
    if !s.contains(old_word) {
        return s.to_string();
    }
    if let Some(idx) = s.rsplit(old_word).next() {
        let idx = s.len() - idx.len() - old_word.len();
        return format!("{}{}{}", &s[..idx], new_word, &s[idx + old_word.len()..]);
    }
    s.to_string()
}

pub(crate) fn get_evaluated_property(
    target_property: &fastn_type::Property,
    properties: &[fastn_type::Property],
    arguments: &[fastn_type::Argument],
    component_name: &str,
    doc_name: &str,
    line_number: usize,
) -> ftd::executor::Result<Option<fastn_type::Property>> {
    use ftd::interpreter::PropertyExt;

    let key = if let Some(key) = target_property.get_local_argument(component_name) {
        key
    } else {
        return Ok(Some(target_property.to_owned()));
    };

    let argument = arguments.iter().find(|v| v.name.eq(key.as_str())).ok_or(
        ftd::executor::Error::ParseError {
            message: format!("Cannot find `{}` argument", key),
            doc_id: doc_name.to_string(),
            line_number,
        },
    )?;
    let sources = argument.to_sources();
    if let Some(property) = ftd::interpreter::utils::find_properties_by_source(
        sources.as_slice(),
        properties,
        doc_name,
        argument,
        line_number,
    )?
    .into_iter()
    .find(|v| v.condition.is_none())
    {
        get_evaluated_property(
            &property,
            properties,
            arguments,
            component_name,
            doc_name,
            line_number,
        )
    } else if argument.kind.is_optional() || argument.kind.is_list() {
        Ok(None)
    } else {
        ftd::executor::utils::parse_error(
            format!("Expected Value for `{}`", key).as_str(),
            doc_name,
            line_number,
        )
    }
}
