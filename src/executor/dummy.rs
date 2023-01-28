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
        let start_index = container.last().unwrap().to_owned();

        DummyElement {
            parent_container,
            start_index,
            element,
        }
    }

    pub(crate) fn from_instruction(
        mut instruction: ftd::interpreter2::Component,
        doc: &mut ftd::executor::TDoc,
        local_container: &[usize],
        inherited_variables: &mut ftd::VecMap<(String, Vec<usize>)>,
    ) -> ftd::executor::Result<ftd::executor::DummyElement> {
        let mut found_elements: std::collections::HashSet<String> =
            std::collections::HashSet::new();

        let element = DummyElement::from_instruction_to_element(
            instruction,
            doc,
            local_container,
            inherited_variables,
            &mut found_elements,
        )?;

        dbg!("DummyElement::from_instruction", &found_elements);

        Ok(DummyElement::from_element_and_container(
            element,
            local_container,
        ))
    }

    pub(crate) fn from_instruction_to_element(
        mut instruction: ftd::interpreter2::Component,
        doc: &mut ftd::executor::TDoc,
        local_container: &[usize],
        inherited_variables: &mut ftd::VecMap<(String, Vec<usize>)>,
        found_elements: &mut std::collections::HashSet<String>,
    ) -> ftd::executor::Result<ftd::executor::Element> {
        if let Some(iteration) = instruction.iteration.as_ref().clone() {
            instruction.iteration = Box::new(None);

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

        let component_definition = {
            // NOTE: doing unwrap to force bug report if we following fails, this function
            // must have validated everything, and must not fail at run time
            doc.itdoc()
                .get_component(instruction.name.as_str(), instruction.line_number)
                .unwrap()
        };

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
            )?
        } else {
            found_elements.insert(instruction.name.to_string());

            ftd::executor::Element::RawElement(ftd::executor::RawElement {
                name: instruction.name.to_string(),
                properties: instruction.properties.to_owned(),
                condition: *instruction.condition.clone(),
                children: vec![],
                line_number: instruction.line_number,
            })
        };

        let mut children_elements = vec![];

        for (idx, mut instruction) in instruction
            .get_children(&doc.itdoc())?
            .into_iter()
            .enumerate()
        {
            let local_container = {
                let mut local_container = local_container.to_vec();
                local_container.push(idx);
                local_container
            };
            children_elements.push(DummyElement::from_instruction_to_element(
                instruction,
                doc,
                local_container.as_slice(),
                inherited_variables,
                found_elements,
            )?);
        }

        if let Some(children) = element.get_children() {
            children.extend(children_elements);
        }

        Ok(element)
    }
}

#[derive(serde::Deserialize, Clone, Debug, PartialEq, serde::Serialize)]
pub struct ElementConstructor {
    pub arguments: Vec<ftd::interpreter2::Argument>,
    pub element: ftd::executor::Element,
}
