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

pub(crate) fn validate_properties_and_set_default(
    properties: &mut Vec<ftd::interpreter2::Property>,
    argument: &ftd::interpreter2::Argument,
    doc_id: &str,
    line_number: usize,
) -> ftd::executor::Result<()> {
    let mut found_default = None;
    let expected_kind = &argument.kind.kind;
    for property in properties.iter() {
        let found_kind = property.value.kind();
        if !found_kind.is_same_as(expected_kind) {
            return ftd::executor::utils::parse_error(
                format!(
                    "Expected kind is `{:?}`, found: `{:?}`",
                    expected_kind, found_kind,
                ),
                doc_id,
                property.line_number,
            );
        }

        if found_default.is_some() && property.condition.is_none() {
            return ftd::executor::utils::parse_error(
                format!(
                    "Already found default property in line number {:?}",
                    found_default
                ),
                doc_id,
                property.line_number,
            );
        }
        if property.condition.is_none() {
            found_default = Some(property.line_number);
        }
    }
    if found_default.is_none() {
        if let Some(ref default_value) = argument.value {
            properties.push(ftd::interpreter2::Property {
                value: default_value.to_owned(),
                source: ftd::interpreter2::PropertySource::Default,
                condition: None,
                line_number: argument.line_number,
            });
        } else if !expected_kind.is_optional() && !expected_kind.is_list() {
            return ftd::executor::utils::parse_error(
                format!(
                    "Need value of kind: `{:?}` for `{}`",
                    expected_kind, argument.name
                ),
                doc_id,
                line_number,
            );
        }
    }
    Ok(())
}

pub(crate) fn get_string_container(local_container: &[usize]) -> String {
    local_container
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<String>>()
        .join(",")
}

pub(crate) fn create_dummy_instruction_for_loop_element(
    instruction: &ftd::interpreter2::Component,
    doc: &mut ftd::executor::TDoc,
    inherited_variables: &mut ftd::VecMap<(String, Vec<usize>)>,
    local_container: &[usize],
) -> ftd::executor::Result<ftd::interpreter2::Component> {
    let mut instruction = instruction.clone();
    /*let reference_replace_pattern = ftd::interpreter2::PropertyValueSource::Loop(alias.to_string())
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
    instruction: &ftd::interpreter2::Component,
    doc: &mut ftd::executor::TDoc,
    index_in_loop: usize,
    alias: &str,
    reference_name: &str,
    inherited_variables: &mut ftd::VecMap<(String, Vec<usize>)>,
    local_container: &[usize],
    doc_name: &str,
) -> ftd::executor::Result<ftd::interpreter2::Component> {
    let mut instruction = instruction.clone();
    let reference_replace_pattern = ftd::interpreter2::PropertyValueSource::Loop(alias.to_string())
        .get_reference_name(alias, &doc.itdoc());
    let replace_with = format!("{}.{}", reference_name, index_in_loop);
    let map =
        std::iter::IntoIterator::into_iter([(reference_replace_pattern, replace_with)]).collect();
    let replace_property_value = std::iter::IntoIterator::into_iter([(
        doc.itdoc()
            .resolve_name(format!("{}#{}", doc_name, ftd::interpreter2::FTD_LOOP_COUNTER).as_str()),
        ftd::interpreter2::Value::Integer {
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
    component: &mut ftd::interpreter2::Component,
    outer_condition: ftd::interpreter2::Expression,
) {
    if let Some(condition) = component.condition.as_mut() {
        let references = {
            let mut reference = outer_condition.references;
            reference.extend(condition.references.to_owned());
            reference
        };
        let new_condition = ftd::interpreter2::Expression {
            expression: ftd::evalexpr::ExprNode::new(ftd::evalexpr::Operator::RootNode)
                .add_children(vec![ftd::evalexpr::ExprNode::new(
                    ftd::evalexpr::Operator::And,
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
    component: &mut ftd::interpreter2::Component,
    outer_event: Vec<ftd::interpreter2::Event>,
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
    component_definition: &mut ftd::interpreter2::Component,
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
    component: &mut ftd::interpreter2::Component,
    local_variable_map: &ftd::Map<String>,
    inherited_variables: &mut ftd::VecMap<(String, Vec<usize>)>,
    replace_property_value: &ftd::Map<ftd::interpreter2::PropertyValue>,
    local_container: &[usize],
    doc: &mut ftd::executor::TDoc,
) {
    if component.is_variable() {
        let mut component_name = ftd::interpreter2::PropertyValue::Reference {
            name: component.name.to_string(),
            kind: ftd::interpreter2::Kind::ui().into_kind_data(),
            source: ftd::interpreter2::PropertyValueSource::Global,
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
        );
    }

    if let Some(ftd::interpreter2::Loop { on, .. }) = component.iteration.as_mut() {
        update_local_variable_reference_in_property_value(
            on,
            local_variable_map,
            inherited_variables,
            replace_property_value,
            local_container,
            doc,
        );
    }

    for child in component.children.iter_mut() {
        update_local_variable_references_in_component(
            child,
            local_variable_map,
            inherited_variables,
            &Default::default(),
            local_container,
            doc,
        );
    }
}

fn update_local_variable_reference_in_property(
    property: &mut ftd::interpreter2::Property,
    local_variable: &ftd::Map<String>,
    inherited_variables: &mut ftd::VecMap<(String, Vec<usize>)>,
    replace_property_value: &ftd::Map<ftd::interpreter2::PropertyValue>,
    local_container: &[usize],
    doc: &mut ftd::executor::TDoc,
) {
    update_local_variable_reference_in_property_value(
        &mut property.value,
        local_variable,
        inherited_variables,
        replace_property_value,
        local_container,
        doc,
    );
    if let Some(ref mut condition) = property.condition {
        update_local_variable_reference_in_condition(
            condition,
            local_variable,
            inherited_variables,
            replace_property_value,
            local_container,
            doc,
        );
    }
}

fn update_local_variable_reference_in_condition(
    condition: &mut ftd::interpreter2::Expression,
    local_variable: &ftd::Map<String>,
    inherited_variables: &mut ftd::VecMap<(String, Vec<usize>)>,
    replace_property_value: &ftd::Map<ftd::interpreter2::PropertyValue>,
    local_container: &[usize],
    doc: &mut ftd::executor::TDoc,
) {
    for reference in condition.references.values_mut() {
        update_local_variable_reference_in_property_value(
            reference,
            local_variable,
            inherited_variables,
            replace_property_value,
            local_container,
            doc,
        );
    }
}

fn update_local_variable_reference_in_property_value(
    property_value: &mut ftd::interpreter2::PropertyValue,
    local_variable: &ftd::Map<String>,
    inherited_variables: &mut ftd::VecMap<(String, Vec<usize>)>,
    replace_property_value: &ftd::Map<ftd::interpreter2::PropertyValue>,
    local_container: &[usize],
    doc: &mut ftd::executor::TDoc,
) {
    let reference_or_clone = match property_value {
        ftd::interpreter2::PropertyValue::Reference { name, .. }
        | ftd::interpreter2::PropertyValue::Clone { name, .. } => name.to_string(),
        ftd::interpreter2::PropertyValue::FunctionCall(function_call) => {
            for property_value in function_call.values.values_mut() {
                update_local_variable_reference_in_property_value(
                    property_value,
                    local_variable,
                    inherited_variables,
                    replace_property_value,
                    local_container,
                    doc,
                );
            }
            return;
        }
        ftd::interpreter2::PropertyValue::Value { value, .. } => {
            return match value {
                ftd::interpreter2::Value::List { data, .. } => {
                    for d in data.iter_mut() {
                        update_local_variable_reference_in_property_value(
                            d,
                            local_variable,
                            inherited_variables,
                            replace_property_value,
                            local_container,
                            doc,
                        );
                    }
                }
                ftd::interpreter2::Value::Record { fields, .. }
                | ftd::interpreter2::Value::Object { values: fields } => {
                    for d in fields.values_mut() {
                        update_local_variable_reference_in_property_value(
                            d,
                            local_variable,
                            inherited_variables,
                            replace_property_value,
                            local_container,
                            doc,
                        );
                    }
                }
                ftd::interpreter2::Value::UI { component, .. } => {
                    update_local_variable_references_in_component(
                        component,
                        local_variable,
                        inherited_variables,
                        &Default::default(),
                        local_container,
                        doc,
                    )
                }
                ftd::interpreter2::Value::OrType { value, .. } => {
                    update_local_variable_reference_in_property_value(
                        value,
                        local_variable,
                        inherited_variables,
                        replace_property_value,
                        local_container,
                        doc,
                    );
                }
                _ => {}
            }
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
        *property_value = replace_with.to_owned();
    }

    update_inherited_reference_in_property_value(
        property_value,
        reference_or_clone.as_str(),
        inherited_variables,
        local_container,
        doc,
    )
}

fn update_inherited_reference_in_property_value(
    property_value: &mut ftd::interpreter2::PropertyValue,
    reference_or_clone: &str,
    inherited_variables: &mut ftd::VecMap<(String, Vec<usize>)>,
    local_container: &[usize],
    doc: &mut ftd::executor::TDoc,
) {
    let values = if reference_or_clone.starts_with(ftd::interpreter2::FTD_INHERITED) {
        let reference_or_clone = reference_or_clone
            .trim_start_matches(format!("{}.", ftd::interpreter2::FTD_INHERITED).as_str());
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

            if let Ok(ftd::interpreter2::StateWithThing::Thing(property)) =
                ftd::interpreter2::PropertyValue::from_ast_value(
                    ftd::ast::VariableValue::String {
                        // TODO: ftd#default-colors, ftd#default-types
                        value: format!("${}", reference_name),
                        line_number: 0,
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
            .starts_with(format!("{}.types", ftd::interpreter2::FTD_INHERITED).as_str())
            || reference_or_clone
                .starts_with(format!("{}.colors", ftd::interpreter2::FTD_INHERITED).as_str()))
    {
        if let Ok(ftd::interpreter2::StateWithThing::Thing(property)) =
            ftd::interpreter2::PropertyValue::from_ast_value(
                ftd::ast::VariableValue::String {
                    // TODO: ftd#default-colors, ftd#default-types
                    value: {
                        format!(
                            "$ftd#default-{}{}",
                            if reference_or_clone.starts_with(
                                format!("{}.types", ftd::interpreter2::FTD_INHERITED).as_str()
                            ) {
                                "types"
                            } else {
                                "colors"
                            },
                            reference_or_clone
                                .trim_start_matches(
                                    format!("{}.types", ftd::interpreter2::FTD_INHERITED).as_str()
                                )
                                .trim_start_matches(
                                    format!("{}.colors", ftd::interpreter2::FTD_INHERITED).as_str()
                                )
                        )
                    },
                    line_number: 0,
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
