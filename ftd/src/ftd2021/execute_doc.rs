#[derive(Debug, PartialEq)]
pub struct ExecuteDoc<'a> {
    pub name: &'a str,
    pub aliases: &'a ftd::Map<String>,
    pub bag: &'a ftd::Map<ftd::ftd2021::p2::Thing>,
    pub local_variables: &'a mut ftd::Map<ftd::ftd2021::p2::Thing>,
    pub instructions: &'a [ftd::Instruction],
    pub invocations: &'a mut ftd::Map<Vec<ftd::Map<ftd::Value>>>,
}

impl ExecuteDoc<'_> {
    pub(crate) fn execute(
        &mut self,
        parent_container: &[usize],
        id: Option<String>,
        referenced_local_variables: &mut ftd::Map<String>,
    ) -> ftd::ftd2021::p1::Result<ftd::ftd2021::component::ElementWithContainer> {
        let mut index = 0;
        self.execute_(
            &mut index,
            false,
            parent_container,
            0,
            None,
            id,
            referenced_local_variables,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn execute_(
        &mut self,
        index: &mut usize,
        is_external: bool,
        parent_container: &[usize],
        parent_children_length: usize, // in case of open container send the current length
        parent_id: Option<String>,
        id: Option<String>,
        referenced_local_variables: &mut ftd::Map<String>,
    ) -> ftd::ftd2021::p1::Result<ftd::ftd2021::component::ElementWithContainer> {
        let mut current_container: Vec<usize> = Default::default();
        let mut named_containers: ftd::Map<Vec<Vec<usize>>> = Default::default();
        let mut children: Vec<ftd::Element> = vec![];
        let mut external_children_count = if is_external { Some(0_usize) } else { None };

        while *index < self.instructions.len() {
            let mut doc = ftd::ftd2021::p2::TDoc {
                name: self.name,
                aliases: self.aliases,
                bag: self.bag,
                local_variables: self.local_variables,
                referenced_local_variables,
            };

            let local_container = {
                let mut local_container = parent_container.to_vec();
                local_container.append(&mut current_container.to_vec());
                let current_length = {
                    let mut current = &children;
                    for i in current_container.iter() {
                        current = match &current[*i] {
                            ftd::Element::Row(ref r) => &r.container.children,
                            ftd::Element::Column(ref r) => &r.container.children,
                            ftd::Element::Scene(ref r) => &r.container.children,
                            ftd::Element::Grid(ref r) => &r.container.children,
                            _ => unreachable!(),
                        };
                    }
                    current.len() + parent_children_length
                };
                local_container.push(current_length);
                local_container
            };

            match &self.instructions[*index] {
                ftd::Instruction::ChangeContainer { name: c } => {
                    if !named_containers.contains_key(c)
                        && is_external
                        && !match_parent_id(c, &parent_id)
                    {
                        *index -= 1;
                        return Ok(ftd::ftd2021::component::ElementWithContainer {
                            element: ftd::Element::Null,
                            children,
                            child_container: Some(named_containers),
                        });
                    }
                    change_container(
                        c,
                        &mut current_container,
                        &mut named_containers,
                        &parent_id,
                        self.name,
                    )?;
                }
                ftd::Instruction::Component {
                    parent,
                    children: inner,
                } => {
                    //assert!(self.arguments.is_empty()); // This clause cant have arguments
                    let (parent, inner) = {
                        let mut parent = parent.clone();
                        let mut inner = inner.clone();
                        doc.insert_local(
                            &mut parent,
                            &mut inner,
                            local_container.as_slice(),
                            &external_children_count,
                        )?;
                        (parent, inner)
                    };

                    let ftd::ftd2021::component::ElementWithContainer {
                        element,
                        children: container_children,
                        child_container,
                    } = parent.super_call(
                        &inner,
                        &mut doc,
                        self.invocations,
                        &local_container,
                        &external_children_count,
                    )?;

                    children = self.add_element(
                        children,
                        &mut current_container,
                        &mut named_containers,
                        element,
                        child_container,
                        index,
                        parent_container,
                        None,
                        container_children,
                        referenced_local_variables,
                        parent_children_length,
                    )?;
                }
                ftd::Instruction::ChildComponent { child: f } if !f.is_recursive => {
                    let (arguments, is_visible) = if let Some(ref condition) = f.condition {
                        ftd::ftd2021::p2::utils::arguments_on_condition(
                            condition,
                            f.line_number,
                            &doc,
                        )?
                    } else {
                        (Default::default(), true)
                    };
                    let f = {
                        let mut f = f.clone();
                        doc.insert_local_from_childcomponent(local_container.as_slice(), &mut f)?;
                        f.properties.extend(arguments.into_iter().map(|(k, v)| {
                            (
                                k,
                                ftd::ftd2021::component::Property {
                                    default: Some(ftd::PropertyValue::Value { value: v }),
                                    conditions: vec![],
                                    nested_properties: Default::default(),
                                },
                            )
                        }));
                        f
                    };

                    let new_id = {
                        if f.condition.is_some()
                            && f.condition.as_ref().unwrap().is_constant()
                            && !f.condition.as_ref().unwrap().eval(f.line_number, &doc)?
                            && f.condition
                                .as_ref()
                                .unwrap()
                                .set_null(f.line_number, doc.name)?
                        {
                            None
                        } else {
                            let new_id = ftd::ftd2021::p2::utils::string_optional(
                                "id",
                                &ftd::ftd2021::component::resolve_properties_by_id(
                                    f.line_number,
                                    &f.properties,
                                    &doc,
                                    Some("id".to_string()),
                                )?,
                                doc.name,
                                f.line_number,
                            )?;
                            if new_id.is_some() && id.is_some() {
                                Some(format!("{}:{}", id.as_ref().unwrap(), new_id.unwrap()))
                            } else {
                                None
                            }
                        }
                    };

                    let ftd::ftd2021::component::ElementWithContainer {
                        element: mut e,
                        child_container,
                        ..
                    } = f.call(
                        &mut doc,
                        self.invocations,
                        true,
                        &local_container,
                        new_id.clone(),
                        &external_children_count,
                    )?;
                    e.set_element_id(new_id);
                    if !is_visible {
                        e.set_non_visibility(!is_visible);
                    }

                    children = self.add_element(
                        children,
                        &mut current_container,
                        &mut named_containers,
                        e,
                        child_container,
                        index,
                        parent_container,
                        id.clone(),
                        vec![],
                        referenced_local_variables,
                        parent_children_length,
                    )?;
                }
                ftd::Instruction::RecursiveChildComponent { child: f }
                | ftd::Instruction::ChildComponent { child: f } => {
                    let elements =
                        f.recursive_call(&mut doc, self.invocations, true, &local_container)?;
                    for e in elements {
                        children = self.add_element(
                            children,
                            &mut current_container,
                            &mut named_containers,
                            e.element,
                            None,
                            index,
                            parent_container,
                            None,
                            vec![],
                            referenced_local_variables,
                            parent_children_length,
                        )?
                    }
                }
            }
            *index += 1;
            if let Some(count) = &mut external_children_count {
                *count += 1;
            }
        }

        Ok(ftd::ftd2021::component::ElementWithContainer {
            element: ftd::Element::Null,
            children,
            child_container: Some(named_containers),
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn add_element(
        &mut self,
        mut main: Vec<ftd::Element>,
        current_container: &mut Vec<usize>,
        named_containers: &mut ftd::Map<Vec<Vec<usize>>>,
        e: ftd::Element,
        container: Option<ftd::Map<Vec<Vec<usize>>>>,
        index: &mut usize,
        parent_container: &[usize],
        id: Option<String>,
        container_children: Vec<ftd::Element>,
        referenced_local_variables: &mut ftd::Map<String>,
        parent_children_length: usize,
    ) -> ftd::ftd2021::p1::Result<Vec<ftd::Element>> {
        let mut current = &mut main;
        for i in current_container.iter() {
            current = match &mut current[*i] {
                ftd::Element::Row(ref mut r) => &mut r.container.children,
                ftd::Element::Column(ref mut r) => &mut r.container.children,
                ftd::Element::Scene(ref mut r) => &mut r.container.children,
                ftd::Element::Grid(ref mut r) => &mut r.container.children,
                _ => unreachable!(),
            };
        }
        let len = current.len();
        let mut container_id = None;
        let parent_id = e.container_id();
        if let Some(ref v) = parent_id {
            let mut c = current_container.clone();
            c.push(len);
            container_id = Some(v.clone());
            if let Some(val) = named_containers.get_mut(v.as_str()) {
                val.push(c);
            } else {
                named_containers.insert(v.to_string(), vec![c]);
            }
        }

        if let Some(child_container) = container {
            let mut c = current_container.clone();
            c.push(len);
            update_named_container(&c, named_containers, &child_container, container_id, true);
        }
        let number_of_children = e.number_of_children();
        let append_at = e.append_at();
        let is_open = e.is_open_container(container_children.is_empty());
        current.push(e);

        if let Some(append_at) = append_at {
            match current.last_mut() {
                Some(ftd::Element::Column(ftd::Column {
                    container: ref mut c,
                    ..
                }))
                | Some(ftd::Element::Row(ftd::Row {
                    container: ref mut c,
                    ..
                }))
                | Some(ftd::Element::Scene(ftd::Scene {
                    container: ref mut c,
                    ..
                }))
                | Some(ftd::Element::Grid(ftd::Grid {
                    container: ref mut c,
                    ..
                })) => {
                    let string_container = {
                        let mut new_parent_container = parent_container.to_vec();
                        new_parent_container
                            .extend(current_container.iter().map(ToOwned::to_owned));
                        new_parent_container.push(len + parent_children_length);
                        ftd::ftd2021::p2::utils::get_string_container(
                            new_parent_container.as_slice(),
                        )
                    };
                    let child = if container_children.is_empty() && is_open {
                        current_container.push(len);
                        let mut new_parent_container = parent_container.to_vec();
                        new_parent_container.append(&mut current_container.to_vec());
                        *index += 1;
                        self.execute_(
                            index,
                            true,
                            &new_parent_container,
                            number_of_children,
                            parent_id,
                            None,
                            referenced_local_variables,
                        )?
                        .children
                    } else {
                        container_children
                    };

                    self.local_variables.insert(
                        ftd::ftd2021::p2::utils::resolve_local_variable_name(
                            0,
                            "CHILDREN-COUNT",
                            string_container.as_str(),
                            self.name,
                            self.aliases,
                        )?,
                        ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                            name: "CHILDREN-COUNT".to_string(),
                            value: ftd::PropertyValue::Value {
                                value: ftd::Value::Integer {
                                    value: child.len() as i64,
                                },
                            },
                            conditions: vec![],
                            flags: Default::default(),
                        }),
                    );

                    self.local_variables.insert(
                        ftd::ftd2021::p2::utils::resolve_local_variable_name(
                            0,
                            "CHILDREN-COUNT-MINUS-ONE",
                            string_container.as_str(),
                            self.name,
                            self.aliases,
                        )?,
                        ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                            name: "CHILDREN-COUNT-MINUS-ONE".to_string(),
                            value: ftd::PropertyValue::Value {
                                value: ftd::Value::Integer {
                                    value: child.len() as i64 - 1,
                                },
                            },
                            conditions: vec![],
                            flags: Default::default(),
                        }),
                    );

                    let external_children = {
                        if child.is_empty() {
                            vec![]
                        } else {
                            let mut main = ftd::ftd2021::p2::interpreter::default_column();
                            main.container.children.extend(child);
                            vec![ftd::Element::Column(main)]
                        }
                    };

                    if let Some((_, _, ref mut e)) = c.external_children {
                        e.extend(external_children);
                    } else {
                        return ftd::ftd2021::p2::utils::e2(
                            format!("expected external_children data for id: {}", append_at),
                            "",
                            0,
                        );
                    }
                }
                _ => unreachable!(),
            }
        } else {
            let string_container = {
                let mut new_parent_container = parent_container.to_vec();
                new_parent_container.extend(current_container.iter().map(ToOwned::to_owned));
                new_parent_container.push(len + parent_children_length);
                ftd::ftd2021::p2::utils::get_string_container(new_parent_container.as_slice())
            };
            let mut child_count = 0;
            let container = match current.last_mut() {
                Some(ftd::Element::Column(ftd::Column {
                    ref mut container, ..
                }))
                | Some(ftd::Element::Row(ftd::Row {
                    ref mut container, ..
                }))
                | Some(ftd::Element::Scene(ftd::Scene {
                    ref mut container, ..
                }))
                | Some(ftd::Element::Grid(ftd::Grid {
                    ref mut container, ..
                })) => {
                    child_count += container_children.len();
                    container.children.extend(container_children);
                    Some(container)
                }
                _ => None,
            };

            if is_open && child_count.eq(&0) {
                current_container.push(len);
                let mut new_parent_container = parent_container.to_vec();
                new_parent_container.append(&mut current_container.to_vec());

                let container = match container {
                    Some(container) => {
                        *index += 1;
                        let child = self.execute_(
                            index,
                            true,
                            &new_parent_container,
                            number_of_children,
                            parent_id.clone(),
                            id,
                            referenced_local_variables,
                        )?;
                        child_count += child.children.len();
                        container.children.extend(child.children);
                        child.child_container
                    }
                    _ => unreachable!(),
                };

                if let Some(child_container) = container {
                    update_named_container(
                        current_container,
                        named_containers,
                        &child_container,
                        None,
                        false,
                    );
                }
            }

            self.local_variables.insert(
                ftd::ftd2021::p2::utils::resolve_local_variable_name(
                    0,
                    "CHILDREN-COUNT",
                    string_container.as_str(),
                    self.name,
                    self.aliases,
                )?,
                ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                    name: "CHILDREN-COUNT".to_string(),
                    value: ftd::PropertyValue::Value {
                        value: ftd::Value::Integer {
                            value: child_count as i64,
                        },
                    },
                    conditions: vec![],
                    flags: Default::default(),
                }),
            );

            self.local_variables.insert(
                ftd::ftd2021::p2::utils::resolve_local_variable_name(
                    0,
                    "CHILDREN-COUNT-MINUS-ONE",
                    string_container.as_str(),
                    self.name,
                    self.aliases,
                )?,
                ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                    name: "CHILDREN-COUNT-MINUS-ONE".to_string(),
                    value: ftd::PropertyValue::Value {
                        value: ftd::Value::Integer {
                            value: (child_count as i64) - 1,
                        },
                    },
                    conditions: vec![],
                    flags: Default::default(),
                }),
            );
        }
        Ok(main)
    }
}

fn match_parent_id(c: &str, parent_id: &Option<String>) -> bool {
    if let Some(p) = parent_id {
        if c == p {
            return true;
        }
    }
    false
}

fn change_container(
    name: &str,
    current_container: &mut Vec<usize>,
    named_containers: &mut ftd::Map<Vec<Vec<usize>>>,
    parent_id: &Option<String>,
    doc_id: &str,
) -> ftd::ftd2021::p1::Result<()> {
    let name = name.replace('.', "#");
    if name == "ftd#main" || match_parent_id(name.as_str(), parent_id) {
        *current_container = vec![];
        return Ok(());
    }
    *current_container = match named_containers.get(name.as_str()) {
        Some(v) => v.get(0).unwrap().to_owned(),
        None => {
            let error_msg = format!("no such container: {}", name);
            return ftd::ftd2021::p2::utils::e2(error_msg.as_str(), doc_id, 0);
        }
    };
    Ok(())
}

fn update_named_container(
    current_container: &[usize],
    named_containers: &mut ftd::Map<Vec<Vec<usize>>>,
    child_container: &ftd::Map<Vec<Vec<usize>>>,
    container_id: Option<String>,
    key_with_container: bool,
) {
    for (key, value) in child_container.iter() {
        for value in value.iter() {
            let mut hierarchy = current_container.to_vec();
            let mut p2 = value.clone();
            hierarchy.append(&mut p2);
            let container_id = if key_with_container {
                container_id
                    .clone()
                    .map_or(format!("#{}", key), |v| format!("{}#{}", v, key))
            } else {
                key.clone()
            };
            if let Some(val) = named_containers.get_mut(container_id.as_str()) {
                val.push(hierarchy);
            } else {
                named_containers.insert(container_id, vec![hierarchy]);
            }
        }
    }
}
