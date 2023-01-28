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
    #[tracing::instrument(skip_all)]
    pub fn from_interpreter(document: ftd::interpreter2::Document) -> ftd::executor::Result<RT> {
        let mut document = document;
        let execute_doc = ExecuteDoc {
            name: document.name.as_str(),
            aliases: &document.aliases,
            bag: &mut document.data,
            instructions: &document.tree,
        }
        .execute()?;
        let mut main = ftd::executor::element::default_column();
        main.container.children.extend(execute_doc);

        Ok(RT {
            name: document.name.to_string(),
            aliases: document.aliases,
            bag: document.data,
            main,
        })
    }
    #[tracing::instrument(skip_all)]
    fn execute(&mut self) -> ftd::executor::Result<Vec<ftd::executor::Element>> {
        let mut doc = ftd::executor::TDoc {
            name: self.name,
            aliases: self.aliases,
            bag: self.bag,
            dummy_instructions: &mut Default::default(),
            element_constructor: &mut Default::default(),
        };

        ExecuteDoc::execute_from_instructions_loop(self.instructions, &mut doc)
    }

    pub(crate) fn get_instructions_from_instructions(
        instructions: &[ftd::interpreter2::Component],
        doc: &mut ftd::executor::TDoc,
        parent_container: &[usize],
        inherited_variables: &mut ftd::VecMap<(String, Vec<usize>)>,
    ) -> ftd::executor::Result<Vec<(bool, Vec<usize>, ftd::interpreter2::Component)>> {
        let mut elements = vec![];
        let mut count = 0;
        for instruction in instructions.iter() {
            let instructions = ExecuteDoc::get_instructions_from_instruction(
                instruction,
                doc,
                parent_container,
                count,
                inherited_variables,
            )?;
            count += instructions.len();
            elements.extend(instructions)
        }
        Ok(elements)
    }

    fn get_instructions_from_instruction(
        instruction: &ftd::interpreter2::Component,
        doc: &mut ftd::executor::TDoc,
        parent_container: &[usize],
        start_index: usize,
        inherited_variables: &mut ftd::VecMap<(String, Vec<usize>)>,
    ) -> ftd::executor::Result<Vec<(bool, Vec<usize>, ftd::interpreter2::Component)>> {
        if instruction.is_loop() {
            ExecuteDoc::get_loop_instructions(
                instruction,
                doc,
                parent_container,
                start_index,
                inherited_variables,
            )
        } else {
            let mut local_container = parent_container.to_vec();
            local_container.push(start_index);
            Ok(vec![(false, local_container, instruction.to_owned())])
        }
    }

    fn get_simple_instruction(
        instruction: &ftd::interpreter2::Component,
        doc: &mut ftd::executor::TDoc,
        local_container: &[usize],
        component_definition: ftd::interpreter2::ComponentDefinition,
        inherited_variables: &mut ftd::VecMap<(String, Vec<usize>)>,
    ) -> ftd::executor::Result<ftd::interpreter2::Component> {
        let mut component_definition = component_definition;
        let local_variable_map = doc.insert_local_variables(
            component_definition.name.as_str(),
            instruction.properties.as_slice(),
            component_definition.arguments.as_slice(),
            local_container,
            instruction.line_number,
        )?;

        ftd::executor::utils::update_local_variable_references_in_component(
            &mut component_definition.definition,
            &local_variable_map,
            inherited_variables,
            &Default::default(),
            local_container,
            doc,
        );

        if let Some(condition) = instruction.condition.as_ref() {
            ftd::executor::utils::update_condition_in_component(
                &mut component_definition.definition,
                condition.to_owned(),
            );
        }

        ftd::executor::utils::update_events_in_component(
            &mut component_definition.definition,
            instruction.events.to_owned(),
        );

        ftd::executor::utils::insert_local_variables(
            &component_definition.name,
            inherited_variables,
            &local_variable_map,
            local_container,
        );

        Ok(component_definition.definition)
    }

    fn get_loop_instructions(
        instruction: &ftd::interpreter2::Component,
        doc: &mut ftd::executor::TDoc,
        parent_container: &[usize],
        start_index: usize,
        inherited_variables: &mut ftd::VecMap<(String, Vec<usize>)>,
    ) -> ftd::executor::Result<Vec<(bool, Vec<usize>, ftd::interpreter2::Component)>> {
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
            let local_container = {
                let mut local_container = parent_container.to_vec();
                local_container.push(index + start_index);
                local_container
            };
            let new_instruction = ftd::executor::utils::update_instruction_for_loop_element(
                instruction,
                doc,
                index,
                iteration.alias.as_str(),
                reference_name,
                inherited_variables,
                local_container.as_slice(),
            )?;
            elements.push((false, local_container, new_instruction));
        }
        if iteration.on.is_mutable() {
            let local_container = {
                let mut local_container = parent_container.to_vec();
                local_container.push(start_index);
                local_container
            };
            let component = ftd::executor::utils::create_dummy_instruction_for_loop_element(
                instruction,
                doc,
                iteration.alias.as_str(),
                reference_name,
                inherited_variables,
                local_container.as_slice(),
            )?;

            dbg!(&component);
            elements.push((true, local_container, component))
        }
        Ok(elements)
    }

    #[tracing::instrument(skip_all)]
    fn execute_from_instructions_loop(
        instructions: &[ftd::interpreter2::Component],
        doc: &mut ftd::executor::TDoc,
    ) -> ftd::executor::Result<Vec<ftd::executor::Element>> {
        let mut elements = vec![];
        let mut inherited_variables: ftd::VecMap<(String, Vec<usize>)> = Default::default();
        let mut instructions = ExecuteDoc::get_instructions_from_instructions(
            instructions,
            doc,
            &[],
            &mut inherited_variables,
        )?;
        while !instructions.is_empty() {
            let (is_dummy, container, mut instruction) = instructions.remove(0);
            loop {
                if let Some(condition) = instruction.condition.as_ref() {
                    if condition.is_static(&doc.itdoc()) && !condition.eval(&doc.itdoc())? {
                        ExecuteDoc::insert_element(
                            &mut elements,
                            container.as_slice(),
                            ftd::executor::Element::Null,
                        );
                        break;
                    }
                }

                if is_dummy {
                    dbg!(&instruction, &container);
                    ftd::executor::DummyElement::from_instruction(
                        instruction,
                        doc,
                        container.as_slice(),
                        &mut inherited_variables,
                    )?;
                    break;
                }
                let component_definition = {
                    // NOTE: doing unwrap to force bug report if we following fails, this function
                    // must have validated everything, and must not fail at run time
                    doc.itdoc()
                        .get_component(instruction.name.as_str(), instruction.line_number)
                        .unwrap()
                };

                if component_definition.definition.name.eq("ftd.kernel") {
                    ftd::executor::utils::update_inherited_reference_in_instruction(
                        &mut instruction,
                        &mut inherited_variables,
                        container.as_slice(),
                        doc,
                    );
                    ExecuteDoc::insert_element(
                        &mut elements,
                        container.as_slice(),
                        ExecuteDoc::execute_kernel_components(
                            &instruction,
                            doc,
                            container.as_slice(),
                            &component_definition,
                            false,
                        )?,
                    );
                    let children_instructions = ExecuteDoc::get_instructions_from_instructions(
                        instruction.get_children(&doc.itdoc())?.as_slice(),
                        doc,
                        container.as_slice(),
                        &mut inherited_variables,
                    )?;
                    instructions.extend(children_instructions);

                    break;
                } else {
                    instruction = ExecuteDoc::get_simple_instruction(
                        &instruction,
                        doc,
                        container.as_slice(),
                        component_definition,
                        &mut inherited_variables,
                    )?;
                }
            }
        }
        Ok(elements)
    }

    fn insert_element(
        elements: &mut Vec<ftd::executor::Element>,
        container: &[usize],
        element: ftd::executor::Element,
    ) {
        let mut current = elements;
        for (idx, i) in container.iter().enumerate() {
            if idx == container.len() - 1 {
                current.insert(*i, element);
                break;
            } else {
                current = match &mut current[*i] {
                    ftd::executor::Element::Row(r) => &mut r.container.children,
                    ftd::executor::Element::Column(r) => &mut r.container.children,
                    t => unreachable!("{:?}", t),
                };
            }
        }
    }

    /*    fn execute_from_instructions(
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
        if let Some(condition) = instruction.condition.as_ref() {
            if condition.is_static(&doc.itdoc()) && !condition.eval(&doc.itdoc())? {
                return Ok(ftd::executor::Element::Null);
            }
        }
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

        if let Some(condition) = instruction.condition.as_ref() {
            update_condition_in_component(
                &mut component_definition.definition,
                condition.to_owned(),
            );
        }

        ExecuteDoc::execute_from_instruction(&component_definition.definition, doc, local_container)
    }*/

    pub(crate) fn execute_kernel_components(
        instruction: &ftd::interpreter2::Component,
        doc: &mut ftd::executor::TDoc,
        local_container: &[usize],
        component_definition: &ftd::interpreter2::ComponentDefinition,
        is_dummy: bool,
    ) -> ftd::executor::Result<ftd::executor::Element> {
        Ok(match component_definition.name.as_str() {
            "ftd#text" => {
                ftd::executor::Element::Text(ftd::executor::element::text_from_properties(
                    instruction.properties.as_slice(),
                    instruction.events.as_slice(),
                    component_definition.arguments.as_slice(),
                    instruction.condition.as_ref(),
                    doc,
                    local_container,
                    is_dummy,
                    instruction.line_number,
                )?)
            }
            "ftd#integer" => {
                ftd::executor::Element::Integer(ftd::executor::element::integer_from_properties(
                    instruction.properties.as_slice(),
                    instruction.events.as_slice(),
                    component_definition.arguments.as_slice(),
                    instruction.condition.as_ref(),
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
                    instruction.condition.as_ref(),
                    doc,
                    local_container,
                    instruction.line_number,
                )?)
            }
            "ftd#decimal" => {
                ftd::executor::Element::Decimal(ftd::executor::element::decimal_from_properties(
                    instruction.properties.as_slice(),
                    instruction.events.as_slice(),
                    component_definition.arguments.as_slice(),
                    instruction.condition.as_ref(),
                    doc,
                    local_container,
                    instruction.line_number,
                )?)
            }
            "ftd#row" => ftd::executor::Element::Row(ftd::executor::element::row_from_properties(
                instruction.properties.as_slice(),
                instruction.events.as_slice(),
                component_definition.arguments.as_slice(),
                instruction.condition.as_ref(),
                doc,
                local_container,
                instruction.line_number,
                vec![],
            )?),
            "ftd#column" => {
                ftd::executor::Element::Column(ftd::executor::element::column_from_properties(
                    instruction.properties.as_slice(),
                    instruction.events.as_slice(),
                    component_definition.arguments.as_slice(),
                    instruction.condition.as_ref(),
                    doc,
                    local_container,
                    instruction.line_number,
                    vec![],
                )?)
            }
            "ftd#image" => {
                ftd::executor::Element::Image(ftd::executor::element::image_from_properties(
                    instruction.properties.as_slice(),
                    instruction.events.as_slice(),
                    component_definition.arguments.as_slice(),
                    instruction.condition.as_ref(),
                    doc,
                    local_container,
                    instruction.line_number,
                )?)
            }
            "ftd#code" => {
                ftd::executor::Element::Code(ftd::executor::element::code_from_properties(
                    instruction.properties.as_slice(),
                    instruction.events.as_slice(),
                    component_definition.arguments.as_slice(),
                    instruction.condition.as_ref(),
                    doc,
                    local_container,
                    instruction.line_number,
                )?)
            }
            "ftd#iframe" => {
                ftd::executor::Element::Iframe(ftd::executor::element::iframe_from_properties(
                    instruction.properties.as_slice(),
                    instruction.events.as_slice(),
                    component_definition.arguments.as_slice(),
                    instruction.condition.as_ref(),
                    doc,
                    local_container,
                    instruction.line_number,
                )?)
            }
            "ftd#text-input" => ftd::executor::Element::TextInput(
                ftd::executor::element::text_input_from_properties(
                    instruction.properties.as_slice(),
                    instruction.events.as_slice(),
                    component_definition.arguments.as_slice(),
                    instruction.condition.as_ref(),
                    doc,
                    local_container,
                    instruction.line_number,
                )?,
            ),
            _ => unimplemented!(),
        })
    }
}
