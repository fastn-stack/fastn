#[derive(Debug, PartialEq)]
pub struct ExecuteDoc<'a> {
    pub name: &'a str,
    pub aliases: &'a std::collections::BTreeMap<String, String>,
    pub bag: &'a std::collections::BTreeMap<String, ftd::p2::Thing>,
    pub instructions: &'a [ftd::Instruction],
    pub arguments: &'a std::collections::BTreeMap<String, ftd::Value>,
    pub invocations: &'a mut std::collections::BTreeMap<
        String,
        Vec<std::collections::BTreeMap<String, ftd::Value>>,
    >,
}

impl<'a> ExecuteDoc<'a> {
    pub(crate) fn execute(
        &mut self,
        parent_container: &[usize],
        all_locals: &mut ftd::Map,
        id: Option<String>,
    ) -> ftd::p1::Result<ftd::component::ElementWithContainer> {
        let mut index = 0;
        self.execute_(&mut index, false, parent_container, all_locals, None, id)
    }

    fn execute_(
        &mut self,
        index: &mut usize,
        is_external: bool,
        parent_container: &[usize],
        all_locals: &mut ftd::Map,
        parent_id: Option<String>,
        id: Option<String>,
    ) -> ftd::p1::Result<ftd::component::ElementWithContainer> {
        let mut current_container: Vec<usize> = Default::default();
        let mut named_containers: std::collections::BTreeMap<String, Vec<Vec<usize>>> =
            Default::default();
        let mut children: Vec<ftd::Element> = vec![];

        while *index < self.instructions.len() {
            let doc = ftd::p2::TDoc {
                name: self.name,
                aliases: self.aliases,
                bag: self.bag,
            };
            // dbg!(&self.instructions[*index]);

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
                    current.len()
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
                        return Ok(ftd::component::ElementWithContainer {
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
                    assert!(self.arguments.is_empty()); // This clause cant have arguments
                    let ftd::component::ElementWithContainer {
                        element,
                        children: container_children,
                        child_container,
                    } = parent.super_call(
                        inner,
                        &doc,
                        self.arguments,
                        self.invocations,
                        all_locals,
                        &local_container,
                    )?;

                    let mut temp_locals: ftd::Map = Default::default();
                    children = self.add_element(
                        children,
                        &mut current_container,
                        &mut named_containers,
                        element,
                        child_container,
                        index,
                        parent_container,
                        &mut temp_locals,
                        None,
                        container_children,
                    )?;
                }
                ftd::Instruction::ChildComponent { child: f } if !f.is_recursive => {
                    let (arguments, is_visible) = if let Some(ref condition) = f.condition {
                        // dbg!(&condition, &self.arguments, &doc);
                        ftd::p2::utils::arguments_on_condition(
                            self.arguments,
                            condition,
                            f.line_number,
                            &doc,
                        )?
                    } else {
                        (self.arguments.to_owned(), true)
                    };

                    let new_id = {
                        if f.condition.is_some()
                            && f.condition.as_ref().unwrap().is_constant()
                            && !f.condition.as_ref().unwrap().eval(
                                f.line_number,
                                self.arguments,
                                &doc,
                            )?
                            && f.condition
                                .as_ref()
                                .unwrap()
                                .set_null(f.line_number, doc.name)?
                        {
                            None
                        } else {
                            let new_id = ftd::p2::utils::string_optional(
                                "id",
                                &ftd::component::resolve_properties(
                                    f.line_number,
                                    &f.properties,
                                    &arguments,
                                    &doc,
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

                    let ftd::component::ElementWithContainer {
                        element: mut e,
                        child_container,
                        ..
                    } = f.call(
                        &doc,
                        &arguments,
                        self.invocations,
                        true,
                        all_locals,
                        &local_container,
                        new_id.clone(),
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
                        all_locals,
                        id.clone(),
                        vec![],
                    )?;
                }
                ftd::Instruction::RecursiveChildComponent { child: f }
                | ftd::Instruction::ChildComponent { child: f } => {
                    let elements = f.recursive_call(
                        &doc,
                        self.arguments,
                        self.invocations,
                        true,
                        all_locals,
                        &local_container,
                    )?;
                    for e in elements {
                        children = self.add_element(
                            children,
                            &mut current_container,
                            &mut named_containers,
                            e.element,
                            None,
                            index,
                            parent_container,
                            all_locals,
                            None,
                            vec![],
                        )?
                    }
                }
            }
            *index += 1;
        }

        Ok(ftd::component::ElementWithContainer {
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
        named_containers: &mut std::collections::BTreeMap<String, Vec<Vec<usize>>>,
        e: ftd::Element,
        container: Option<std::collections::BTreeMap<String, Vec<Vec<usize>>>>,
        index: &mut usize,
        parent_container: &[usize],
        all_locals: &mut ftd::Map,
        id: Option<String>,
        container_children: Vec<ftd::Element>,
    ) -> ftd::p1::Result<Vec<ftd::Element>> {
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

        let open_id = e.is_open_container(container_children.is_empty()).1;
        let container_id = e.container_id();
        let is_open = e.is_open_container(container_children.is_empty()).0;

        current.push(e);

        if let Some(id) = open_id {
            let open_id = container_id.map_or(id.clone(), |v| format!("{}#{}", v, id));

            let container =
                get_external_children(current, &open_id, named_containers, current_container);

            let id = if id.contains('.') {
                ftd::p2::utils::split(id, ".")?.1
            } else {
                id
            };

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
                    let child = if container_children.is_empty() {
                        current_container.push(len);
                        let mut new_parent_container = parent_container.to_vec();
                        new_parent_container.append(&mut current_container.to_vec());

                        let mut temp_locals: ftd::Map = Default::default();

                        *index += 1;
                        self.execute_(
                            index,
                            true,
                            &new_parent_container,
                            &mut temp_locals,
                            parent_id,
                            None,
                        )?
                        .children
                    } else {
                        container_children
                    };

                    let external_children = {
                        if child.is_empty() {
                            vec![]
                        } else {
                            let mut main = ftd::p2::interpreter::default_column();
                            main.container.children = child;
                            vec![ftd::Element::Column(main)]
                        }
                    };
                    c.external_children = Some((id, container, external_children));
                }
                _ => unreachable!(),
            }
        } else {
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
                    container.children.extend(container_children);
                    Some(container)
                }
                _ => None,
            };

            if is_open {
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
                            all_locals,
                            parent_id.clone(),
                            id,
                        )?;
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
        }
        Ok(main)
    }
}

fn get_external_children(
    children: &[ftd::Element],
    open_id: &str,
    named_containers: &std::collections::BTreeMap<String, Vec<Vec<usize>>>,
    current_container: &[usize],
) -> Vec<Vec<usize>> {
    let open_id = if !open_id.contains('#') {
        format!("#{}", open_id.replace(".", "#"))
    } else {
        open_id.to_string()
    };

    let container = match named_containers.get(&open_id) {
        Some(c) => {
            let mut container = vec![];
            for c in c {
                let matching = c
                    .iter()
                    .zip(current_container.iter())
                    .filter(|&(a, b)| a == b)
                    .count();
                if let Some(cc) = c.get(matching) {
                    if matching == current_container.len() && cc == &(children.len() - 1) {
                        container.push(c[matching + 1..].to_vec());
                    }
                }
            }
            container
        }
        None => vec![],
    };
    container
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
    named_containers: &mut std::collections::BTreeMap<String, Vec<Vec<usize>>>,
    parent_id: &Option<String>,
    doc_id: &str,
) -> ftd::p1::Result<()> {
    if name == "ftd#main" || match_parent_id(name, parent_id) {
        *current_container = vec![];
        return Ok(());
    }
    *current_container = match named_containers.get(name) {
        Some(v) => v.get(0).unwrap().to_owned(),
        None => {
            return ftd::e2("no such container", doc_id, 0);
        }
    };
    Ok(())
}

fn update_named_container(
    current_container: &[usize],
    named_containers: &mut std::collections::BTreeMap<String, Vec<Vec<usize>>>,
    child_container: &std::collections::BTreeMap<String, Vec<Vec<usize>>>,
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
