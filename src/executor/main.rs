#![allow(dead_code)]

#[derive(Debug, PartialEq)]
pub struct ExecuteDoc<'a> {
    pub name: &'a str,
    pub aliases: &'a ftd::Map<String>,
    pub bag: &'a mut ftd::Map<ftd::interpreter2::Thing>,
    pub instructions: &'a [ftd::interpreter2::Component],
}

#[derive(serde::Deserialize, Debug, Default, PartialEq, Clone, serde::Serialize)]
pub struct RT {
    pub name: String,
    pub aliases: ftd::Map<String>,
    pub bag: ftd::Map<ftd::interpreter2::Thing>,
    pub main: ftd::executor::Column,
}

impl<'a> ExecuteDoc<'a> {
    pub fn from_interpreter(document: ftd::interpreter2::Document) -> ftd::executor::Result<RT> {
        let mut document = document;
        let execute_doc = ExecuteDoc {
            name: document.name.as_str(),
            aliases: &document.aliases,
            bag: &mut document.data,
            instructions: &document.instructions,
        }
        .execute(&[])?;
        let mut main = ftd::executor::element::default_column();
        main.container.children.extend(execute_doc);

        Ok(RT {
            name: document.name.to_string(),
            aliases: document.aliases,
            bag: document.data,
            main,
        })
    }

    fn execute(
        &mut self,
        parent_container: &[usize],
    ) -> ftd::executor::Result<Vec<ftd::executor::Element>> {
        let mut doc = ftd::executor::TDoc {
            name: self.name,
            aliases: self.aliases,
            bag: self.bag,
        };

        ExecuteDoc::execute_from_instructions(self.instructions, &mut doc, parent_container)
    }

    fn execute_from_instructions(
        instructions: &[ftd::interpreter2::Component],
        doc: &mut ftd::executor::TDoc,
        parent_container: &[usize],
    ) -> ftd::executor::Result<Vec<ftd::executor::Element>> {
        let mut elements = vec![];
        for (idx, instruction) in instructions.iter().enumerate() {
            let local_container = {
                let mut local_container = parent_container.to_vec();
                local_container.push(idx);
                local_container
            };
            if instruction.is_loop() {
                elements.extend(ExecuteDoc::execute_recursive_component(
                    instruction,
                    doc,
                    local_container.as_slice(),
                )?);
            } else {
                elements.push(ExecuteDoc::execute_from_instruction(
                    instruction,
                    doc,
                    local_container.as_slice(),
                )?);
            }
        }

        Ok(elements)
    }

    fn execute_from_instruction(
        instruction: &ftd::interpreter2::Component,
        doc: &mut ftd::executor::TDoc,
        local_container: &[usize],
    ) -> ftd::executor::Result<ftd::executor::Element> {
        let component_definition = {
            // NOTE: doing unwrap to force bug report if we following fails, this function
            // must have validated everything, and must not fail at run time
            doc.itdoc()
                .get_component(instruction.name.as_str(), instruction.line_number)
                .unwrap()
        };

        if component_definition.definition.name.eq("ftd.kernel") {
            return ExecuteDoc::execute_kernel_components(
                instruction,
                doc,
                local_container,
                &component_definition,
            );
        }

        ExecuteDoc::execute_simple_component(
            instruction,
            doc,
            local_container,
            component_definition,
        )
    }

    fn execute_recursive_component(
        instruction: &ftd::interpreter2::Component,
        doc: &mut ftd::executor::TDoc,
        local_container: &[usize],
    ) -> ftd::executor::Result<Vec<ftd::executor::Element>> {
        let iteration = if let Some(iteration) = instruction.iteration.as_ref() {
            iteration
        } else {
            return ftd::executor::utils::parse_error(
                format!("Expected recursive, found: `{:?}`", instruction),
                doc.name,
                instruction.line_number,
            );
        };

        let children_length = iteration.children(&doc.itdoc())?.0.len();
        let reference_name =
            iteration
                .on
                .get_reference_or_clone()
                .ok_or(ftd::executor::Error::ParseError {
                    message: format!(
                        "Expected reference for loop object, found: `{:?}`",
                        iteration.on
                    ),
                    doc_id: doc.name.to_string(),
                    line_number: iteration.line_number,
                })?;
        let mut elements = vec![];
        for index in 0..children_length {
            let new_instruction = update_instruction_for_loop_element(
                instruction,
                doc,
                index,
                iteration.alias.as_str(),
                reference_name,
            )?;
            let local_container = {
                let mut local_container = local_container.to_vec();
                local_container.push(index);
                local_container
            };
            elements.push(ExecuteDoc::execute_from_instruction(
                &new_instruction,
                doc,
                local_container.as_slice(),
            )?);
        }
        Ok(elements)
    }

    fn execute_simple_component(
        instruction: &ftd::interpreter2::Component,
        doc: &mut ftd::executor::TDoc,
        local_container: &[usize],
        component_definition: ftd::interpreter2::ComponentDefinition,
    ) -> ftd::executor::Result<ftd::executor::Element> {
        let mut component_definition = component_definition;
        let local_variable_map = doc.insert_local_variables(
            component_definition.name.as_str(),
            instruction.properties.as_slice(),
            component_definition.arguments.as_slice(),
            local_container,
            instruction.line_number,
        )?;

        update_local_variable_references_in_component(
            &mut component_definition.definition,
            &local_variable_map,
        );

        ExecuteDoc::execute_from_instruction(&component_definition.definition, doc, local_container)
    }

    fn execute_kernel_components(
        instruction: &ftd::interpreter2::Component,
        doc: &mut ftd::executor::TDoc,
        local_container: &[usize],
        component_definition: &ftd::interpreter2::ComponentDefinition,
    ) -> ftd::executor::Result<ftd::executor::Element> {
        Ok(match component_definition.name.as_str() {
            "ftd#text" => {
                ftd::executor::Element::Text(ftd::executor::element::text_from_properties(
                    instruction.properties.as_slice(),
                    instruction.events.as_slice(),
                    component_definition.arguments.as_slice(),
                    doc,
                    local_container,
                    instruction.line_number,
                )?)
            }
            "ftd#integer" => {
                ftd::executor::Element::Integer(ftd::executor::element::integer_from_properties(
                    instruction.properties.as_slice(),
                    instruction.events.as_slice(),
                    component_definition.arguments.as_slice(),
                    doc,
                    local_container,
                    instruction.line_number,
                )?)
            }
            "ftd#boolean" => {
                ftd::executor::Element::Boolean(ftd::executor::element::boolean_from_properties(
                    instruction.properties.as_slice(),
                    instruction.events.as_slice(),
                    component_definition.arguments.as_slice(),
                    doc,
                    local_container,
                    instruction.line_number,
                )?)
            }
            "ftd#row" => {
                let children = ExecuteDoc::execute_from_instructions(
                    instruction.children.as_slice(),
                    doc,
                    local_container,
                )?;
                ftd::executor::Element::Row(ftd::executor::element::row_from_properties(
                    instruction.properties.as_slice(),
                    instruction.events.as_slice(),
                    component_definition.arguments.as_slice(),
                    doc,
                    local_container,
                    instruction.line_number,
                    children,
                )?)
            }
            "ftd#column" => {
                let children = ExecuteDoc::execute_from_instructions(
                    instruction.children.as_slice(),
                    doc,
                    local_container,
                )?;
                ftd::executor::Element::Column(ftd::executor::element::column_from_properties(
                    instruction.properties.as_slice(),
                    instruction.events.as_slice(),
                    component_definition.arguments.as_slice(),
                    doc,
                    local_container,
                    instruction.line_number,
                    children,
                )?)
            }
            _ => unimplemented!(),
        })
    }
}

fn update_instruction_for_loop_element(
    instruction: &ftd::interpreter2::Component,
    doc: &mut ftd::executor::TDoc,
    index_in_loop: usize,
    alias: &str,
    reference_name: &str,
) -> ftd::executor::Result<ftd::interpreter2::Component> {
    let mut instruction = instruction.clone();
    let reference_replace_pattern = ftd::interpreter2::PropertyValueSource::Loop(alias.to_string())
        .get_reference_name(alias, &doc.itdoc());
    let replace_with = format!("{}.{}", reference_name, index_in_loop);
    let map =
        std::iter::IntoIterator::into_iter([(reference_replace_pattern, replace_with)]).collect();
    update_local_variable_references_in_component(&mut instruction, &map);
    Ok(instruction)
}

fn update_reference_value(
    property_value: &mut ftd::interpreter2::PropertyValue,
    reference_replace_pattern: &str,
    replace_with: &str,
) {
    match property_value {
        ftd::interpreter2::PropertyValue::Clone { name, .. }
        | ftd::interpreter2::PropertyValue::Reference { name, .. } => {
            *name = name.replace(reference_replace_pattern, replace_with);
        }
        _ => {}
    }
}

fn update_local_variable_references_in_component(
    component: &mut ftd::interpreter2::Component,
    local_variable_map: &ftd::Map<String>,
) {
    for property in component.properties.iter_mut() {
        update_local_variable_reference_in_property(property, local_variable_map);
    }

    for events in component.events.iter_mut() {
        for action in events.action.values.values_mut() {
            update_local_variable_reference_in_property_value(action, local_variable_map);
        }
    }

    if let Some(condition) = component.condition.as_mut() {
        update_local_variable_reference_in_condition(condition, local_variable_map);
    }

    if let Some(ftd::interpreter2::Loop { on, .. }) = component.iteration.as_mut() {
        update_local_variable_reference_in_property_value(on, local_variable_map);
    }

    for child in component.children.iter_mut() {
        update_local_variable_references_in_component(child, local_variable_map);
    }
}

fn update_local_variable_reference_in_property(
    property: &mut ftd::interpreter2::Property,
    local_variable: &ftd::Map<String>,
) {
    update_local_variable_reference_in_property_value(&mut property.value, local_variable);
    if let Some(ref mut condition) = property.condition {
        update_local_variable_reference_in_condition(condition, local_variable);
    }
}

fn update_local_variable_reference_in_condition(
    condition: &mut ftd::interpreter2::Boolean,
    local_variable: &ftd::Map<String>,
) {
    match condition {
        ftd::interpreter2::Boolean::IsNotNull { value, .. } => {
            update_local_variable_reference_in_property_value(value, local_variable)
        }
        ftd::interpreter2::Boolean::IsNull { value, .. } => {
            update_local_variable_reference_in_property_value(value, local_variable)
        }
        ftd::interpreter2::Boolean::IsNotEmpty { value, .. } => {
            update_local_variable_reference_in_property_value(value, local_variable)
        }
        ftd::interpreter2::Boolean::IsEmpty { value, .. } => {
            update_local_variable_reference_in_property_value(value, local_variable)
        }
        ftd::interpreter2::Boolean::Equal { left, right, .. } => {
            update_local_variable_reference_in_property_value(left, local_variable);
            update_local_variable_reference_in_property_value(right, local_variable);
        }
        ftd::interpreter2::Boolean::NotEqual { left, right, .. } => {
            update_local_variable_reference_in_property_value(left, local_variable);
            update_local_variable_reference_in_property_value(right, local_variable);
        }
        ftd::interpreter2::Boolean::Literal { .. } => {}
    }
}

fn update_local_variable_reference_in_property_value(
    property_value: &mut ftd::interpreter2::PropertyValue,
    local_variable: &ftd::Map<String>,
) {
    let reference_or_clone = match property_value {
        ftd::interpreter2::PropertyValue::Reference { name, .. }
        | ftd::interpreter2::PropertyValue::Clone { name, .. } => name.to_string(),
        ftd::interpreter2::PropertyValue::FunctionCall(function_call) => {
            for property_value in function_call.values.values_mut() {
                update_local_variable_reference_in_property_value(property_value, local_variable);
            }
            return;
        }
        _ => return,
    };

    if let Some(local_variable) = local_variable.iter().find_map(|(k, v)| {
        if reference_or_clone.starts_with(format!("{}.", k).as_str()) || reference_or_clone.eq(k) {
            Some(reference_or_clone.replace(k, v))
        } else {
            None
        }
    }) {
        property_value.set_reference_or_clone(local_variable.as_str())
    }
}
