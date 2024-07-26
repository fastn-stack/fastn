#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Component {
    pub root: String,
    pub full_name: String,
    pub arguments: ftd::Map<ftd::ftd2021::p2::Kind>,
    pub locals: ftd::Map<ftd::ftd2021::p2::Kind>,
    pub properties: ftd::Map<Property>,
    pub instructions: Vec<Instruction>,
    pub events: Vec<ftd::ftd2021::p2::Event>,
    pub condition: Option<ftd::ftd2021::p2::Boolean>,
    pub kernel: bool,
    pub invocations: Vec<ftd::Map<ftd::Value>>,
    pub line_number: usize,
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
    pub fn without_line_number(&mut self) {
        match self {
            Instruction::ChildComponent { child } => {
                child.line_number = 0;
            }
            Instruction::Component { parent, children } => {
                parent.line_number = 0;
                for child in children {
                    child.line_number = 0;
                }
            }
            Instruction::RecursiveChildComponent { child } => {
                child.line_number = 0;
            }
            _ => {}
        };
    }

    pub fn resolve_id(&self) -> Option<&str> {
        let id = match self {
            Instruction::ChildComponent { child } => child.properties.get("id"),
            Instruction::Component { parent, .. } => parent.properties.get("id"),
            _ => None,
        };
        if let Some(property) = id {
            if let Some(ftd::PropertyValue::Value {
                value: ftd::ftd2021::variable::Value::String { text, .. },
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
    pub condition: Option<ftd::ftd2021::p2::Boolean>,
    pub properties: ftd::Map<Property>,
    pub arguments: ftd::Map<ftd::ftd2021::p2::Kind>,
    pub events: Vec<ftd::ftd2021::p2::Event>,
    pub is_recursive: bool,
    pub line_number: usize,
    pub reference: Option<(String, ftd::ftd2021::p2::Kind)>,
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Property {
    pub default: Option<ftd::PropertyValue>,
    pub conditions: Vec<(ftd::ftd2021::p2::Boolean, ftd::PropertyValue)>,
    pub nested_properties: ftd::Map<ftd::ftd2021::component::Property>,
}

#[derive(Debug, Clone)]
pub struct ElementWithContainer {
    pub element: ftd::Element,
    pub children: Vec<ftd::Element>,
    pub child_container: Option<ftd::Map<Vec<Vec<usize>>>>,
}

impl Property {
    fn eval(
        &self,
        line_number: usize,
        name: &str,
        doc: &ftd::ftd2021::p2::TDoc,
    ) -> ftd::ftd2021::p1::Result<&ftd::PropertyValue> {
        let mut property_value = ftd::ftd2021::p2::utils::e2(
            format!("condition is not complete, name: {}", name),
            doc.name,
            line_number,
        );
        if let Some(property) = &self.default {
            property_value = Ok(property);
        }
        for (boolean, property) in &self.conditions {
            if boolean.eval(line_number, doc)? {
                property_value = Ok(property);
            }
        }
        property_value
    }

    pub(crate) fn add_default_properties(
        reference: &ftd::Map<Property>,
        properties: &mut ftd::Map<Property>,
    ) {
        for (key, arg) in reference {
            if universal_arguments().contains_key(key) {
                properties
                    .entry(key.to_string())
                    .or_insert_with(|| arg.to_owned());
            }
        }
    }

    /// returns the value as string from property.default
    ///
    /// returns empty string in case if it's None
    pub fn resolve_default_value_string(
        &self,
        doc: &ftd::ftd2021::p2::TDoc,
        line_number: usize,
    ) -> ftd::ftd2021::p1::Result<String> {
        if let Some(property_value) = &self.default {
            if let Some(val) = property_value.resolve(line_number, doc)?.to_string() {
                return Ok(val);
            }
        }
        Ok("".to_string())
    }
}

impl ChildComponent {
    pub fn super_call(
        &self,
        children: &[Self],
        doc: &mut ftd::ftd2021::p2::TDoc,
        invocations: &mut ftd::Map<Vec<ftd::Map<ftd::Value>>>,
        local_container: &[usize],
        external_children_count: &Option<usize>,
    ) -> ftd::ftd2021::p1::Result<ElementWithContainer> {
        let id = ftd::ftd2021::p2::utils::string_optional(
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
                            ftd::Instruction::RecursiveChildComponent {
                                child: child.to_owned(),
                            }
                        } else {
                            ftd::Instruction::ChildComponent {
                                child: child.to_owned(),
                            }
                        }
                    })
                    .collect::<Vec<ftd::Instruction>>();
                let elements = ftd::ftd2021::execute_doc::ExecuteDoc {
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
                let root_name = ftd::ftd2021::p2::utils::get_root_component_name(
                    doc,
                    self.root.as_str(),
                    self.line_number,
                )?;
                match root_name.as_str() {
                    "ftd#row" | "ftd#column" | "ftd#scene" | "ftd#grid" | "ftd#text" => {}
                    t => {
                        return ftd::ftd2021::p2::utils::e2(
                            format!("{} cant have children", t),
                            doc.name,
                            self.line_number,
                        )
                    }
                }
            }
            (ftd::Element::Markup(_), _) => {}
            (t, false) => {
                return ftd::ftd2021::p2::utils::e2(
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
        doc: &mut ftd::ftd2021::p2::TDoc,
        invocations: &mut ftd::Map<Vec<ftd::Map<ftd::Value>>>,
        is_child: bool,
        local_container: &[usize],
    ) -> ftd::ftd2021::p1::Result<Vec<ElementWithContainer>> {
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
                if let Ok(ftd::PropertyValue::Reference { name, .. }) = value.eval(0, "$loop$", doc)
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

        fn construct_tmp_data(kind: &ftd::ftd2021::p2::Kind) -> Option<ftd::PropertyValue> {
            // todo: fix it for all kind (Arpita)
            match kind {
                ftd::ftd2021::p2::Kind::String { .. } => Some(ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: "$loop$".to_string(),
                        source: ftd::TextSource::Header,
                    },
                }),
                ftd::ftd2021::p2::Kind::Integer { .. } => Some(ftd::PropertyValue::Value {
                    value: ftd::Value::Integer { value: 0 },
                }),
                ftd::ftd2021::p2::Kind::Decimal { .. } => Some(ftd::PropertyValue::Value {
                    value: ftd::Value::Decimal { value: 0.0 },
                }),
                ftd::ftd2021::p2::Kind::Boolean { .. } => Some(ftd::PropertyValue::Value {
                    value: ftd::Value::Boolean { value: false },
                }),
                ftd::ftd2021::p2::Kind::Optional { kind, .. } => {
                    construct_tmp_data(kind).map(|v| v.into_optional())
                }
                _ => None,
            }
        }

        #[allow(clippy::too_many_arguments)]
        fn construct_element(
            child_component: &ChildComponent,
            d: &ftd::PropertyValue,
            index: usize,
            root: &ftd::Component,
            doc: &mut ftd::ftd2021::p2::TDoc,
            invocations: &mut ftd::Map<Vec<ftd::Map<ftd::Value>>>,
            is_child: bool,
            local_container: &[usize],
        ) -> ftd::ftd2021::p1::Result<ElementWithContainer> {
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
                ftd::ftd2021::p2::utils::get_string_container(local_container.as_slice());
            let loop_name = doc.resolve_name(0, format!("$loop$@{}", string_container).as_str())?;
            doc.local_variables.insert(
                loop_name,
                ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
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
        doc: &mut ftd::ftd2021::p2::TDoc,
        invocations: &mut ftd::Map<Vec<ftd::Map<ftd::Value>>>,
        is_child: bool,
        local_container: &[usize],
        id: Option<String>,
        external_children_count: &Option<usize>,
    ) -> ftd::ftd2021::p1::Result<ElementWithContainer> {
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
        p1: &ftd::ftd2021::p1::Header,
        caption: &Option<String>,
        body: &Option<(usize, String)>,
        doc: &ftd::ftd2021::p2::TDoc,
        arguments: &ftd::Map<ftd::ftd2021::p2::Kind>,
    ) -> ftd::ftd2021::p1::Result<Self> {
        let mut reference = None;
        let root = if let Some(ftd::ftd2021::p2::Kind::UI { default }) =
            arguments.get(name).map(|v| v.inner())
        {
            reference = Some((
                name.to_string(),
                ftd::ftd2021::p2::Kind::UI {
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
                Some(expr) => Some(ftd::ftd2021::p2::Boolean::from_expression(
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
            doc: &ftd::ftd2021::p2::TDoc,
            arguments: &ftd::Map<ftd::ftd2021::p2::Kind>,
            inherits: Vec<String>,
        ) -> ftd::ftd2021::p1::Result<ftd::Map<Property>> {
            let mut properties: ftd::Map<Property> =
                root_properties_from_inherits(line_number, arguments, inherits, doc)?;
            if let Some(caption) = caption {
                if let Ok(name) = doc.resolve_name(line_number, name) {
                    let kind = match name.as_str() {
                        "ftd#integer" => ftd::ftd2021::p2::Kind::integer(),
                        "ftd#boolean" => ftd::ftd2021::p2::Kind::boolean(),
                        "ftd#decimal" => ftd::ftd2021::p2::Kind::decimal(),
                        _ => return Ok(properties),
                    };
                    if let Ok(property_value) = ftd::PropertyValue::resolve_value(
                        line_number,
                        caption,
                        Some(kind),
                        doc,
                        arguments,
                        None,
                    ) {
                        properties.insert(
                            "value".to_string(),
                            ftd::ftd2021::component::Property {
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

fn markup_get_named_container(
    children: &[ChildComponent],
    root: &str,
    line_number: usize,
    doc: &mut ftd::ftd2021::p2::TDoc,
    invocations: &mut ftd::Map<Vec<ftd::Map<ftd::Value>>>,
    local_container: &[usize],
) -> ftd::ftd2021::p1::Result<ftd::Map<ftd::Element>> {
    let children = {
        let mut children = children.to_vec();
        let root_name = ftd::ftd2021::p2::utils::get_root_component_name(doc, root, line_number)?;
        if root_name.eq("ftd#text") {
            let mut name = root.to_string();
            while name != "ftd.kernel" {
                let component = doc.get_component(line_number, name.as_str())?;
                for instruction in component.instructions {
                    if let ftd::Instruction::ChildComponent { child } = instruction {
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
                ftd::Instruction::RecursiveChildComponent { child }
            } else {
                ftd::Instruction::ChildComponent { child }
            }
        })
        .collect::<Vec<ftd::Instruction>>();

    let container_children = ftd::ftd2021::execute_doc::ExecuteDoc {
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
        doc: &ftd::ftd2021::p2::TDoc,
    ) -> ftd::ftd2021::p1::Result<ftd::Map<ftd::Element>> {
        let mut named_container = ftd::Map::new();
        for (idx, container) in container_children.iter().enumerate() {
            match elements_name.get(idx) {
                Some(name) => {
                    named_container.insert(name.to_string(), container.to_owned());
                }
                None => {
                    return ftd::ftd2021::p2::utils::e2(
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

/// In case markup the behaviour of container_children is not the same.
/// They act as the component variables which are, then, referred to in markup text
/// container_children copy there properties to the reference in markup text
fn reevalute_markups(
    markups: &mut ftd::Markups,
    named_container: ftd::Map<ftd::Element>,
    doc: &mut ftd::ftd2021::p2::TDoc,
) -> ftd::ftd2021::p1::Result<()> {
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
                    ftd::ftd2021::rendered::markup(v)
                } else {
                    ftd::ftd2021::rendered::markup_line(v)
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
    doc: &mut ftd::ftd2021::p2::TDoc,
) -> ftd::ftd2021::p1::Result<()> {
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
                    text: ftd::ftd2021::rendered::markup_line(traverse_string.as_str()),
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
                text: ftd::ftd2021::rendered::markup_line(traverse_string.as_str()),
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
    fn get_inner_text(
        text: &[char],
        idx: &mut usize,
        doc_id: &str,
    ) -> ftd::ftd2021::p1::Result<String> {
        let mut stack = vec!['{'];
        let mut traverse_string = "".to_string();
        while !stack.is_empty() {
            *idx += 1;
            if *idx >= text.len() {
                return ftd::ftd2021::p2::utils::e2(
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
        doc: &mut ftd::ftd2021::p2::TDoc,
        text: Option<&str>,
        root: &str,
        named_container: &ftd::Map<ftd::Element>,
    ) -> ftd::ftd2021::p1::Result<ftd::IText> {
        Ok(match element {
            ftd::Element::Integer(t) => {
                let t = {
                    let mut t = t.clone();
                    if let Some(text) = text {
                        t.text = ftd::ftd2021::rendered::markup_line(text);
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
                        t.text = ftd::ftd2021::rendered::markup_line(text);
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
                        t.text = ftd::ftd2021::rendered::markup_line(text);
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
                        t.text = ftd::ftd2021::rendered::markup_line(text);
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
                        t.text = ftd::ftd2021::rendered::markup_line(text);
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
                return ftd::ftd2021::p2::utils::e2(
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
        doc: &mut ftd::ftd2021::p2::TDoc,
        name: &str,
    ) -> ftd::ftd2021::p1::Result<ftd::Element> {
        let mut root =
            doc.get_component(0, name)
                .map_err(|_| ftd::ftd2021::p1::Error::ParseError {
                    message: format!("This component not found in ftd.text {}", name),
                    doc_id: doc.name.to_string(),
                    line_number: 0,
                })?;

        let property_value = if let Some(p) = root.properties.get("text") {
            p
        } else if let Some(p) = root.properties.get("value") {
            p
        } else {
            return ftd::ftd2021::p2::utils::e2(
                format!(
                    "expected type for ftd.text are text, integer, decimal and boolean, {:?}",
                    root
                ),
                doc.name,
                0,
            );
        };

        if let ftd::ftd2021::component::Property {
            default: Some(ftd::PropertyValue::Variable { kind, .. }),
            ..
        } = property_value
        {
            if !kind.has_default_value() {
                let property = ftd::ftd2021::component::Property {
                    default: Some(ftd::PropertyValue::Value {
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

fn resolve_recursive_property(
    line_number: usize,
    self_properties: &ftd::Map<Property>,
    doc: &ftd::ftd2021::p2::TDoc,
) -> ftd::ftd2021::p1::Result<ftd::Value> {
    if let Some(value) = self_properties.get("$loop$") {
        if let Ok(property_value) = value.eval(line_number, "$loop$", doc) {
            return property_value.resolve(line_number, doc);
        }
    }
    ftd::ftd2021::p2::utils::e2(
        format!("$loop$ not found in properties {:?}", self_properties),
        doc.name,
        line_number,
    )
}

pub fn resolve_properties(
    line_number: usize,
    self_properties: &ftd::Map<Property>,
    doc: &ftd::ftd2021::p2::TDoc,
) -> ftd::ftd2021::p1::Result<ftd::Map<ftd::Value>> {
    resolve_properties_by_id(line_number, self_properties, doc, None)
}

pub fn resolve_properties_by_id(
    line_number: usize,
    self_properties: &ftd::Map<Property>,
    doc: &ftd::ftd2021::p2::TDoc,
    id: Option<String>,
) -> ftd::ftd2021::p1::Result<ftd::Map<ftd::Value>> {
    let mut properties: ftd::Map<ftd::Value> = Default::default();
    for (name, value) in self_properties.iter() {
        if name == "$loop$" {
            continue;
        }
        if let Some(ref id) = id {
            if !id.eq(name) {
                continue;
            }
        }
        if let Ok(property_value) = value.eval(line_number, name, doc) {
            properties.insert(name.to_string(), property_value.resolve(line_number, doc)?);
        }
    }
    Ok(properties)
}

fn get_conditional_attributes(
    line_number: usize,
    properties: &ftd::Map<Property>,
    doc: &ftd::ftd2021::p2::TDoc,
) -> ftd::ftd2021::p1::Result<ftd::Map<ftd::ConditionalAttribute>> {
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
        condition: &ftd::ftd2021::p2::Boolean,
        pv: &ftd::PropertyValue,
        value: &ftd::Value,
    ) -> bool {
        let bool_name = if let ftd::ftd2021::p2::Boolean::IsNotNull {
            value:
                ftd::PropertyValue::Reference { name, .. } | ftd::PropertyValue::Variable { name, .. },
        } = condition
        {
            name
        } else {
            return false;
        };

        let pv_name = match pv {
            ftd::PropertyValue::Reference { name, .. }
            | ftd::PropertyValue::Variable { name, .. } => name,
            _ => return false,
        };

        if !bool_name.eq(pv_name) {
            return false;
        }

        match value {
            ftd::Value::None { .. } => true,
            ftd::Value::Optional { data, .. } if data.as_ref().eq(&None) => true,
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
        value: ftd::Value,
        doc: &ftd::ftd2021::p2::TDoc,
        line_number: usize,
        reference: Option<String>,
    ) -> ftd::ftd2021::p1::Result<ftd::ConditionalValue> {
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

        let style_length = [
            "width",
            "min-width",
            "max-width",
            "height",
            "min-height",
            "max-height",
        ];

        let style_color = ["background-color", "color", "border-color", "shadow-color"];

        let style_integer_important = [
            "border-left-width",
            "border-right-width",
            "border-top-width",
            "border-bottom-width",
            "border-top-left-radius",
            "border-top-right-radius",
            "border-bottom-left-radius",
            "border-bottom-right-radius",
        ];

        let style_string = [
            "cursor",
            "position",
            "align",
            "background-image",
            "grid-template-columns",
            "grid-template-rows",
            "grid-area",
        ];

        let style_overflow = ["overflow-x", "overflow-y"];

        let style_boolean = ["background-repeat"];

        Ok(if style_integer.contains(&name) {
            match value {
                ftd::Value::Integer { value: v } => ftd::ConditionalValue {
                    value: serde_json::Value::String(format!("{}px", v)),
                    important: false,
                    reference,
                },
                v => {
                    return ftd::ftd2021::p2::utils::e2(
                        format!("expected int, found3: {:?}", v),
                        doc.name,
                        line_number,
                    )
                }
            }
        } else if style_integer_important.contains(&name) {
            match value {
                ftd::Value::Integer { value: v } => ftd::ConditionalValue {
                    value: serde_json::Value::String(format!("{}px", v)),
                    important: true,
                    reference,
                },
                v => {
                    return ftd::ftd2021::p2::utils::e2(
                        format!("expected int, found4: {:?}", v),
                        doc.name,
                        line_number,
                    )
                }
            }
        } else if style_length.contains(&name) {
            match value {
                ftd::Value::String { text: v, .. } => ftd::ConditionalValue {
                    value: serde_json::Value::String(
                        ftd::length(&ftd::Length::from(Some(v), doc.name)?.unwrap(), name).1,
                    ),
                    important: false,
                    reference,
                },
                v => {
                    return ftd::ftd2021::p2::utils::e2(
                        format!("expected string, found 8: {:?}", v),
                        doc.name,
                        line_number,
                    )
                }
            }
        } else if style_color.contains(&name) {
            match value {
                ftd::Value::Record { fields, .. } => {
                    let properties = fields
                        .iter()
                        .map(|(k, v)| v.resolve(line_number, doc).map(|v| (k.to_string(), v)))
                        .collect::<ftd::ftd2021::p1::Result<ftd::Map<ftd::Value>>>()?;
                    let light = if let Some(light) = ftd::ftd2021::p2::element::color_from(
                        ftd::ftd2021::p2::utils::string_optional(
                            "light",
                            &properties,
                            doc.name,
                            0,
                        )?,
                        doc.name,
                    )? {
                        ftd::ftd2021::html::color(&light)
                    } else {
                        "auto".to_string()
                    };
                    let dark = if let Some(dark) = ftd::ftd2021::p2::element::color_from(
                        ftd::ftd2021::p2::utils::string_optional("dark", &properties, doc.name, 0)?,
                        doc.name,
                    )? {
                        ftd::ftd2021::html::color(&dark)
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
                    return ftd::ftd2021::p2::utils::e2(
                        format!("expected string, found 9: {:?}", v),
                        doc.name,
                        line_number,
                    )
                }
            }
        } else if style_overflow.contains(&name) {
            match value {
                ftd::Value::String { text: v, .. } => ftd::ConditionalValue {
                    value: serde_json::Value::String(
                        ftd::overflow(&ftd::Overflow::from(Some(v), doc.name)?.unwrap(), name).1,
                    ),
                    important: false,
                    reference,
                },
                v => {
                    return ftd::ftd2021::p2::utils::e2(
                        format!("expected string, found 10: {:?}", v),
                        doc.name,
                        line_number,
                    )
                }
            }
        } else if style_string.contains(&name) {
            match value {
                ftd::Value::String { text: v, .. } => ftd::ConditionalValue {
                    value: serde_json::Value::String(v),
                    important: false,
                    reference,
                },
                v => {
                    return ftd::ftd2021::p2::utils::e2(
                        format!("expected string, found 11: {:?}", v),
                        doc.name,
                        line_number,
                    )
                }
            }
        } else if style_boolean.contains(&name) {
            match value {
                ftd::Value::Boolean { value: v } => ftd::ConditionalValue {
                    value: serde_json::Value::Bool(v),
                    important: false,
                    reference,
                },
                v => {
                    return ftd::ftd2021::p2::utils::e2(
                        format!("expected string, found 12: {:?}", v),
                        doc.name,
                        line_number,
                    )
                }
            }
        } else if name.eq("sticky") {
            match value {
                ftd::Value::Boolean { value: v } => ftd::ConditionalValue {
                    value: serde_json::Value::String({
                        if v { "sticky" } else { "inherit" }.to_string()
                    }),
                    important: false,
                    reference,
                },
                v => {
                    return ftd::ftd2021::p2::utils::e2(
                        format!("expected boolean, found: {:?}", v),
                        doc.name,
                        line_number,
                    )
                }
            }
        } else if name.eq("background-attachment") {
            match value {
                ftd::Value::Boolean { value: v } => ftd::ConditionalValue {
                    value: serde_json::Value::String({
                        if v { "fixed" } else { "inherit" }.to_string()
                    }),
                    important: false,
                    reference,
                },
                v => {
                    return ftd::ftd2021::p2::utils::e2(
                        format!("expected boolean, found: {:?}", v),
                        doc.name,
                        line_number,
                    )
                }
            }
        } else if name.eq("line-clamp") {
            match value {
                ftd::Value::Integer { value: v } => ftd::ConditionalValue {
                    value: serde_json::json!(v),
                    important: false,
                    reference,
                },
                v => {
                    return ftd::ftd2021::p2::utils::e2(
                        format!("expected int, found5: {:?}", v),
                        doc.name,
                        line_number,
                    )
                }
            }
        } else if name.eq("grid-template-areas") {
            match value {
                ftd::Value::String { text: v, .. } => {
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
                    return ftd::ftd2021::p2::utils::e2(
                        format!("expected string, found 13: {:?}", v),
                        doc.name,
                        line_number,
                    )
                }
            }
        } else {
            return ftd::ftd2021::p2::utils::e2(
                format!("unknown style name: `{}` value:`{:?}`", name, value),
                doc.name,
                line_number,
            );
        })
    }
}

pub(crate) fn resolve_properties_with_ref(
    line_number: usize,
    self_properties: &ftd::Map<Property>,
    doc: &ftd::ftd2021::p2::TDoc,
) -> ftd::ftd2021::p1::Result<ftd::Map<(ftd::Value, Option<String>)>> {
    let mut properties: ftd::Map<(ftd::Value, Option<String>)> = Default::default();
    for (name, value) in self_properties.iter() {
        if name == "$loop$" {
            continue;
        }
        if let Ok(property_value) = value.eval(line_number, name, doc) {
            let reference = match property_value {
                ftd::PropertyValue::Reference { name, .. } => Some(name.to_string()),
                ftd::PropertyValue::Variable { name, .. } => Some(name.to_string()),
                _ => None,
            };
            let resolved_value = {
                let mut resolved_value = property_value.resolve(line_number, doc)?;
                if let ftd::Value::UI { data, .. } = &mut resolved_value {
                    data.extend(value.nested_properties.clone())
                }
                resolved_value
            };

            properties.insert(name.to_string(), (resolved_value, reference));
        }
    }
    Ok(properties)
}

impl Component {
    fn call_sub_functions(
        &self,
        doc: &mut ftd::ftd2021::p2::TDoc,
        invocations: &mut ftd::Map<Vec<ftd::Map<ftd::Value>>>,
        call_container: &[usize],
        id: Option<String>,
    ) -> ftd::ftd2021::p1::Result<ElementWithContainer> {
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

        return ftd::ftd2021::execute_doc::ExecuteDoc {
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
            doc: &ftd::ftd2021::p2::TDoc,
        ) -> ftd::ftd2021::p1::Result<()> {
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
                                if matches!(kind, ftd::ftd2021::p2::Kind::UI { .. }) =>
                            {
                                if let Some(ftd::ftd2021::p2::Boolean::IsNotNull {
                                    value:
                                        ftd::PropertyValue::Reference { ref name, .. }
                                        | ftd::PropertyValue::Variable { ref name, .. },
                                }) = child.condition
                                {
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
                            }
                            _ => {}
                        }
                        return ftd::ftd2021::p2::utils::e2(
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

    pub fn get_caption(&self) -> Option<String> {
        let mut new_caption_title = None;
        for (arg, arg_kind) in self.arguments.clone() {
            if let ftd::ftd2021::p2::Kind::String { caption, .. } = arg_kind {
                if caption {
                    new_caption_title = Some(arg);
                }
            }
        }
        new_caption_title
    }

    pub fn from_p1(
        p1: &ftd::ftd2021::p1::Section,
        doc: &ftd::ftd2021::p2::TDoc,
    ) -> ftd::ftd2021::p1::Result<Self> {
        let var_data = ftd::ftd2021::variable::VariableData::get_name_kind(
            &p1.name,
            doc,
            p1.line_number,
            vec![].as_slice(),
        )?;
        if var_data.is_variable() {
            return ftd::ftd2021::p2::utils::e2(
                format!("expected component, found: {}", p1.name),
                doc.name,
                p1.line_number,
            );
        }
        let name = var_data.name;
        let root = doc.resolve_name(p1.line_number, var_data.kind.as_str())?;
        let root_component = doc.get_component(p1.line_number, root.as_str())?;
        let (mut arguments, inherits) = read_arguments(
            &p1.header,
            root.as_str(),
            &root_component.arguments,
            &Default::default(),
            doc,
        )?;

        // Extend the local arguments with universal arguments
        arguments.extend(universal_arguments());

        assert_no_extra_properties(
            p1.line_number,
            &p1.header,
            root.as_str(),
            &root_component.arguments,
            &p1.name,
            doc,
        )?;
        let mut instructions: Vec<Instruction> = Default::default();

        for sub in p1.sub_sections.0.iter() {
            if sub.is_commented {
                continue;
            }
            if let Ok(loop_data) = sub.header.str(doc.name, p1.line_number, "$loop$") {
                instructions.push(Instruction::RecursiveChildComponent {
                    child: recursive_child_component(
                        loop_data,
                        sub,
                        doc,
                        &arguments,
                        Some((name.to_string(), root_component.to_owned())),
                    )?,
                });
                continue;
            }

            instructions.push(if sub.name == "container" {
                Instruction::ChangeContainer {
                    name: doc.resolve_name_without_full_path(
                        sub.line_number,
                        sub.caption(doc.name)?.as_str(),
                    )?,
                }
            } else {
                let child = if ftd::ftd2021::p2::utils::get_root_component_name(
                    doc,
                    root_component.full_name.as_str(),
                    sub.line_number,
                )?
                .eq("ftd#text")
                {
                    ftd::ftd2021::p2::utils::get_markup_child(sub, doc, &arguments)?
                } else {
                    ftd::ChildComponent::from_p1(
                        sub.line_number,
                        sub.name.as_str(),
                        &sub.header,
                        &sub.caption,
                        &sub.body,
                        doc,
                        &arguments,
                    )?
                };
                Instruction::ChildComponent { child }
            });
        }

        let condition = match p1.header.str_optional(doc.name, p1.line_number, "if")? {
            Some(expr) => Some(ftd::ftd2021::p2::Boolean::from_expression(
                expr,
                doc,
                &arguments,
                (None, None),
                p1.line_number,
            )?),
            None => None,
        };

        let events = p1.header.get_events(p1.line_number, doc, &arguments)?;

        assert_caption_body_checks(
            &root,
            &p1.header,
            doc,
            &p1.caption,
            &p1.body,
            p1.line_number,
        )?;

        Ok(Component {
            full_name: doc.resolve_name(p1.line_number, &name)?,
            properties: read_properties(
                p1.line_number,
                &p1.header,
                &p1.caption,
                &p1.body,
                name.as_str(),
                root.as_str(),
                &root_component.arguments,
                &arguments,
                doc,
                &root_properties_from_inherits(p1.line_number, &arguments, inherits, doc)?,
                false,
            )?,
            arguments,
            locals: Default::default(),
            root,
            instructions,
            kernel: false,
            invocations: Default::default(),
            condition,
            events,
            line_number: p1.line_number,
        })
    }

    fn call_without_values(
        &self,
        doc: &mut ftd::ftd2021::p2::TDoc,
    ) -> ftd::ftd2021::p1::Result<ElementWithContainer> {
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
        doc: &mut ftd::ftd2021::p2::TDoc,
        invocations: &mut ftd::Map<Vec<ftd::Map<ftd::Value>>>,
        condition: &Option<ftd::ftd2021::p2::Boolean>,
        is_child: bool,
        events: &[ftd::ftd2021::p2::Event],
        local_container: &[usize],
        id: Option<String>,
        external_children_count: &Option<usize>,
    ) -> ftd::ftd2021::p1::Result<ElementWithContainer> {
        invocations
            .entry(self.full_name.clone())
            .or_default()
            .push(resolve_properties(0, arguments, doc)?);
        if self.root == "ftd.kernel" {
            let element = match self.full_name.as_str() {
                "ftd#text-block" => {
                    ftd::Element::TextBlock(ftd::ftd2021::p2::element::text_block_from_properties(
                        arguments, doc, condition, is_child, events,
                    )?)
                }
                "ftd#code" => ftd::Element::Code(ftd::ftd2021::p2::element::code_from_properties(
                    arguments, doc, condition, is_child, events,
                )?),
                "ftd#image" => {
                    ftd::Element::Image(ftd::ftd2021::p2::element::image_from_properties(
                        arguments, doc, condition, is_child, events,
                    )?)
                }
                "ftd#row" => ftd::Element::Row(ftd::ftd2021::p2::element::row_from_properties(
                    arguments, doc, condition, is_child, events,
                )?),
                "ftd#column" => {
                    ftd::Element::Column(ftd::ftd2021::p2::element::column_from_properties(
                        arguments, doc, condition, is_child, events,
                    )?)
                }
                "ftd#iframe" => {
                    ftd::Element::IFrame(ftd::ftd2021::p2::element::iframe_from_properties(
                        arguments, doc, condition, is_child, events,
                    )?)
                }
                "ftd#integer" => {
                    ftd::Element::Integer(ftd::ftd2021::p2::element::integer_from_properties(
                        arguments, doc, condition, is_child, events,
                    )?)
                }
                "ftd#decimal" => {
                    ftd::Element::Decimal(ftd::ftd2021::p2::element::decimal_from_properties(
                        arguments, doc, condition, is_child, events,
                    )?)
                }
                "ftd#boolean" => {
                    ftd::Element::Boolean(ftd::ftd2021::p2::element::boolean_from_properties(
                        arguments, doc, condition, is_child, events,
                    )?)
                }
                "ftd#input" => {
                    ftd::Element::Input(ftd::ftd2021::p2::element::input_from_properties(
                        arguments, doc, condition, is_child, events,
                    )?)
                }
                "ftd#scene" => {
                    ftd::Element::Scene(ftd::ftd2021::p2::element::scene_from_properties(
                        arguments, doc, condition, is_child, events,
                    )?)
                }
                "ftd#grid" => ftd::Element::Grid(ftd::ftd2021::p2::element::grid_from_properties(
                    arguments, doc, condition, is_child, events,
                )?),
                "ftd#text" => {
                    ftd::Element::Markup(ftd::ftd2021::p2::element::markup_from_properties(
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

            let events = ftd::ftd2021::p2::Event::get_events(self.line_number, events, doc)?;

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
                                ftd::ftd2021::p2::utils::split(append_at.to_string(), ".")?.1
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

    pub fn to_value(&self, kind: &ftd::ftd2021::p2::Kind) -> ftd::ftd2021::p1::Result<ftd::Value> {
        Ok(ftd::Value::UI {
            name: self.full_name.to_string(),
            kind: kind.to_owned(),
            data: Default::default(),
        })
    }
}

pub fn recursive_child_component(
    loop_data: &str,
    sub: &ftd::ftd2021::p1::SubSection,
    doc: &ftd::ftd2021::p2::TDoc,
    arguments: &ftd::Map<ftd::ftd2021::p2::Kind>,
    name_with_component: Option<(String, ftd::Component)>,
) -> ftd::ftd2021::p1::Result<ftd::ChildComponent> {
    let mut loop_ref = "object".to_string();
    let mut loop_on_component = loop_data.to_string();

    if loop_data.contains("as") {
        let parts = ftd::ftd2021::p2::utils::split(loop_data.to_string(), " as ")?;
        loop_on_component = parts.0;
        loop_ref = if let Some(loop_ref) = parts.1.strip_prefix('$') {
            loop_ref.to_string()
        } else {
            return ftd::ftd2021::p2::utils::e2(
                format!("loop variable should start with $, found: {}", parts.1),
                doc.name,
                sub.line_number,
            );
        };
    }

    let recursive_property_value = ftd::PropertyValue::resolve_value(
        sub.line_number,
        &loop_on_component,
        None,
        doc,
        arguments,
        None,
    )?;

    let recursive_kind = if let ftd::ftd2021::p2::Kind::List { kind, .. } =
        recursive_property_value.kind().inner()
    {
        kind.as_ref().to_owned()
    } else {
        return ftd::ftd2021::p2::utils::e2(
            format!(
                "expected list for loop, found: {:?}",
                recursive_property_value.kind(),
            ),
            doc.name,
            sub.line_number,
        );
    };

    let mut properties: ftd::Map<Property> = Default::default();

    properties.insert(
        "$loop$".to_string(),
        ftd::ftd2021::component::Property {
            default: Some(recursive_property_value),
            conditions: vec![],
            ..Default::default()
        },
    );

    let mut new_header = ftd::ftd2021::p1::Header(vec![]);
    let (mut left_boolean, mut right_boolean) = (None, None);
    for (i, k, v) in &sub.header.0 {
        if k == "$loop$" {
            continue;
        }

        if k == "if" && contains_loop_ref(&loop_ref, v) {
            let v = v.replace(&format!("${}", loop_ref), "$loop$");
            let (_, left, right) =
                ftd::ftd2021::p2::Boolean::boolean_left_right(i.to_owned(), &v, doc.name)?;
            if left.contains("$loop$") {
                left_boolean = resolve_loop_reference(i, &recursive_kind, doc, left)?.default;
            }
            if let Some(r) = right {
                if r.contains("$loop$") {
                    right_boolean = resolve_loop_reference(i, &recursive_kind, doc, r)?.default;
                }
            }
        }

        if contains_loop_ref(&loop_ref, v) && v.starts_with(&format!("${}", loop_ref)) {
            let reference = v.to_string().replace(&format!("${}", loop_ref), "$loop$");
            let value = resolve_loop_reference(i, &recursive_kind, doc, reference)?;
            properties.insert(k.to_string(), value);
        } else {
            new_header.add(i, k, v);
        }
    }

    let mut reference = None;

    let (root_arguments, full_name, caption) = match name_with_component {
        Some((name, root_component)) if sub.name == name => (
            root_component.arguments.clone(),
            root_component.full_name.to_string(),
            root_component.get_caption(),
        ),
        _ => {
            let root = if let Some(ftd::ftd2021::p2::Kind::UI { default }) =
                arguments.get(&sub.name).map(|v| v.inner())
            {
                reference = Some((
                    sub.name.to_string(),
                    ftd::ftd2021::p2::Kind::UI {
                        default: (*default).clone(),
                    },
                ));
                ftd::Component {
                    root: "ftd.kernel".to_string(),
                    full_name: "ftd#ui".to_string(),
                    arguments: Default::default(),
                    locals: Default::default(),
                    properties: Default::default(),
                    instructions: vec![],
                    events: vec![],
                    condition: None,
                    kernel: false,
                    invocations: vec![],
                    line_number: sub.line_number,
                }
            } else {
                doc.get_component(sub.line_number, sub.name.as_str())?
            };
            let root_arguments = root.arguments.clone();
            assert_no_extra_properties(
                sub.line_number,
                &new_header,
                root.full_name.as_str(),
                &root_arguments,
                sub.name.as_str(),
                doc,
            )?;
            (
                root_arguments,
                root.full_name.to_string(),
                root.get_caption(),
            )
        }
    };

    let mut new_caption = sub.caption.clone();
    if let (Some(caption), Some(caption_arg)) = (sub.caption.clone(), caption) {
        if contains_loop_ref(&loop_ref, &caption) {
            let reference = caption.replace(&format!("${}", loop_ref), "$loop$");
            let value = resolve_loop_reference(&sub.line_number, &recursive_kind, doc, reference)?;
            properties.insert(caption_arg, value);
            new_caption = None;
        }
    }

    assert_caption_body_checks(
        full_name.as_str(),
        &sub.header,
        doc,
        &sub.caption,
        &sub.body,
        sub.line_number,
    )?;

    properties.extend(read_properties(
        sub.line_number,
        &new_header,
        &new_caption,
        &sub.body,
        &sub.name,
        full_name.as_str(),
        &root_arguments,
        arguments,
        doc,
        &properties,
        reference.is_some(),
    )?);

    return Ok(ftd::ChildComponent {
        root: doc.resolve_name(sub.line_number, &sub.name.to_string())?,
        condition: match sub.header.str_optional(doc.name, sub.line_number, "if")? {
            Some(expr) => Some(ftd::ftd2021::p2::Boolean::from_expression(
                expr,
                doc,
                arguments,
                (left_boolean, right_boolean),
                sub.line_number,
            )?),
            None => None,
        },
        properties,
        arguments: Default::default(),
        events: vec![],
        is_recursive: true,
        line_number: sub.line_number,
        reference,
    });

    fn resolve_loop_reference(
        line_number: &usize,
        recursive_kind: &ftd::ftd2021::p2::Kind,
        doc: &ftd::ftd2021::p2::TDoc,
        reference: String,
    ) -> ftd::ftd2021::p1::Result<Property> {
        let mut arguments: ftd::Map<ftd::ftd2021::p2::Kind> = Default::default();
        arguments.insert("$loop$".to_string(), recursive_kind.to_owned());
        let property = ftd::PropertyValue::resolve_value(
            *line_number,
            &format!("${}", reference),
            None,
            doc,
            &arguments,
            None,
        )?;
        Ok(ftd::ftd2021::component::Property {
            default: Some(property),
            conditions: vec![],
            ..Default::default()
        })
    }

    fn contains_loop_ref(loop_ref: &str, pattern: &str) -> bool {
        let ref1 = format!("${}.", loop_ref);
        let pattern_vec: Vec<&str> = pattern.split(' ').collect();
        let partern_bool = pattern_vec
            .iter()
            .map(|v| v.contains(&ref1) || v == &format!("${}", loop_ref))
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
        || (name == "ftd.text-block")
        || (name == "ftd.code")
        || (name == "ftd.image")
        || (name == "ftd.row")
        || (name == "ftd.column")
        || (name == "ftd.iframe")
        || (name == "ftd.integer")
        || (name == "ftd.decimal")
        || (name == "ftd.boolean")
        || (name == "ftd.input")
        || (name == "ftd.scene")
        || (name == "ftd.grid")
        || (name == "ftd.markup"))
}

fn assert_no_extra_properties(
    line_number: usize,
    p1: &ftd::ftd2021::p1::Header,
    root: &str,
    root_arguments: &ftd::Map<ftd::ftd2021::p2::Kind>,
    name: &str,
    doc: &ftd::ftd2021::p2::TDoc,
) -> ftd::ftd2021::p1::Result<()> {
    for (i, k, _) in p1.0.iter() {
        if k == "component"
            || k.starts_with('$')
            || k == "if"
            || ftd::ftd2021::variable::VariableData::get_name_kind(
                k,
                doc,
                line_number,
                vec![].as_slice(),
            )
            .is_ok()
        {
            continue;
        }
        let key = if k.contains(" if ") {
            let mut parts = k.splitn(2, " if ");
            parts.next().unwrap().trim()
        } else {
            k
        };

        if !(root_arguments.contains_key(key)
            || (is_component(name) && universal_arguments().contains_key(key)))
        {
            return ftd::ftd2021::p2::utils::e2(
                format!(
                    "unknown key found: {}, {} has: {}",
                    k,
                    root,
                    root_arguments
                        .keys()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                doc.name,
                i.to_owned(),
            );
        }
    }

    Ok(())
}

/// Throws error if the user specifies both value and default-value for ftd.input
/// otherwise returns Ok(())
///
/// # No error in these cases
///
/// ```markup
/// -- ftd.input:
/// value: v1
///
/// -- ftd.input:
/// default-value: d1
/// ```
///
/// # Error in this case
///
/// ```markup
/// -- ftd.input:
/// value: v2
/// default-value: d2
/// ```
fn check_input_conflicting_values(
    properties: &ftd::Map<Property>,
    doc: &ftd::ftd2021::p2::TDoc,
    line_number: usize,
) -> ftd::ftd2021::p1::Result<()> {
    fn get_property_default_value(
        property_name: &str,
        properties: &ftd::Map<Property>,
        doc: &ftd::ftd2021::p2::TDoc,
        line_number: usize,
    ) -> ftd::ftd2021::p1::Result<String> {
        if let Some(property) = properties.get(property_name) {
            return property.resolve_default_value_string(doc, line_number);
        }
        Err(ftd::ftd2021::p1::Error::NotFound {
            doc_id: doc.name.to_string(),
            line_number,
            key: property_name.to_string(),
        })
    }

    let contains_value = properties.contains_key("value");
    let contains_default_value = properties.contains_key("default-value");

    match (contains_value, contains_default_value) {
        (true, true) => {
            let value = get_property_default_value("value", properties, doc, line_number)?;
            let default_value =
                get_property_default_value("default-value", properties, doc, line_number)?;

            Err(ftd::ftd2021::p1::Error::ForbiddenUsage {
                message: format!(
                    "value: \'{}\', default-value: \'{}\' both are used in ftd.input",
                    value, default_value
                ),
                doc_id: doc.name.to_string(),
                line_number,
            })
        }
        (_, _) => Ok(()),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn read_properties(
    line_number: usize,
    p1: &ftd::ftd2021::p1::Header,
    caption: &Option<String>,
    body: &Option<(usize, String)>,
    fn_name: &str,
    root: &str,
    root_arguments: &ftd::Map<ftd::ftd2021::p2::Kind>,
    arguments: &ftd::Map<ftd::ftd2021::p2::Kind>,
    doc: &ftd::ftd2021::p2::TDoc,
    root_properties: &ftd::Map<Property>,
    is_reference: bool,
) -> ftd::ftd2021::p1::Result<ftd::Map<Property>> {
    let mut properties: ftd::Map<Property> = Default::default();

    for (name, kind) in root_arguments.iter() {
        if let Some(prop) = root_properties.get(name) {
            properties.insert(name.to_string(), prop.clone());
            continue;
        }
        let (conditional_vector, source) = match (
            p1.conditional_str(doc, line_number, name, arguments),
            kind.inner(),
        ) {
            (Ok(v), _) => (
                v.iter()
                    .map(|(a, b, c, d)| (Some(a.to_owned()), b.to_owned(), c.to_owned(), *d))
                    .collect::<Vec<(Option<usize>, String, Option<String>, bool)>>(),
                ftd::TextSource::Header,
            ),
            (
                Err(ftd::ftd2021::p1::Error::NotFound { .. }),
                ftd::ftd2021::p2::Kind::String {
                    caption: c,
                    body: b,
                    default: d,
                    is_reference: r,
                },
            ) => {
                if *c && caption.is_some() {
                    (
                        vec![(None, caption.as_ref().unwrap().to_string(), None, *r)],
                        ftd::TextSource::Caption,
                    )
                } else if *b && body.is_some() {
                    (
                        vec![(None, body.as_ref().unwrap().1.to_string(), None, *r)],
                        ftd::TextSource::Body,
                    )
                } else if matches!(kind, ftd::ftd2021::p2::Kind::Optional { .. }) {
                    continue;
                } else if let Some(d) = d {
                    (
                        vec![(None, d.to_string(), None, *r)],
                        ftd::TextSource::Default,
                    )
                } else if is_reference {
                    continue;
                } else {
                    return ftd::ftd2021::p2::utils::e2(
                        format!(
                            "{} is calling {}, without a required argument 1 `{}`",
                            fn_name, root, name
                        ),
                        doc.name,
                        line_number,
                    );
                }
            }
            (Err(ftd::ftd2021::p1::Error::NotFound { .. }), k) => {
                if matches!(kind, ftd::ftd2021::p2::Kind::Optional { .. }) {
                    continue;
                }

                if let Some(d) = k.get_default_value_str() {
                    (
                        vec![(None, d.to_string(), None, k.is_reference())],
                        ftd::TextSource::Default,
                    )
                } else if is_reference {
                    continue;
                } else {
                    return ftd::ftd2021::p2::utils::e2(
                        format!(
                            "{} is calling {}, without a required argument `{}`",
                            fn_name, root, name
                        ),
                        doc.name,
                        line_number,
                    );
                }
            }
            (Err(e), _) => {
                return Err(e);
            }
        };
        for (idx, value, conditional_attribute, is_referenced) in conditional_vector {
            if kind.is_reference() && !is_referenced {
                return ftd::ftd2021::p2::utils::e2(
                    format!(
                        "{} is calling {}, without a referenced argument `{}`",
                        fn_name, root, value
                    ),
                    doc.name,
                    line_number,
                );
            }
            let mut property_value = match ftd::PropertyValue::resolve_value(
                line_number,
                value.as_str(),
                Some(kind.to_owned()),
                doc,
                arguments,
                Some(source.clone()),
            ) {
                Ok(p) => p,
                _ if source.eq(&ftd::TextSource::Default) => ftd::PropertyValue::resolve_value(
                    line_number,
                    value.as_str(),
                    Some(kind.to_owned()),
                    doc,
                    root_arguments,
                    Some(source.clone()),
                )?,
                Err(e) => return Err(e),
            };

            if is_referenced {
                property_value.set_reference();
            }

            let nested_properties = match property_value {
                ftd::PropertyValue::Reference { ref kind, .. }
                    if matches!(kind.inner(), ftd::ftd2021::p2::Kind::UI { .. }) =>
                {
                    let headers = if source.eq(&ftd::TextSource::Default) {
                        let mut headers = Default::default();
                        if let ftd::ftd2021::p2::Kind::UI {
                            default: Some((_, h)),
                        } = kind.inner()
                        {
                            headers = h.clone();
                        }
                        headers
                    } else {
                        let mut headers = vec![];
                        if let Some(idx) = idx {
                            let p1 = &p1.0;
                            for i in idx + 1..p1.len() {
                                let p1 = p1.get(i).unwrap();
                                if let Some(k) = p1.1.strip_prefix('>') {
                                    headers.push((p1.0, k.trim().to_string(), p1.2.to_string()));
                                } else {
                                    break;
                                }
                            }
                        }
                        ftd::ftd2021::p1::Header(headers)
                    };
                    ftd::ftd2021::p2::utils::structure_header_to_properties(
                        &value,
                        arguments,
                        doc,
                        line_number,
                        &headers,
                    )?
                }
                _ => Default::default(),
            };

            let (condition_value, default_value) =
                if let Some(ref attribute) = conditional_attribute {
                    let condition = ftd::ftd2021::p2::Boolean::from_expression(
                        attribute,
                        doc,
                        arguments,
                        (None, None),
                        line_number,
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
                property.nested_properties = nested_properties;
            } else {
                let value = Property {
                    default: default_value,
                    conditions: condition_value,
                    nested_properties,
                };
                properties.insert(name.to_string(), value);
            }
        }
    }

    // Checking if the user has entered conflicting values for ftd.input
    if root.eq("ftd#input") {
        check_input_conflicting_values(&properties, doc, line_number)?;
    }

    Ok(properties)
}

/// asserts caption-body checks on components.
///
/// It includes
///
/// # Caption/Body/Header_value conflicts
/// - This happens if any argument accepts data from more than one way
///   and the user doesn't pass this data in exactly one way
///
/// # Missing data checks
/// - This happens if any (required) argument doesn't get the data from any way it takes
///
/// # Unknown data checks
/// - This happens when there is no argument to accept the data passed from caption/body
///
fn assert_caption_body_checks(
    root: &str,
    p1: &ftd::ftd2021::p1::Header,
    doc: &ftd::ftd2021::p2::TDoc,
    caption: &Option<String>,
    body: &Option<(usize, String)>,
    line_number: usize,
) -> ftd::ftd2021::p1::Result<()> {
    // No checks on ftd#ui
    if is_it_ui(root) {
        return Ok(());
    }

    let mut has_caption = caption.is_some();
    let mut has_body = body.is_some();

    let mut properties = None;
    let mut header_list: Option<&ftd::ftd2021::p1::Header> = Some(p1);

    let mut thing = doc.get_thing(line_number, root)?;
    loop {
        // Either the component is kernel or variable/derived component
        if let ftd::ftd2021::p2::Thing::Component(c) = thing {
            let local_arguments = &c.arguments;

            check_caption_body_conflicts(
                &c.full_name,
                local_arguments,
                properties,
                header_list,
                doc,
                has_caption,
                has_body,
                line_number,
            )?;

            // stop checking once you hit the top-most kernel component or ftd#ui component
            if c.kernel || is_it_ui(&c.root) {
                break;
            }

            // get the parent component and do the same checks
            thing = doc.get_thing(line_number, &c.root)?;
            properties = Some(c.properties.clone());

            // These things are only available to the lowest level component
            has_caption = false;
            has_body = false;
            header_list = None;
        }
    }

    return Ok(());

    /// checks if the root == ftd#ui
    fn is_it_ui(root: &str) -> bool {
        root.eq("ftd#ui")
    }

    /// checks for body and caption conflicts using the given header list,
    /// arguments and properties map of the component
    #[allow(clippy::too_many_arguments)]
    fn check_caption_body_conflicts(
        full_name: &str,
        arguments: &std::collections::BTreeMap<String, ftd::ftd2021::p2::Kind>,
        properties: Option<std::collections::BTreeMap<String, Property>>,
        p1: Option<&ftd::ftd2021::p1::Header>,
        doc: &ftd::ftd2021::p2::TDoc,
        has_caption: bool,
        has_body: bool,
        line_number: usize,
    ) -> ftd::ftd2021::p1::Result<()> {
        /// returns a hashset`<key>` of header keys which have non-empty values
        fn get_header_set_with_values(
            p1: Option<&ftd::ftd2021::p1::Header>,
        ) -> std::collections::HashSet<String> {
            let mut header_set = std::collections::HashSet::new();

            // For Some(header = p1) we need to make a set of headers with values
            if let Some(header) = p1 {
                for (_ln, k, v) in header.0.iter() {
                    if !v.is_empty() {
                        header_set.insert(k.to_string());
                    }
                }
            }

            header_set
        }

        /// checks if the hashset of headers has this particular argument or not
        fn has_header_value(
            argument: &str,
            header_set: Option<&std::collections::HashSet<String>>,
        ) -> bool {
            if let Some(s) = header_set {
                s.contains(argument)
            } else {
                false
            }
        }

        /// checks if the argument has been passed down as property
        fn has_property_value(
            argument: &str,
            properties: &Option<std::collections::BTreeMap<String, Property>>,
        ) -> bool {
            if let Some(p) = properties {
                p.contains_key(argument)
            } else {
                false
            }
        }

        let mut caption_pass = false;
        let mut body_pass = false;
        let header_set = get_header_set_with_values(p1);

        for (arg, kind) in arguments.iter() {
            // in case the kind is optional
            let inner_kind = kind.inner();

            let has_value = has_header_value(arg, Some(&header_set));
            let has_property = has_property_value(arg, &properties);

            match inner_kind {
                ftd::ftd2021::p2::Kind::String {
                    caption,
                    body,
                    default,
                    ..
                } => {
                    let has_default = default.is_some();
                    match (caption, body) {
                        (true, true) => {
                            // accepts data from either body or caption or header_value
                            // if passed by 2 or more ways then throw error
                            if ((has_property || has_body || has_caption && has_value)
                                && (has_caption || has_value))
                                || (has_property && has_body)
                            {
                                return Err(ftd::ftd2021::p1::Error::ForbiddenUsage {
                                    message: format!(
                                        "pass either body or caption or header_value, ambiguity in \'{}\'",
                                        arg
                                    ),
                                    doc_id: doc.name.to_string(),
                                    line_number,
                                });
                            }

                            // check if data is available in exactly one way
                            // also avoid throwing error when argument is optional kind or has default value
                            // and no data is passed in any way
                            if !(has_caption
                                || has_body
                                || has_value
                                || has_property
                                || has_default
                                || kind.is_optional())
                            {
                                return Err(ftd::ftd2021::p1::Error::MissingData {
                                    message: format!(
                                        "body or caption or header_value, none of them are passed for \'{}\'",
                                        arg
                                    ),
                                    doc_id: doc.name.to_string(),
                                    line_number,
                                });
                            }

                            // check if caption is utilized if passed
                            if has_caption {
                                caption_pass = true;
                            }

                            // check if body is utilized if passed
                            if has_body {
                                body_pass = true;
                            }
                        }
                        (true, false) => {
                            // check if the component has caption or header_value (not both)
                            // if data conflicts from any 2 ways
                            if ((has_property || has_value) && has_caption)
                                || (has_value && has_property)
                            {
                                return Err(ftd::ftd2021::p1::Error::ForbiddenUsage {
                                    message: format!(
                                        "pass either caption or header_value for header \'{}\'",
                                        arg
                                    ),
                                    doc_id: doc.name.to_string(),
                                    line_number,
                                });
                            }

                            // check if data is available from either caption/header_value
                            // also avoid throwing error when argument is optional kind or has default value
                            // and no data is passed in any way
                            if !(has_caption
                                || has_value
                                || has_property
                                || has_default
                                || kind.is_optional())
                            {
                                return Err(ftd::ftd2021::p1::Error::MissingData {
                                    message: format!(
                                        "caption or header_value, none of them are passed for \'{}\'",
                                        arg
                                    ),
                                    doc_id: doc.name.to_string(),
                                    line_number,
                                });
                            }

                            // check if caption is utilized if passed
                            if has_caption {
                                caption_pass = true;
                            }
                        }
                        (false, true) => {
                            // check if the component has body or not
                            // if body is not passed throw error
                            if ((has_property || has_value) && has_body)
                                || (has_property && has_value)
                            {
                                return Err(ftd::ftd2021::p1::Error::ForbiddenUsage {
                                    message: format!(
                                        "pass either body or header_value for header \'{}\'",
                                        arg
                                    ),
                                    doc_id: doc.name.to_string(),
                                    line_number,
                                });
                            }

                            // check if data is available from either body/header_value
                            // also avoid throwing error when argument is optional kind or has default value
                            // and no data is passed in any way
                            if !(has_body
                                || has_value
                                || has_property
                                || has_default
                                || kind.is_optional())
                            {
                                return Err(ftd::ftd2021::p1::Error::MissingData {
                                    message: format!(
                                        "body or header_value, none of them are passed for \'{}\'",
                                        arg
                                    ),
                                    doc_id: doc.name.to_string(),
                                    line_number,
                                });
                            }

                            // check if body is utilized if passed
                            if has_body {
                                body_pass = true;
                            }
                        }
                        (false, false) => continue,
                    }
                }
                ftd::ftd2021::p2::Kind::Integer { default, .. }
                | ftd::ftd2021::p2::Kind::Decimal { default, .. }
                | ftd::ftd2021::p2::Kind::Boolean { default, .. }
                    if arg.eq("value")
                        && matches!(full_name, "ftd#integer" | "ftd#boolean" | "ftd#decimal") =>
                {
                    // checks on ftd.integer, ftd.decimal, ftd.boolean
                    // these components take data from either caption or
                    // header_value when invoked or when data is passed to it
                    // from any variable component
                    let has_default = default.is_some();

                    // check if data conflicts from any 2 two ways
                    if ((has_property || has_value) && has_caption) || (has_value && has_property) {
                        return Err(ftd::ftd2021::p1::Error::ForbiddenUsage {
                            message: format!(
                                "pass either caption or header_value for header \'{}\'",
                                arg
                            ),
                            doc_id: doc.name.to_string(),
                            line_number,
                        });
                    }

                    // check if data is available in exactly one way
                    if !(has_caption
                        || has_value
                        || has_property
                        || has_default
                        || kind.is_optional())
                    {
                        return Err(ftd::ftd2021::p1::Error::MissingData {
                            message: format!(
                                "caption or header_value, none of them are passed for \'{}\'",
                                arg
                            ),
                            doc_id: doc.name.to_string(),
                            line_number,
                        });
                    }

                    // check if caption is utilized if passed
                    if has_caption {
                        caption_pass = true;
                    }
                }
                _ => continue,
            }
        }

        // check if both caption and body is utilized
        if !(caption_pass && body_pass) {
            // if caption is passed and caption not utilized then throw error
            if !caption_pass && has_caption {
                return Err(ftd::ftd2021::p1::Error::UnknownData {
                    message: "caption passed with no header accepting it !!".to_string(),
                    doc_id: doc.name.to_string(),
                    line_number,
                });
            }

            // if body is passed and body not utilized then throw error
            if !body_pass && has_body {
                return Err(ftd::ftd2021::p1::Error::UnknownData {
                    message: "body passed with no header accepting it !!".to_string(),
                    doc_id: doc.name.to_string(),
                    line_number,
                });
            }
        }

        Ok(())
    }
}

pub(crate) fn universal_arguments() -> ftd::Map<ftd::ftd2021::p2::Kind> {
    let mut universal_arguments: ftd::Map<ftd::ftd2021::p2::Kind> = Default::default();
    universal_arguments.insert(
        "heading-number".to_string(),
        ftd::ftd2021::p2::Kind::list(ftd::ftd2021::p2::Kind::string()).into_optional(),
    );
    universal_arguments.insert(
        "id".to_string(),
        ftd::ftd2021::p2::Kind::string().into_optional(),
    );
    universal_arguments.insert(
        "top".to_string(),
        ftd::ftd2021::p2::Kind::integer().into_optional(),
    );
    universal_arguments.insert(
        "bottom".to_string(),
        ftd::ftd2021::p2::Kind::integer().into_optional(),
    );
    universal_arguments.insert(
        "left".to_string(),
        ftd::ftd2021::p2::Kind::integer().into_optional(),
    );
    universal_arguments.insert(
        "move-up".to_string(),
        ftd::ftd2021::p2::Kind::integer().into_optional(),
    );
    universal_arguments.insert(
        "move-down".to_string(),
        ftd::ftd2021::p2::Kind::integer().into_optional(),
    );
    universal_arguments.insert(
        "move-left".to_string(),
        ftd::ftd2021::p2::Kind::integer().into_optional(),
    );
    universal_arguments.insert(
        "move-right".to_string(),
        ftd::ftd2021::p2::Kind::integer().into_optional(),
    );
    universal_arguments.insert(
        "right".to_string(),
        ftd::ftd2021::p2::Kind::integer().into_optional(),
    );

    universal_arguments.insert(
        "align".to_string(),
        ftd::ftd2021::p2::Kind::string().into_optional(),
    );
    universal_arguments.insert(
        "scale".to_string(),
        ftd::ftd2021::p2::Kind::decimal().into_optional(),
    );
    universal_arguments.insert(
        "rotate".to_string(),
        ftd::ftd2021::p2::Kind::integer().into_optional(),
    );
    universal_arguments.insert(
        "scale-x".to_string(),
        ftd::ftd2021::p2::Kind::decimal().into_optional(),
    );
    universal_arguments.insert(
        "scale-y".to_string(),
        ftd::ftd2021::p2::Kind::decimal().into_optional(),
    );
    universal_arguments.insert(
        "slot".to_string(),
        ftd::ftd2021::p2::Kind::string().into_optional(),
    );

    universal_arguments
}

fn root_properties_from_inherits(
    line_number: usize,
    arguments: &ftd::Map<ftd::ftd2021::p2::Kind>,
    inherits: Vec<String>,
    doc: &ftd::ftd2021::p2::TDoc,
) -> ftd::ftd2021::p1::Result<ftd::Map<Property>> {
    let mut root_properties: ftd::Map<Property> = Default::default();
    for inherit in inherits {
        let pv = ftd::PropertyValue::resolve_value(
            line_number,
            &format!("${}", inherit),
            None,
            doc,
            arguments,
            None,
        )?;
        root_properties.insert(
            inherit,
            ftd::ftd2021::component::Property {
                default: Some(pv),
                conditions: vec![],
                ..Default::default()
            },
        );
    }
    Ok(root_properties)
}

fn read_arguments(
    p1: &ftd::ftd2021::p1::Header,
    root: &str,
    root_arguments: &ftd::Map<ftd::ftd2021::p2::Kind>,
    arguments: &ftd::Map<ftd::ftd2021::p2::Kind>,
    doc: &ftd::ftd2021::p2::TDoc,
) -> ftd::ftd2021::p1::Result<(ftd::Map<ftd::ftd2021::p2::Kind>, Vec<String>)> {
    let mut args: ftd::Map<ftd::ftd2021::p2::Kind> = Default::default();
    let mut inherits: Vec<String> = Default::default();

    // contains parent arguments and current arguments
    let mut all_args = arguments.clone();

    // Set of all universal arguments available to all components
    let universal_arguments_set: std::collections::HashSet<String> =
        universal_arguments().keys().cloned().collect();

    // Set of root arguments which are invoked once
    let mut root_args_set: std::collections::HashSet<String> = std::collections::HashSet::new();
    for (idx, (i, k, v)) in p1.0.iter().enumerate() {
        if (k.starts_with('$') && k.ends_with('$')) || k.starts_with('>') {
            // event and loop matches
            continue;
        }

        let var_data = match ftd::ftd2021::variable::VariableData::get_name_kind(
            k,
            doc,
            i.to_owned(),
            vec![].as_slice(),
        ) {
            Ok(v) => v,
            _ => {
                // Duplicate header usage check
                if root_args_set.contains(k) {
                    if let Some(kind) = root_arguments.get(k) {
                        if kind.inner().is_list() {
                            continue;
                        }
                        return Err(ftd::ftd2021::p1::Error::ForbiddenUsage {
                            message: format!("repeated usage of \'{}\' not allowed !!", k),
                            doc_id: doc.name.to_string(),
                            line_number: *i,
                        });
                    }
                } else {
                    root_args_set.insert(k.to_string());
                }

                continue;
            }
        };

        let option_v = if v.is_empty() {
            None
        } else {
            Some(v.to_string())
        };

        let mut kind = if var_data.kind.eq("inherit") {
            match root_arguments.get(&var_data.name) {
                Some(kind) => {
                    inherits.push(var_data.name.to_string());
                    let default = {
                        // resolve the default value
                        let mut default = option_v;
                        if let Some(ref v) = default {
                            default =
                                Some(doc.resolve_reference_name(i.to_owned(), v, &all_args)?);
                        }
                        default
                    };
                    kind.clone().set_default(default)
                }
                None => {
                    return ftd::ftd2021::p2::utils::e2(
                        format!("'{}' is not an argument of {}", var_data.name, root),
                        doc.name,
                        i.to_owned(),
                    )
                }
            }
        } else {
            ftd::ftd2021::p2::Kind::for_variable(i.to_owned(), k, option_v, doc, None, &all_args)?
        };
        if let ftd::ftd2021::p2::Kind::UI {
            default: Some((ui_id, h)),
        } = &mut kind.mut_inner()
        {
            let headers = {
                let mut headers = vec![];
                let p1 = &p1.0;
                for i in idx + 1..p1.len() {
                    let p1 = p1.get(i).unwrap();
                    if let Some(k) = p1.1.strip_prefix('>') {
                        headers.push((p1.0, k.trim().to_string(), p1.2.to_string()));
                    } else {
                        break;
                    }
                }
                ftd::ftd2021::p1::Header(headers)
            };
            *h = headers;
            *ui_id = doc.resolve_name(*i, ui_id.as_str())?;
        }

        // Duplicate header definition check
        if args.contains_key(var_data.name.as_str()) {
            return Err(ftd::ftd2021::p1::Error::ForbiddenUsage {
                message: format!(
                    "\'{}\' is already used as header name/identifier !!",
                    &var_data.name
                ),
                doc_id: doc.name.to_string(),
                line_number: *i,
            });
        }

        // checking if any universal argument is declared by the user (forbidden)
        if universal_arguments_set.contains(&var_data.name) {
            return Err(ftd::ftd2021::p1::Error::ForbiddenUsage {
                message: format!(
                    "redundant declaration of universal argument \'{}\' !!",
                    &var_data.name
                ),
                doc_id: doc.name.to_string(),
                line_number: *i,
            });
        }

        args.insert(var_data.name.to_string(), kind.clone());
        all_args.insert(var_data.name.to_string(), kind);
    }

    Ok((args, inherits))
}
