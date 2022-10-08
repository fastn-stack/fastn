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

            elements.push(ExecuteDoc::execute_from_instruction(
                instruction,
                doc,
                local_container.as_slice(),
            )?);
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

        let _local_variables = doc.insert_local_variables(
            component_definition.name.as_str(),
            instruction.properties.as_slice(),
            component_definition.arguments.as_slice(),
            local_container,
            instruction.line_number,
        )?;

        todo!()
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
                    component_definition.arguments.as_slice(),
                    doc,
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
                    component_definition.arguments.as_slice(),
                    doc,
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
                    component_definition.arguments.as_slice(),
                    doc,
                    instruction.line_number,
                    children,
                )?)
            }
            _ => unimplemented!(),
        })
    }
}
