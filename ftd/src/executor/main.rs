#[derive(Debug, PartialEq)]
pub struct ExecuteDoc<'a> {
    pub name: &'a str,
    pub aliases: &'a ftd::Map<String>,
    pub bag: &'a mut indexmap::IndexMap<String, ftd::interpreter::Thing>,
    pub instructions: &'a [ftd::interpreter::Component],
    pub dummy_instructions: &'a mut ftd::VecMap<ftd::executor::DummyElement>,
    pub element_constructor: &'a mut ftd::Map<ftd::executor::ElementConstructor>,
    pub js: &'a mut std::collections::HashSet<String>,
    pub css: &'a mut std::collections::HashSet<String>,
    pub rive_data: &'a mut Vec<ftd::executor::RiveData>,
}

#[derive(serde::Deserialize, Debug, PartialEq, Clone, serde::Serialize)]
pub struct RT {
    pub name: String,
    pub aliases: ftd::Map<String>,
    pub bag: indexmap::IndexMap<String, ftd::interpreter::Thing>,
    pub main: ftd::executor::Column,
    pub html_data: ftd::executor::HTMLData,
    pub dummy_instructions: ftd::VecMap<ftd::executor::DummyElement>,
    pub element_constructor: ftd::Map<ftd::executor::ElementConstructor>,
    pub js: std::collections::HashSet<String>,
    pub css: std::collections::HashSet<String>,
    pub rive_data: Vec<ftd::executor::RiveData>,
}

impl Default for RT {
    fn default() -> RT {
        RT {
            name: "".to_string(),
            aliases: Default::default(),
            bag: Default::default(),
            main: Default::default(),
            html_data: Default::default(),
            dummy_instructions: ftd::VecMap::new(),
            element_constructor: Default::default(),
            js: Default::default(),
            css: Default::default(),
            rive_data: vec![],
        }
    }
}

impl ExecuteDoc<'_> {
    #[tracing::instrument(skip_all)]
    pub fn from_interpreter(document: ftd::interpreter::Document) -> ftd::executor::Result<RT> {
        let mut document = document;
        let mut dummy_instructions = ftd::VecMap::new();
        let mut element_constructor = Default::default();
        let mut js: std::collections::HashSet<String> = document.js;
        let mut css: std::collections::HashSet<String> = document.css;
        let mut rive_data: Vec<ftd::executor::RiveData> = vec![];
        let execute_doc = ExecuteDoc {
            name: document.name.as_str(),
            aliases: &document.aliases,
            bag: &mut document.data,
            instructions: &document.tree,
            dummy_instructions: &mut dummy_instructions,
            element_constructor: &mut element_constructor,
            js: &mut js,
            css: &mut css,
            rive_data: &mut rive_data,
        }
        .execute()?;

        let (html_data, children) = match execute_doc.first() {
            Some(first) if first.is_document() => {
                if execute_doc.len() != 1 {
                    return ftd::executor::utils::parse_error(
                        "ftd.document can't have siblings.",
                        document.name.as_str(),
                        first.line_number(),
                    );
                }

                if let ftd::executor::Element::Document(d) = first {
                    // setting document breakpoint here
                    if let Some(breakpoint) = d.breakpoint_width.value.as_ref() {
                        ExecuteDoc::set_document_breakpoint(
                            &mut document.data,
                            breakpoint.mobile.value,
                            d.line_number,
                        );
                    }
                    (d.data.to_owned(), d.children.to_vec())
                } else {
                    unreachable!()
                }
            }
            _ => (ftd::executor::HTMLData::default(), execute_doc),
        };

        let mut main = ftd::executor::element::default_column();
        main.container.children.extend(children);

        Ok(RT {
            name: document.name.to_string(),
            aliases: document.aliases,
            bag: document.data,
            main,
            html_data,
            dummy_instructions,
            element_constructor,
            js,
            css,
            rive_data,
        })
    }
    #[tracing::instrument(skip_all)]
    fn execute(&mut self) -> ftd::executor::Result<Vec<ftd::executor::Element>> {
        let mut doc = ftd::executor::TDoc {
            name: self.name,
            aliases: self.aliases,
            bag: self.bag,
            dummy_instructions: self.dummy_instructions,
            element_constructor: self.element_constructor,
            js: self.js,
            css: self.css,
            rive_data: self.rive_data,
        };

        ExecuteDoc::execute_from_instructions_loop(self.instructions, &mut doc)
    }

    pub fn set_document_breakpoint(
        bag: &mut indexmap::IndexMap<String, ftd::interpreter::Thing>,
        breakpoint_width: i64,
        line_number: usize,
    ) {
        let breakpoint_width_from_bag = bag.get_mut(ftd::interpreter::FTD_BREAKPOINT_WIDTH);

        if let Some(ftd::interpreter::Thing::Variable(v)) = breakpoint_width_from_bag {
            v.value = fastn_type::PropertyValue::Value {
                value: fastn_type::Value::Record {
                    name: ftd::interpreter::FTD_BREAKPOINT_WIDTH_DATA.to_string(),
                    fields: std::iter::IntoIterator::into_iter([(
                        "mobile".to_string(),
                        fastn_type::PropertyValue::Value {
                            value: fastn_type::Value::Integer {
                                value: breakpoint_width,
                            },
                            is_mutable: false,
                            line_number,
                        },
                    )])
                    .collect(),
                },
                is_mutable: true,
                line_number,
            };
        }
    }

    #[allow(clippy::type_complexity)]
    pub(crate) fn get_instructions_from_instructions(
        instructions: &[ftd::interpreter::Component],
        doc: &mut ftd::executor::TDoc,
        parent_container: &[usize],
        inherited_variables: &mut ftd::VecMap<(String, Vec<usize>)>,
        device: Option<Device>,
    ) -> ftd::executor::Result<
        Vec<(
            Option<String>,
            Vec<usize>,
            ftd::interpreter::Component,
            Option<Device>,
        )>,
    > {
        use itertools::Itertools;
        let mut elements = vec![];
        let mut count = 0;
        for instruction in instructions.iter() {
            let instructions = ExecuteDoc::get_instructions_from_instruction(
                instruction,
                doc,
                parent_container,
                count,
                inherited_variables,
            )?
            .into_iter()
            .map(|v| (v.0, v.1, v.2, device.clone()))
            .collect_vec();
            count += instructions
                .iter()
                .filter(|(v, _, _, _)| v.is_none())
                .count();
            elements.extend(instructions)
        }
        Ok(elements)
    }

    #[allow(clippy::type_complexity)]
    fn get_instructions_from_instruction(
        instruction: &ftd::interpreter::Component,
        doc: &mut ftd::executor::TDoc,
        parent_container: &[usize],
        start_index: usize,
        inherited_variables: &mut ftd::VecMap<(String, Vec<usize>)>,
    ) -> ftd::executor::Result<Vec<(Option<String>, Vec<usize>, ftd::interpreter::Component)>> {
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
            Ok(vec![(None, local_container, instruction.to_owned())])
        }
    }

    fn execute_web_component(
        instruction: &ftd::interpreter::Component,
        doc: &mut ftd::executor::TDoc,
        local_container: &[usize],
        web_component_definition: ftd::interpreter::WebComponentDefinition,
        inherited_variables: &mut ftd::VecMap<(String, Vec<usize>)>,
        device: Option<ftd::executor::Device>,
    ) -> ftd::executor::Result<ftd::executor::Element> {
        let local_variable_map = doc.insert_local_variables(
            web_component_definition.name.as_str(),
            instruction.properties.as_slice(),
            web_component_definition.arguments.as_slice(),
            local_container,
            instruction.line_number,
            inherited_variables,
            true,
        )?;

        let mut properties: ftd::Map<fastn_type::PropertyValue> = Default::default();

        for argument in web_component_definition.arguments.as_slice() {
            let property_value = if let Some(local_variable) = local_variable_map
                .get(format!("{}.{}", instruction.name, argument.name.as_str()).as_str())
            {
                fastn_type::PropertyValue::Reference {
                    name: local_variable.to_string(),
                    kind: argument.kind.to_owned(),
                    source: fastn_type::PropertyValueSource::Global,
                    is_mutable: argument.mutable,
                    line_number: instruction.line_number,
                }
            } else if let Some(ref value) = argument.value {
                value.to_owned()
            } else {
                unreachable!()
            };

            properties.insert(argument.name.to_string(), property_value);
        }

        let web_component = ftd::executor::WebComponent {
            name: instruction.name.to_string(),
            properties,
            line_number: instruction.line_number,
            device,
        };

        Ok(ftd::executor::Element::WebComponent(web_component))
    }

    fn get_simple_instruction(
        instruction: &ftd::interpreter::Component,
        doc: &mut ftd::executor::TDoc,
        local_container: &[usize],
        component_definition: ftd::interpreter::ComponentDefinition,
        inherited_variables: &mut ftd::VecMap<(String, Vec<usize>)>,
    ) -> ftd::executor::Result<ftd::interpreter::Component> {
        let mut component_definition = component_definition;
        let local_variable_map = doc.insert_local_variables(
            component_definition.name.as_str(),
            instruction.properties.as_slice(),
            component_definition.arguments.as_slice(),
            local_container,
            instruction.line_number,
            inherited_variables,
            true,
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

    fn get_instruction_from_variable(
        instruction: &ftd::interpreter::Component,
        doc: &mut ftd::executor::TDoc,
    ) -> ftd::executor::Result<ftd::interpreter::Component> {
        use ftd::interpreter::{PropertyValueExt, ValueExt};

        if doc
            .itdoc()
            .get_component(instruction.name.as_str(), instruction.line_number)
            .is_ok()
        {
            let mut component = instruction.to_owned();
            component.source = ftd::interpreter::ComponentSource::Declaration;
            return Ok(component);
        }
        let mut component = doc
            .itdoc()
            .get_variable(instruction.name.as_str(), instruction.line_number)?
            .value
            .resolve(&doc.itdoc(), instruction.line_number)?
            .ui(doc.name, instruction.line_number)?;
        if let Some(condition) = instruction.condition.as_ref() {
            ftd::executor::utils::update_condition_in_component(
                &mut component,
                condition.to_owned(),
            );
        }

        ftd::executor::utils::update_events_in_component(
            &mut component,
            instruction.events.to_owned(),
        );

        component.source = ftd::interpreter::ComponentSource::Declaration;

        Ok(component)
    }

    #[allow(clippy::type_complexity)]
    fn get_loop_instructions(
        instruction: &ftd::interpreter::Component,
        doc: &mut ftd::executor::TDoc,
        parent_container: &[usize],
        start_index: usize,
        inherited_variables: &mut ftd::VecMap<(String, Vec<usize>)>,
    ) -> ftd::executor::Result<Vec<(Option<String>, Vec<usize>, ftd::interpreter::Component)>> {
        let iteration = if let Some(iteration) = instruction.iteration.as_ref() {
            iteration
        } else {
            return ftd::executor::utils::parse_error(
                format!("Expected recursive, found: `{:?}`", instruction),
                doc.name,
                instruction.line_number,
            );
        };

        let doc_name = ftd::interpreter::utils::get_doc_name_and_thing_name_and_remaining(
            iteration.alias.as_str(),
            doc.name,
            instruction.line_number,
        )
        .0;

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
                &doc_name,
            )?;
            elements.push((None, local_container, new_instruction));
        }
        if iteration.on.is_mutable() && iteration.on.reference_name().is_some() {
            let local_container = {
                let mut local_container = parent_container.to_vec();
                local_container.push(start_index);
                local_container
            };
            let component = ftd::executor::utils::create_dummy_instruction_for_loop_element(
                instruction,
                doc,
                inherited_variables,
                local_container.as_slice(),
            )?;

            elements.push((
                iteration.on.reference_name().map(|v| v.to_string()),
                local_container,
                component,
            ))
        }
        Ok(elements)
    }

    #[tracing::instrument(skip_all)]
    // TODO: Remove this after: Throw error when dummy is ready
    #[allow(unused_must_use)]
    fn execute_from_instructions_loop(
        instructions: &[ftd::interpreter::Component],
        doc: &mut ftd::executor::TDoc,
    ) -> ftd::executor::Result<Vec<ftd::executor::Element>> {
        let mut elements = vec![];
        let mut inherited_variables: ftd::VecMap<(String, Vec<usize>)> = Default::default();
        let mut instructions = ExecuteDoc::get_instructions_from_instructions(
            instructions,
            doc,
            &[],
            &mut inherited_variables,
            None,
        )?;
        while !instructions.is_empty() {
            let (dummy_reference, container, mut instruction, mut device) = instructions.remove(0);
            loop {
                if let Some(dummy_reference) = dummy_reference {
                    // TODO: Throw error when dummy is ready
                    ftd::executor::DummyElement::from_instruction(
                        instruction,
                        doc,
                        dummy_reference,
                        container.as_slice(),
                        &mut inherited_variables,
                    );
                    break;
                }

                if let Some(condition) = instruction.condition.as_ref() {
                    if condition.is_static(&doc.itdoc()) && !condition.eval(&doc.itdoc())? {
                        ExecuteDoc::insert_element(
                            &mut elements,
                            container.as_slice(),
                            ftd::executor::Element::Null {
                                line_number: instruction.line_number,
                            },
                        );
                        break;
                    }
                }

                if let Ok(web_component_definition) = doc
                    .itdoc()
                    .get_web_component(instruction.name.as_str(), instruction.line_number)
                {
                    let js = web_component_definition
                        .js
                        .clone()
                        .resolve(&doc.itdoc(), web_component_definition.line_number)?
                        .string(doc.name, web_component_definition.line_number)?;
                    doc.js.insert(format!("{}:type=\"module\"", js));
                    ExecuteDoc::insert_element(
                        &mut elements,
                        container.as_slice(),
                        ExecuteDoc::execute_web_component(
                            &instruction,
                            doc,
                            container.as_slice(),
                            web_component_definition,
                            &mut inherited_variables,
                            device,
                        )?,
                    );
                    break;
                }

                if instruction.is_variable() {
                    instruction = ExecuteDoc::get_instruction_from_variable(&instruction, doc)?;
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
                    if let Some(found_device) =
                        Device::from_component_name(component_definition.name.as_str())
                    {
                        if device.is_some() && device.as_ref().unwrap().ne(&found_device) {
                            ExecuteDoc::insert_element(
                                &mut elements,
                                container.as_slice(),
                                ftd::executor::Element::Null {
                                    line_number: instruction.line_number,
                                },
                            );
                            break;
                        }

                        ftd::executor::ExecuteDoc::add_colors_and_types_local_variable(
                            &instruction,
                            doc,
                            container.as_slice(),
                            &component_definition,
                            &mut inherited_variables,
                        )?;
                        let children_instructions = instruction.get_children(&doc.itdoc())?;
                        if children_instructions.len().ne(&1) {
                            return ftd::executor::utils::parse_error(
                                format!(
                                    "Expected one child for {}",
                                    component_definition.name.replace('#', ".")
                                ),
                                doc.name,
                                component_definition.line_number,
                            );
                        }
                        let line_number = instruction.line_number;
                        instruction = children_instructions[0].clone();
                        if device.is_none() {
                            found_device.add_condition(&mut instruction, line_number);
                            device = Some(found_device);
                        }
                        continue;
                    }

                    ExecuteDoc::insert_element(
                        &mut elements,
                        container.as_slice(),
                        ExecuteDoc::execute_kernel_components(
                            &instruction,
                            doc,
                            container.as_slice(),
                            &component_definition,
                            false,
                            &mut inherited_variables,
                            device.clone(),
                        )?,
                    );
                    let children_instructions = ExecuteDoc::get_instructions_from_instructions(
                        instruction.get_children(&doc.itdoc())?.as_slice(),
                        doc,
                        container.as_slice(),
                        &mut inherited_variables,
                        device,
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
                    ftd::executor::Element::Container(e) => &mut e.children,
                    ftd::executor::Element::Document(r) => &mut r.children,
                    t => unreachable!("{:?}", t),
                };
            }
        }
    }

    /*    fn execute_from_instructions(
        instructions: &[ftd::interpreter::Component],
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
        instruction: &ftd::interpreter::Component,
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
        instruction: &ftd::interpreter::Component,
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
        instruction: &ftd::interpreter::Component,
        doc: &mut ftd::executor::TDoc,
        local_container: &[usize],
        component_definition: ftd::interpreter::ComponentDefinition,
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

    pub(crate) fn add_colors_and_types_local_variable(
        instruction: &ftd::interpreter::Component,
        doc: &mut ftd::executor::TDoc,
        local_container: &[usize],
        component_definition: &ftd::interpreter::ComponentDefinition,
        inherited_variables: &mut ftd::VecMap<(String, Vec<usize>)>,
    ) -> ftd::executor::Result<()> {
        use itertools::Itertools;

        match component_definition.name.as_str() {
            "ftd#row" | "ftd#column" | "ftd#container" | "ftd#document" | "ftd#desktop"
            | "ftd#mobile" => {
                doc.insert_local_variables(
                    component_definition.name.as_str(),
                    instruction.properties.as_slice(),
                    component_definition
                        .arguments
                        .iter()
                        .filter(|&k| k.name.eq("colors") || k.name.eq("types"))
                        .cloned()
                        .collect_vec()
                        .as_slice(),
                    local_container,
                    instruction.line_number,
                    inherited_variables,
                    false,
                )?;
            }
            _ => {}
        };
        Ok(())
    }

    pub(crate) fn execute_kernel_components(
        instruction: &ftd::interpreter::Component,
        doc: &mut ftd::executor::TDoc,
        local_container: &[usize],
        component_definition: &ftd::interpreter::ComponentDefinition,
        is_dummy: bool,
        inherited_variables: &mut ftd::VecMap<(String, Vec<usize>)>,
        device: Option<ftd::executor::Device>,
    ) -> ftd::executor::Result<ftd::executor::Element> {
        ftd::executor::ExecuteDoc::add_colors_and_types_local_variable(
            instruction,
            doc,
            local_container,
            component_definition,
            inherited_variables,
        )?;

        Ok(match component_definition.name.as_str() {
            "ftd#text" => {
                let mut text = ftd::executor::element::text_from_properties(
                    instruction.properties.as_slice(),
                    instruction.events.as_slice(),
                    component_definition.arguments.as_slice(),
                    instruction.condition.as_ref(),
                    doc,
                    local_container,
                    is_dummy,
                    instruction.line_number,
                    inherited_variables,
                    device,
                )?;
                text.set_auto_id();
                ftd::executor::Element::Text(text)
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
                    inherited_variables,
                    device,
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
                    inherited_variables,
                    device,
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
                    inherited_variables,
                    device,
                )?)
            }
            "ftd#rive" => {
                ftd::executor::Element::Rive(ftd::executor::element::rive_from_properties(
                    instruction.properties.as_slice(),
                    instruction.events.as_slice(),
                    component_definition.arguments.as_slice(),
                    instruction.condition.as_ref(),
                    doc,
                    local_container,
                    instruction.line_number,
                    inherited_variables,
                    device,
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
                inherited_variables,
                device,
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
                    inherited_variables,
                    device,
                )?)
            }
            "ftd#container" => ftd::executor::Element::Container(
                ftd::executor::element::container_element_from_properties(
                    instruction.properties.as_slice(),
                    instruction.events.as_slice(),
                    component_definition.arguments.as_slice(),
                    instruction.condition.as_ref(),
                    doc,
                    local_container,
                    instruction.line_number,
                    vec![],
                    inherited_variables,
                    device,
                )?,
            ),
            "ftd#document" => {
                if !instruction.events.is_empty() {
                    return ftd::executor::utils::parse_error(
                        "Events are not expected for ftd.document type",
                        doc.name,
                        instruction.events.first().unwrap().line_number,
                    );
                }

                if instruction.condition.is_some() {
                    return ftd::executor::utils::parse_error(
                        "Condition is not expected for ftd.document type",
                        doc.name,
                        instruction.condition.clone().unwrap().line_number,
                    );
                }

                if local_container.len().ne(&1) || local_container.first().unwrap().ne(&0) {
                    return ftd::executor::utils::parse_error(
                        "ftd.document can occur only once and must be the root",
                        doc.name,
                        instruction.line_number,
                    );
                }

                ftd::executor::Element::Document(Box::new(
                    ftd::executor::element::document_from_properties(
                        instruction.properties.as_slice(),
                        component_definition.arguments.as_slice(),
                        doc,
                        instruction.line_number,
                        vec![],
                        inherited_variables,
                    )?,
                ))
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
                    inherited_variables,
                    device,
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
                    inherited_variables,
                    device,
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
                    inherited_variables,
                    device,
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
                    inherited_variables,
                    device,
                )?,
            ),
            "ftd#checkbox" => {
                ftd::executor::Element::CheckBox(ftd::executor::element::checkbox_from_properties(
                    instruction.properties.as_slice(),
                    instruction.events.as_slice(),
                    component_definition.arguments.as_slice(),
                    instruction.condition.as_ref(),
                    doc,
                    local_container,
                    instruction.line_number,
                    inherited_variables,
                    device,
                )?)
            }
            _ => unimplemented!(),
        })
    }
}

#[derive(Debug, PartialEq, Clone, serde::Deserialize, Default, serde::Serialize)]
pub enum Device {
    #[default]
    Desktop,
    Mobile,
}

impl Device {
    fn from_component_name(name: &str) -> Option<Device> {
        match name {
            "ftd#desktop" => Some(Device::Desktop),
            "ftd#mobile" => Some(Device::Mobile),
            _ => None,
        }
    }

    fn to_str(&self) -> &'static str {
        match self {
            Device::Desktop => "desktop",
            Device::Mobile => "mobile",
        }
    }

    pub(crate) fn is_mobile(&self) -> bool {
        matches!(self, Device::Mobile)
    }

    pub(crate) fn is_desktop(&self) -> bool {
        matches!(self, Device::Desktop)
    }

    fn add_condition(&self, instruction: &mut ftd::interpreter::Component, line_number: usize) {
        let expression =
            fastn_grammar::evalexpr::ExprNode::new(fastn_grammar::evalexpr::Operator::Eq)
                .add_children(vec![
                    fastn_grammar::evalexpr::ExprNode::new(
                        fastn_grammar::evalexpr::Operator::VariableIdentifierRead {
                            identifier: "ftd.device".to_string(),
                        },
                    ),
                    fastn_grammar::evalexpr::ExprNode::new(
                        fastn_grammar::evalexpr::Operator::Const {
                            value: fastn_grammar::evalexpr::Value::String(
                                self.to_str().to_string(),
                            ),
                        },
                    ),
                ]);

        if let Some(condition) = instruction.condition.as_mut() {
            let expression =
                fastn_grammar::evalexpr::ExprNode::new(fastn_grammar::evalexpr::Operator::RootNode)
                    .add_children(vec![fastn_grammar::evalexpr::ExprNode::new(
                        fastn_grammar::evalexpr::Operator::And,
                    )
                    .add_children(vec![expression, condition.expression.to_owned()])]);

            condition.expression = expression;

            condition.references.insert(
                "ftd.device".to_string(),
                fastn_type::PropertyValue::Reference {
                    name: "ftd#device".to_string(),
                    kind: fastn_type::Kind::record("ftd#device-data").into_kind_data(),
                    source: fastn_type::PropertyValueSource::Global,
                    is_mutable: false,
                    line_number,
                },
            );
        } else {
            let expression =
                fastn_grammar::evalexpr::ExprNode::new(fastn_grammar::evalexpr::Operator::RootNode)
                    .add_children(vec![expression]);

            let condition = ftd::interpreter::Expression {
                expression,
                references: std::iter::IntoIterator::into_iter([(
                    "ftd.device".to_string(),
                    fastn_type::PropertyValue::Reference {
                        name: "ftd#device".to_string(),
                        kind: fastn_type::Kind::record("ftd#device-data").into_kind_data(),
                        source: fastn_type::PropertyValueSource::Global,
                        is_mutable: false,
                        line_number,
                    },
                )])
                .collect(),
                line_number,
            };

            instruction.condition = Box::new(Some(condition));
        }
    }
}
