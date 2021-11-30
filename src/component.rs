#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Component {
    pub root: String,
    pub full_name: String,
    pub arguments: std::collections::BTreeMap<String, ftd::p2::Kind>,
    pub locals: std::collections::BTreeMap<String, ftd::p2::Kind>,
    pub properties: std::collections::BTreeMap<String, Property>,
    pub instructions: Vec<Instruction>,
    pub events: Vec<ftd::p2::Event>,
    pub condition: Option<ftd::p2::Boolean>,
    pub kernel: bool,
    pub invocations: Vec<std::collections::BTreeMap<String, ftd::Value>>,
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
                for mut child in children {
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
                value: ftd::variable::Value::String { text, .. },
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
    pub arguments: std::collections::BTreeMap<String, ftd::p2::Kind>,
    pub events: Vec<ftd::p2::Event>,
    pub is_recursive: bool,
    pub line_number: usize,
    pub reference: Option<(String, ftd::p2::Kind)>,
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Property {
    pub default: Option<ftd::PropertyValue>,
    pub conditions: Vec<(ftd::p2::Boolean, ftd::PropertyValue)>,
    pub nested_properties: std::collections::BTreeMap<String, ftd::component::Property>,
}

#[derive(Debug, Clone)]
pub struct ElementWithContainer {
    pub element: ftd::Element,
    pub children: Vec<ftd::Element>,
    pub child_container: Option<std::collections::BTreeMap<String, Vec<Vec<usize>>>>,
}

impl Property {
    fn eval(
        &self,
        line_number: usize,
        name: &str,
        arguments: &std::collections::BTreeMap<String, ftd::Value>,
        doc: &ftd::p2::TDoc,
    ) -> ftd::p1::Result<&ftd::PropertyValue> {
        let mut property_value = ftd::e2(
            format!("condition is not complete, name: {}", name),
            doc.name,
            line_number,
        );
        if let Some(property) = &self.default {
            property_value = Ok(property);
        }
        for (boolean, property) in &self.conditions {
            if boolean.eval(line_number, arguments, doc)? {
                property_value = Ok(property);
            }
        }
        property_value
    }
}

impl ChildComponent {
    pub fn super_call(
        &self,
        children: &[Self],
        doc: &ftd::p2::TDoc,
        arguments: &std::collections::BTreeMap<String, ftd::Value>,
        invocations: &mut std::collections::BTreeMap<
            String,
            Vec<std::collections::BTreeMap<String, ftd::Value>>,
        >,
        all_locals: &mut ftd::Map,
        local_container: &[usize],
    ) -> ftd::p1::Result<ElementWithContainer> {
        let id = ftd::p2::utils::string_optional(
            "id",
            &resolve_properties(self.line_number, &self.properties, arguments, doc, None)?,
            doc.name,
            self.line_number,
        )?;

        let string_container: String = local_container
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<String>>()
            .join(",");

        for k in self.arguments.keys() {
            all_locals.insert(k.to_string(), string_container.to_string());
        }

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
            id.clone(),
        )?;
        element.set_container_id(id.clone());
        element.set_element_id(id);

        if let Some(common) = element.get_mut_common() {
            for (k, v) in &self.arguments {
                if let Ok(v) = v.to_value(self.line_number, doc.name) {
                    if let Some(v) = v.to_string() {
                        common
                            .locals
                            .insert(format!("{}@{}", k, string_container), v.to_string());
                    }
                }
            }
        }

        let mut container_children = vec![];
        match (&mut element, children.is_empty()) {
            (ftd::Element::Column(_), _)
            | (ftd::Element::Row(_), _)
            | (ftd::Element::Scene(_), _) => {
                let instructions = children
                    .iter()
                    .map(|child| {
                        if child.is_recursive {
                            ftd::Instruction::RecursiveChildComponent {
                                child: child.clone(),
                            }
                        } else {
                            ftd::Instruction::ChildComponent {
                                child: child.clone(),
                            }
                        }
                    })
                    .collect::<Vec<ftd::Instruction>>();
                let elements = ftd::execute_doc::ExecuteDoc {
                    name: doc.name,
                    aliases: doc.aliases,
                    bag: doc.bag,
                    instructions: &instructions,
                    arguments,
                    invocations,
                    root_name: None,
                }
                .execute(local_container, all_locals, None)?
                .children;
                container_children.extend(elements);
            }
            (ftd::Element::Null, false) => {
                let mut root = doc
                    .get_component(self.line_number, self.root.as_str())
                    .unwrap();
                while root.root != "ftd.kernel" {
                    root = doc
                        .get_component(self.line_number, self.root.as_str())
                        .unwrap();
                }
                match root.full_name.as_str() {
                    "ftd#row" | "ftd#column" | "ftd#scene" => {}
                    t => {
                        return ftd::e2(
                            format!("{} cant have children", t),
                            doc.name,
                            self.line_number,
                        )
                    }
                }
            }
            (t, false) => {
                return ftd::e2(
                    format!("cant have children: {:?}", t),
                    doc.name,
                    self.line_number,
                );
            }
            (_, true) => {}
        }
        Ok(ElementWithContainer {
            element,
            children: container_children,
            child_container,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn recursive_call(
        &self,
        doc: &ftd::p2::TDoc,
        arguments: &std::collections::BTreeMap<String, ftd::Value>,
        invocations: &mut std::collections::BTreeMap<
            String,
            Vec<std::collections::BTreeMap<String, ftd::Value>>,
        >,
        is_child: bool,
        root_name: Option<&str>,
        all_locals: &mut ftd::Map,
        local_container: &[usize],
    ) -> ftd::p1::Result<Vec<ElementWithContainer>> {
        let root = {
            // NOTE: doing unwrap to force bug report if we following fails, this function
            // must have validated everything, and must not fail at run time
            doc.get_component_with_root(self.line_number, self.root.as_str(), root_name)
                .unwrap()
        };
        let loop_property = resolve_recursive_property(
            self.line_number,
            &self.properties,
            arguments,
            doc,
            root_name,
        )?;
        let mut elements = vec![];

        if let ftd::Value::List { data, .. } = loop_property {
            for (i, d) in data.iter().enumerate() {
                let mut new_arguments: std::collections::BTreeMap<String, ftd::Value> =
                    arguments.clone();
                new_arguments.insert("$loop$".to_string(), d.clone());
                let new_properties = resolve_properties_with_ref(
                    self.line_number,
                    &self.properties,
                    &new_arguments,
                    doc,
                    root_name,
                )?;
                let mut temp_locals: ftd::Map = Default::default();
                let conditional_attribute = get_conditional_attributes(
                    self.line_number,
                    &self.properties,
                    &new_arguments,
                    doc,
                    &mut temp_locals,
                )?;
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
                        if b.is_constant() && !b.eval(self.line_number, &new_arguments, doc)? {
                            visible = false;
                            if let Ok(true) = b.set_null(self.line_number, doc.name) {
                                elements.push(ElementWithContainer {
                                    element: ftd::Element::Null,
                                    children: vec![],
                                    child_container: None,
                                });
                                continue;
                            }
                        }
                    }
                    visible
                };

                let mut all_old_locals = all_locals.clone();

                let mut element = root.call(
                    &new_properties,
                    doc,
                    invocations,
                    &None,
                    is_child,
                    doc.get_root(self.root.as_str(), self.line_number)?
                        .or(root_name),
                    &self.events,
                    &mut all_old_locals,
                    local_container.as_slice(),
                    None,
                )?;

                if let Some(condition) = &self.condition {
                    element.element.set_non_visibility(!condition.eval(
                        self.line_number,
                        &new_arguments,
                        doc,
                    )?);
                    element.element.set_condition(
                        condition
                            .to_condition(
                                self.line_number,
                                all_locals,
                                &Default::default(),
                                doc.name,
                            )
                            .ok(),
                    );
                }
                if !is_visible {
                    element.element.set_non_visibility(!is_visible);
                }
                if let Some(common) = element.element.get_mut_common() {
                    common.conditional_attribute.extend(conditional_attribute);
                }
                elements.push(element);
            }
        }
        Ok(elements)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn call(
        &self,
        doc: &ftd::p2::TDoc,
        arguments: &std::collections::BTreeMap<String, ftd::Value>,
        invocations: &mut std::collections::BTreeMap<
            String,
            Vec<std::collections::BTreeMap<String, ftd::Value>>,
        >,
        is_child: bool,
        root_name: Option<&str>,
        all_locals: &mut ftd::Map,
        local_container: &[usize],
        id: Option<String>,
    ) -> ftd::p1::Result<ElementWithContainer> {
        if let Some(ref b) = self.condition {
            if b.is_constant() && !b.eval(self.line_number, arguments, doc)? {
                if let Ok(true) = b.set_null(self.line_number, doc.name) {
                    return Ok(ElementWithContainer {
                        element: ftd::Element::Null,
                        children: vec![],
                        child_container: None,
                    });
                }
            }
        }

        let root = {
            // NOTE: doing unwrap to force bug report if we following fails, this function
            // must have validated everything, and must not fail at run time
            doc.get_component(self.line_number, self.root.as_str())
                .unwrap()
        };

        let root_properties = {
            let mut root_properties = resolve_properties_with_ref(
                self.line_number,
                &self.properties,
                arguments,
                doc,
                root_name,
            )?;
            //pass argument of component to its children
            for (k, v) in arguments {
                root_properties.insert(format!("${}", k), (v.to_owned(), None));
            }
            for (k, v) in &self.arguments {
                if let Ok(v) = v.to_value(self.line_number, doc.name) {
                    root_properties.insert(format!("${}", k), (v.to_owned(), None));
                }
            }
            root_properties
        };

        let conditional_attribute = get_conditional_attributes(
            self.line_number,
            &self.properties,
            arguments,
            doc,
            all_locals,
        )?;

        let mut element = root.call(
            &root_properties,
            doc,
            invocations,
            &self.condition,
            is_child,
            doc.get_root(self.root.as_str(), self.line_number)?,
            &self.events,
            all_locals,
            local_container,
            id,
        )?;

        if let Some(common) = element.element.get_mut_common() {
            common.conditional_attribute.extend(conditional_attribute);
        }

        Ok(element)
    }

    pub fn from_p1(
        line_number: usize,
        name: &str,
        p1: &ftd::p1::Header,
        caption: &Option<String>,
        body: &Option<(usize, String)>,
        doc: &ftd::p2::TDoc,
        arguments: &std::collections::BTreeMap<String, ftd::p2::Kind>,
    ) -> ftd::p1::Result<Self> {
        let mut reference = None;
        let root = if let Some(ftd::p2::Kind::UI { default }) = arguments.get(name) {
            reference = Some((
                name.to_string(),
                ftd::p2::Kind::UI {
                    default: default.clone(),
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
                line_number,
            }
        } else {
            doc.get_component(line_number, name)?
        };
        let mut root_arguments = root.arguments;
        assert_no_extra_properties(
            line_number,
            p1,
            root.full_name.as_str(),
            &root_arguments,
            name,
            doc,
        )?;
        let (local_arguments, inherits) = read_arguments(p1, name, &root_arguments, doc)?;

        let mut all_arguments = local_arguments.clone();
        all_arguments.extend(arguments.clone());

        let root_property =
            get_root_property(line_number, name, caption, doc, &all_arguments, inherits)?;

        return Ok(Self {
            line_number,
            properties: read_properties(
                line_number,
                p1,
                caption,
                body,
                name,
                root.full_name.as_str(),
                &mut root_arguments,
                &all_arguments,
                doc,
                &root_property,
                reference.is_some(),
            )?,
            condition: match p1.str_optional(doc.name, line_number, "if")? {
                Some(expr) => Some(ftd::p2::Boolean::from_expression(
                    expr,
                    doc,
                    &all_arguments,
                    (None, None),
                    line_number,
                )?),
                None => None,
            },
            root: root.full_name.clone(),
            events: p1.get_events(line_number, doc, &all_arguments)?,
            is_recursive: false,
            arguments: local_arguments,
            reference,
        });

        fn get_root_property(
            line_number: usize,
            name: &str,
            caption: &Option<String>,
            doc: &ftd::p2::TDoc,
            arguments: &std::collections::BTreeMap<String, ftd::p2::Kind>,
            inherits: Vec<String>,
        ) -> ftd::p1::Result<std::collections::BTreeMap<String, Property>> {
            let mut properties: std::collections::BTreeMap<String, Property> =
                root_properties_from_inherits(line_number, arguments, inherits, doc)?;
            if let Some(caption) = caption {
                if let Ok(name) = doc.resolve_name(line_number, name) {
                    let kind = match name.as_str() {
                        "ftd#integer" => ftd::p2::Kind::integer(),
                        "ftd#boolean" => ftd::p2::Kind::boolean(),
                        "ftd#decimal" => ftd::p2::Kind::decimal(),
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

fn resolve_recursive_property(
    line_number: usize,
    self_properties: &std::collections::BTreeMap<String, Property>,
    arguments: &std::collections::BTreeMap<String, ftd::Value>,
    doc: &ftd::p2::TDoc,
    root_name: Option<&str>,
) -> ftd::p1::Result<ftd::Value> {
    if let Some(value) = self_properties.get("$loop$") {
        if let Ok(property_value) = value.eval(line_number, "$loop$", arguments, doc) {
            return property_value.resolve_with_root(line_number, arguments, doc, root_name);
        }
    }
    ftd::e2(
        format!("$loop$ not found in properties {:?}", self_properties),
        doc.name,
        line_number,
    )
}

pub fn resolve_properties(
    line_number: usize,
    self_properties: &std::collections::BTreeMap<String, Property>,
    arguments: &std::collections::BTreeMap<String, ftd::Value>,
    doc: &ftd::p2::TDoc,
    root_name: Option<&str>,
) -> ftd::p1::Result<std::collections::BTreeMap<String, ftd::Value>> {
    let mut properties: std::collections::BTreeMap<String, ftd::Value> = Default::default();
    for (name, value) in self_properties.iter() {
        if name == "$loop$" {
            continue;
        }
        if let Ok(property_value) = value.eval(line_number, name, arguments, doc) {
            properties.insert(
                name.to_string(),
                property_value.resolve_with_root(line_number, arguments, doc, root_name)?,
            );
        }
    }
    Ok(properties)
}

fn get_conditional_attributes(
    line_number: usize,
    properties: &std::collections::BTreeMap<String, Property>,
    arguments: &std::collections::BTreeMap<String, ftd::Value>,
    doc: &ftd::p2::TDoc,
    all_locals: &mut ftd::Map,
) -> ftd::p1::Result<std::collections::BTreeMap<String, ftd::ConditionalAttribute>> {
    let mut conditional_attribute: std::collections::BTreeMap<String, ftd::ConditionalAttribute> =
        Default::default();

    let mut dictionary: std::collections::BTreeMap<String, Vec<String>> = Default::default();
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
                        let cond = condition.to_condition(
                            line_number,
                            all_locals,
                            &Default::default(),
                            doc.name,
                        )?;
                        let value = pv.resolve(line_number, arguments, doc)?;
                        let string = get_string_value(&name, value, doc.name, line_number)?;
                        conditions_with_value.push((cond, string));
                    }
                }
                let default = {
                    let mut default = None;
                    if let Some(pv) = &value.default {
                        let value = pv.resolve(line_number, arguments, doc)?;
                        let string = get_string_value(&name, value, doc.name, line_number)?;
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
        doc_id: &str,
        line_number: usize,
    ) -> ftd::p1::Result<ftd::ConditionalValue> {
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

        let style_string = vec!["cursor", "position", "align", "background-image"];

        let style_overflow = vec!["overflow-x", "overflow-y"];

        let style_boolean = vec!["background-repeat"];

        Ok(if style_integer.contains(&name) {
            match value {
                ftd::Value::Integer { value: v } => ftd::ConditionalValue {
                    value: format!("{}px", v),
                    important: false,
                },
                v => return ftd::e2(format!("expected int, found: {:?}", v), doc_id, line_number),
            }
        } else if style_integer_important.contains(&name) {
            match value {
                ftd::Value::Integer { value: v } => ftd::ConditionalValue {
                    value: format!("{}px", v),
                    important: true,
                },
                v => return ftd::e2(format!("expected int, found: {:?}", v), doc_id, line_number),
            }
        } else if style_length.contains(&name) {
            match value {
                ftd::Value::String { text: v, .. } => ftd::ConditionalValue {
                    value: ftd::length(&ftd::Length::from(Some(v), doc_id)?.unwrap(), name).1,
                    important: false,
                },
                v => {
                    return ftd::e2(
                        format!("expected string, found: {:?}", v),
                        doc_id,
                        line_number,
                    )
                }
            }
        } else if style_color.contains(&name) {
            match value {
                ftd::Value::String { text: v, .. } => ftd::ConditionalValue {
                    value: ftd::color(&ftd::p2::element::color_from(Some(v), doc_id)?.unwrap()),
                    important: false,
                },
                v => {
                    return ftd::e2(
                        format!("expected string, found: {:?}", v),
                        doc_id,
                        line_number,
                    )
                }
            }
        } else if style_overflow.contains(&name) {
            match value {
                ftd::Value::String { text: v, .. } => ftd::ConditionalValue {
                    value: ftd::overflow(&ftd::Overflow::from(Some(v), doc_id)?.unwrap(), name).1,
                    important: false,
                },
                v => {
                    return ftd::e2(
                        format!("expected string, found: {:?}", v),
                        doc_id,
                        line_number,
                    )
                }
            }
        } else if style_string.contains(&name) {
            match value {
                ftd::Value::String { text: v, .. } => ftd::ConditionalValue {
                    value: v,
                    important: false,
                },
                v => {
                    return ftd::e2(
                        format!("expected string, found: {:?}", v),
                        doc_id,
                        line_number,
                    )
                }
            }
        } else if style_boolean.contains(&name) {
            match value {
                ftd::Value::Boolean { value: v } => ftd::ConditionalValue {
                    value: v.to_string(),
                    important: false,
                },
                v => {
                    return ftd::e2(
                        format!("expected string, found: {:?}", v),
                        doc_id,
                        line_number,
                    )
                }
            }
        } else if name.eq("sticky") {
            match value {
                ftd::Value::Boolean { value: v } => ftd::ConditionalValue {
                    value: { if v { "sticky" } else { "inherit" }.to_string() },
                    important: false,
                },
                v => {
                    return ftd::e2(
                        format!("expected boolean, found: {:?}", v),
                        doc_id,
                        line_number,
                    )
                }
            }
        } else if name.eq("background-attachment") {
            match value {
                ftd::Value::Boolean { value: v } => ftd::ConditionalValue {
                    value: { if v { "fixed" } else { "inherit" }.to_string() },
                    important: false,
                },
                v => {
                    return ftd::e2(
                        format!("expected boolean, found: {:?}", v),
                        doc_id,
                        line_number,
                    )
                }
            }
        } else if name.eq("line-clamp") {
            match value {
                ftd::Value::Integer { value: v } => ftd::ConditionalValue {
                    value: v.to_string(),
                    important: false,
                },
                v => return ftd::e2(format!("expected int, found: {:?}", v), doc_id, line_number),
            }
        } else {
            return ftd::e2(
                format!("unknown style name: `{}` value:`{:?}`", name, value),
                doc_id,
                line_number,
            );
        })
    }
}

fn resolve_properties_with_ref(
    line_number: usize,
    self_properties: &std::collections::BTreeMap<String, Property>,
    arguments: &std::collections::BTreeMap<String, ftd::Value>,
    doc: &ftd::p2::TDoc,
    root_name: Option<&str>,
) -> ftd::p1::Result<std::collections::BTreeMap<String, (ftd::Value, Option<String>)>> {
    let mut properties: std::collections::BTreeMap<String, (ftd::Value, Option<String>)> =
        Default::default();
    for (name, value) in self_properties.iter() {
        if name == "$loop$" {
            continue;
        }
        if let Ok(property_value) = value.eval(line_number, name, arguments, doc) {
            let reference = match property_value {
                ftd::PropertyValue::Reference { name, .. } => Some(name.to_string()),
                ftd::PropertyValue::Variable { name, .. } => Some(format!("@{}", name)),
                _ => None,
            };
            properties.insert(
                name.to_string(),
                (
                    property_value.resolve_with_root(line_number, arguments, doc, root_name)?,
                    reference,
                ),
            );
        }
    }
    Ok(properties)
}

impl Component {
    #[allow(clippy::too_many_arguments)]
    fn call_sub_functions(
        &self,
        arguments: &std::collections::BTreeMap<String, ftd::Value>,
        doc: &ftd::p2::TDoc,
        invocations: &mut std::collections::BTreeMap<
            String,
            Vec<std::collections::BTreeMap<String, ftd::Value>>,
        >,
        root_name: Option<&str>,
        call_container: &[usize],
        all_locals: &mut ftd::Map,
        id: Option<String>,
    ) -> ftd::p1::Result<ElementWithContainer> {
        let new_instruction = {
            let mut instructions: Vec<Instruction> = self.instructions.clone();
            for instruction in instructions.iter_mut() {
                match instruction {
                    Instruction::ChildComponent { child } => reference_to_child_component(
                        child,
                        arguments,
                        self.line_number,
                        doc,
                        root_name,
                    )?,
                    Instruction::Component { parent, children } => {
                        reference_to_child_component(
                            parent,
                            arguments,
                            self.line_number,
                            doc,
                            root_name,
                        )?;
                        for child in children.iter_mut() {
                            reference_to_child_component(
                                child,
                                arguments,
                                self.line_number,
                                doc,
                                root_name,
                            )?;
                        }
                    }
                    Instruction::ChangeContainer { .. } => {}
                    Instruction::RecursiveChildComponent { child } => reference_to_child_component(
                        child,
                        arguments,
                        self.line_number,
                        doc,
                        root_name,
                    )?,
                }
            }
            instructions
        };

        return ftd::execute_doc::ExecuteDoc {
            name: doc.name,
            aliases: doc.aliases,
            bag: doc.bag,
            instructions: &new_instruction,
            arguments,
            invocations,
            root_name,
        }
        .execute(call_container, all_locals, id);

        fn reference_to_child_component(
            child: &mut ChildComponent,
            arguments: &std::collections::BTreeMap<String, crate::Value>,
            line_number: usize,
            doc: &ftd::p2::TDoc,
            root_name: Option<&str>,
        ) -> ftd::p1::Result<()> {
            if let Some(ref c) = child.reference {
                dbg!("reference_to_child_component", &c, arguments);
                if let Some(ftd::Value::UI { name, .. }) = arguments.get(&c.0) {
                    match doc.get_component_with_root(line_number, name, root_name) {
                        Ok(_) => {
                            *child = ChildComponent {
                                root: name.to_string(),
                                condition: None,
                                properties: Default::default(),
                                arguments: Default::default(),
                                events: vec![],
                                is_recursive: false,
                                line_number,
                                reference: None,
                            };
                        }
                        Err(e) => return ftd::e2(format!("{:?}", e), doc.name, line_number),
                    }
                }
            }
            Ok(())
        }
    }

    pub fn get_caption(&self) -> Option<String> {
        let mut new_caption_title = None;
        for (arg, arg_kind) in self.arguments.clone() {
            if let ftd::p2::Kind::String { caption, .. } = arg_kind {
                if caption {
                    new_caption_title = Some(arg);
                }
            }
        }
        new_caption_title
    }

    pub fn from_p1(p1: &ftd::p1::Section, doc: &ftd::p2::TDoc) -> ftd::p1::Result<Self> {
        let var_data = ftd::variable::VariableData::get_name_kind(
            &p1.name,
            doc,
            p1.line_number,
            vec![].as_slice(),
        )?;
        if var_data.is_variable() {
            return ftd::e2(
                format!("expected component, found: {}", p1.name),
                doc.name,
                p1.line_number,
            );
        }
        let name = var_data.name;
        let root = var_data.kind;
        let root_component = doc.get_component(p1.line_number, root.as_str())?;
        let mut root_arguments = root_component.arguments.clone();
        let (arguments, inherits) =
            read_arguments(&p1.header, root.as_str(), &root_arguments, doc)?;

        assert_no_extra_properties(
            p1.line_number,
            &p1.header,
            root.as_str(),
            &root_arguments,
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
                let s = ChildComponent::from_p1(
                    sub.line_number,
                    sub.name.as_str(),
                    &sub.header,
                    &sub.caption,
                    &sub.body_without_comment(),
                    doc,
                    &arguments,
                )?;
                Instruction::ChildComponent { child: s }
            });
        }

        let condition = match p1.header.str_optional(doc.name, p1.line_number, "if")? {
            Some(expr) => Some(ftd::p2::Boolean::from_expression(
                expr,
                doc,
                &arguments,
                (None, None),
                p1.line_number,
            )?),
            None => None,
        };

        let events = p1.header.get_events(p1.line_number, doc, &arguments)?;

        return Ok(Component {
            full_name: doc.resolve_name(p1.line_number, &name)?,
            properties: read_properties(
                p1.line_number,
                &p1.header,
                &p1.caption,
                &p1.body_without_comment(),
                name.as_str(),
                root.as_str(),
                &mut root_arguments,
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
        });
    }

    #[allow(clippy::too_many_arguments)]
    fn call(
        &self,
        arguments: &std::collections::BTreeMap<String, (ftd::Value, Option<String>)>,
        doc: &ftd::p2::TDoc,
        invocations: &mut std::collections::BTreeMap<
            String,
            Vec<std::collections::BTreeMap<String, ftd::Value>>,
        >,
        condition: &Option<ftd::p2::Boolean>,
        is_child: bool,
        root_name: Option<&str>,
        events: &[ftd::p2::Event],
        all_locals: &mut ftd::Map,
        local_container: &[usize],
        id: Option<String>,
    ) -> ftd::p1::Result<ElementWithContainer> {
        let property = {
            //remove arguments
            let mut properties_without_arguments: std::collections::BTreeMap<String, ftd::Value> =
                Default::default();
            for (k, v) in &ftd::p2::utils::properties(arguments) {
                if k.starts_with('$') {
                    continue;
                }
                properties_without_arguments.insert(k.to_string(), v.to_owned());
            }
            properties_without_arguments
        };
        invocations
            .entry(self.full_name.clone())
            .or_default()
            .push(property.to_owned());
        if self.root == "ftd.kernel" {
            let element = match self.full_name.as_str() {
                "ftd#text" => ftd::Element::Text(ftd::p2::element::text_from_properties(
                    arguments, doc, condition, is_child, events, all_locals, root_name,
                )?),
                "ftd#text-block" => {
                    ftd::Element::TextBlock(ftd::p2::element::text_block_from_properties(
                        arguments, doc, condition, is_child, events, all_locals, root_name,
                    )?)
                }
                "ftd#code" => ftd::Element::Code(ftd::p2::element::code_from_properties(
                    arguments, doc, condition, is_child, events, all_locals, root_name,
                )?),
                "ftd#image" => ftd::Element::Image(ftd::p2::element::image_from_properties(
                    arguments, doc, condition, is_child, events, all_locals, root_name,
                )?),
                "ftd#row" => ftd::Element::Row(ftd::p2::element::row_from_properties(
                    arguments, doc, condition, is_child, events, all_locals, root_name,
                )?),
                "ftd#column" => ftd::Element::Column(ftd::p2::element::column_from_properties(
                    arguments, doc, condition, is_child, events, all_locals, root_name,
                )?),
                "ftd#iframe" => ftd::Element::IFrame(ftd::p2::element::iframe_from_properties(
                    arguments, doc, condition, is_child, events, all_locals, root_name,
                )?),
                "ftd#integer" => ftd::Element::Integer(ftd::p2::element::integer_from_properties(
                    arguments, doc, condition, is_child, events, all_locals, root_name,
                )?),
                "ftd#decimal" => ftd::Element::Decimal(ftd::p2::element::decimal_from_properties(
                    arguments, doc, condition, is_child, events, all_locals, root_name,
                )?),
                "ftd#boolean" => ftd::Element::Boolean(ftd::p2::element::boolean_from_properties(
                    arguments, doc, condition, is_child, events, all_locals, root_name,
                )?),
                "ftd#input" => ftd::Element::Input(ftd::p2::element::input_from_properties(
                    arguments, doc, condition, is_child, events, all_locals, root_name,
                )?),
                "ftd#scene" => ftd::Element::Scene(ftd::p2::element::scene_from_properties(
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
                doc.get_component(self.line_number, self.root.as_str())
                    .unwrap()
            };
            let arguments = ftd::p2::utils::properties(arguments);
            let root_properties = {
                let mut properties = resolve_properties_with_ref(
                    self.line_number,
                    &self.properties,
                    &property,
                    doc,
                    root_name,
                )?;
                update_properties(&mut properties, &property, &self.arguments);
                properties
            };

            let string_container: String = local_container
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<String>>()
                .join(",");

            let mut all_new_locals: ftd::Map = self.get_locals_map(&string_container);

            let (get_condition, is_visible, is_null_element) = match condition {
                Some(c) => {
                    let arguments = {
                        //remove properties
                        let mut arguments_without_properties: std::collections::BTreeMap<
                            String,
                            ftd::Value,
                        > = Default::default();
                        for (k, v) in &arguments {
                            if let Some(k) = k.strip_prefix('$') {
                                arguments_without_properties.insert(k.to_string(), v.to_owned());
                            }
                        }
                        arguments_without_properties
                    };
                    let is_visible = c.eval(self.line_number, &arguments, doc)?;
                    if !c.is_arg_constant() {
                        (
                            Some(c.to_condition(
                                self.line_number,
                                all_locals,
                                &arguments,
                                doc.name,
                            )?),
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

            let events = ftd::p2::Event::get_events(
                self.line_number,
                events,
                all_locals,
                &arguments,
                doc,
                root_name,
            )?;

            // let mut all_locals = all_locals.clone();
            all_locals.extend(all_new_locals.clone());

            let mut element = if !is_null_element {
                root.call(
                    &root_properties,
                    doc,
                    invocations,
                    &self.condition,
                    is_child,
                    root_name,
                    &self.events,
                    /*&mut */ all_locals,
                    local_container,
                    None,
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

            let conditional_attribute = get_conditional_attributes(
                self.line_number,
                &self.properties,
                &property,
                doc,
                &mut all_new_locals,
            )?;

            let mut containers = None;
            match &mut element {
                ftd::Element::Text(_)
                | ftd::Element::TextBlock(_)
                | ftd::Element::Code(_)
                | ftd::Element::Image(_)
                | ftd::Element::IFrame(_)
                | ftd::Element::Input(_)
                | ftd::Element::Integer(_)
                | ftd::Element::Decimal(_)
                | ftd::Element::Boolean(_)
                | ftd::Element::Null => {}
                ftd::Element::Column(ftd::Column {
                    ref mut container, ..
                })
                | ftd::Element::Row(ftd::Row {
                    ref mut container, ..
                })
                | ftd::Element::Scene(ftd::Scene {
                    ref mut container, ..
                }) => {
                    let ElementWithContainer {
                        children,
                        child_container,
                        ..
                    } = self.call_sub_functions(
                        &property,
                        doc,
                        invocations,
                        root_name,
                        local_container,
                        &mut all_new_locals,
                        id,
                    )?;
                    containers = child_container;
                    container.children = children;
                }
            }

            element.set_locals(self.get_all_locals(
                &all_new_locals,
                &arguments,
                &string_container,
            )?);

            if let Some(common) = element.get_mut_common() {
                common.conditional_attribute.extend(conditional_attribute);
                common.events.extend(events);
                common
                    .events
                    .extend(ftd::p2::Event::mouse_event(&mut all_new_locals)?);
            }

            Ok(ElementWithContainer {
                element,
                children: vec![],
                child_container: containers,
            })
        }
    }

    fn get_all_locals(
        &self,
        all_locals: &ftd::Map,
        arguments: &std::collections::BTreeMap<String, ftd::Value>,
        string_container: &str,
    ) -> ftd::p1::Result<ftd::Map> {
        let mut locals: ftd::Map = Default::default();

        for k in all_locals.keys() {
            if k.eq("MOUSE-IN-TEMP") {
                continue;
            }
            if k.eq("MOUSE-IN") {
                locals.insert(
                    format!("MOUSE-IN@{}", string_container),
                    "false".to_string(),
                );
            }
            if let Some(arg) = arguments.get(k) {
                if let Some(value) = arg.to_string() {
                    locals.insert(format!("{}@{}", k, string_container), value.to_string());
                }
            } else if let Some(arg) = self.arguments.get(k) {
                match arg {
                    ftd::p2::Kind::String {
                        default: Some(d), ..
                    }
                    | ftd::p2::Kind::Integer { default: Some(d) }
                    | ftd::p2::Kind::Decimal { default: Some(d) }
                    | ftd::p2::Kind::Boolean { default: Some(d) } => {
                        locals.insert(format!("{}@{}", k, string_container), d.to_string());
                    }
                    _ => {}
                };
            }
        }
        Ok(locals)
    }

    fn get_locals_map(&self, string_container: &str) -> ftd::Map {
        let mut all_locals: ftd::Map = Default::default();

        for k in self.arguments.keys() {
            all_locals.insert(k.to_string(), string_container.to_string());
        }
        all_locals.insert("MOUSE-IN-TEMP".to_string(), string_container.to_string());
        all_locals
    }

    pub fn to_value(&self, kind: &ftd::p2::Kind) -> ftd::p1::Result<ftd::Value> {
        Ok(ftd::Value::UI {
            name: self.full_name.to_string(),
            kind: kind.to_owned(),
        })
    }
}

pub fn recursive_child_component(
    loop_data: &str,
    sub: &ftd::p1::SubSection,
    doc: &ftd::p2::TDoc,
    arguments: &std::collections::BTreeMap<String, ftd::p2::Kind>,
    name_with_component: Option<(String, ftd::Component)>,
) -> ftd::p1::Result<ftd::ChildComponent> {
    let mut loop_ref = "object".to_string();
    let mut loop_on_component = loop_data.to_string();

    if loop_data.contains("as") {
        let parts = ftd::p2::utils::split(loop_data.to_string(), "as")?;
        loop_on_component = parts.0;
        loop_ref = if let Some(loop_ref) = parts.1.strip_prefix('$') {
            loop_ref.to_string()
        } else {
            return ftd::e2(
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

    let recursive_kind = if let ftd::p2::Kind::List { kind } = recursive_property_value.kind() {
        kind.as_ref().to_owned()
    } else {
        return ftd::e2(
            format!(
                "expected list for loop, found: {:?}",
                recursive_property_value.kind(),
            ),
            doc.name,
            sub.line_number,
        );
    };

    let mut properties: std::collections::BTreeMap<String, Property> = Default::default();

    properties.insert(
        "$loop$".to_string(),
        ftd::component::Property {
            default: Some(recursive_property_value),
            conditions: vec![],
            ..Default::default()
        },
    );

    let mut new_header = ftd::p1::Header(vec![]);
    let (mut left_boolean, mut right_boolean) = (None, None);
    for (i, k, v) in &sub.header.0 {
        if k == "$loop$" || k.starts_with('/') {
            continue;
        }

        if k == "if" && contains_loop_ref(&loop_ref, v) {
            let v = v.replace(&format!("${}", loop_ref), "$loop$");
            let (_, left, right) =
                ftd::p2::Boolean::boolean_left_right(i.to_owned(), &v, doc.name)?;
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

    let (mut root_arguments, full_name, caption) =
        if name_with_component.is_some() && sub.name == name_with_component.clone().expect("").0 {
            let root_component = name_with_component.expect("").1;
            (
                root_component.arguments.clone(),
                root_component.full_name.to_string(),
                root_component.get_caption(),
            )
        } else {
            let root = if let Some(ftd::p2::Kind::UI { default }) = arguments.get(&sub.name) {
                reference = Some((
                    sub.name.to_string(),
                    ftd::p2::Kind::UI {
                        default: default.clone(),
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

    properties.extend(read_properties(
        sub.line_number,
        &new_header,
        &new_caption,
        &sub.body_without_comment(),
        &sub.name,
        full_name.as_str(),
        &mut root_arguments,
        arguments,
        doc,
        &properties,
        reference.is_some(),
    )?);

    return Ok(ftd::ChildComponent {
        root: sub.name.to_string(),
        condition: match sub.header.str_optional(doc.name, sub.line_number, "if")? {
            Some(expr) => Some(ftd::p2::Boolean::from_expression(
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
        recursive_kind: &ftd::p2::Kind,
        doc: &ftd::p2::TDoc,
        reference: String,
    ) -> ftd::p1::Result<Property> {
        let mut arguments: std::collections::BTreeMap<String, ftd::p2::Kind> = Default::default();
        arguments.insert("$loop$".to_string(), recursive_kind.to_owned());
        let property = ftd::PropertyValue::resolve_value(
            *line_number,
            &format!("${}", reference),
            None,
            doc,
            &arguments,
            None,
        )?;
        Ok(ftd::component::Property {
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

fn update_properties(
    properties: &mut std::collections::BTreeMap<String, (ftd::Value, Option<String>)>,
    arguments_value: &std::collections::BTreeMap<String, ftd::Value>,
    arguments: &std::collections::BTreeMap<String, ftd::p2::Kind>,
) {
    let default_property = vec![
        "id", "top", "bottom", "left", "right", "align", "scale", "rotate", "scale-x", "scale-y",
    ];
    for p in default_property {
        if !properties.contains_key(p) {
            if let Some(v) = arguments_value.get(p) {
                properties.insert(p.to_string(), (v.to_owned(), None));
            }
        }
    }
    for p in arguments.keys() {
        if let Some(v) = arguments_value.get(p) {
            properties.insert(format!("${}", p), (v.to_owned(), None));
        }
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
        || (name == "ftd.scene"))
}

fn assert_no_extra_properties(
    line_number: usize,
    p1: &ftd::p1::Header,
    root: &str,
    root_arguments: &std::collections::BTreeMap<String, ftd::p2::Kind>,
    name: &str,
    doc: &ftd::p2::TDoc,
) -> ftd::p1::Result<()> {
    for (i, k, _) in p1.0.iter() {
        if k == "component"
            || k.starts_with('$')
            || k == "if"
            || k.starts_with('/')
            || ftd::variable::VariableData::get_name_kind(k, doc, line_number, vec![].as_slice())
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
            || (is_component(name)
                && vec![
                    "id", "top", "bottom", "left", "right", "align", "scale", "rotate", "scale-x",
                    "scale-y",
                ]
                .contains(&key)))
        {
            return ftd::e2(
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

#[allow(clippy::too_many_arguments)]
pub fn read_properties(
    line_number: usize,
    p1: &ftd::p1::Header,
    caption: &Option<String>,
    body: &Option<(usize, String)>,
    fn_name: &str,
    root: &str,
    root_arguments: &mut std::collections::BTreeMap<String, ftd::p2::Kind>,
    arguments: &std::collections::BTreeMap<String, ftd::p2::Kind>,
    doc: &ftd::p2::TDoc,
    root_properties: &std::collections::BTreeMap<String, Property>,
    is_reference: bool,
) -> ftd::p1::Result<std::collections::BTreeMap<String, Property>> {
    let mut properties: std::collections::BTreeMap<String, Property> = Default::default();
    update_root_arguments(root_arguments);

    for (name, kind) in root_arguments.iter() {
        if let Some(prop) = root_properties.get(name) {
            properties.insert(name.to_string(), prop.clone());
            continue;
        }
        let (conditional_vector, source) = match (
            p1.conditional_str(doc.name, line_number, name),
            kind.inner(),
        ) {
            (Ok(v), _) => (v, ftd::TextSource::Header),
            (
                Err(ftd::p1::Error::NotFound { .. }),
                ftd::p2::Kind::String {
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
                        vec![(body.as_ref().unwrap().1.to_string(), None)],
                        ftd::TextSource::Body,
                    )
                } else if matches!(kind, ftd::p2::Kind::Optional { .. }) {
                    continue;
                } else if let Some(d) = d {
                    (vec![(d.to_string(), None)], ftd::TextSource::Default)
                } else if is_reference {
                    continue;
                } else {
                    return ftd::e2(
                        format!(
                            "{} is calling {}, without a required argument `{}`",
                            fn_name, root, name
                        ),
                        doc.name,
                        line_number,
                    );
                }
            }
            (Err(ftd::p1::Error::NotFound { .. }), k) => {
                if matches!(kind, ftd::p2::Kind::Optional { .. }) {
                    continue;
                }

                if let Some(d) = k.get_default_value_str() {
                    (vec![(d.to_string(), None)], ftd::TextSource::Default)
                } else if is_reference {
                    continue;
                } else {
                    return ftd::e2(
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
        for (value, conditional_attribute) in conditional_vector {
            let property_value = ftd::PropertyValue::resolve_value(
                line_number,
                value.as_str(),
                Some(kind.to_owned()),
                doc,
                arguments,
                Some(source.clone()),
            )?;
            dbg!(&property_value, &value, "hh");
            let nested_properties = if let ftd::PropertyValue::Reference {
                kind: ftd::p2::Kind::UI { .. },
                ..
            } = &property_value
            {
                ftd::p2::utils::structure_header_to_properties(
                    &value,
                    &arguments,
                    doc,
                    line_number,
                )?
            } else {
                Default::default()
            };

            let (condition_value, default_value) = if let Some(attribute) = conditional_attribute {
                let condition = ftd::p2::Boolean::from_expression(
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
    return Ok(properties);

    fn update_root_arguments(
        root_arguments: &mut std::collections::BTreeMap<String, ftd::p2::Kind>,
    ) {
        let mut default_argument: std::collections::BTreeMap<String, ftd::p2::Kind> =
            Default::default();
        default_argument.insert("id".to_string(), ftd::p2::Kind::string().into_optional());
        default_argument.insert("top".to_string(), ftd::p2::Kind::integer().into_optional());
        default_argument.insert(
            "bottom".to_string(),
            ftd::p2::Kind::integer().into_optional(),
        );
        default_argument.insert("left".to_string(), ftd::p2::Kind::integer().into_optional());
        default_argument.insert(
            "right".to_string(),
            ftd::p2::Kind::integer().into_optional(),
        );
        default_argument.insert("align".to_string(), ftd::p2::Kind::string().into_optional());
        default_argument.insert(
            "scale".to_string(),
            ftd::p2::Kind::decimal().into_optional(),
        );
        default_argument.insert(
            "rotate".to_string(),
            ftd::p2::Kind::integer().into_optional(),
        );
        default_argument.insert(
            "scale-x".to_string(),
            ftd::p2::Kind::decimal().into_optional(),
        );
        default_argument.insert(
            "scale-y".to_string(),
            ftd::p2::Kind::decimal().into_optional(),
        );

        for (key, arg) in default_argument {
            root_arguments.entry(key).or_insert(arg);
        }
    }
}

fn root_properties_from_inherits(
    line_number: usize,
    arguments: &std::collections::BTreeMap<String, ftd::p2::Kind>,
    inherits: Vec<String>,
    doc: &ftd::p2::TDoc,
) -> ftd::p1::Result<std::collections::BTreeMap<String, Property>> {
    let mut root_properties: std::collections::BTreeMap<String, Property> = Default::default();
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
            ftd::component::Property {
                default: Some(pv),
                conditions: vec![],
                ..Default::default()
            },
        );
    }
    Ok(root_properties)
}

fn read_arguments(
    p1: &ftd::p1::Header,
    root: &str,
    root_arguments: &std::collections::BTreeMap<String, ftd::p2::Kind>,
    doc: &ftd::p2::TDoc,
) -> ftd::p1::Result<(
    std::collections::BTreeMap<String, ftd::p2::Kind>,
    Vec<String>,
)> {
    let mut args: std::collections::BTreeMap<String, ftd::p2::Kind> = Default::default();
    let mut inherits: Vec<String> = Default::default();

    for (i, k, v) in p1.0.iter() {
        if (k.starts_with('$') && k.ends_with('$')) || k.starts_with('/') {
            // event and loop matches
            continue;
        }

        let var_data = match ftd::variable::VariableData::get_name_kind(
            k,
            doc,
            i.to_owned(),
            vec![].as_slice(),
        ) {
            Ok(v) => v,
            _ => continue,
        };

        let v = if v.is_empty() {
            None
        } else {
            Some(v.to_string())
        };

        let kind = if var_data.kind.eq("inherit") {
            match root_arguments.get(&var_data.name) {
                Some(kind) => {
                    inherits.push(var_data.name.to_string());
                    kind.clone().set_default(v)
                }
                None => {
                    return ftd::e2(
                        format!("'{}' is not an argument of {}", var_data.name, root),
                        doc.name,
                        i.to_owned(),
                    )
                }
            }
        } else {
            ftd::p2::Kind::for_variable(i.to_owned(), k, v, doc, None)?
        };
        args.insert(var_data.name.to_string(), kind);
    }

    Ok((args, inherits))
}

#[cfg(test)]
mod test {
    use ftd::component::Property;
    use ftd::test::*;

    macro_rules! p2 {
        ($s:expr, $doc: expr, $t: expr,) => {
            p2!($s, $doc, $t)
        };
        ($s:expr, $doc: expr, $t: expr) => {
            let p1 = ftd::p1::parse(indoc::indoc!($s), $doc.name).unwrap();
            pretty_assertions::assert_eq!(super::Component::from_p1(&p1[0], &$doc).unwrap(), $t)
        };
    }

    fn s(s: &str) -> String {
        s.to_string()
    }

    #[test]
    fn component() {
        let mut bag = ftd::p2::interpreter::default_bag();
        let aliases = ftd::p2::interpreter::default_aliases();
        let d = ftd::p2::TDoc {
            name: "foo",
            bag: &mut bag,
            aliases: &aliases,
        };
        p2!(
            "-- ftd.text foo:
            string foo:
            optional integer bar:
            text: hello
            ",
            d,
            super::Component {
                full_name: s("foo#foo"),
                root: "ftd.text".to_string(),
                arguments: std::array::IntoIter::new([
                    (s("foo"), ftd::p2::Kind::string()),
                    (s("bar"), ftd::p2::Kind::optional(ftd::p2::Kind::integer()))
                ])
                .collect(),
                properties: std::array::IntoIter::new([(
                    s("text"),
                    ftd::component::Property {
                        default: Some(ftd::PropertyValue::Value {
                            value: ftd::Value::String {
                                text: s("hello"),
                                source: ftd::TextSource::Header
                            }
                        }),
                        conditions: vec![],
                        ..Default::default()
                    }
                ),])
                .collect(),
                ..Default::default()
            }
        );
    }

    #[test]
    fn properties() {
        let mut bag = ftd::p2::interpreter::default_bag();
        let aliases = ftd::p2::interpreter::default_aliases();
        let d = ftd::p2::TDoc {
            name: "foo",
            bag: &mut bag,
            aliases: &aliases,
        };
        p2!(
            "-- ftd.text foo:
            text: hello
            ",
            d,
            super::Component {
                root: "ftd.text".to_string(),
                full_name: s("foo#foo"),
                properties: std::array::IntoIter::new([(
                    s("text"),
                    ftd::component::Property {
                        default: Some(ftd::PropertyValue::Value {
                            value: ftd::Value::String {
                                text: s("hello"),
                                source: ftd::TextSource::Header
                            }
                        }),
                        conditions: vec![],
                        ..Default::default()
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
            ftd::p2::Thing::Variable(ftd::Variable {
                name: "name".to_string(),
                value: ftd::Value::String {
                    text: s("Amit"),
                    source: ftd::TextSource::Caption,
                },
                conditions: vec![],
            }),
        );
        let mut main = default_column();
        main.container.children.push(ftd::Element::Text(ftd::Text {
            text: ftd::markdown_line("Amit"),
            line: true,
            common: ftd::Common {
                reference: Some(s("foo/bar#name")),
                ..Default::default()
            },
            ..Default::default()
        }));

        p!(
            "
            -- string name: Amit

            -- ftd.text:
            text: $name
            ",
            (bag.clone(), main.clone()),
        );

        p!(
            "
            -- string name: Amit

            -- ftd.text: $name
            ",
            (bag.clone(), main.clone()),
        );

        p!(
            "
            -- string name: Amit

            -- ftd.text:
            $name
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
            ftd::p2::Thing::Record(ftd::p2::Record {
                name: "foo/bar#person".to_string(),
                fields: person_fields(),
                instances: Default::default(),
            }),
        );
        bag.insert(
            "foo/bar#x".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: "x".to_string(),
                value: ftd::Value::Integer { value: 20 },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#abrar".to_string(),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: "abrar".to_string(),
                value: ftd::Value::Record {
                    name: "foo/bar#person".to_string(),
                    fields: abrar(),
                },
                conditions: vec![],
            }),
        );

        let mut main = default_column();
        main.container.children.push(ftd::Element::Text(ftd::Text {
            text: ftd::markdown_line("Abrar Khan"),
            line: true,
            ..Default::default()
        }));

        p!(
            "
            -- record person:
            caption name:
            string address:
            body bio:
            integer age:

            -- integer x: 10

            -- person abrar: Abrar Khan
            address: Bihar
            age: $x

            Software developer working at fifthtry.

            -- ftd.text:
            text: $abrar.name
            ",
            (bag.clone(), main.clone()),
        );
    }
}
