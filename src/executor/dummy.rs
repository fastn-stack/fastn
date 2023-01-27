#[derive(Debug, PartialEq)]
pub struct DummyInstruction {
    pub parent_container: Vec<usize>,
    pub start_index: usize,
    pub instruction: ftd::executor::Element,
}

impl DummyInstruction {
    pub(crate) fn from_instruction(
        mut instruction: ftd::interpreter2::Component,
        doc: &mut ftd::executor::TDoc,
        local_container: &[usize],
        inherited_variables: &mut ftd::VecMap<(String, Vec<usize>)>,
    ) -> ftd::executor::Result<ftd::executor::Element> {
        if let Some(iteration) = instruction.iteration.as_ref().clone() {
            instruction.iteration = Box::new(None);
            return Ok(ftd::executor::Element::IterativeElement(
                ftd::executor::IterativeElement {
                    element: DummyInstruction::from_instruction(
                        instruction,
                        doc,
                        local_container,
                        inherited_variables,
                    )?,
                    iteration: Some(iteration),
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
    }
}
