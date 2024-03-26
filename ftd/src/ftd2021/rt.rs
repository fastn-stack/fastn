#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub struct RT {
    pub name: String,
    pub aliases: ftd::Map<String>,
    pub bag: ftd::Map<ftd::ftd2021::p2::Thing>,
    pub instructions: Vec<ftd::Instruction>,
}

impl RT {
    pub fn from(
        name: &str,
        aliases: ftd::Map<String>,
        bag: ftd::Map<ftd::ftd2021::p2::Thing>,
        instructions: Vec<ftd::Instruction>,
    ) -> Self {
        Self {
            name: name.to_string(),
            aliases,
            bag,
            instructions,
        }
    }

    // pub fn set_bool(&mut self, variable: &str, value: bool, doc_id: &str) -> ftd_p1::Result<bool> {
    //     match self.bag.get(variable) {
    //         Some(ftd::p2::Thing::Variable(v)) => match v.value {
    //             ftd::Value::Boolean { value: old } => {
    //                 let conditions = v.conditions.to_vec();
    //                 self.bag.insert(
    //                     variable.to_string(),
    //                     ftd::p2::Thing::Variable(ftd::Variable {
    //                         name: variable.to_string(),
    //                         value: ftd::Value::Boolean { value },
    //                         conditions,
    //                     }),
    //                 );
    //                 Ok(old)
    //             }
    //             ref t => ftd::p2::utils::e2(
    //                 format!("{} is not a boolean", variable),
    //                 format!("{:?}", t).as_str(),
    //             ),
    //         },
    //         Some(t) => ftd::p2::utils::e2(
    //             format!("{} is not a variable", variable),
    //             format!("{:?}", t).as_str(),
    //         ),
    //         None => ftd::p2::utils::e2(format!("{} not found", variable), doc_id),
    //     }
    // }

    pub fn render(&mut self) -> ftd::ftd2021::p1::Result<ftd::Column> {
        let mut main = self.render_();
        if let Ok(main) = &mut main {
            ftd::Element::set_id(&mut main.container.children, &[], None);
        }
        main
    }

    pub fn render_(&mut self) -> ftd::ftd2021::p1::Result<ftd::Column> {
        let mut main = ftd::ftd2021::p2::interpreter::default_column();
        let mut invocations = Default::default();
        let mut local_variables = Default::default();
        let mut element = ftd::ftd2021::execute_doc::ExecuteDoc {
            name: self.name.as_str(),
            aliases: &self.aliases,
            bag: &self.bag,
            local_variables: &mut local_variables,
            instructions: &self.instructions,
            invocations: &mut invocations,
        }
        .execute(&[], None, &mut Default::default())?
        .children;

        ftd::Element::set_children_count_variable(&mut element, &local_variables);
        ftd::Element::set_default_locals(&mut element);
        // ftd::Element::renest_on_region(&mut element);
        ftd::ftd2021::p2::document::set_region_id(&mut element);
        ftd::ftd2021::p2::document::default_scene_children_position(&mut element);

        main.container.children.extend(element);
        store_invocations(&mut self.bag, &mut local_variables, invocations);
        self.bag.extend(local_variables);
        Ok(main)
    }
}

pub(crate) fn store_invocations(
    bag: &mut ftd::Map<ftd::ftd2021::p2::Thing>,
    local_variables: &mut ftd::Map<ftd::ftd2021::p2::Thing>,
    invocations: ftd::Map<Vec<ftd::Map<ftd::Value>>>,
) {
    for (k, v) in invocations.into_iter() {
        if let Some(c) = bag.get_mut(k.as_str()) {
            match c {
                ftd::ftd2021::p2::Thing::Component(ref mut c) => {
                    if !c.kernel {
                        c.invocations.extend(v)
                    }
                    continue;
                }
                _ => unreachable!(),
            }
        }
        if let Some(c) = local_variables.get_mut(k.as_str()) {
            match c {
                ftd::ftd2021::p2::Thing::Component(ref mut c) => {
                    if !c.kernel {
                        c.invocations.extend(v)
                    }
                }
                _ => unreachable!(),
            }
        }
    }
}
