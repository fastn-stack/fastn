#[derive(Debug, Clone)]
pub struct ElementWithContainer {
    pub element: ftd::Element,
    pub children: Vec<ftd::Element>,
    pub child_container: Option<ftd::Map<Vec<Vec<usize>>>>,
}

/*impl ftd::interpreter::ChildComponent {
    pub fn super_call(
        &self,
        children: &[Self],
        doc: &mut ftd::interpreter::TDoc,
        invocations: &mut ftd::Map<Vec<ftd::Map<ftd::Value>>>,
        local_container: &[usize],
        external_children_count: &Option<usize>,
    ) -> ftd::p11::Result<ElementWithContainer> {
        let id = ftd::executer::utils::string_optional(
            "id",
            &resolve_properties(self.line_number, &self.properties, doc)?,
            doc.name,
            self.line_number,
        )?;

        let ElementWithContainer {
            mut element,
            child_container,
            ..
        } = self.call(
            doc,
            invocations,
            false,
            local_container,
            id.clone(),
            external_children_count,
        )?;
        element.set_container_id(id.clone());
        element.set_element_id(id);

        let mut container_children = vec![];
        match (&mut element, children.is_empty()) {
            (ftd::Element::Column(_), _)
            | (ftd::Element::Row(_), _)
            | (ftd::Element::Scene(_), _)
            | (ftd::Element::Grid(_), _) => {
                let instructions = children
                    .iter()
                    .map(|child| {
                        if child.is_recursive {
                            ftd::interpreter::Instruction::RecursiveChildComponent {
                                child: child.to_owned(),
                            }
                        } else {
                            ftd::interpreter::Instruction::ChildComponent {
                                child: child.to_owned(),
                            }
                        }
                    })
                    .collect::<Vec<ftd::Instruction>>();
                let elements = ftd::execute_doc::ExecuteDoc {
                    name: doc.name,
                    aliases: doc.aliases,
                    bag: doc.bag,
                    local_variables: doc.local_variables,
                    instructions: &instructions,
                    invocations,
                }
                .execute(local_container, None, doc.referenced_local_variables)?
                .children;
                container_children.extend(elements);
            }
            (ftd::Element::Null, false) => {
                let root_name = ftd::interpreter::utils::get_root_component_name(
                    doc,
                    self.root.as_str(),
                    self.line_number,
                )?;
                match root_name.as_str() {
                    "ftd#row" | "ftd#column" | "ftd#scene" | "ftd#grid" | "ftd#text" => {}
                    t => {
                        return ftd::interpreter::utils::e2(
                            format!("{} cant have children", t),
                            doc.name,
                            self.line_number,
                        )
                    }
                }
            }
            (ftd::Element::Markup(_), _) => {}
            (t, false) => {
                return ftd::interpreter::utils::e2(
                    format!("cant have children: {:?}", t),
                    doc.name,
                    self.line_number,
                );
            }
            (_, true) => {}
        }

        // In case markup the behaviour of container_children is not the same.
        // They act as the component variables which are, then, referred to in markup text
        // container_children copy there properties to the reference in markup text

        if let ftd::Element::Markup(ref mut markups) = element {
            if !children.is_empty() {
                let named_container = markup_get_named_container(
                    children,
                    self.root.as_str(),
                    self.line_number,
                    doc,
                    invocations,
                    local_container,
                )?;
                reevalute_markups(markups, named_container, doc)?;
            }
        }

        Ok(ElementWithContainer {
            element,
            children: container_children,
            child_container,
        })
    }

    pub fn recursive_call(
        &self,
        doc: &mut ftd::interpreter::TDoc,
        invocations: &mut ftd::Map<Vec<ftd::Map<ftd::Value>>>,
        is_child: bool,
        local_container: &[usize],
    ) -> ftd::p11::Result<Vec<ElementWithContainer>> {
        let root = {
            // NOTE: doing unwrap to force bug report if we following fails, this function
            // must have validated everything, and must not fail at run time
            doc.get_component(self.line_number, self.root.as_str())
                .unwrap()
        };
        let loop_property = resolve_recursive_property(self.line_number, &self.properties, doc)?;
        let mut elements = vec![];

        let reference_name = {
            let mut reference_name = None;
            if let Some(value) = self.properties.get("$loop$") {
                if let Ok(ftd::interpreter::Property::Reference { name, .. }) =
                    value.eval(0, "$loop$", doc)
                {
                    reference_name = Some(name);
                }
            }
            reference_name
        };

        if let ftd::Value::List { data, kind } = loop_property {
            for (i, d) in data.iter().enumerate() {
                let mut element = construct_element(
                    self,
                    d,
                    i,
                    &root,
                    doc,
                    invocations,
                    is_child,
                    local_container,
                )?;
                if let Some(name) = reference_name {
                    if let Some(common) = element.element.get_mut_common() {
                        common.reference = Some(name.to_string());
                    }
                }
                elements.push(element);
            }
            if let Some(tmp_data) = construct_tmp_data(&kind) {
                if let Some(name) = reference_name {
                    let mut element = construct_element(
                        self,
                        &tmp_data,
                        data.len(),
                        &root,
                        doc,
                        invocations,
                        is_child,
                        local_container,
                    )?;
                    if let Some(common) = element.element.get_mut_common() {
                        common.reference = Some(name.to_string());
                        common.is_dummy = true;
                        elements.push(element);
                    }
                }
            }
        }
        return Ok(elements);

        fn construct_tmp_data(kind: &ftd::interpreter::Kind) -> Option<ftd::interpreter::Property> {
            // todo: fix it for all kind (Arpita)
            match kind {
                ftd::interpreter::Kind::String { .. } => Some(ftd::interpreter::Property::Value {
                    value: ftd::Value::String {
                        text: "$loop$".to_string(),
                        source: ftd::TextSource::Header,
                    },
                }),
                ftd::interpreter::Kind::Integer { .. } => Some(ftd::interpreter::Property::Value {
                    value: ftd::Value::Integer { value: 0 },
                }),
                ftd::interpreter::Kind::Decimal { .. } => Some(ftd::interpreter::Property::Value {
                    value: ftd::Value::Decimal { value: 0.0 },
                }),
                ftd::interpreter::Kind::Boolean { .. } => Some(ftd::interpreter::Property::Value {
                    value: ftd::Value::Boolean { value: false },
                }),
                ftd::interpreter::Kind::Optional { kind, .. } => {
                    construct_tmp_data(kind).map(|v| v.into_optional())
                }
                _ => None,
            }
        }

        #[allow(clippy::too_many_arguments)]
        fn construct_element(
            child_component: &ChildComponent,
            d: &ftd::interpreter::Property,
            index: usize,
            root: &ftd::Component,
            doc: &mut ftd::interpreter::TDoc,
            invocations: &mut ftd::Map<Vec<ftd::Map<ftd::Value>>>,
            is_child: bool,
            local_container: &[usize],
        ) -> ftd::p11::Result<ElementWithContainer> {
            let mut root = root.to_owned();
            let local_container = {
                let mut container = local_container[..local_container.len() - 1].to_vec();
                match local_container.last() {
                    Some(val) => container.push(val + index),
                    None => container.push(index),
                }
                container
            };
            let string_container =
                ftd::interpreter::utils::get_string_container(local_container.as_slice());
            let loop_name = doc.resolve_name(0, format!("$loop$@{}", string_container).as_str())?;
            doc.local_variables.insert(
                loop_name,
                ftd::interpreter::Thing::Variable(ftd::Variable {
                    name: "$loop$".to_string(),
                    value: d.to_owned(),
                    conditions: vec![],
                    flags: Default::default(),
                }),
            );
            doc.insert_local_from_component(
                &mut root,
                &child_component.properties,
                local_container.as_slice(),
                &None,
            )?;
            let child_component = {
                let mut child_component = child_component.clone();
                doc.update_component_data(
                    string_container.as_str(),
                    string_container.as_str(),
                    &mut child_component.properties,
                    &mut child_component.reference,
                    &mut child_component.condition,
                    &mut child_component.events,
                    false,
                    false,
                    false,
                )?;
                child_component
            };

            let is_visible = {
                let mut visible = true;
                if let Some(ref b) = child_component.condition {
                    if b.is_constant() && !b.eval(child_component.line_number, doc)? {
                        visible = false;
                        if let Ok(true) = b.set_null(child_component.line_number, doc.name) {
                            return Ok(ElementWithContainer {
                                element: ftd::Element::Null,
                                children: vec![],
                                child_container: None,
                            });
                        }
                    }
                }
                visible
            };
            let conditional_attribute = get_conditional_attributes(
                child_component.line_number,
                &child_component.properties,
                doc,
            )?;

            let mut element = root.call(
                &child_component.properties,
                doc,
                invocations,
                &None,
                is_child,
                &child_component.events,
                local_container.as_slice(),
                None,
                &None,
            )?;

            if let Some(condition) = &child_component.condition {
                element
                    .element
                    .set_non_visibility(!condition.eval(child_component.line_number, doc)?);
                element.element.set_condition(
                    condition
                        .to_condition(child_component.line_number, doc)
                        .ok(),
                );
            }
            if !is_visible {
                element.element.set_non_visibility(!is_visible);
            }
            if let Some(common) = element.element.get_mut_common() {
                common.conditional_attribute.extend(conditional_attribute);
            }
            // doc.local_variables.remove(loop_name.as_str());
            Ok(element)
        }
    }

    pub fn call(
        &self,
        doc: &mut ftd::interpreter::TDoc,
        invocations: &mut ftd::Map<Vec<ftd::Map<ftd::Value>>>,
        is_child: bool,
        local_container: &[usize],
        id: Option<String>,
        external_children_count: &Option<usize>,
    ) -> ftd::p11::Result<ElementWithContainer> {
        if let Some(ref b) = self.condition {
            if b.is_constant() && !b.eval(self.line_number, doc)? {
                if let Ok(true) = b.set_null(self.line_number, doc.name) {
                    return Ok(ElementWithContainer {
                        element: ftd::Element::Null,
                        children: vec![],
                        child_container: None,
                    });
                }
            }
        }

        let mut root = {
            // NOTE: doing unwrap to force bug report if we following fails, this function
            // must have validated everything, and must not fail at run time
            doc.get_component(self.line_number, self.root.as_str())
                .unwrap()
        };

        doc.insert_local_from_component(
            &mut root,
            &self.properties,
            local_container,
            external_children_count,
        )?;

        let conditional_attribute =
            get_conditional_attributes(self.line_number, &self.properties, doc)?;

        let mut element = root.call(
            &self.properties,
            doc,
            invocations,
            &self.condition,
            is_child,
            &self.events,
            local_container,
            id,
            external_children_count,
        )?;

        if let Some(common) = element.element.get_mut_common() {
            common.conditional_attribute.extend(conditional_attribute);
        }

        if let ftd::Element::Markup(ref mut markups) = element.element {
            let named_container = match markup_get_named_container(
                &[],
                self.root.as_str(),
                self.line_number,
                doc,
                invocations,
                local_container,
            ) {
                Ok(n) => n,
                _ => return Ok(element),
            };
            reevalute_markups(markups, named_container, doc).ok();
        }

        Ok(element)
    }

    pub fn from_p1(
        line_number: usize,
        name: &str,
        p1: &ftd::p11::Header,
        caption: &Option<String>,
        body: &Option<(usize, String)>,
        doc: &ftd::interpreter::TDoc,
        arguments: &ftd::Map<ftd::interpreter::Kind>,
    ) -> ftd::p11::Result<Self> {
        let mut reference = None;
        let root = if let Some(ftd::interpreter::Kind::UI { default }) =
            arguments.get(name).map(|v| v.inner())
        {
            reference = Some((
                name.to_string(),
                ftd::interpreter::Kind::UI {
                    default: (*default).clone(),
                },
            ));
            ftd::Component {
                root: "ftd.kernel".to_string(),
                full_name: "ftd#ui".to_string(),
                line_number,
                ..Default::default()
            }
        } else {
            doc.get_component(line_number, name)?
        };

        assert_no_extra_properties(
            line_number,
            p1,
            root.full_name.as_str(),
            &root.arguments,
            name,
            doc,
        )?;
        let (local_arguments, inherits) =
            read_arguments(p1, name, &root.arguments, arguments, doc)?;

        let mut all_arguments = local_arguments.clone();
        all_arguments.extend(arguments.clone());

        let root_property =
            get_root_property(line_number, name, caption, doc, &all_arguments, inherits)?;

        assert_caption_body_checks(&root.full_name, p1, doc, caption, body, line_number)?;

        return Ok(Self {
            line_number,
            properties: read_properties(
                line_number,
                p1,
                caption,
                body,
                name,
                root.full_name.as_str(),
                &root.arguments,
                &all_arguments,
                doc,
                &root_property,
                reference.is_some(),
            )?,
            condition: match p1.str_optional(doc.name, line_number, "if")? {
                Some(expr) => Some(ftd::interpreter::Boolean::from_expression(
                    expr,
                    doc,
                    &all_arguments,
                    (None, None),
                    line_number,
                )?),
                None => None,
            },
            root: doc.resolve_name(line_number, root.full_name.as_str())?,
            events: p1.get_events(line_number, doc, &all_arguments)?,
            is_recursive: false,
            arguments: local_arguments,
            reference,
        });

        fn get_root_property(
            line_number: usize,
            name: &str,
            caption: &Option<String>,
            doc: &ftd::interpreter::TDoc,
            arguments: &ftd::Map<ftd::interpreter::Kind>,
            inherits: Vec<String>,
        ) -> ftd::p11::Result<ftd::Map<Property>> {
            let mut properties: ftd::Map<Property> =
                root_properties_from_inherits(line_number, arguments, inherits, doc)?;
            if let Some(caption) = caption {
                if let Ok(name) = doc.resolve_name(line_number, name) {
                    let kind = match name.as_str() {
                        "ftd#integer" => ftd::interpreter::Kind::integer(),
                        "ftd#boolean" => ftd::interpreter::Kind::boolean(),
                        "ftd#decimal" => ftd::interpreter::Kind::decimal(),
                        _ => return Ok(properties),
                    };
                    if let Ok(property_value) = ftd::interpreter::Property::resolve_value(
                        line_number,
                        caption,
                        Some(kind),
                        doc,
                        arguments,
                        None,
                    ) {
                        properties.insert(
                            "value".to_string(),
                            ftd::component::Property {
                                default: Some(property_value),
                                conditions: vec![],
                                ..Default::default()
                            },
                        );
                    }
                }
            }
            Ok(properties)
        }
    }
}

impl ftd::interpreter::Component {
    fn call_sub_functions(
        &self,
        doc: &mut ftd::interpreter::TDoc,
        invocations: &mut ftd::Map<Vec<ftd::Map<ftd::Value>>>,
        call_container: &[usize],
        id: Option<String>,
    ) -> ftd::p11::Result<ElementWithContainer> {
        let new_instruction = {
            let mut instructions: Vec<Instruction> = self.instructions.clone();
            for instruction in instructions.iter_mut() {
                match instruction {
                    Instruction::ChildComponent { child } => {
                        reference_to_child_component(child, self.line_number, doc)?
                    }
                    Instruction::Component { parent, children } => {
                        reference_to_child_component(parent, self.line_number, doc)?;
                        for child in children.iter_mut() {
                            reference_to_child_component(child, self.line_number, doc)?;
                        }
                    }
                    Instruction::ChangeContainer { .. } => {}
                    Instruction::RecursiveChildComponent { child } => {
                        reference_to_child_component(child, self.line_number, doc)?
                    }
                }
            }
            instructions
        };

        return ftd::execute_doc::ExecuteDoc {
            name: doc.name,
            aliases: doc.aliases,
            bag: doc.bag,
            local_variables: doc.local_variables,
            instructions: &new_instruction,
            invocations,
        }
        .execute(call_container, id, doc.referenced_local_variables);

        fn reference_to_child_component(
            child: &mut ChildComponent,
            line_number: usize,
            doc: &ftd::interpreter::TDoc,
        ) -> ftd::p11::Result<()> {
            if let Some(ref c) = child.reference {
                match doc.get_component(line_number, &c.0) {
                    Ok(_) => {
                        *child = ChildComponent {
                            root: c.0.to_string(),
                            condition: None,
                            properties: Default::default(),
                            arguments: Default::default(),
                            events: vec![],
                            is_recursive: false,
                            line_number,
                            reference: None,
                        };
                    }
                    Err(e) => {
                        match doc.get_value(line_number, &c.0) {
                            Ok(ftd::Value::Optional { kind, .. })
                            | Ok(ftd::Value::None { kind })
                                if matches!(kind, ftd::interpreter::Kind::UI { .. }) =>
                            {
                                if let Some(ftd::interpreter::Boolean::IsNotNull { ref value }) =
                                    child.condition
                                {
                                    match value {
                                        ftd::interpreter::Property::Reference { name, .. }
                                        | ftd::interpreter::Property::Variable { name, .. } => {
                                            if name.eq({
                                                if let Some(reference) = c.0.strip_prefix('@') {
                                                    reference
                                                } else {
                                                    c.0.as_str()
                                                }
                                            }) {
                                                *child = ChildComponent {
                                                    root: "ftd#null".to_string(),
                                                    condition: None,
                                                    properties: Default::default(),
                                                    arguments: Default::default(),
                                                    events: vec![],
                                                    is_recursive: false,
                                                    line_number,
                                                    reference: None,
                                                };
                                                return Ok(());
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            _ => {}
                        }
                        return ftd::interpreter::utils::e2(
                            format!("{:?}", e),
                            doc.name,
                            line_number,
                        );
                    }
                }
            }
            Ok(())
        }
    }

    fn call_without_values(
        &self,
        doc: &mut ftd::interpreter::TDoc,
    ) -> ftd::p11::Result<ElementWithContainer> {
        self.call(
            &Default::default(),
            doc,
            &mut Default::default(),
            &Default::default(),
            false,
            &[],
            &[],
            Default::default(),
            &None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn call(
        &self,
        arguments: &ftd::Map<Property>,
        doc: &mut ftd::interpreter::TDoc,
        invocations: &mut ftd::Map<Vec<ftd::Map<ftd::Value>>>,
        condition: &Option<ftd::interpreter::Boolean>,
        is_child: bool,
        events: &[ftd::interpreter::Event],
        local_container: &[usize],
        id: Option<String>,
        external_children_count: &Option<usize>,
    ) -> ftd::p11::Result<ElementWithContainer> {
        invocations
            .entry(self.full_name.clone())
            .or_default()
            .push(resolve_properties(0, arguments, doc)?);
        if self.root == "ftd.kernel" {
            let element = match self.full_name.as_str() {
                "ftd#text-block" => {
                    ftd::Element::TextBlock(ftd::interpreter::element::text_block_from_properties(
                        arguments, doc, condition, is_child, events,
                    )?)
                }
                "ftd#code" => ftd::Element::Code(ftd::interpreter::element::code_from_properties(
                    arguments, doc, condition, is_child, events,
                )?),
                "ftd#image" => {
                    ftd::Element::Image(ftd::interpreter::element::image_from_properties(
                        arguments, doc, condition, is_child, events,
                    )?)
                }
                "ftd#row" => ftd::Element::Row(ftd::interpreter::element::row_from_properties(
                    arguments, doc, condition, is_child, events,
                )?),
                "ftd#column" => {
                    ftd::Element::Column(ftd::interpreter::element::column_from_properties(
                        arguments, doc, condition, is_child, events,
                    )?)
                }
                "ftd#iframe" => {
                    ftd::Element::IFrame(ftd::interpreter::element::iframe_from_properties(
                        arguments, doc, condition, is_child, events,
                    )?)
                }
                "ftd#integer" => {
                    ftd::Element::Integer(ftd::interpreter::element::integer_from_properties(
                        arguments, doc, condition, is_child, events,
                    )?)
                }
                "ftd#decimal" => {
                    ftd::Element::Decimal(ftd::interpreter::element::decimal_from_properties(
                        arguments, doc, condition, is_child, events,
                    )?)
                }
                "ftd#boolean" => {
                    ftd::Element::Boolean(ftd::interpreter::element::boolean_from_properties(
                        arguments, doc, condition, is_child, events,
                    )?)
                }
                "ftd#input" => {
                    ftd::Element::Input(ftd::interpreter::element::input_from_properties(
                        arguments, doc, condition, is_child, events,
                    )?)
                }
                "ftd#scene" => {
                    ftd::Element::Scene(ftd::interpreter::element::scene_from_properties(
                        arguments, doc, condition, is_child, events,
                    )?)
                }
                "ftd#grid" => ftd::Element::Grid(ftd::interpreter::element::grid_from_properties(
                    arguments, doc, condition, is_child, events,
                )?),
                "ftd#text" => {
                    ftd::Element::Markup(ftd::interpreter::element::markup_from_properties(
                        arguments, doc, condition, is_child, events,
                    )?)
                }
                "ftd#null" => ftd::Element::Null,
                _ => unreachable!(),
            };
            Ok(ElementWithContainer {
                element,
                children: vec![],
                child_container: None,
            })
        } else {
            let mut root = {
                // NOTE: doing unwrap to force bug report if we following fails, this function
                // must have validated everything, and must not fail at run time
                doc.get_component(self.line_number, self.root.as_str())
                    .unwrap()
            };
            doc.insert_local_from_component(
                &mut root,
                &self.properties,
                local_container,
                external_children_count,
            )?;

            let (get_condition, is_visible, is_null_element) = match condition {
                Some(c) => {
                    let is_visible = c.eval(self.line_number, doc)?;
                    if !c.is_arg_constant() {
                        (
                            Some(c.to_condition(self.line_number, doc)?),
                            is_visible,
                            false,
                        )
                    } else {
                        (
                            None,
                            is_visible,
                            !is_visible
                                && c.set_null(self.line_number, doc.name).is_ok()
                                && c.set_null(self.line_number, doc.name)?,
                        )
                    }
                }
                _ => (None, true, false),
            };

            let events = ftd::interpreter::Event::get_events(self.line_number, events, doc)?;

            let mut element = if !is_null_element {
                root.call(
                    &self.properties,
                    doc,
                    invocations,
                    &self.condition,
                    is_child,
                    &self.events,
                    local_container,
                    None,
                    external_children_count,
                )?
            } else {
                ElementWithContainer {
                    element: ftd::Element::Null,
                    children: vec![],
                    child_container: None,
                }
            }
            .element;

            if get_condition.is_some() {
                let mut is_visible = is_visible;
                if let Some(common) = element.get_common() {
                    is_visible &= !common.is_not_visible;
                }
                element.set_condition(get_condition);
                element.set_non_visibility(!is_visible);
            }

            let conditional_attribute =
                get_conditional_attributes(self.line_number, &self.properties, doc)?;

            let mut containers: Option<ftd::Map<Vec<Vec<usize>>>> = None;
            match &mut element {
                ftd::Element::TextBlock(_)
                | ftd::Element::Code(_)
                | ftd::Element::Image(_)
                | ftd::Element::IFrame(_)
                | ftd::Element::Input(_)
                | ftd::Element::Integer(_)
                | ftd::Element::Decimal(_)
                | ftd::Element::Boolean(_)
                | ftd::Element::Markup(_)
                | ftd::Element::Null => {}
                ftd::Element::Column(ftd::Column {
                    ref mut container, ..
                })
                | ftd::Element::Row(ftd::Row {
                    ref mut container, ..
                })
                | ftd::Element::Scene(ftd::Scene {
                    ref mut container, ..
                })
                | ftd::Element::Grid(ftd::Grid {
                    ref mut container, ..
                }) => {
                    let ElementWithContainer {
                        children,
                        child_container,
                        ..
                    } = self.call_sub_functions(doc, invocations, local_container, id)?;

                    if let Some(ref append_at) = container.append_at {
                        if let Some(ref child_container) = child_container {
                            let id = if append_at.contains('.') {
                                ftd::interpreter::utils::split(append_at.to_string(), ".")?.1
                            } else {
                                append_at.to_string()
                            };
                            if let Some(c) =
                                child_container.get(append_at.replace('.', "#").as_str())
                            {
                                container.external_children = Some((id, c.to_owned(), vec![]));
                            }
                        }
                    }
                    if let Some(child_container) = child_container {
                        match containers {
                            Some(ref mut containers) => {
                                containers.extend(child_container);
                            }
                            None => {
                                containers = Some(child_container);
                            }
                        }
                    }
                    container.children.extend(children);
                }
            }

            if let Some(common) = element.get_mut_common() {
                common.conditional_attribute.extend(conditional_attribute);
                common.events.extend(events);
            }

            Ok(ElementWithContainer {
                element,
                children: vec![],
                child_container: containers,
            })
        }
    }
}

fn markup_get_named_container(
    children: &[ChildComponent],
    root: &str,
    line_number: usize,
    doc: &mut ftd::interpreter::TDoc,
    invocations: &mut ftd::Map<Vec<ftd::Map<ftd::Value>>>,
    local_container: &[usize],
) -> ftd::p11::Result<ftd::Map<ftd::Element>> {
    let children = {
        let mut children = children.to_vec();
        let root_name = ftd::interpreter::utils::get_root_component_name(doc, root, line_number)?;
        if root_name.eq("ftd#text") {
            let mut name = root.to_string();
            while name != "ftd.kernel" {
                let component = doc.get_component(line_number, name.as_str())?;
                for instruction in component.instructions {
                    if let ftd::interpreter::Instruction::ChildComponent { child } = instruction {
                        children.push(child);
                    }
                }
                name = component.root;
            }
        }
        children
    };
    let mut elements_name = vec![];

    let instructions = children
        .iter()
        .map(|child| {
            // ftd#markup modifies the child
            let child = get_modified_child(child, &mut elements_name);
            if child.is_recursive {
                ftd::interpreter::Instruction::RecursiveChildComponent { child }
            } else {
                ftd::interpreter::Instruction::ChildComponent { child }
            }
        })
        .collect::<Vec<ftd::Instruction>>();

    let container_children = ftd::execute_doc::ExecuteDoc {
        name: doc.name,
        aliases: doc.aliases,
        bag: doc.bag,
        local_variables: doc.local_variables,
        instructions: &instructions,
        invocations,
    }
    .execute(local_container, None, doc.referenced_local_variables)?
    .children;

    return convert_to_named_container(&container_children, &elements_name, doc);

    /// ftd#markup modifies the child because root name contains both the component and also the variable name
    /// Like this: --- <component-name> <variable-name>:
    /// Example: --- ftd.text name:
    /// We need to remove variable name before passing it to instruction and later append it to resolve variables
    /// in ftd#markup.
    fn get_modified_child(
        child: &ftd::ChildComponent,
        elements_name: &mut Vec<String>,
    ) -> ftd::ChildComponent {
        let mut child = child.clone();
        if let Some((ref c, ref element_name)) = child.root.split_once(' ') {
            elements_name.push(element_name.to_string());
            child.root = c.to_string();
        }
        child
    }

    /// first convert the container children into the named container.
    /// which basically make a map of component's variable name with the container_child.
    /// which was initially  seperated by `get_modified_child()`
    fn convert_to_named_container(
        container_children: &[ftd::Element],
        elements_name: &[String],
        doc: &ftd::interpreter::TDoc,
    ) -> ftd::p11::Result<ftd::Map<ftd::Element>> {
        let mut named_container = ftd::Map::new();
        for (idx, container) in container_children.iter().enumerate() {
            match elements_name.get(idx) {
                Some(name) => {
                    named_container.insert(name.to_string(), container.to_owned());
                }
                None => {
                    return ftd::interpreter::utils::e2(
                        format!("cannot find name for container {:?}", container),
                        doc.name,
                        0,
                    )
                }
            }
        }
        Ok(named_container)
    }
}

fn reevalute_markups(
    markups: &mut ftd::Markups,
    named_container: ftd::Map<ftd::Element>,
    doc: &mut ftd::interpreter::TDoc,
) -> ftd::p11::Result<()> {
    if !markups.children.is_empty() {
        // no need to re-evalute
        // already evaluted
        return Ok(());
    }
    let mut all_children = markups.children.to_owned();
    if markups.text.original.contains("\n\n") {
        for v in markups.text.original.split("\n\n") {
            let itext = ftd::IText::Markup(ftd::Markups {
                text: if !markups.line {
                    ftd::rendered::markup(v)
                } else {
                    ftd::rendered::markup_line(v)
                },
                ..Default::default()
            });
            all_children.push(ftd::Markup {
                itext,
                children: vec![],
            });
        }
    }
    if all_children.is_empty() {
        let mut markup = ftd::Markup {
            itext: ftd::IText::Markup(markups.clone()),
            children: vec![],
        };
        reevalute_markup(&mut markup, &named_container, doc)?;
        if let ftd::IText::Markup(m) = markup.itext {
            *markups = m;
        }
        markups.line = true;
        markups.children = markup.children;
        return Ok(());
    }
    for markup in all_children.iter_mut() {
        reevalute_markup(markup, &named_container, doc)?;
    }
    markups.children = all_children;

    Ok(())
}

fn reevalute_markup(
    markup: &mut ftd::Markup,
    named_container: &ftd::Map<ftd::Element>,
    doc: &mut ftd::interpreter::TDoc,
) -> ftd::p11::Result<()> {
    let text = match &markup.itext {
        ftd::IText::Text(ftd::Text { text, .. })
        | ftd::IText::TextBlock(ftd::TextBlock { text, .. })
        | ftd::IText::Integer(ftd::Text { text, .. })
        | ftd::IText::Boolean(ftd::Text { text, .. })
        | ftd::IText::Decimal(ftd::Text { text, .. })
        | ftd::IText::Markup(ftd::Markups { text, .. }) => {
            text.original.chars().collect::<Vec<_>>()
        }
    };
    let mut children = vec![];
    let mut idx = 0;
    let mut traverse_string = "".to_string();
    while idx < text.len() {
        if text[idx].eq(&'{') {
            children.push(ftd::Markup {
                itext: ftd::IText::Text(ftd::Text {
                    text: ftd::rendered::markup_line(traverse_string.as_str()),
                    ..Default::default()
                }),
                children: vec![],
            });
            traverse_string = get_inner_text(&text, &mut idx, doc.name)?;
            let (style, text) = traverse_string
                .split_once(':')
                .map(|(v, n)| (v.trim(), Some(n)))
                .unwrap_or((traverse_string.trim(), None));

            let container = match named_container.get(style) {
                Some(style) => style.to_owned(),
                None => get_element_doc(doc, style)?,
            };

            let itext = element_to_itext(&container, doc, text, style, named_container)?;

            children.push(ftd::Markup {
                itext,
                children: vec![],
            });

            traverse_string = "".to_string();
        } else {
            traverse_string.push(text[idx]);
        }
        idx += 1;
    }

    if !traverse_string.is_empty() && !children.is_empty() {
        children.push(ftd::Markup {
            itext: ftd::IText::Text(ftd::Text {
                text: ftd::rendered::markup_line(traverse_string.as_str()),
                ..Default::default()
            }),
            children: vec![],
        });
    }
    for child in children.iter_mut() {
        if let ftd::IText::Markup(_) = child.itext {
            continue;
        }
        reevalute_markup(child, named_container, doc)?;
    }
    markup.children = children;

    return Ok(());

    /// Get text between `{` and `}`
    fn get_inner_text(text: &[char], idx: &mut usize, doc_id: &str) -> ftd::p11::Result<String> {
        let mut stack = vec!['{'];
        let mut traverse_string = "".to_string();
        while !stack.is_empty() {
            *idx += 1;
            if *idx >= text.len() {
                return ftd::interpreter::utils::e2(
                    format!(
                        "cannot find closing-parenthesis before the string ends: {}",
                        traverse_string
                    ),
                    doc_id,
                    0,
                );
            }
            if text[*idx].eq(&'{') {
                stack.push('{');
            } else if text[*idx].eq(&'}') {
                stack.pop();
            }
            if !stack.is_empty() {
                traverse_string.push(text[*idx]);
            }
        }
        Ok(traverse_string)
    }

    fn element_to_itext(
        element: &ftd::Element,
        doc: &mut ftd::interpreter::TDoc,
        text: Option<&str>,
        root: &str,
        named_container: &ftd::Map<ftd::Element>,
    ) -> ftd::p11::Result<ftd::IText> {
        Ok(match element {
            ftd::Element::Integer(t) => {
                let t = {
                    let mut t = t.clone();
                    if let Some(text) = text {
                        t.text = ftd::rendered::markup_line(text);
                        t.common.reference = None;
                    }
                    t
                };
                ftd::IText::Integer(t)
            }
            ftd::Element::Boolean(t) => {
                let t = {
                    let mut t = t.clone();
                    if let Some(text) = text {
                        t.text = ftd::rendered::markup_line(text);
                        t.common.reference = None;
                    }
                    t
                };
                ftd::IText::Boolean(t)
            }
            ftd::Element::Decimal(t) => {
                let t = {
                    let mut t = t.clone();
                    if let Some(text) = text {
                        t.text = ftd::rendered::markup_line(text);
                        t.common.reference = None;
                    }
                    t
                };
                ftd::IText::Decimal(t)
            }
            ftd::Element::TextBlock(t) => {
                let t = {
                    let mut t = t.clone();
                    if let Some(text) = text {
                        t.text = ftd::rendered::markup_line(text);
                        t.common.reference = None;
                    }
                    t
                };
                ftd::IText::TextBlock(t)
            }
            ftd::Element::Markup(t) => {
                let mut t = {
                    let mut t = t.clone();
                    if let Some(text) = text {
                        t.text = ftd::rendered::markup_line(text);
                        t.common.reference = None;
                    }
                    t
                };
                let named_container = if let Ok(mut get) =
                    markup_get_named_container(&[], root, 0, doc, &mut Default::default(), &[])
                {
                    get.extend(named_container.clone());
                    get
                } else {
                    // In case of component variable of markup defined internally,
                    // it won't be present inside doc.bag
                    // Example:
                    // -- ftd.text foo: {bar: Hello}
                    // --- ftd.text bar:
                    // color: red
                    //
                    // `bar` here won't be present inside doc.bag
                    named_container.clone()
                };
                reevalute_markups(&mut t, named_container, doc)?;
                ftd::IText::Markup(t)
            }
            t => {
                return ftd::interpreter::utils::e2(
                    format!(
                        "expected type istext, integer, boolean, decimal. found: {:?}",
                        t
                    ),
                    doc.name,
                    0,
                )
            }
        })
    }

    fn get_element_doc(
        doc: &mut ftd::interpreter::TDoc,
        name: &str,
    ) -> ftd::p11::Result<ftd::Element> {
        let mut root = doc
            .get_component(0, name)
            .map_err(|_| ftd::p11::Error::ParseError {
                message: format!("This component not found in ftd.text {}", name),
                doc_id: doc.name.to_string(),
                line_number: 0,
            })?;

        let property_value = if let Some(p) = root.properties.get("text") {
            p
        } else if let Some(p) = root.properties.get("value") {
            p
        } else {
            return ftd::interpreter::utils::e2(
                format!(
                    "expected type for ftd.text are text, integer, decimal and boolean, {:?}",
                    root
                ),
                doc.name,
                0,
            );
        };

        if let ftd::interpreter::Property {
            default: Some(ftd::interpreter::Property::Variable { kind, .. }),
            ..
        } = property_value
        {
            if !kind.has_default_value() {
                let property = ftd::interpreter::Property {
                    default: Some(ftd::interpreter::Property::Value {
                        value: ftd::Value::String {
                            text: name.to_string(),
                            source: ftd::TextSource::Header,
                        },
                    }),
                    ..Default::default()
                };
                root.properties.insert("text".to_string(), property.clone());
                root.properties.insert("value".to_string(), property);
            }
        }
        root.arguments = Default::default();
        Ok(root.call_without_values(doc)?.element)
    }
}

fn get_conditional_attributes(
    line_number: usize,
    properties: &ftd::Map<Property>,
    doc: &ftd::interpreter::TDoc,
) -> ftd::p11::Result<ftd::Map<ftd::ConditionalAttribute>> {
    let mut conditional_attribute: ftd::Map<ftd::ConditionalAttribute> = Default::default();

    let mut dictionary: ftd::Map<Vec<String>> = Default::default();
    dictionary.insert(
        "padding-vertical".to_string(),
        vec!["padding-top".to_string(), "padding-bottom".to_string()],
    );
    dictionary.insert(
        "padding-horizontal".to_string(),
        vec!["padding-left".to_string(), "padding-right".to_string()],
    );
    dictionary.insert(
        "border-left".to_string(),
        vec!["border-left-width".to_string()],
    );
    dictionary.insert(
        "border-right".to_string(),
        vec!["border-right-width".to_string()],
    );
    dictionary.insert(
        "border-top".to_string(),
        vec!["border-top-width".to_string()],
    );
    dictionary.insert(
        "background-parallax".to_string(),
        vec!["background-attachment".to_string()],
    );
    dictionary.insert("size".to_string(), vec!["font-size".to_string()]);
    dictionary.insert(
        "border-bottom".to_string(),
        vec!["border-bottom-width".to_string()],
    );
    dictionary.insert(
        "border-top-radius".to_string(),
        vec![
            "border-top-left-radius".to_string(),
            "border-top-right-radius".to_string(),
        ],
    );
    dictionary.insert(
        "border-left-radius".to_string(),
        vec![
            "border-top-left-radius".to_string(),
            "border-bottom-left-radius".to_string(),
        ],
    );
    dictionary.insert(
        "border-right-radius".to_string(),
        vec![
            "border-bottom-right-radius".to_string(),
            "border-top-right-radius".to_string(),
        ],
    );
    dictionary.insert(
        "border-bottom-radius".to_string(),
        vec![
            "border-bottom-left-radius".to_string(),
            "border-bottom-right-radius".to_string(),
        ],
    );

    dictionary.insert("slots".to_string(), vec!["grid-template-areas".to_string()]);
    dictionary.insert(
        "slot-widths".to_string(),
        vec!["grid-template-columns".to_string()],
    );
    dictionary.insert(
        "slot-heights".to_string(),
        vec!["grid-template-rows".to_string()],
    );
    if properties.contains_key("slots") {
        dictionary.insert("spacing".to_string(), vec!["grid-gap".to_string()]);
    }
    dictionary.insert("slot".to_string(), vec!["grid-area".to_string()]);

    for (name, value) in properties {
        if !value.conditions.is_empty() {
            let styles = if let Some(styles) = dictionary.get(name) {
                styles.to_owned()
            } else {
                vec![name.to_string()]
            };

            for name in styles {
                let mut conditions_with_value = vec![];
                for (condition, pv) in &value.conditions {
                    if !condition.is_arg_constant() {
                        let cond = condition.to_condition(line_number, doc)?;
                        let value = pv.resolve(line_number, doc)?;
                        if check_for_none(condition, pv, &value) {
                            // todo: send default value
                            continue;
                        }
                        let string =
                            get_string_value(&name, value, doc, line_number, pv.get_reference())?;
                        conditions_with_value.push((cond, string));
                    }
                }
                let default = {
                    let mut default = None;
                    if let Some(pv) = &value.default {
                        let value = pv.resolve(line_number, doc)?;
                        let string =
                            get_string_value(&name, value, doc, line_number, pv.get_reference())?;
                        default = Some(string);
                    }
                    default
                };

                conditional_attribute.insert(
                    get_style_name(name),
                    ftd::ConditionalAttribute {
                        attribute_type: ftd::AttributeType::Style,
                        conditions_with_value,
                        default,
                    },
                );
            }
        }
    }
    return Ok(conditional_attribute);

    fn check_for_none(
        condition: &ftd::interpreter::Boolean,
        pv: &ftd::interpreter::Property,
        value: &ftd::interpreter::Value,
    ) -> bool {
        let bool_name = if let ftd::interpreter::Boolean::IsNotNull { value } = condition {
            match value {
                ftd::interpreter::Property::Reference { name, .. }
                | ftd::interpreter::Property::Variable { name, .. } => name,
                _ => return false,
            }
        } else {
            return false;
        };

        let pv_name = match pv {
            ftd::interpreter::Property::Reference { name, .. }
            | ftd::interpreter::Property::Variable { name, .. } => name,
            _ => return false,
        };

        if !bool_name.eq(pv_name) {
            return false;
        }

        match value {
            ftd::interpreter::Value::None { .. } => true,
            ftd::interpreter::Value::Optional { data, .. } if data.as_ref().eq(&None) => true,
            _ => false,
        }
    }

    fn get_style_name(name: String) -> String {
        match name.as_str() {
            "sticky" => "position",
            t => t,
        }
        .to_string()
    }

    fn get_string_value(
        name: &str,
        value: ftd::interpreter::Value,
        doc: &ftd::interpreter::TDoc,
        line_number: usize,
        reference: Option<String>,
    ) -> ftd::p11::Result<ftd::ConditionalValue> {
        let style_integer = vec![
            "padding",
            "padding-left",
            "padding-right",
            "padding-top",
            "padding-bottom",
            "margin-left",
            "margin-right",
            "margin-top",
            "margin-bottom",
            "top",
            "bottom",
            "left",
            "right",
            "shadow-offset-x",
            "shadow-offset-y",
            "shadow-size",
            "shadow-blur",
            "font-size",
            "border-width",
            "grid-gap",
            "line-height",
        ];

        let style_length = vec![
            "width",
            "min-width",
            "max-width",
            "height",
            "min-height",
            "max-height",
        ];

        let style_color = vec!["background-color", "color", "border-color", "shadow-color"];

        let style_integer_important = vec![
            "border-left-width",
            "border-right-width",
            "border-top-width",
            "border-bottom-width",
            "border-top-left-radius",
            "border-top-right-radius",
            "border-bottom-left-radius",
            "border-bottom-right-radius",
        ];

        let style_string = vec![
            "cursor",
            "position",
            "align",
            "background-image",
            "grid-template-columns",
            "grid-template-rows",
            "grid-area",
        ];

        let style_overflow = vec!["overflow-x", "overflow-y"];

        let style_boolean = vec!["background-repeat"];

        Ok(if style_integer.contains(&name) {
            match value {
                ftd::interpreter::Value::Integer { value: v } => ftd::ConditionalValue {
                    value: serde_json::Value::String(format!("{}px", v)),
                    important: false,
                    reference,
                },
                v => {
                    return ftd::interpreter::utils::e2(
                        format!("expected int, found3: {:?}", v),
                        doc.name,
                        line_number,
                    )
                }
            }
        } else if style_integer_important.contains(&name) {
            match value {
                ftd::interpreter::Value::Integer { value: v } => ftd::ConditionalValue {
                    value: serde_json::Value::String(format!("{}px", v)),
                    important: true,
                    reference,
                },
                v => {
                    return ftd::interpreter::utils::e2(
                        format!("expected int, found4: {:?}", v),
                        doc.name,
                        line_number,
                    )
                }
            }
        } else if style_length.contains(&name) {
            match value {
                ftd::interpreter::Value::String { text: v, .. } => ftd::ConditionalValue {
                    value: serde_json::Value::String(
                        ftd::length(&ftd::Length::from(Some(v), doc.name)?.unwrap(), name).1,
                    ),
                    important: false,
                    reference,
                },
                v => {
                    return ftd::interpreter::utils::e2(
                        format!("expected string, found 8: {:?}", v),
                        doc.name,
                        line_number,
                    )
                }
            }
        } else if style_color.contains(&name) {
            match value {
                ftd::interpreter::Value::Record { fields, .. } => {
                    let properties = fields
                        .iter()
                        .map(|(k, v)| v.resolve(line_number, doc).map(|v| (k.to_string(), v)))
                        .collect::<ftd::p11::Result<ftd::Map<ftd::interpreter::Value>>>()?;
                    let light = if let Some(light) = ftd::interpreter::element::color_from(
                        ftd::interpreter::utils::string_optional(
                            "light",
                            &properties,
                            doc.name,
                            0,
                        )?,
                        doc.name,
                    )? {
                        ftd::html::color(&light)
                    } else {
                        "auto".to_string()
                    };
                    let dark = if let Some(dark) = ftd::interpreter::element::color_from(
                        ftd::interpreter::utils::string_optional("dark", &properties, doc.name, 0)?,
                        doc.name,
                    )? {
                        ftd::html::color(&dark)
                    } else {
                        "auto".to_string()
                    };

                    ftd::ConditionalValue {
                        value: serde_json::json!({ "light": light, "dark": dark, "$kind$": "light" }),
                        important: false,
                        reference,
                    }
                }
                v => {
                    return ftd::interpreter::utils::e2(
                        format!("expected string, found 9: {:?}", v),
                        doc.name,
                        line_number,
                    )
                }
            }
        } else if style_overflow.contains(&name) {
            match value {
                ftd::interpreter::Value::String { text: v, .. } => ftd::ConditionalValue {
                    value: serde_json::Value::String(
                        ftd::overflow(&ftd::Overflow::from(Some(v), doc.name)?.unwrap(), name).1,
                    ),
                    important: false,
                    reference,
                },
                v => {
                    return ftd::interpreter::utils::e2(
                        format!("expected string, found 10: {:?}", v),
                        doc.name,
                        line_number,
                    )
                }
            }
        } else if style_string.contains(&name) {
            match value {
                ftd::interpreter::Value::String { text: v, .. } => ftd::ConditionalValue {
                    value: serde_json::Value::String(v),
                    important: false,
                    reference,
                },
                v => {
                    return ftd::interpreter::utils::e2(
                        format!("expected string, found 11: {:?}", v),
                        doc.name,
                        line_number,
                    )
                }
            }
        } else if style_boolean.contains(&name) {
            match value {
                ftd::interpreter::Value::Boolean { value: v } => ftd::ConditionalValue {
                    value: serde_json::Value::Bool(v),
                    important: false,
                    reference,
                },
                v => {
                    return ftd::interpreter::utils::e2(
                        format!("expected string, found 12: {:?}", v),
                        doc.name,
                        line_number,
                    )
                }
            }
        } else if name.eq("sticky") {
            match value {
                ftd::interpreter::Value::Boolean { value: v } => ftd::ConditionalValue {
                    value: serde_json::Value::String({
                        if v { "sticky" } else { "inherit" }.to_string()
                    }),
                    important: false,
                    reference,
                },
                v => {
                    return ftd::interpreter::utils::e2(
                        format!("expected boolean, found: {:?}", v),
                        doc.name,
                        line_number,
                    )
                }
            }
        } else if name.eq("background-attachment") {
            match value {
                ftd::interpreter::Value::Boolean { value: v } => ftd::ConditionalValue {
                    value: serde_json::Value::String({
                        if v { "fixed" } else { "inherit" }.to_string()
                    }),
                    important: false,
                    reference,
                },
                v => {
                    return ftd::interpreter::utils::e2(
                        format!("expected boolean, found: {:?}", v),
                        doc.name,
                        line_number,
                    )
                }
            }
        } else if name.eq("line-clamp") {
            match value {
                ftd::interpreter::Value::Integer { value: v } => ftd::ConditionalValue {
                    value: serde_json::json!(v),
                    important: false,
                    reference,
                },
                v => {
                    return ftd::interpreter::utils::e2(
                        format!("expected int, found5: {:?}", v),
                        doc.name,
                        line_number,
                    )
                }
            }
        } else if name.eq("grid-template-areas") {
            match value {
                ftd::interpreter::Value::String { text: v, .. } => {
                    let areas = v.split('|').map(|v| v.trim()).collect::<Vec<&str>>();
                    let mut css_areas = "".to_string();
                    for area in areas {
                        css_areas = format!("{}'{}'", css_areas, area);
                    }
                    ftd::ConditionalValue {
                        value: serde_json::Value::String(css_areas),
                        important: false,
                        reference,
                    }
                }
                v => {
                    return ftd::interpreter::utils::e2(
                        format!("expected string, found 13: {:?}", v),
                        doc.name,
                        line_number,
                    )
                }
            }
        } else {
            return ftd::interpreter::utils::e2(
                format!("unknown style name: `{}` value:`{:?}`", name, value),
                doc.name,
                line_number,
            );
        })
    }
}*/
