#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub struct RT {
    pub name: String,
    pub aliases: std::collections::BTreeMap<String, String>,
    pub bag: std::collections::BTreeMap<String, crate::p2::Thing>,
    pub instructions: Vec<ftd::Instruction>,
}

impl RT {
    pub fn from(
        name: &str,
        aliases: std::collections::BTreeMap<String, String>,
        bag: std::collections::BTreeMap<String, crate::p2::Thing>,
        instructions: Vec<ftd::Instruction>,
    ) -> Self {
        Self {
            name: name.to_string(),
            aliases,
            bag,
            instructions,
        }
    }

    pub fn set_bool(&mut self, variable: &str, value: bool) -> crate::p1::Result<bool> {
        match self.bag.get(variable) {
            Some(ftd::p2::Thing::Variable(v)) => match v.value {
                ftd::Value::Boolean { value: old } => {
                    self.bag.insert(
                        variable.to_string(),
                        ftd::p2::Thing::Variable(ftd::Variable {
                            name: variable.to_string(),
                            value: ftd::Value::Boolean { value },
                        }),
                    );
                    Ok(old)
                }
                ref t => crate::e2(
                    format!("{} is not a boolean", variable),
                    format!("{:?}", t).as_str(),
                ),
            },
            Some(t) => crate::e2(
                format!("{} is not a variable", variable),
                format!("{:?}", t).as_str(),
            ),
            None => crate::e(format!("{} not found", variable)),
        }
    }

    pub fn render(&mut self) -> crate::p1::Result<ftd_rt::Column> {
        let mut main = ftd::p2::interpreter::default_column();
        let mut invocations = Default::default();
        let element = execute(
            self.name.as_str(),
            &self.aliases,
            &self.bag,
            &self.instructions,
            &Default::default(),
            &mut invocations,
            None,
        )?
        .children;
        main.container.children = element;
        store_invocations(&mut self.bag, invocations);
        Ok(main)
    }
}

pub(crate) fn execute(
    name: &str,
    aliases: &std::collections::BTreeMap<String, String>,
    bag: &std::collections::BTreeMap<String, crate::p2::Thing>,
    instructions: &[ftd::Instruction],
    arguments: &std::collections::BTreeMap<String, crate::Value>,
    invocations: &mut std::collections::BTreeMap<
        String,
        Vec<std::collections::BTreeMap<String, crate::Value>>,
    >,
    root_name: Option<&str>,
) -> crate::p1::Result<crate::component::ElementWithContainer> {
    let mut current_container: Vec<usize> = Default::default();
    let mut named_containers: std::collections::BTreeMap<String, Vec<usize>> = Default::default();
    let mut children = vec![];
    for instruction in instructions.iter() {
        let doc = crate::p2::TDoc { name, aliases, bag };
        match instruction {
            ftd::Instruction::ChangeContainer { name: c } => {
                change_container(c, &mut current_container, &mut named_containers)?
            }
            ftd::Instruction::Component {
                parent,
                children: inner,
            } => {
                assert!(arguments.is_empty()); // This clause cant have arguments
                let crate::component::ElementWithContainer {
                    element,
                    child_container,
                    ..
                } = parent.super_call(inner, &doc, arguments, invocations)?;
                children = add_element(
                    children,
                    &mut current_container,
                    &mut named_containers,
                    element,
                    child_container,
                )?
            }
            ftd::Instruction::ChildComponent { child: f } => {
                let e = f
                    .call(&doc, arguments, invocations, true, root_name)?
                    .element;
                children = add_element(
                    children,
                    &mut current_container,
                    &mut named_containers,
                    e,
                    None,
                )?
            }
        }
    }
    Ok(crate::component::ElementWithContainer {
        element: ftd_rt::Element::Null,
        children,
        child_container: Some(named_containers),
    })
}

pub(crate) fn store_invocations(
    bag: &mut std::collections::BTreeMap<String, crate::p2::Thing>,
    invocations: std::collections::BTreeMap<
        String,
        Vec<std::collections::BTreeMap<String, crate::Value>>,
    >,
) {
    for (k, v) in invocations.into_iter() {
        match bag.get_mut(k.as_str()).unwrap() {
            crate::p2::Thing::Component(ref mut c) => {
                if !c.kernel {
                    c.invocations.extend(v)
                }
            }
            _ => unreachable!(),
        }
    }
}

fn add_element(
    mut main: Vec<ftd_rt::Element>,
    current_container: &mut Vec<usize>,
    named_containers: &mut std::collections::BTreeMap<String, Vec<usize>>,
    e: ftd_rt::Element,
    container: Option<std::collections::BTreeMap<String, Vec<usize>>>,
) -> crate::p1::Result<Vec<ftd_rt::Element>> {
    let mut current = &mut main;
    for i in current_container.iter() {
        current = match &mut current[*i] {
            ftd_rt::Element::Row(ref mut r) => &mut r.container.children,
            ftd_rt::Element::Column(ref mut r) => &mut r.container.children,
            _ => unreachable!(),
        };
    }
    let len = current.len();
    let mut container_id = None;
    if let Some(v) = e.container_id() {
        let mut c = current_container.clone();
        c.push(len);
        container_id = Some(v.clone());
        named_containers.insert(v, c);
    }
    if e.is_open_container().0 {
        current_container.push(len)
    }
    if let Some(child_container) = container {
        let mut c = current_container.clone();
        c.push(len);
        for (key, value) in child_container.into_iter() {
            let mut hierarchy = c.clone();
            let mut p2 = value.clone();
            hierarchy.append(&mut p2);
            named_containers.insert(
                container_id
                    .clone()
                    .map_or(key.clone(), |v| format!("{}#{}", v, key)),
                hierarchy,
            );
        }
    }
    if let Some(id) = e.is_open_container().1 {
        change_container(
            {
                e.container_id()
                    .map_or(id.clone(), |v| format!("{}#{}", v, id))
                    .as_str()
            },
            current_container,
            named_containers,
        )?;
    }
    current.push(e);
    Ok(main)
}

fn change_container(
    name: &str,
    current_container: &mut Vec<usize>,
    named_containers: &mut std::collections::BTreeMap<String, Vec<usize>>,
) -> crate::p1::Result<()> {
    if name == "ftd#main" {
        *current_container = vec![];
        return Ok(());
    }
    *current_container = match named_containers.get(name) {
        Some(v) => v.to_owned(),
        None => {
            return crate::e2("no such container", name);
        }
    };
    Ok(())
}

/*instruction = Component {
    parent: ChildComponent {
        root: "foo/bar#foo",
        condition: None,
        properties: {
            "id": Property {
                default: Some(
                    Value {
                        value: String {
                            text: "foo-1",
                            source: Header,
                        },
                    },
                ),
                conditions: [],
            },
        },
    },
    children: [],
}*/
