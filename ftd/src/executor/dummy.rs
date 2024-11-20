#[derive(serde::Deserialize, Clone, Debug, PartialEq, serde::Serialize)]
pub struct DummyElement {
    pub parent_container: Vec<usize>,
    pub start_index: usize,
    pub element: ftd::executor::Element,
}

impl DummyElement {
    pub(crate) fn from_element_and_container(
        element: ftd::executor::Element,
        container: &[usize],
    ) -> ftd::executor::DummyElement {
        let parent_container = container[..container.len() - 1].to_vec();
        let start_index = *container.last().unwrap();

        DummyElement {
            parent_container,
            start_index,
            element,
        }
    }

    pub(crate) fn from_instruction(
        instruction: fastn_resolved::ComponentInvocation,
        doc: &mut ftd::executor::TDoc,
        dummy_reference: String,
        local_container: &[usize],
        inherited_variables: &mut ftd::VecMap<(String, Vec<usize>)>,
    ) -> ftd::executor::Result<()> {
        let mut found_elements: std::collections::HashSet<String> =
            std::collections::HashSet::new();

        let line_number = instruction.line_number;

        let element = DummyElement::from_instruction_to_element(
            instruction,
            doc,
            local_container,
            inherited_variables,
            &mut found_elements,
        )?;

        ElementConstructor::from_list(doc, inherited_variables, line_number, &mut found_elements)?;

        let dummy_element = DummyElement::from_element_and_container(element, local_container);

        doc.dummy_instructions
            .insert(dummy_reference, dummy_element);

        Ok(())
    }

    pub(crate) fn from_instruction_to_element(
        mut instruction: fastn_resolved::ComponentInvocation,
        doc: &mut ftd::executor::TDoc,
        local_container: &[usize],
        inherited_variables: &mut ftd::VecMap<(String, Vec<usize>)>,
        found_elements: &mut std::collections::HashSet<String>,
    ) -> ftd::executor::Result<ftd::executor::Element> {
        use ftd::executor::fastn_type_functions::ComponentExt;

        if let Some(iteration) = instruction.iteration.take() {
            return Ok(ftd::executor::Element::IterativeElement(
                ftd::executor::IterativeElement {
                    element: Box::new(DummyElement::from_instruction_to_element(
                        instruction,
                        doc,
                        local_container,
                        inherited_variables,
                        found_elements,
                    )?),
                    iteration,
                },
            ));
        }

        let component_definition = doc
            .itdoc()
            .get_component(instruction.name.as_str(), instruction.line_number)
            .unwrap();

        let mut element = if component_definition.definition.name.eq("ftd.kernel") {
            ftd::executor::utils::update_inherited_reference_in_instruction(
                &mut instruction,
                inherited_variables,
                local_container,
                doc,
            );

            ftd::executor::ExecuteDoc::execute_kernel_components(
                &instruction,
                doc,
                local_container,
                &component_definition,
                true,
                &mut Default::default(),
                None,
            )?
        } else {
            found_elements.insert(instruction.name.to_string());

            let mut properties = vec![];
            for argument in component_definition.arguments.iter() {
                let sources = argument.to_sources();
                properties.extend(
                    ftd::interpreter::utils::find_properties_by_source(
                        sources.as_slice(),
                        instruction.properties.as_slice(),
                        doc.name,
                        argument,
                        argument.line_number,
                    )?
                    .into_iter()
                    .map(|v| (argument.name.to_string(), v)),
                );
            }

            ftd::executor::Element::RawElement(ftd::executor::RawElement {
                name: instruction.name.to_string(),
                properties,
                condition: *instruction.condition.clone(),
                children: vec![],
                events: instruction.events.clone(),
                line_number: instruction.line_number,
            })
        };

        let children_elements = instruction
            .get_children(&doc.itdoc())?
            .into_iter()
            .enumerate()
            .map(|(idx, instruction)| {
                let mut local_container = local_container.to_vec();
                local_container.push(idx);
                DummyElement::from_instruction_to_element(
                    instruction,
                    doc,
                    &local_container,
                    inherited_variables,
                    found_elements,
                )
            })
            .collect::<ftd::executor::Result<Vec<_>>>()?;

        if let Some(children) = element.get_children() {
            children.extend(children_elements);
        }

        Ok(element)
    }
}

#[derive(serde::Deserialize, Clone, Debug, PartialEq, serde::Serialize)]
pub struct ElementConstructor {
    pub arguments: Vec<fastn_resolved::Argument>,
    pub element: ftd::executor::Element,
    pub name: String,
}

impl ElementConstructor {
    pub(crate) fn new(
        arguments: &[fastn_resolved::Argument],
        element: ftd::executor::Element,
        name: &str,
    ) -> ElementConstructor {
        ElementConstructor {
            arguments: arguments.to_vec(),
            element,
            name: name.to_string(),
        }
    }

    pub(crate) fn from_list(
        doc: &mut ftd::executor::TDoc,
        inherited_variables: &mut ftd::VecMap<(String, Vec<usize>)>,
        line_number: usize,
        found_elements: &mut std::collections::HashSet<String>,
    ) -> ftd::executor::Result<()> {
        for element_name in found_elements.clone() {
            found_elements.remove(element_name.as_str());
            if doc.element_constructor.contains_key(element_name.as_str()) {
                continue;
            }
            let element_constructor = ElementConstructor::get(
                doc,
                inherited_variables,
                element_name.as_str(),
                line_number,
                found_elements,
            )?;
            doc.element_constructor
                .insert(element_name.to_string(), element_constructor);
        }
        Ok(())
    }

    pub(crate) fn get(
        doc: &mut ftd::executor::TDoc,
        inherited_variables: &mut ftd::VecMap<(String, Vec<usize>)>,
        component: &str,
        line_number: usize,
        found_elements: &mut std::collections::HashSet<String>,
    ) -> ftd::executor::Result<ElementConstructor> {
        let component_definition = doc.itdoc().get_component(component, line_number)?;
        let element = DummyElement::from_instruction_to_element(
            component_definition.definition,
            doc,
            &[],
            inherited_variables,
            found_elements,
        )?;

        Ok(ElementConstructor::new(
            component_definition.arguments.as_slice(),
            element,
            component,
        ))
    }
}
