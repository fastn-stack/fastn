#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Component {
    pub root: String,
    pub full_name: String,
    pub arguments: std::collections::BTreeMap<String, crate::p2::Kind>,
    pub locals: std::collections::BTreeMap<String, crate::p2::Kind>,
    pub properties: std::collections::BTreeMap<String, Property>,
    pub instructions: Vec<Instruction>,
    pub events: Vec<ftd::p2::Event>,
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
    pub events: Vec<ftd::p2::Event>,
    pub is_recursive: bool,
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Property {
    pub default: Option<crate::PropertyValue>,
    pub conditions: Vec<(crate::p2::Boolean, crate::PropertyValue)>,
}

#[derive(Debug, Clone)]
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
        let mut property_value = ftd::e2(format!("{:?}", name), "condition is not complete");
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
            (ftd_rt::Element::Column(ftd_rt::Column { container, .. }), _)
            | (ftd_rt::Element::Row(ftd_rt::Row { container, .. }), _) => {
                for (i, child) in children.iter().enumerate() {
                    let local_container = {
                        let mut local_container = local_container.to_vec();
                        local_container.push(i);
                        local_container
                    };
                    if child.is_recursive {
                        container.children.extend(
                            child
                                .recursive_call(
                                    doc,
                                    arguments,
                                    invocations,
                                    false,
                                    None,
                                    all_locals,
                                    &local_container,
                                )?
                                .iter()
                                .map(|c| c.element.clone())
                                .collect::<Vec<ftd_rt::Element>>(),
                        );
                        continue;
                    }
                    container.children.push(
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
        let root = {
            // NOTE: doing unwrap to force bug report if we following fails, this function
            // must have validated everything, and must not fail at run time
            doc.get_component_with_root(self.root.as_str(), root_name)
                .unwrap()
        };
        let loop_property =
            resolve_recursive_property(&self.properties, arguments, doc, root_name)?;
        let mut elements = vec![];

        if let crate::Value::List { data, .. } = loop_property {
            for (i, d) in data.iter().enumerate() {
                let mut new_arguments: std::collections::BTreeMap<String, crate::Value> =
                    arguments.clone();
                new_arguments.insert("$loop$".to_string(), d.clone());
                let new_properties =
                    resolve_properties_with_ref(&self.properties, &new_arguments, doc, root_name)?;
                let local_container = {
                    let mut container = local_container[..local_container.len() - 1].to_vec();
                    match local_container.last() {
                        Some(val) => container.push(val + i),
                        None => container.push(i),
                    }
                    container
                };
                let is_visible = {
                    let mut visible = true;
                    if let Some(ref b) = self.condition {
                        if b.is_constant() && !b.eval(&new_arguments, doc)? {
                            visible = false;
                            if let Ok(true) = b.set_null() {
                                elements.push(ElementWithContainer {
                                    element: ftd_rt::Element::Null,
                                    children: vec![],
                                    child_container: None,
                                });
                                continue;
                            }
                        }
                    }
                    visible
                };

                let mut element = root.call(
                    &new_properties,
                    doc,
                    invocations,
                    &self.condition,
                    is_child,
                    doc.get_root(self.root.as_str())?.or(root_name),
                    &self.events,
                    all_locals,
                    local_container.as_slice(),
                )?;

                if let Some(condition) = &self.condition {
                    element.element.set_condition(
                        condition.to_condition(all_locals, &Default::default()).ok(),
                    );
                }
                if !is_visible {
                    element.element.set_non_visibility(!is_visible);
                }
                elements.push(element);
            }
        }
        Ok(elements)
    }

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
        let is_visible = {
            let mut visible = true;
            if let Some(ref b) = self.condition {
                if b.is_constant() && !b.eval(arguments, doc)? {
                    visible = false;
                    if let Ok(true) = b.set_null() {
                        return Ok(ElementWithContainer {
                            element: ftd_rt::Element::Null,
                            children: vec![],
                            child_container: None,
                        });
                    }
                }
            }
            visible
        };

        let root = {
            // NOTE: doing unwrap to force bug report if we following fails, this function
            // must have validated everything, and must not fail at run time
            doc.get_component(self.root.as_str()).unwrap()
        };

        let root_properties = {
            let mut root_properties =
                resolve_properties_with_ref(&self.properties, arguments, doc, root_name)?;
            //pass argument of component to its children
            for (k, v) in arguments {
                if !root_properties.contains_key(k) {
                    root_properties.insert(format!("${}", k), (v.to_owned(), None));
                }
            }
            root_properties
        };

        let mut element = root.call(
            &root_properties,
            doc,
            invocations,
            &self.condition,
            is_child,
            doc.get_root(self.root.as_str())?,
            &self.events,
            all_locals,
            local_container,
        )?;

        if let Some(condition) = &self.condition {
            element
                .element
                .set_condition(condition.to_condition(all_locals, &Default::default()).ok());
        }
        if !is_visible {
            element.element.set_non_visibility(!is_visible);
        }

        Ok(element)
    }

    pub fn from_p1(
        name: &str,
        p1: &crate::p1::Header,
        caption: &Option<String>,
        body: &Option<String>,
        doc: &crate::p2::TDoc,
        arguments: &std::collections::BTreeMap<String, crate::p2::Kind>,
        locals: &std::collections::BTreeMap<String, crate::p2::Kind>,
    ) -> crate::p1::Result<Self> {
        let root = doc.get_component(name)?;
        let mut root_arguments = root.arguments;
        assert_no_extra_properties(p1, root.full_name.as_str(), &root_arguments, name)?;
        let root_property = get_root_property(name, caption, doc);

        return Ok(Self {
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
                &root_property,
            )?,
            condition: match p1.str_optional("if")? {
                Some(expr) => Some(crate::p2::Boolean::from_expression(
                    expr,
                    doc,
                    arguments,
                    locals,
                    (None, None),
                )?),
                None => None,
            },
            root: root.full_name.clone(),
            events: p1.get_events(doc, locals, arguments)?,
            is_recursive: false,
        });

        fn get_root_property(
            name: &str,
            caption: &Option<String>,
            doc: &ftd::p2::TDoc,
        ) -> std::collections::BTreeMap<String, Property> {
            let mut properties: std::collections::BTreeMap<String, Property> = Default::default();
            if let Some(caption) = caption {
                if let Ok(name) = doc.resolve_name(name) {
                    let kind = match name.as_str() {
                        "ftd#integer" => ftd::p2::Kind::integer(),
                        "ftd#boolean" => ftd::p2::Kind::boolean(),
                        "ftd#decimal" => ftd::p2::Kind::decimal(),
                        _ => return properties,
                    };
                    if let Ok(property_value) = ftd::PropertyValue::resolve_value(
                        caption,
                        Some(kind),
                        doc,
                        &Default::default(),
                        &Default::default(),
                        None,
                        true,
                    ) {
                        properties.insert(
                            "value".to_string(),
                            Property {
                                default: Some(property_value),
                                conditions: vec![],
                            },
                        );
                    }
                }
            }
            properties
        }
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

fn resolve_properties_with_ref(
    self_properties: &std::collections::BTreeMap<String, Property>,
    arguments: &std::collections::BTreeMap<String, crate::Value>,
    doc: &crate::p2::TDoc,
    root_name: Option<&str>,
) -> crate::p1::Result<std::collections::BTreeMap<String, (crate::Value, Option<String>)>> {
    let mut properties: std::collections::BTreeMap<String, (crate::Value, Option<String>)> =
        Default::default();
    for (name, value) in self_properties.iter() {
        if name == "$loop$" {
            continue;
        }
        if let Ok(property_value) = value.eval(name, arguments, doc) {
            let reference = match property_value {
                ftd::PropertyValue::Reference { name, .. } => Some(name.to_string()),
                ftd::PropertyValue::LocalVariable { name, .. } => Some(format!("@{}", name)),
                _ => None,
            };
            properties.insert(
                name.to_string(),
                (
                    property_value.resolve_with_root(arguments, doc, root_name)?,
                    reference,
                ),
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
        let root_component = doc.get_component(root.as_str())?;
        let mut root_arguments = root_component.arguments.clone();
        let (arguments, _inherits) =
            read_arguments(&p1.header, root.as_str(), &root_arguments, doc)?;
        let locals = read_locals(&p1.header, doc)?;

        assert_no_extra_properties(&p1.header, root.as_str(), &root_arguments, &p1.name)?;
        let mut instructions: Vec<Instruction> = Default::default();

        for sub in p1.sub_sections.0.iter() {
            if sub.is_commented {
                continue;
            }
            if let Ok(loop_data) = sub.header.str("$loop$") {
                instructions.push(Instruction::RecursiveChildComponent {
                    child: recursive_child_component(
                        loop_data,
                        sub,
                        doc,
                        &arguments,
                        Some((name.to_string(), root_component.to_owned())),
                        &locals,
                    )?,
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
                    &sub.body_without_comment(),
                    doc,
                    &arguments,
                    &locals,
                )?;
                Instruction::ChildComponent { child: s }
            });
        }

        let condition = match p1.header.str_optional("if")? {
            Some(expr) => Some(crate::p2::Boolean::from_expression(
                expr,
                doc,
                &arguments,
                &locals,
                (None, None),
            )?),
            None => None,
        };

        let events = p1.header.get_events(doc, &locals, &arguments)?;

        Ok(Component {
            full_name: doc.resolve_name(&name)?,
            properties: read_properties(
                &p1.header,
                &p1.caption,
                &p1.body_without_comment(),
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
        arguments: &std::collections::BTreeMap<String, (crate::Value, Option<String>)>,
        doc: &crate::p2::TDoc,
        invocations: &mut std::collections::BTreeMap<
            String,
            Vec<std::collections::BTreeMap<String, crate::Value>>,
        >,
        condition: &Option<ftd::p2::Boolean>,
        is_child: bool,
        root_name: Option<&str>,
        events: &[ftd::p2::Event],
        all_locals: &ftd_rt::Map,
        local_container: &[usize],
    ) -> crate::p1::Result<ElementWithContainer> {
        let argument = ftd::p2::utils::properties(arguments);
        invocations
            .entry(self.full_name.clone())
            .or_default()
            .push(argument.to_owned());

        if self.root == "ftd.kernel" {
            let element = match self.full_name.as_str() {
                "ftd#text" => ftd_rt::Element::Text(ftd::p2::element::text_from_properties(
                    arguments, doc, condition, is_child, events, all_locals, root_name,
                )?),
                "ftd#image" => ftd_rt::Element::Image(ftd::p2::element::image_from_properties(
                    arguments, doc, condition, is_child, events, all_locals, root_name,
                )?),
                "ftd#row" => ftd_rt::Element::Row(ftd::p2::element::row_from_properties(
                    arguments, doc, condition, is_child, events, all_locals, root_name,
                )?),
                "ftd#column" => ftd_rt::Element::Column(ftd::p2::element::column_from_properties(
                    arguments, doc, condition, is_child, events, all_locals, root_name,
                )?),
                "ftd#iframe" => ftd_rt::Element::IFrame(ftd::p2::element::iframe_from_properties(
                    arguments, doc, condition, is_child, events, all_locals, root_name,
                )?),
                "ftd#integer" => {
                    ftd_rt::Element::Integer(ftd::p2::element::integer_from_properties(
                        arguments, doc, condition, is_child, events, all_locals, root_name,
                    )?)
                }
                "ftd#decimal" => {
                    ftd_rt::Element::Decimal(ftd::p2::element::decimal_from_properties(
                        arguments, doc, condition, is_child, events, all_locals, root_name,
                    )?)
                }
                "ftd#boolean" => {
                    ftd_rt::Element::Boolean(ftd::p2::element::boolean_from_properties(
                        arguments, doc, condition, is_child, events, all_locals, root_name,
                    )?)
                }
                "ftd#input" => ftd_rt::Element::Input(ftd::p2::element::input_from_properties(
                    arguments, doc, condition, is_child, events, all_locals, root_name,
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
            let arguments = ftd::p2::utils::properties(arguments);
            let root_properties = {
                let mut properties =
                    resolve_properties_with_ref(&self.properties, &argument, doc, root_name)?;
                if !properties.contains_key("id") {
                    if let Some(id) = arguments.get("id") {
                        properties.insert("id".to_string(), (id.to_owned(), None));
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
                let mut is_visible = true;
                element.set_condition({
                    match &self.condition {
                        Some(c) if !c.is_arg_constant() => {
                            if !c.eval(&arguments, doc)? {
                                is_visible = false;
                            }
                            Some(c.to_condition(&all_locals, &arguments)?)
                        }
                        _ => None,
                    }
                });
                element.set_non_visibility(!is_visible);
            }

            element.set_events(&mut ftd::p2::Event::get_events(
                &self.events,
                &all_locals,
                &arguments,
                doc,
                root_name,
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
                ftd_rt::Element::Column(ftd_rt::Column {
                    ref mut container, ..
                })
                | ftd_rt::Element::Row(ftd_rt::Row {
                    ref mut container, ..
                }) => {
                    let ElementWithContainer {
                        children,
                        child_container,
                        ..
                    } = self.call_sub_functions(
                        &argument,
                        doc,
                        invocations,
                        root_name,
                        local_container,
                        &all_locals,
                    )?;
                    containers = child_container;
                    container.children = children;
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
                ftd::p2::Kind::String { default: Some(d), .. } |
                ftd::p2::Kind::Integer { default: Some(d) } |
                ftd::p2::Kind::Decimal { default: Some(d) } |
                ftd::p2::Kind::Boolean { default: Some(d) } => d,
                _ => return crate::e("local variable supports string, integer, boolean and decimal type with default value"),
            };
            locals.insert(format!("{}@{}", k, string_container), value.to_string());
            all_locals.insert(k.to_string(), string_container.to_string());
        }
        Ok(locals)
    }
}

pub fn recursive_child_component(
    loop_data: &str,
    sub: &ftd::p1::SubSection,
    doc: &ftd::p2::TDoc,
    arguments: &std::collections::BTreeMap<String, ftd::p2::Kind>,
    name_with_component: Option<(String, ftd::Component)>,
    locals: &std::collections::BTreeMap<String, ftd::p2::Kind>,
) -> ftd::p1::Result<ftd::ChildComponent> {
    let mut loop_ref = "object".to_string();
    let mut loop_on_component = loop_data.to_string();

    if loop_data.contains("as") {
        let parts = ftd::p2::utils::split(loop_data.to_string(), "as")?;
        loop_on_component = parts.0;
        loop_ref = parts.1;
    }

    let recursive_property_value = ftd::PropertyValue::resolve_value(
        &loop_on_component,
        None,
        doc,
        arguments,
        locals,
        None,
        false,
    )?;

    let recursive_kind = if let ftd::p2::Kind::List { kind } = recursive_property_value.kind() {
        kind.as_ref().to_owned()
    } else {
        return ftd::e(format!(
            "expected list for loop, found: {:?}",
            recursive_property_value.kind()
        ));
    };

    let mut properties: std::collections::BTreeMap<String, Property> = Default::default();

    properties.insert(
        "$loop$".to_string(),
        ftd::component::Property {
            default: Some(recursive_property_value),
            conditions: vec![],
        },
    );

    let mut new_header = ftd::p1::Header(vec![]);
    let (mut left_boolean, mut right_boolean) = (None, None);
    for (k, v) in &sub.header.0 {
        if k == "$loop$" || k.starts_with('/') {
            continue;
        }

        if k == "if" && contains_loop_ref(&loop_ref, v) {
            let v = v.replace(&loop_ref, "$loop$");
            let (_, left, right) = ftd::p2::Boolean::boolean_left_right(&v)?;
            if left.contains("$loop$") {
                left_boolean = resolve_loop_reference(&recursive_kind, doc, left)?.default;
            }
            if let Some(r) = right {
                if r.contains("$loop$") {
                    right_boolean = resolve_loop_reference(&recursive_kind, doc, r)?.default;
                }
            }
        }

        if contains_loop_ref(&loop_ref, v) && v.starts_with("ref ") {
            let reference = ftd_rt::get_name("ref", &*v)?
                .to_string()
                .replace(&loop_ref, "$loop$");
            let value = resolve_loop_reference(&recursive_kind, doc, reference)?;
            properties.insert(k.to_string(), value);
        } else {
            new_header.add(k, v);
        }
    }

    let (mut root_arguments, full_name, caption) =
        if name_with_component.is_some() && sub.name == name_with_component.clone().expect("").0 {
            let root_component = name_with_component.expect("").1;
            (
                root_component.arguments.clone(),
                root_component.full_name.to_string(),
                root_component.get_caption(),
            )
        } else {
            let root = doc.get_component(sub.name.as_str())?;
            let root_arguments = root.arguments.clone();
            assert_no_extra_properties(
                &new_header,
                root.full_name.as_str(),
                &root_arguments,
                sub.name.as_str(),
            )?;
            (
                root_arguments,
                root.full_name.to_string(),
                root.get_caption(),
            )
        };

    let mut new_caption = sub.caption.clone();
    if let (Some(caption), Some(caption_arg)) = (sub.caption.clone(), caption) {
        if caption.starts_with("ref ") && contains_loop_ref(&loop_ref, &caption) {
            let reference = ftd_rt::get_name("ref", &*caption)?
                .to_string()
                .replace(&loop_ref, "$loop$");
            let value = resolve_loop_reference(&recursive_kind, doc, reference)?;
            properties.insert(caption_arg, value);
            new_caption = None;
        }
    }

    properties.extend(read_properties(
        &new_header,
        &new_caption,
        &sub.body_without_comment(),
        "",
        full_name.as_str(),
        &mut root_arguments,
        arguments,
        locals,
        doc,
        &properties,
    )?);

    return Ok(ftd::ChildComponent {
        root: sub.name.to_string(),
        condition: match sub.header.str_optional("if")? {
            Some(expr) => Some(ftd::p2::Boolean::from_expression(
                expr,
                doc,
                arguments,
                locals,
                (left_boolean, right_boolean),
            )?),
            None => None,
        },
        properties,
        events: vec![],
        is_recursive: true,
    });

    fn resolve_loop_reference(
        recursive_kind: &ftd::p2::Kind,
        doc: &ftd::p2::TDoc,
        reference: String,
    ) -> ftd::p1::Result<Property> {
        let mut arguments: std::collections::BTreeMap<String, ftd::p2::Kind> = Default::default();
        arguments.insert("$loop$".to_string(), recursive_kind.to_owned());
        let property = ftd::PropertyValue::resolve_value(
            &format!("${}", reference),
            None,
            doc,
            &arguments,
            &Default::default(),
            None,
            false,
        )?;
        Ok(ftd::component::Property {
            default: Some(property),
            conditions: vec![],
        })
    }

    fn contains_loop_ref(loop_ref: &str, pattern: &str) -> bool {
        let ref1 = format!("{}.", loop_ref);
        let pattern_vec: Vec<&str> = pattern.split(' ').collect();
        let partern_bool = pattern_vec
            .iter()
            .map(|v| v.contains(&ref1) || v == &loop_ref)
            .collect::<Vec<bool>>();
        for p in partern_bool {
            if p {
                return p;
            }
        }
        false
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
        if k == "component"
            || k.starts_with('$')
            || k.starts_with('@')
            || k == "if"
            || k.starts_with('/')
        {
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
            let (reference, is_data) = if value.starts_with("ref ") {
                (ftd_rt::get_name("ref", &value)?, false)
            } else {
                (value.as_str(), true)
            };

            let property_value = ftd::PropertyValue::resolve_value(
                reference,
                Some(kind.to_owned()),
                doc,
                arguments,
                locals,
                Some(source.clone()),
                is_data,
            )?;
            let (condition_value, default_value) = if let Some(attribute) = conditional_attribute {
                let condition = crate::p2::Boolean::from_expression(
                    attribute,
                    doc,
                    arguments,
                    locals,
                    (None, None),
                )?;
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
        if (k.starts_with('$') && k.ends_with('$')) || k.starts_with('/') {
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
        if k.starts_with('/') {
            continue;
        }
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
                conditions: vec![],
            }),
        );
        let mut main = default_column();
        main.container
            .children
            .push(ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("Amit"),
                line: true,
                common: ftd_rt::Common {
                    reference: Some(s("foo/bar#name")),
                    ..Default::default()
                },
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
                conditions: vec![],
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
                conditions: vec![],
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
