use crate::p2::Kind;

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Component {
    pub root: String,
    pub full_name: String,
    pub arguments: std::collections::BTreeMap<String, crate::p2::Kind>,
    pub locals: std::collections::BTreeMap<String, crate::p2::Kind>,
    pub properties: std::collections::BTreeMap<String, Property>,
    pub instructions: Vec<Instruction>,
    pub events: Vec<ftd::p2::expression::Event>,
    pub condition: Option<ftd::p2::Boolean>,
    pub kernel: bool,
    pub invocations: Vec<std::collections::BTreeMap<String, crate::Value>>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum Instruction {
    ChildComponent {
        child: ChildComponent,
    },
    Component {
        parent: ChildComponent,
        children: Vec<ChildComponent>,
    },
    ChangeContainer {
        name: String,
    },
    RecursiveChildComponent {
        child: ChildComponent,
    },
}

impl Instruction {
    pub fn resolve_id(&self) -> Option<&str> {
        let id = match self {
            Instruction::ChildComponent { child } => child.properties.get("id"),
            Instruction::Component { parent, .. } => parent.properties.get("id"),
            _ => None,
        };
        if let Some(property) = id {
            if let Some(crate::PropertyValue::Value {
                value: crate::variable::Value::String { text, .. },
            }) = &property.default
            {
                return Some(text.as_str());
            }
        }
        None
    }
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ChildComponent {
    pub root: String,
    pub condition: Option<ftd::p2::Boolean>,
    pub properties: std::collections::BTreeMap<String, Property>,
    pub events: Vec<ftd::p2::expression::Event>,
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Property {
    pub default: Option<crate::PropertyValue>,
    pub conditions: Vec<(crate::p2::Boolean, crate::PropertyValue)>,
}

pub struct ElementWithContainer {
    pub element: ftd_rt::Element,
    pub children: Vec<ftd_rt::Element>,
    pub child_container: Option<std::collections::BTreeMap<String, Vec<Vec<usize>>>>,
}

impl Property {
    fn eval(
        &self,
        name: &str,
        arguments: &std::collections::BTreeMap<String, crate::Value>,
        doc: &crate::p2::TDoc,
    ) -> crate::p1::Result<&crate::PropertyValue> {
        let mut property_value = crate::e2(format!("{:?}", name), "condition is not complete");
        if let Some(property) = &self.default {
            property_value = Ok(property);
        }
        for (boolean, property) in &self.conditions {
            if boolean.eval(arguments, doc)? {
                property_value = Ok(property);
            }
        }
        property_value
    }

    pub fn get_property_for_loop(
        reference: String,
        fields: &std::collections::BTreeMap<String, crate::PropertyValue>,
        property_title: &str,
        property_kind: Option<&crate::p2::Kind>,
    ) -> crate::p1::Result<Self> {
        let kind = match property_kind {
            Some(kind) => kind,
            None => return crate::e(format!("{} property is not required", property_title)),
        };

        let (part_1, part_2) = ftd::p2::utils::split(reference, ".")?;

        let property_value = match fields.get(&*part_2) {
            Some(property_value) => property_value,
            None => return crate::e(format!("{} not found", part_1)),
        };

        if !property_value.clone().kind().is_same_as(kind) {
            return crate::e(format!(
                "'{}' is expected to be {:?}, but its of {:?}",
                property_title, property_kind, kind
            ));
        }

        Ok(crate::component::Property {
            default: Some(property_value.clone()),
            conditions: vec![],
        })
    }
}

impl ChildComponent {
    pub fn children_for_loop(
        value: crate::Value,
        p1: &crate::p1::Section,
        root: &crate::Component,
        doc: &crate::p2::TDoc,
        loop_ref: &str,
        arguments: &std::collections::BTreeMap<String, crate::p2::Kind>,
    ) -> crate::p1::Result<Self> {
        let new_caption_title = root.get_caption();
        let mut new_caption = p1.caption.clone();
        let mut new_properties: std::collections::BTreeMap<String, crate::component::Property> =
            Default::default();
        let mut new_header: crate::p1::Header = Default::default();
        let mut boolean_condition = None;
        match value {
            crate::Value::Record { .. }
            | crate::Value::String { .. }
            | crate::Value::Integer { .. }
            | crate::Value::Boolean { .. }
            | crate::Value::Decimal { .. } => {
                for (h1, h2) in &p1.header.0 {
                    if h1 == "$loop$" {
                        continue;
                    }
                    if h1 == "if" {
                        if h2.contains(loop_ref) {
                            let mut right = ftd::PropertyValue::Value {
                                value: ftd::Value::Boolean { value: true },
                            };

                            if let Some((first, _)) = h2.split_once(' ') {
                                if first == "not" {
                                    right = ftd::PropertyValue::Value {
                                        value: ftd::Value::Boolean { value: false },
                                    };
                                }
                            }

                            let (part_1, part_2) = ftd::p2::utils::split(h2.to_string(), ".")?;
                            boolean_condition = match value.clone() {
                                crate::Value::Record { fields, .. } => {
                                    let field = match fields.get(&*part_2) {
                                        Some(field) => field,
                                        None => return crate::e(format!("{} not found", part_1)),
                                    };
                                    Some(crate::p2::Boolean::Equal {
                                        left: field.clone(),
                                        right,
                                    })
                                }
                                crate::Value::Boolean { .. } => Some(crate::p2::Boolean::Equal {
                                    left: crate::variable::PropertyValue::Value {
                                        value: value.clone(),
                                    },
                                    right,
                                }),
                                _ => {
                                    return crate::e("if should contain value of type boolean");
                                }
                            };
                        } else {
                            new_header.add(h1, h2);
                        }
                        continue;
                    }

                    if h2.contains("ref") {
                        let reference = ftd_rt::get_name("ref", &*h2)?.to_string();
                        if reference.contains(loop_ref) {
                            let property = {
                                let mut property = Property {
                                    default: Some(crate::variable::PropertyValue::Value {
                                        value: value.clone(),
                                    }),
                                    conditions: vec![],
                                };
                                if let crate::Value::Record { fields, .. } = value.clone() {
                                    if reference.contains('.') {
                                        property = Property::get_property_for_loop(
                                            reference,
                                            &fields,
                                            h1,
                                            root.arguments.get(h1),
                                        )?
                                    }
                                }
                                property
                            };
                            new_properties.insert(h1.to_string(), property);
                            continue;
                        }
                    }
                    new_header.add(h1, h2);
                }

                if let (Some(caption), Some(caption_arg)) = (p1.caption.clone(), new_caption_title)
                {
                    let reference = ftd_rt::get_name("ref", &*caption)?.to_string();
                    if caption.contains("ref") && caption.contains(loop_ref) {
                        let property = if let crate::Value::Record { fields, .. } = value.clone() {
                            Property::get_property_for_loop(
                                reference,
                                &fields,
                                &*caption_arg,
                                root.arguments.get(&*caption_arg),
                            )?
                        } else {
                            Property {
                                default: Some(crate::variable::PropertyValue::Value { value }),
                                conditions: vec![],
                            }
                        };
                        new_properties.insert(caption_arg, property);
                        new_caption = None;
                    }
                }
            }
            _ => unimplemented!("{:?}", value),
        }
        ftd::ChildComponent::from_p1(
            p1.name.as_str(),
            &new_header,
            &new_caption,
            &p1.body,
            doc,
            p1.name.as_str(),
            arguments,
            &new_properties,
            boolean_condition,
            &Default::default(),
        )
    }

    pub fn super_call(
        &self,
        children: &[Self],
        doc: &crate::p2::TDoc,
        arguments: &std::collections::BTreeMap<String, crate::Value>,
        invocations: &mut std::collections::BTreeMap<
            String,
            Vec<std::collections::BTreeMap<String, crate::Value>>,
        >,
        all_locals: &ftd_rt::Map,
        local_container: &[usize],
    ) -> crate::p1::Result<ElementWithContainer> {
        let ElementWithContainer {
            mut element,
            child_container,
            ..
        } = self.call(
            doc,
            arguments,
            invocations,
            false,
            None,
            all_locals,
            local_container,
        )?;
        element.set_container_id(crate::p2::utils::string_optional(
            "id",
            &resolve_properties(&self.properties, arguments, doc, None)?,
        )?);

        match (&mut element, children.is_empty()) {
            (ftd_rt::Element::Column(c), _) => {
                for (i, child) in children.iter().enumerate() {
                    let local_container = {
                        let mut local_container = local_container.to_vec();
                        local_container.push(i);
                        local_container
                    };
                    c.container.children.push(
                        child
                            .call(
                                doc,
                                arguments,
                                invocations,
                                false,
                                None,
                                all_locals,
                                &local_container,
                            )?
                            .element,
                    )
                }
            }
            (ftd_rt::Element::Row(c), _) => {
                for (i, child) in children.iter().enumerate() {
                    let local_container = {
                        let mut local_container = local_container.to_vec();
                        local_container.push(i);
                        local_container
                    };
                    c.container.children.push(
                        child
                            .call(
                                doc,
                                arguments,
                                invocations,
                                false,
                                None,
                                all_locals,
                                &local_container,
                            )?
                            .element,
                    )
                }
            }
            (t, false) => {
                return crate::e2(format!("{:?}", t), "cant have children");
            }
            (_, true) => {}
        }
        Ok(ElementWithContainer {
            element,
            children: vec![],
            child_container,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn recursive_call(
        &self,
        doc: &crate::p2::TDoc,
        arguments: &std::collections::BTreeMap<String, crate::Value>,
        invocations: &mut std::collections::BTreeMap<
            String,
            Vec<std::collections::BTreeMap<String, crate::Value>>,
        >,
        is_child: bool,
        root_name: Option<&str>,
        all_locals: &ftd_rt::Map,
        local_container: &[usize],
    ) -> crate::p1::Result<Vec<ElementWithContainer>> {
        if let Some(ref b) = self.condition {
            if b.is_constant() && !b.eval(arguments, doc)? {
                return Ok(vec![ElementWithContainer {
                    element: ftd_rt::Element::Null,
                    children: vec![],
                    child_container: None,
                }]);
            }
        }

        let root = {
            // NOTE: doing unwrap to force bug report if we following fails, this function
            // must have validated everything, and must not fail at run time
            doc.get_component_with_root(self.root.as_str(), root_name)
                .unwrap()
        };
        let loop_property =
            resolve_recursive_property(&self.properties, arguments, doc, root_name)?;
        let mut element = vec![];

        if let crate::Value::List { data, .. } = loop_property {
            for (i, d) in data.iter().enumerate() {
                let mut new_arguments: std::collections::BTreeMap<String, crate::Value> =
                    arguments.clone();
                new_arguments.insert("$loop$".to_string(), d.clone());
                let new_properties =
                    resolve_properties(&self.properties, &new_arguments, doc, root_name)?;
                let local_container = {
                    let mut container = local_container[..local_container.len() - 1].to_vec();
                    match local_container.last() {
                        Some(val) => container.push(val + i),
                        None => container.push(i),
                    }
                    container
                };

                element.push(root.call(
                    &new_properties,
                    doc,
                    invocations,
                    &self.condition,
                    is_child,
                    doc.get_root(self.root.as_str())?.or(root_name),
                    &self.events,
                    all_locals,
                    local_container.as_slice(),
                )?);
            }
        }
        Ok(element)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn call(
        &self,
        doc: &crate::p2::TDoc,
        arguments: &std::collections::BTreeMap<String, crate::Value>,
        invocations: &mut std::collections::BTreeMap<
            String,
            Vec<std::collections::BTreeMap<String, crate::Value>>,
        >,
        is_child: bool,
        root_name: Option<&str>,
        all_locals: &ftd_rt::Map,
        local_container: &[usize],
    ) -> crate::p1::Result<ElementWithContainer> {
        if let Some(ref b) = self.condition {
            if b.is_constant() && !b.eval(arguments, doc)? {
                return Ok(ElementWithContainer {
                    element: ftd_rt::Element::Null,
                    children: vec![],
                    child_container: None,
                });
            }
        }

        let root = {
            // NOTE: doing unwrap to force bug report if we following fails, this function
            // must have validated everything, and must not fail at run time
            doc.get_component(self.root.as_str()).unwrap()
        };
        let root_properties = resolve_properties(&self.properties, arguments, doc, root_name)?;
        root.call(
            &root_properties,
            doc,
            invocations,
            &self.condition,
            is_child,
            doc.get_root(self.root.as_str())?,
            &self.events,
            all_locals,
            local_container,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn from_p1(
        name: &str,
        p1: &crate::p1::Header,
        caption: &Option<String>,
        body: &Option<String>,
        doc: &crate::p2::TDoc,
        component: &str,
        arguments: &std::collections::BTreeMap<String, crate::p2::Kind>,
        properties: &std::collections::BTreeMap<String, Property>,
        boolean_condition: Option<ftd::p2::Boolean>,
        locals: &std::collections::BTreeMap<String, crate::p2::Kind>,
    ) -> crate::p1::Result<Self> {
        let root = doc.get_component(name)?;
        let mut root_arguments = root.arguments;
        assert_no_extra_properties(p1, root.full_name.as_str(), &root_arguments, name)?;

        Ok(Self {
            properties: read_properties(
                p1,
                caption,
                body,
                "",
                root.full_name.as_str(),
                &mut root_arguments,
                arguments,
                locals,
                doc,
                properties,
            )?,
            condition: match p1.str_optional("if")? {
                Some(expr) => Some(crate::p2::Boolean::from_expression(
                    expr, doc, component, arguments, locals,
                )?),
                None => boolean_condition,
            },
            root: root.full_name.clone(),
            events: p1.get_events(doc, locals)?,
        })
    }
}

fn resolve_recursive_property(
    self_properties: &std::collections::BTreeMap<String, Property>,
    arguments: &std::collections::BTreeMap<String, crate::Value>,
    doc: &crate::p2::TDoc,
    root_name: Option<&str>,
) -> crate::p1::Result<crate::Value> {
    if let Some(value) = self_properties.get("$loop$") {
        if let Ok(property_value) = value.eval("$loop$", arguments, doc) {
            return property_value.resolve_with_root(arguments, doc, root_name);
        }
    }
    crate::e(format!(
        "$loop$ not found in properties {:?}",
        self_properties
    ))
}

fn resolve_properties(
    self_properties: &std::collections::BTreeMap<String, Property>,
    arguments: &std::collections::BTreeMap<String, crate::Value>,
    doc: &crate::p2::TDoc,
    root_name: Option<&str>,
) -> crate::p1::Result<std::collections::BTreeMap<String, crate::Value>> {
    let mut properties: std::collections::BTreeMap<String, crate::Value> = Default::default();
    for (name, value) in self_properties.iter() {
        if name == "$loop$" {
            continue;
        }
        if let Ok(property_value) = value.eval(name, arguments, doc) {
            properties.insert(
                name.to_string(),
                property_value.resolve_with_root(arguments, doc, root_name)?,
            );
        }
    }
    Ok(properties)
}

impl Component {
    fn call_sub_functions(
        &self,
        arguments: &std::collections::BTreeMap<String, crate::Value>,
        doc: &crate::p2::TDoc,
        invocations: &mut std::collections::BTreeMap<
            String,
            Vec<std::collections::BTreeMap<String, crate::Value>>,
        >,
        root_name: Option<&str>,
        call_container: &[usize],
        all_locals: &ftd_rt::Map,
    ) -> crate::p1::Result<ElementWithContainer> {
        ftd::execute_doc::ExecuteDoc {
            name: doc.name,
            aliases: doc.aliases,
            bag: doc.bag,
            instructions: &self.instructions,
            arguments,
            invocations,
            root_name,
        }
        .execute(call_container, all_locals)
    }

    pub fn get_caption(&self) -> Option<String> {
        let mut new_caption_title = None;
        for (arg, arg_kind) in self.arguments.clone() {
            if let crate::p2::Kind::String { caption, .. } = arg_kind {
                if caption {
                    new_caption_title = Some(arg);
                }
            }
        }
        new_caption_title
    }

    pub fn from_p1(p1: &crate::p1::Section, doc: &crate::p2::TDoc) -> crate::p1::Result<Self> {
        let name = ftd_rt::get_name("component", p1.name.as_str())?.to_string();
        let root = p1.header.string("component")?;
        let mut root_arguments = doc.get_component(root.as_str())?.arguments;
        let (arguments, _inherits) =
            read_arguments(&p1.header, root.as_str(), &root_arguments, doc)?;
        let locals = read_locals(&p1.header, doc)?;
        assert_no_extra_properties(&p1.header, root.as_str(), &root_arguments, &p1.name)?;
        let mut instructions: Vec<Instruction> = Default::default();

        for sub in p1.sub_sections.0.iter() {
            if let Ok(loop_data) = sub.header.str("$loop$") {
                let mut loop_ref = "object".to_string();
                let mut loop_on_component = loop_data.to_string();

                if loop_data.contains("as") {
                    let parts = ftd::p2::utils::split(loop_data.to_string(), "as")?;
                    loop_on_component = parts.0;
                    loop_ref = parts.1;
                }

                let (is_arg, v) = match loop_on_component.strip_prefix('$') {
                    Some(v) => (true, v.to_string()),
                    None => (
                        false,
                        doc.resolve_name(loop_on_component.as_str())
                            .unwrap_or_else(|_| loop_on_component.to_string()),
                    ),
                };
                let mut part_2 = None;

                let n = if v.contains('.') {
                    let (p1, p2) = ftd::p2::utils::split(v.to_string(), ".")?;
                    part_2 = Some(p2);
                    p1
                } else {
                    v.to_string()
                };

                let arg = if is_arg {
                    let arg = match arguments.get(n.as_str()) {
                        Some(arg) => arg,
                        None => return crate::e(format!("{} in not present in arguments", v)),
                    };
                    arg.clone()
                } else {
                    doc.get_value(loop_on_component.as_str())?.kind()
                };

                let (recursive_kind, component_loop) = match arg {
                    crate::p2::Kind::Record { ref name } => {
                        let component_loop = doc.resolve_name(name)?;

                        let field = match part_2 {
                            Some(p) => p,
                            None => {
                                return crate::e(format!(
                                    "{} should be a list but it's a {:?}",
                                    n, arg
                                ))
                            }
                        };

                        let rec = doc.get_record(&*component_loop)?;
                        match rec.fields.get(field.as_str()) {
                            Some(crate::p2::Kind::List { .. }) => {}
                            _ => {
                                return crate::e(format!(
                                    "{} is not present in {} of type {:?}",
                                    field, component_loop, rec
                                ));
                            }
                        }

                        (
                            crate::p2::Kind::Record {
                                name: component_loop.to_string(),
                            },
                            component_loop,
                        )
                    }
                    crate::p2::Kind::List { kind } => {
                        let name = match kind.as_ref() {
                            crate::p2::Kind::Record { name } => name,
                            _ => {
                                return crate::e(format!(
                                    "list should be of record type, found: {:?}",
                                    kind
                                ))
                            }
                        };

                        let component_loop = doc.resolve_name(name)?;
                        (
                            crate::p2::Kind::Record {
                                name: component_loop.to_string(),
                            },
                            component_loop,
                        )
                    }
                    _ => unimplemented!(),
                };

                let mut properties: std::collections::BTreeMap<String, Property> =
                    Default::default();

                let property = if is_arg {
                    crate::component::Property {
                        default: Some(crate::PropertyValue::Argument {
                            name: v.to_string(),
                            kind: recursive_kind.clone(),
                        }),
                        conditions: vec![],
                    }
                } else {
                    crate::component::Property {
                        default: Some(crate::PropertyValue::Reference {
                            name: v.to_string(),
                            kind: recursive_kind.clone(),
                        }),
                        conditions: vec![],
                    }
                };
                properties.insert("$loop$".to_string(), property);

                for (k, v) in &sub.header.0 {
                    if k == "$loop$" {
                        continue;
                    }
                    if v.contains(&loop_ref) && v.starts_with("ref ") {
                        let reference = ftd_rt::get_name("ref", &*v)?.to_string();
                        let rec = doc.get_record(&*component_loop)?;
                        if reference.contains('.') {
                            let part_2 = ftd::p2::utils::split(reference, ".")?.1;
                            let field = match rec.fields.get(part_2.as_str()) {
                                Some(field) => field,
                                None => {
                                    return crate::e(format!("{} not present in {:?}", part_2, rec))
                                }
                            };
                            let property = crate::component::Property {
                                default: Some(crate::PropertyValue::Argument {
                                    name: format!("$loop$.{}", part_2),
                                    kind: field.clone(),
                                }),
                                conditions: vec![],
                            };
                            properties.insert(k.to_string(), property);
                        } else {
                            let kind = crate::p2::Kind::Record {
                                name: component_loop.to_string(),
                            };
                            let property = crate::component::Property {
                                default: Some(crate::PropertyValue::Argument {
                                    name: "$loop$".to_string(),
                                    kind,
                                }),
                                conditions: vec![],
                            };
                            properties.insert(k.to_string(), property);
                        }
                    }
                }

                instructions.push(Instruction::RecursiveChildComponent {
                    child: ftd::ChildComponent {
                        root: sub.name.to_string(),
                        condition: match sub.header.str_optional("if")? {
                            Some(expr) => Some(crate::p2::Boolean::from_expression(
                                expr,
                                doc,
                                sub.name.as_str(),
                                &arguments,
                                &locals,
                            )?),
                            None => None,
                        },
                        properties,
                        events: vec![],
                    },
                });
                continue;
            }

            instructions.push(if sub.name == "container" {
                Instruction::ChangeContainer {
                    name: doc.resolve_name_without_full_path(sub.caption()?.as_str())?,
                }
            } else {
                let s = ChildComponent::from_p1(
                    sub.name.as_str(),
                    &sub.header,
                    &sub.caption,
                    &sub.body,
                    doc,
                    name.as_str(),
                    &arguments,
                    &std::collections::BTreeMap::new(),
                    None,
                    &locals,
                )?;
                Instruction::ChildComponent { child: s }
            });
        }

        let condition = match p1.header.str_optional("if")? {
            Some(expr) => Some(crate::p2::Boolean::from_expression(
                expr,
                doc,
                p1.name.as_str(),
                &arguments,
                &locals,
            )?),
            None => None,
        };

        let events = p1.header.get_events(doc, &locals)?;

        Ok(Component {
            full_name: doc.resolve_name(&name)?,
            properties: read_properties(
                &p1.header,
                &p1.caption,
                &p1.body,
                name.as_str(),
                root.as_str(),
                &mut root_arguments,
                &arguments,
                &locals,
                doc,
                &std::collections::BTreeMap::new(),
            )?,
            arguments,
            locals,
            root,
            instructions,
            kernel: false,
            invocations: Default::default(),
            condition,
            events,
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn call(
        &self,
        arguments: &std::collections::BTreeMap<String, crate::Value>,
        doc: &crate::p2::TDoc,
        invocations: &mut std::collections::BTreeMap<
            String,
            Vec<std::collections::BTreeMap<String, crate::Value>>,
        >,
        condition: &Option<ftd::p2::Boolean>,
        is_child: bool,
        root_name: Option<&str>,
        events: &[ftd::p2::expression::Event],
        all_locals: &ftd_rt::Map,
        local_container: &[usize],
    ) -> crate::p1::Result<ElementWithContainer> {
        invocations
            .entry(self.full_name.clone())
            .or_default()
            .push(arguments.to_owned());

        if self.root == "ftd.kernel" {
            let element = match self.full_name.as_str() {
                "ftd#text" => ftd_rt::Element::Text(ftd::p2::element::text_from_properties(
                    arguments, doc, condition, is_child, events, all_locals,
                )?),
                "ftd#image" => ftd_rt::Element::Image(ftd::p2::element::image_from_properties(
                    arguments, doc, condition, is_child, events, all_locals,
                )?),
                "ftd#row" => ftd_rt::Element::Row(ftd::p2::element::row_from_properties(
                    arguments, doc, condition, is_child, events, all_locals,
                )?),
                "ftd#column" => ftd_rt::Element::Column(ftd::p2::element::column_from_properties(
                    arguments, doc, condition, is_child, events, all_locals,
                )?),
                "ftd#iframe" => ftd_rt::Element::IFrame(ftd::p2::element::iframe_from_properties(
                    arguments, doc, condition, is_child, events, all_locals,
                )?),
                "ftd#integer" => {
                    ftd_rt::Element::Integer(ftd::p2::element::integer_from_properties(
                        arguments, doc, condition, is_child, events, all_locals,
                    )?)
                }
                "ftd#decimal" => {
                    ftd_rt::Element::Decimal(ftd::p2::element::decimal_from_properties(
                        arguments, doc, condition, is_child, events, all_locals,
                    )?)
                }
                "ftd#boolean" => {
                    ftd_rt::Element::Boolean(ftd::p2::element::boolean_from_properties(
                        arguments, doc, condition, is_child, events, all_locals,
                    )?)
                }
                "ftd#input" => ftd_rt::Element::Input(ftd::p2::element::input_from_properties(
                    arguments, doc, condition, is_child, events, all_locals,
                )?),
                _ => unreachable!(),
            };
            Ok(ElementWithContainer {
                element,
                children: vec![],
                child_container: None,
            })
        } else {
            let root = {
                // NOTE: doing unwrap to force bug report if we following fails, this function
                // must have validated everything, and must not fail at run time
                doc.get_component(self.root.as_str()).unwrap()
            };
            let root_properties = {
                let mut properties =
                    resolve_properties(&self.properties, arguments, doc, root_name)?;
                if !properties.contains_key("id") {
                    if let Some(id) = arguments.get("id") {
                        properties.insert("id".to_string(), id.to_owned());
                    }
                }
                properties
            };
            let mut element = root
                .call(
                    &root_properties,
                    doc,
                    invocations,
                    condition,
                    is_child,
                    root_name,
                    events,
                    all_locals,
                    local_container,
                )?
                .element;

            let local_string_container: String = local_container
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<String>>()
                .join(",");

            let mut all_locals: ftd_rt::Map = Default::default();

            element.set_locals(self.get_locals_map(&local_string_container, &mut all_locals)?);

            if condition.is_none() {
                element.set_condition({
                    match &self.condition {
                        Some(c) if !c.is_constant() => Some(c.to_condition(&all_locals)?),
                        _ => None,
                    }
                });
            }

            element.set_events(&mut ftd::p2::expression::Event::get_events(
                &self.events,
                &all_locals,
            )?);

            let mut containers = None;
            match &mut element {
                ftd_rt::Element::Text(_)
                | ftd_rt::Element::Image(_)
                | ftd_rt::Element::IFrame(_)
                | ftd_rt::Element::Input(_)
                | ftd_rt::Element::Integer(_)
                | ftd_rt::Element::Decimal(_)
                | ftd_rt::Element::Boolean(_)
                | ftd_rt::Element::Null => {}
                ftd_rt::Element::Column(ref mut e) => {
                    let ElementWithContainer {
                        children,
                        child_container,
                        ..
                    } = self.call_sub_functions(
                        arguments,
                        doc,
                        invocations,
                        root_name,
                        local_container,
                        &all_locals,
                    )?;
                    containers = child_container;
                    e.container.children = children;
                }
                ftd_rt::Element::Row(ref mut e) => {
                    let ElementWithContainer {
                        children,
                        child_container,
                        ..
                    } = self.call_sub_functions(
                        arguments,
                        doc,
                        invocations,
                        root_name,
                        local_container,
                        &all_locals,
                    )?;
                    containers = child_container;
                    e.container.children = children;
                }
            }

            Ok(ElementWithContainer {
                element,
                children: vec![],
                child_container: containers,
            })
        }
    }

    fn get_locals_map(
        &self,
        string_container: &str,
        all_locals: &mut ftd_rt::Map,
    ) -> crate::p1::Result<ftd_rt::Map> {
        let mut locals: ftd_rt::Map = Default::default();
        for (k, v) in &self.locals {
            let value = match v {
                Kind::String { default: Some(d), .. } => d,
                Kind::Integer { default: Some(d) } => d,
                Kind::Decimal { default: Some(d) } => d,
                Kind::Boolean { default: Some(d) } => d,
                _ => return crate::e("local variable supports string, integer, boolean and decimal type with default value"),
            };
            locals.insert(format!("{}@{}", k, string_container), value.to_string());
            all_locals.insert(k.to_string(), string_container.to_string());
        }
        Ok(locals)
    }
}

fn is_component(name: &str) -> bool {
    !(name.starts_with("component ")
        || name.starts_with("var ")
        || name.starts_with("record ")
        || name.starts_with("or-type")
        || name.starts_with("list ")
        || name.starts_with("map ")
        || (name == "container")
        || (name == "ftd.text")
        || (name == "ftd.image")
        || (name == "ftd.row")
        || (name == "ftd.column")
        || (name == "ftd.iframe")
        || (name == "ftd.integer")
        || (name == "ftd.decimal")
        || (name == "ftd.boolean")
        || (name == "ftd.input"))
}

fn assert_no_extra_properties(
    p1: &crate::p1::Header,
    root: &str,
    root_arguments: &std::collections::BTreeMap<String, crate::p2::Kind>,
    name: &str,
) -> crate::p1::Result<()> {
    for (k, _) in p1.0.iter() {
        if k == "component" || k.starts_with('$') || k.starts_with('@') || k == "if" {
            continue;
        }
        let key = if k.contains(" if ") {
            let mut parts = k.splitn(2, " if ");
            parts.next().unwrap().trim()
        } else {
            k
        };

        if !(root_arguments.contains_key(key) || (is_component(name) && key == "id")) {
            return crate::e(format!(
                "unknown key found: {}, {} has: {}",
                k,
                root,
                root_arguments
                    .keys()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
    }

    Ok(())
}

pub fn read_value(
    name: &str,
    value: &str,
    source: crate::TextSource,
    kind: &crate::p2::Kind,
) -> crate::p1::Result<crate::Value> {
    match kind.inner() {
        crate::p2::Kind::Integer { .. } => {
            if let Ok(v) = value.parse::<i64>() {
                return Ok(crate::Value::Integer { value: v });
            }
        }
        crate::p2::Kind::Boolean { .. } => {
            if let Ok(v) = value.parse::<bool>() {
                return Ok(crate::Value::Boolean { value: v });
            }
        }
        crate::p2::Kind::Decimal { .. } => {
            if let Ok(v) = value.parse::<f64>() {
                return Ok(crate::Value::Decimal { value: v });
            }
        }
        crate::p2::Kind::String { .. } => {
            return Ok(crate::Value::String {
                text: value.to_string(),
                source,
            });
        }
        _ => {
            todo!("{:?} not yet implemented", kind)
        }
    }

    crate::e(format!("'{}' is not a valid {:?}: {}", name, kind, value))
}

fn read_reference(
    name: &str,
    value: &str,
    kind: &crate::p2::Kind,
    doc: &crate::p2::TDoc,
    arguments: &std::collections::BTreeMap<String, crate::p2::Kind>,
) -> crate::p1::Result<crate::PropertyValue> {
    let ref_name = ftd_rt::get_name("ref", value)?.to_string();
    match ref_name.as_str().strip_prefix('$') {
        Some(v) => {
            let found_kind = if v.contains('.') {
                let (part_1, part_2) = ftd::p2::utils::split(v.to_string(), ".")?;

                let rec_name = match arguments.get(part_1.as_str()) {
                    Some(crate::p2::Kind::Record { name }) => name,
                    _ => return crate::e(format!("'{}' is not an argument of '{}'", v, name)),
                };

                let record = doc.get_record(rec_name)?;
                match record.fields.get(part_2.as_str()) {
                    Some(field) => field.clone(),
                    None => {
                        return crate::e(format!("'{}' is not an argument of '{}'", v, rec_name))
                    }
                }
            } else {
                match arguments.get(v) {
                    Some(k) => k.clone(),
                    None => {
                        return crate::e(format!("'{}' is not an argument of '{}'", v, name));
                    }
                }
            };
            if !found_kind.is_same_as(kind) {
                return crate::e(format!(
                    "'{}' is expected to be {:?}, but its referring to '{}', which is {:?}",
                    name, kind, ref_name, found_kind
                ));
            }
            Ok(crate::PropertyValue::Argument {
                name: v.to_string(),
                kind: kind.to_owned(),
            })
        }
        None => {
            let found_kind = doc.get_value(ref_name.as_str())?.kind();
            if !found_kind.is_same_as(kind) {
                return crate::e(format!(
                    "'{}' is expected to be {:?}, but its referring to '{}', which is {:?}",
                    name, kind, ref_name, found_kind
                ));
            }
            Ok(crate::PropertyValue::Reference {
                name: doc.resolve_name(ref_name.as_str()).unwrap_or(ref_name),
                kind: kind.to_owned(),
            })
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn read_properties(
    p1: &crate::p1::Header,
    caption: &Option<String>,
    body: &Option<String>,
    fn_name: &str,
    root: &str,
    root_arguments: &mut std::collections::BTreeMap<String, crate::p2::Kind>,
    arguments: &std::collections::BTreeMap<String, crate::p2::Kind>,
    locals: &std::collections::BTreeMap<String, crate::p2::Kind>,
    doc: &crate::p2::TDoc,
    root_properties: &std::collections::BTreeMap<String, Property>,
) -> crate::p1::Result<std::collections::BTreeMap<String, Property>> {
    let mut properties: std::collections::BTreeMap<String, Property> = Default::default();
    let id_already_present = root_arguments.contains_key("id");
    if !id_already_present {
        // to add "id" property by default for component as "-- component foo:"
        root_arguments.insert(
            "id".to_string(),
            crate::p2::Kind::Optional {
                kind: Box::new(crate::p2::Kind::string()),
            },
        );
    }

    for (name, kind) in root_arguments.iter() {
        if let Some(prop) = root_properties.get(name) {
            properties.insert(name.to_string(), prop.clone());
            continue;
        }
        let (conditional_vector, source) = match (p1.conditional_str(name), kind.inner()) {
            (Ok(v), _) => (v, ftd::TextSource::Header),
            (
                Err(crate::p1::Error::NotFound { .. }),
                crate::p2::Kind::String {
                    caption: c,
                    body: b,
                    default: d,
                },
            ) => {
                if *c && caption.is_some() {
                    (
                        vec![(caption.as_ref().unwrap().to_string(), None)],
                        ftd::TextSource::Caption,
                    )
                } else if *b && body.is_some() {
                    (
                        vec![(body.as_ref().unwrap().to_string(), None)],
                        ftd::TextSource::Body,
                    )
                } else if matches!(kind, crate::p2::Kind::Optional { .. }) {
                    continue;
                } else if let Some(d) = d {
                    (vec![(d.to_string(), None)], ftd::TextSource::Default)
                } else {
                    return crate::e(format!(
                        "{} is calling {}, without a required argument `{}`",
                        fn_name, root, name,
                    ));
                }
            }
            (Err(crate::p1::Error::NotFound { .. }), k) => {
                if matches!(kind, crate::p2::Kind::Optional { .. }) {
                    continue;
                }

                if let Some(d) = k.get_default_value_str() {
                    (vec![(d.to_string(), None)], ftd::TextSource::Default)
                } else {
                    return crate::e(format!(
                        "{} is calling {}, without a required argument `{}`",
                        fn_name, root, name,
                    ));
                }
            }
            (Err(e), _) => {
                return Err(e);
            }
        };
        for (value, conditional_attribute) in conditional_vector {
            let property_value = if value.starts_with("ref ") {
                read_reference(name, value.as_ref(), kind, doc, arguments)?
            } else {
                crate::PropertyValue::Value {
                    value: read_value(name, value.as_ref(), source.clone(), kind)?,
                }
            };
            let (condition_value, default_value) = if let Some(attribute) = conditional_attribute {
                let condition =
                    crate::p2::Boolean::from_expression(attribute, doc, "", arguments, locals)?;
                (vec![(condition, property_value)], None)
            } else {
                (vec![], Some(property_value))
            };
            if let Some(property) = properties.get_mut(name) {
                if default_value.is_some() {
                    property.default = default_value;
                } else {
                    property.conditions.append(&mut condition_value.clone());
                }
            } else {
                let value = Property {
                    default: default_value,
                    conditions: condition_value,
                };
                properties.insert(name.to_string(), value);
            }
        }
    }
    Ok(properties)
}

fn read_arguments(
    p1: &crate::p1::Header,
    root: &str,
    root_arguments: &std::collections::BTreeMap<String, crate::p2::Kind>,
    doc: &crate::p2::TDoc,
) -> crate::p1::Result<(
    std::collections::BTreeMap<String, crate::p2::Kind>,
    Vec<String>,
)> {
    let mut args: std::collections::BTreeMap<String, crate::p2::Kind> = Default::default();
    let mut inherits: Vec<String> = Default::default();

    for (k, v) in p1.0.iter() {
        if k.starts_with('$') && k.ends_with('$') {
            // event and loop matches
            continue;
        }

        let name = match k.strip_prefix('$') {
            Some(v) => v,
            None => {
                continue;
            }
        };

        let kind = if v == "inherit" {
            match root_arguments.get(name) {
                Some(v) => {
                    inherits.push(name.to_string());
                    v.clone()
                }
                None => return crate::e(format!("'{}' is not an argument of {}", name, root)),
            }
        } else {
            crate::p2::Kind::from(v, doc, None)?
        };
        args.insert(name.to_string(), kind);
    }

    Ok((args, inherits))
}

fn read_locals(
    p1: &crate::p1::Header,
    doc: &crate::p2::TDoc,
) -> crate::p1::Result<std::collections::BTreeMap<String, crate::p2::Kind>> {
    let mut args: std::collections::BTreeMap<String, crate::p2::Kind> = Default::default();

    for (k, v) in p1.0.iter() {
        let name = match k.strip_prefix('@') {
            Some(v) => v,
            None => {
                continue;
            }
        };
        let kind = crate::p2::Kind::from(v, doc, None)?;
        args.insert(name.to_string(), kind);
    }
    Ok(args)
}

#[cfg(test)]
mod test {
    use crate::component::Property;
    use crate::test::*;

    macro_rules! p2 {
        ($s:expr, $doc: expr, $t: expr,) => {
            p2!($s, $doc, $t)
        };
        ($s:expr, $doc: expr, $t: expr) => {
            let p1 = crate::p1::parse(indoc::indoc!($s)).unwrap();
            pretty_assertions::assert_eq!(super::Component::from_p1(&p1[0], &$doc).unwrap(), $t)
        };
    }

    fn s(s: &str) -> String {
        s.to_string()
    }

    #[test]
    fn component() {
        let mut bag = crate::p2::interpreter::default_bag();
        let aliases = crate::p2::interpreter::default_aliases();
        let d = crate::p2::TDoc {
            name: "foo",
            bag: &mut bag,
            aliases: &aliases,
        };
        p2!(
            "-- component foo:
            component: ftd.text
            $foo: string
            $bar: optional integer
            text: hello
            ",
            d,
            super::Component {
                full_name: s("foo#foo"),
                root: "ftd.text".to_string(),
                arguments: std::array::IntoIter::new([
                    (s("foo"), crate::p2::Kind::string()),
                    (
                        s("bar"),
                        crate::p2::Kind::optional(crate::p2::Kind::integer())
                    )
                ])
                .collect(),
                properties: std::array::IntoIter::new([(
                    s("text"),
                    Property {
                        default: Some(crate::PropertyValue::Value {
                            value: crate::Value::String {
                                text: s("hello"),
                                source: crate::TextSource::Header
                            }
                        }),
                        conditions: vec![]
                    }
                ),])
                .collect(),
                ..Default::default()
            }
        );
    }

    #[test]
    fn properties() {
        let mut bag = crate::p2::interpreter::default_bag();
        let aliases = crate::p2::interpreter::default_aliases();
        let d = crate::p2::TDoc {
            name: "foo",
            bag: &mut bag,
            aliases: &aliases,
        };
        p2!(
            "-- component foo:
            component: ftd.text
            text: hello
            ",
            d,
            super::Component {
                root: "ftd.text".to_string(),
                full_name: s("foo#foo"),
                properties: std::array::IntoIter::new([(
                    s("text"),
                    Property {
                        default: Some(crate::PropertyValue::Value {
                            value: crate::Value::String {
                                text: s("hello"),
                                source: crate::TextSource::Header
                            }
                        }),
                        conditions: vec![]
                    }
                ),])
                .collect(),
                ..Default::default()
            }
        );
    }

    #[test]
    fn referring_variables() {
        let mut bag = default_bag();
        bag.insert(
            "foo/bar#name".to_string(),
            crate::p2::Thing::Variable(crate::Variable {
                name: "name".to_string(),
                value: crate::Value::String {
                    text: s("Amit"),
                    source: crate::TextSource::Caption,
                },
            }),
        );
        let mut main = default_column();
        main.container
            .children
            .push(ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("Amit"),
                line: true,
                ..Default::default()
            }));

        p!(
            "
            -- var name: Amit
            -- ftd.text:
            text: ref name
            ",
            (bag.clone(), main.clone()),
        );

        p!(
            "
            -- var name: Amit
            -- ftd.text: ref name
            ",
            (bag.clone(), main.clone()),
        );

        p!(
            "
            -- var name: Amit
            -- ftd.text:
            ref name
            ",
            (bag, main),
        );
    }

    #[test]
    #[ignore]
    fn referring_record_fields() {
        let mut bag = default_bag();
        bag.insert(
            "foo/bar#person".to_string(),
            crate::p2::Thing::Record(crate::p2::Record {
                name: "foo/bar#person".to_string(),
                fields: person_fields(),
                instances: Default::default(),
            }),
        );
        bag.insert(
            "foo/bar#x".to_string(),
            crate::p2::Thing::Variable(crate::Variable {
                name: "x".to_string(),
                value: crate::Value::Integer { value: 20 },
            }),
        );
        bag.insert(
            "foo/bar#abrar".to_string(),
            crate::p2::Thing::Variable(crate::Variable {
                name: "abrar".to_string(),
                value: crate::Value::Record {
                    name: "foo/bar#person".to_string(),
                    fields: abrar(),
                },
            }),
        );

        let mut main = default_column();
        main.container
            .children
            .push(ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("Abrar Khan"),
                line: true,
                ..Default::default()
            }));

        p!(
            "
            -- record person:
            name: caption
            address: string
            bio: body
            age: integer

            -- var x: 10

            -- var abrar: Abrar Khan
            type: person
            address: Bihar
            age: ref x

            Software developer working at fifthtry.

            -- ftd.text:
            text: ref abrar.name
            ",
            (bag.clone(), main.clone()),
        );
    }
}
