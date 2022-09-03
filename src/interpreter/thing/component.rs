#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Component {
    pub root: String,
    pub full_name: String,
    pub arguments: ftd::Map<ftd::interpreter::Kind>,
    pub locals: ftd::Map<ftd::interpreter::Kind>,
    pub properties: ftd::Map<Property>,
    pub instructions: Vec<Instruction>,
    pub events: Vec<ftd::interpreter::Event>,
    pub condition: Option<ftd::interpreter::Boolean>,
    pub kernel: bool,
    pub invocations: Vec<ftd::Map<ftd::interpreter::Value>>,
    pub line_number: usize,
}

pub fn row_function() -> ftd::interpreter::Component {
    ftd::interpreter::Component {
        kernel: true,
        full_name: "ftd#row".to_string(),
        root: "ftd.kernel".to_string(),
        arguments: [
            container_arguments(),
            common_arguments(),
            vec![(
                "spacing".to_string(),
                ftd::interpreter::Kind::string().into_optional(),
            )],
        ]
        .concat()
        .into_iter()
        .collect(),
        locals: Default::default(),
        properties: Default::default(),
        instructions: Default::default(),
        invocations: Default::default(),
        condition: None,
        events: vec![],
        line_number: 0,
    }
}

fn container_arguments() -> Vec<(String, ftd::interpreter::Kind)> {
    vec![
        (
            "open".to_string(),
            ftd::interpreter::Kind::boolean().into_optional(),
        ),
        (
            "append-at".to_string(),
            ftd::interpreter::Kind::string().into_optional(),
        ),
        (
            "align".to_string(),
            ftd::interpreter::Kind::string().into_optional(),
        ),
        (
            "wrap".to_string(),
            ftd::interpreter::Kind::boolean().into_optional(),
        ),
    ]
}

fn common_arguments() -> Vec<(String, ftd::interpreter::Kind)> {
    vec![
        (
            "padding".to_string(),
            ftd::interpreter::Kind::integer().into_optional(),
        ),
        (
            "padding-vertical".to_string(),
            ftd::interpreter::Kind::integer().into_optional(),
        ),
        (
            "padding-horizontal".to_string(),
            ftd::interpreter::Kind::integer().into_optional(),
        ),
        (
            "padding-left".to_string(),
            ftd::interpreter::Kind::integer().into_optional(),
        ),
        (
            "padding-right".to_string(),
            ftd::interpreter::Kind::integer().into_optional(),
        ),
        (
            "padding-top".to_string(),
            ftd::interpreter::Kind::integer().into_optional(),
        ),
        (
            "padding-bottom".to_string(),
            ftd::interpreter::Kind::integer().into_optional(),
        ),
        (
            "border-top-radius".to_string(),
            ftd::interpreter::Kind::integer().into_optional(),
        ),
        (
            "border-bottom-radius".to_string(),
            ftd::interpreter::Kind::integer().into_optional(),
        ),
        (
            "border-left-radius".to_string(),
            ftd::interpreter::Kind::integer().into_optional(),
        ),
        (
            "border-right-radius".to_string(),
            ftd::interpreter::Kind::integer().into_optional(),
        ),
        (
            "width".to_string(),
            ftd::interpreter::Kind::string().into_optional(),
        ),
        (
            "min-width".to_string(),
            ftd::interpreter::Kind::string().into_optional(),
        ),
        (
            "max-width".to_string(),
            ftd::interpreter::Kind::string().into_optional(),
        ),
        (
            "height".to_string(),
            ftd::interpreter::Kind::string().into_optional(),
        ),
        (
            "min-height".to_string(),
            ftd::interpreter::Kind::string().into_optional(),
        ),
        (
            "max-height".to_string(),
            ftd::interpreter::Kind::string().into_optional(),
        ),
        (
            // TODO: remove this after verifying that no existing document is using this
            "explain".to_string(),
            ftd::interpreter::Kind::boolean().into_optional(),
        ),
        (
            "region".to_string(),
            ftd::interpreter::Kind::string().into_optional(),
        ),
        (
            "color".to_string(),
            ftd::interpreter::Kind::Record {
                name: "ftd#color".to_string(),
                default: None,
                is_reference: false,
            }
            .into_optional(),
        ),
        (
            "background-color".to_string(),
            ftd::interpreter::Kind::Record {
                name: "ftd#color".to_string(),
                default: None,
                is_reference: false,
            }
            .into_optional(),
        ),
        (
            "border-color".to_string(),
            ftd::interpreter::Kind::Record {
                name: "ftd#color".to_string(),
                default: None,
                is_reference: false,
            }
            .into_optional(),
        ),
        (
            "border-width".to_string(),
            ftd::interpreter::Kind::integer().into_optional(),
        ),
        (
            "border-radius".to_string(),
            ftd::interpreter::Kind::integer().into_optional(),
        ),
        (
            "id".to_string(),
            ftd::interpreter::Kind::string().into_optional(),
        ),
        (
            "overflow-x".to_string(),
            ftd::interpreter::Kind::string().into_optional(),
        ),
        (
            "overflow-y".to_string(),
            ftd::interpreter::Kind::string().into_optional(),
        ),
        (
            "border-top".to_string(),
            ftd::interpreter::Kind::integer().into_optional(),
        ),
        (
            "border-bottom".to_string(),
            ftd::interpreter::Kind::integer().into_optional(),
        ),
        (
            "border-left".to_string(),
            ftd::interpreter::Kind::integer().into_optional(),
        ),
        (
            "border-right".to_string(),
            ftd::interpreter::Kind::integer().into_optional(),
        ),
        (
            "border-top-color".to_string(),
            ftd::interpreter::Kind::record("ftd#color").into_optional(),
        ),
        (
            "border-left-color".to_string(),
            ftd::interpreter::Kind::record("ftd#color").into_optional(),
        ),
        (
            "border-right-color".to_string(),
            ftd::interpreter::Kind::record("ftd#color").into_optional(),
        ),
        (
            "border-bottom-color".to_string(),
            ftd::interpreter::Kind::record("ftd#color").into_optional(),
        ),
        (
            "margin-top".to_string(),
            ftd::interpreter::Kind::integer().into_optional(),
        ),
        (
            "margin-bottom".to_string(),
            ftd::interpreter::Kind::integer().into_optional(),
        ),
        (
            "margin-left".to_string(),
            ftd::interpreter::Kind::integer().into_optional(),
        ),
        (
            "margin-right".to_string(),
            ftd::interpreter::Kind::integer().into_optional(),
        ),
        (
            "link".to_string(),
            ftd::interpreter::Kind::string().into_optional(),
        ),
        (
            "submit".to_string(),
            ftd::interpreter::Kind::string().into_optional(),
        ),
        (
            "open-in-new-tab".to_string(),
            ftd::interpreter::Kind::boolean().into_optional(),
        ),
        (
            "sticky".to_string(),
            ftd::interpreter::Kind::boolean().into_optional(),
        ),
        (
            "top".to_string(),
            ftd::interpreter::Kind::integer().into_optional(),
        ),
        (
            "bottom".to_string(),
            ftd::interpreter::Kind::integer().into_optional(),
        ),
        (
            "left".to_string(),
            ftd::interpreter::Kind::integer().into_optional(),
        ),
        (
            "right".to_string(),
            ftd::interpreter::Kind::integer().into_optional(),
        ),
        (
            "cursor".to_string(),
            ftd::interpreter::Kind::string().into_optional(),
        ),
        (
            "anchor".to_string(),
            ftd::interpreter::Kind::string().into_optional(),
        ),
        (
            "gradient-direction".to_string(),
            ftd::interpreter::Kind::string().into_optional(),
        ),
        (
            "gradient-colors".to_string(),
            ftd::interpreter::Kind::string().into_optional(),
        ),
        (
            "shadow-offset-x".to_string(),
            ftd::interpreter::Kind::integer().into_optional(),
        ),
        (
            "shadow-offset-y".to_string(),
            ftd::interpreter::Kind::integer().into_optional(),
        ),
        (
            "shadow-blur".to_string(),
            ftd::interpreter::Kind::integer().into_optional(),
        ),
        (
            "shadow-size".to_string(),
            ftd::interpreter::Kind::integer().into_optional(),
        ),
        (
            "shadow-color".to_string(),
            ftd::interpreter::Kind::record("ftd#color").into_optional(),
        ),
        (
            "background-image".to_string(),
            ftd::interpreter::Kind::record("ftd#image-src").into_optional(),
        ),
        (
            "background-repeat".to_string(),
            ftd::interpreter::Kind::boolean().into_optional(),
        ),
        (
            "background-parallax".to_string(),
            ftd::interpreter::Kind::boolean().into_optional(),
        ),
        (
            "scale".to_string(),
            ftd::interpreter::Kind::decimal().into_optional(),
        ),
        (
            "scale-x".to_string(),
            ftd::interpreter::Kind::decimal().into_optional(),
        ),
        (
            "scale-y".to_string(),
            ftd::interpreter::Kind::decimal().into_optional(),
        ),
        (
            "rotate".to_string(),
            ftd::interpreter::Kind::integer().into_optional(),
        ),
        (
            "move-up".to_string(),
            ftd::interpreter::Kind::integer().into_optional(),
        ),
        (
            "move-down".to_string(),
            ftd::interpreter::Kind::integer().into_optional(),
        ),
        (
            "move-left".to_string(),
            ftd::interpreter::Kind::integer().into_optional(),
        ),
        (
            "move-right".to_string(),
            ftd::interpreter::Kind::integer().into_optional(),
        ),
        (
            "position".to_string(),
            ftd::interpreter::Kind::string().into_optional(),
        ),
        (
            "z-index".to_string(),
            ftd::interpreter::Kind::integer().into_optional(),
        ),
        (
            "slot".to_string(),
            ftd::interpreter::Kind::string().into_optional(),
        ),
        (
            "white-space".to_string(),
            ftd::interpreter::Kind::string().into_optional(),
        ),
        (
            "border-style".to_string(),
            ftd::interpreter::Kind::string().into_optional(),
        ),
        (
            "text-transform".to_string(),
            ftd::interpreter::Kind::string().into_optional(),
        ),
        /*(
            "grid-column".to_string(),
            ftd::interpreter::Kind::string().into_optional(),
        ),
        (
            "grid-row".to_string(),
            ftd::interpreter::Kind::string().into_optional(),
        ),*/
    ]
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
            if let Some(ftd::interpreter::Property::Value {
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
    pub condition: Option<ftd::interpreter::Boolean>,
    pub properties: ftd::Map<Property>,
    pub arguments: ftd::Map<ftd::interpreter::Kind>,
    pub events: Vec<ftd::interpreter::Event>,
    pub is_recursive: bool,
    pub line_number: usize,
    pub reference: Option<(String, ftd::interpreter::Kind)>,
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Property {
    pub default: Option<ftd::interpreter::PropertyValue>,
    pub conditions: Vec<(ftd::interpreter::Boolean, ftd::interpreter::PropertyValue)>,
    pub nested_properties: ftd::Map<ftd::interpreter::Property>,
}

impl Property {
    fn eval(
        &self,
        line_number: usize,
        name: &str,
        doc: &ftd::interpreter::TDoc,
    ) -> ftd::p11::Result<&ftd::interpreter::PropertyValue> {
        let mut property_value = ftd::interpreter::utils::e2(
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
    fn resolve_default_value_string(
        &self,
        doc: &ftd::interpreter::TDoc,
        line_number: usize,
    ) -> ftd::p11::Result<String> {
        if let Some(property_value) = &self.default {
            if let Some(val) = property_value.resolve(line_number, doc)?.to_string() {
                return Ok(val);
            }
        }
        Ok("".to_string())
    }
}

impl ChildComponent {
    pub fn from_p1(
        line_number: usize,
        name: &str,
        p1: &[ftd::p11::Header],
        caption: &Option<ftd::p11::Header>,
        body: &Option<ftd::p11::Body>,
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
            ftd::interpreter::Component {
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
            caption: &Option<ftd::p11::Header>,
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
                            ftd::interpreter::Property {
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

/// In case markup the behaviour of container_children is not the same.
/// They act as the component variables which are, then, referred to in markup text
/// container_children copy there properties to the reference in markup text

fn resolve_recursive_property(
    line_number: usize,
    self_properties: &ftd::Map<Property>,
    doc: &ftd::interpreter::TDoc,
) -> ftd::p11::Result<ftd::interpreter::Value> {
    if let Some(value) = self_properties.get("$loop$") {
        if let Ok(property_value) = value.eval(line_number, "$loop$", doc) {
            return property_value.resolve(line_number, doc);
        }
    }
    ftd::interpreter::utils::e2(
        format!("$loop$ not found in properties {:?}", self_properties),
        doc.name,
        line_number,
    )
}

pub fn resolve_properties(
    line_number: usize,
    self_properties: &ftd::Map<Property>,
    doc: &ftd::interpreter::TDoc,
) -> ftd::p11::Result<ftd::Map<ftd::interpreter::Value>> {
    resolve_properties_by_id(line_number, self_properties, doc, None)
}

pub fn resolve_properties_by_id(
    line_number: usize,
    self_properties: &ftd::Map<Property>,
    doc: &ftd::interpreter::TDoc,
    id: Option<String>,
) -> ftd::p11::Result<ftd::Map<ftd::interpreter::Value>> {
    let mut properties: ftd::Map<ftd::interpreter::Value> = Default::default();
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

pub(crate) fn resolve_properties_with_ref(
    line_number: usize,
    self_properties: &ftd::Map<Property>,
    doc: &ftd::interpreter::TDoc,
) -> ftd::p11::Result<ftd::Map<(ftd::interpreter::Value, Option<String>)>> {
    let mut properties: ftd::Map<(ftd::interpreter::Value, Option<String>)> = Default::default();
    for (name, value) in self_properties.iter() {
        if name == "$loop$" {
            continue;
        }
        if let Ok(property_value) = value.eval(line_number, name, doc) {
            let reference = match property_value {
                ftd::interpreter::Property::Reference { name, .. } => Some(name.to_string()),
                ftd::interpreter::Property::Variable { name, .. } => Some(name.to_string()),
                _ => None,
            };
            let resolved_value = {
                let mut resolved_value = property_value.resolve(line_number, doc)?;
                if let ftd::interpreter::Value::UI { data, .. } = &mut resolved_value {
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
    pub fn get_caption(&self) -> Option<String> {
        let mut new_caption_title = None;
        for (arg, arg_kind) in self.arguments.clone() {
            if let ftd::interpreter::Kind::String { caption, .. } = arg_kind {
                if caption {
                    new_caption_title = Some(arg);
                }
            }
        }
        new_caption_title
    }

    pub fn from_p1(p1: &ftd::p11::Section, doc: &ftd::interpreter::TDoc) -> ftd::p11::Result<Self> {
        let var_data = ftd::interpreter::variable::VariableData::get_name_kind(
            &p1.name,
            &p1.kind,
            doc,
            p1.line_number,
            vec![].as_slice(),
        )?;
        if var_data.is_variable() {
            return ftd::interpreter::utils::e2(
                format!("expected component, found: {}", p1.name),
                doc.name,
                p1.line_number,
            );
        }
        let name = var_data.name;
        let root = doc.resolve_name(p1.line_number, var_data.kind.as_str())?;
        let root_component = doc.get_component(p1.line_number, root.as_str())?;
        let (mut arguments, inherits) = read_arguments(
            &p1.headers,
            root.as_str(),
            &root_component.arguments,
            &Default::default(),
            doc,
        )?;

        // Extend the local arguments with universal arguments
        arguments.extend(universal_arguments());

        assert_no_extra_properties(
            p1.line_number,
            &p1.headers,
            &p1.kind,
            root.as_str(),
            &root_component.arguments,
            &p1.name,
            doc,
        )?;
        let mut instructions: Vec<Instruction> = Default::default();

        for sub in p1.sub_sections.iter() {
            if sub.is_commented {
                continue;
            }
            if let Ok(loop_data) = sub.headers.str(doc.name, p1.line_number, "$loop$") {
                let loop_data = loop_data.ok_or(|| ftd::p11::Error::ParseError {
                    message: format!("Expected value for $loop$"),
                    doc_id: doc.name.to_string(),
                    line_number: sub.line_number,
                })?;
                instructions.push(Instruction::RecursiveChildComponent {
                    child: recursive_child_component(
                        loop_data.as_str(),
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
                let child = if ftd::interpreter::utils::get_root_component_name(
                    doc,
                    root_component.full_name.as_str(),
                    sub.line_number,
                )?
                .eq("ftd#text")
                {
                    ftd::interpreter::utils::get_markup_child(sub, doc, &arguments)?
                } else {
                    ftd::interpreter::ChildComponent::from_p1(
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
            Some(expr) => Some(ftd::interpreter::Boolean::from_expression(
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

    pub fn to_value(
        &self,
        kind: &ftd::interpreter::Kind,
    ) -> ftd::p11::Result<ftd::interpreter::Value> {
        Ok(ftd::interpreter::Value::UI {
            name: self.full_name.to_string(),
            kind: kind.to_owned(),
            data: Default::default(),
        })
    }
}

pub fn recursive_child_component(
    loop_data: &str,
    sub: &ftd::p11::Section,
    doc: &ftd::interpreter::TDoc,
    arguments: &ftd::Map<ftd::interpreter::Kind>,
    name_with_component: Option<(String, ftd::interpreter::Component)>,
) -> ftd::p11::Result<ftd::interpreter::ChildComponent> {
    let mut loop_ref = "object".to_string();
    let mut loop_on_component = loop_data.to_string();

    if loop_data.contains("as") {
        let parts = ftd::interpreter::utils::split(loop_data.to_string(), " as ")?;
        loop_on_component = parts.0;
        loop_ref = if let Some(loop_ref) = parts.1.strip_prefix('$') {
            loop_ref.to_string()
        } else {
            return ftd::interpreter::utils::e2(
                format!("loop variable should start with $, found: {}", parts.1),
                doc.name,
                sub.line_number,
            );
        };
    }

    let recursive_property_value = ftd::interpreter::PropertyValue::resolve_value(
        sub.line_number,
        &loop_on_component,
        None,
        doc,
        arguments,
        None,
    )?;

    let recursive_kind =
        if let ftd::interpreter::Kind::List { kind, .. } = recursive_property_value.kind() {
            kind.as_ref().to_owned()
        } else {
            return ftd::interpreter::utils::e2(
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
        ftd::interpreter::Property {
            default: Some(recursive_property_value),
            conditions: vec![],
            ..Default::default()
        },
    );

    let mut new_header = ftd::p11::Headers(vec![]);
    let (mut left_boolean, mut right_boolean) = (None, None);
    for header in &sub.headers.0 {
        if header.is_section() {
            continue;
        }
        let i = header.get_line_number();
        let k = header.get_key();
        let v = if let Some(value) = header.get_value(doc.name)? {
            value
        } else {
            new_header.push(header.to_owned());
            continue;
        };
        if k == "$loop$" {
            continue;
        }

        if k == "if" && contains_loop_ref(&loop_ref, v.as_str()) {
            let v = v.replace(&format!("${}", loop_ref), "$loop$");
            let (_, left, right) = ftd::interpreter::Boolean::boolean_left_right(i, &v, doc.name)?;
            if left.contains("$loop$") {
                left_boolean = resolve_loop_reference(i, &recursive_kind, doc, left)?.default;
            }
            if let Some(r) = right {
                if r.contains("$loop$") {
                    right_boolean = resolve_loop_reference(i, &recursive_kind, doc, r)?.default;
                }
            }
        }

        if contains_loop_ref(&loop_ref, v.as_str()) && v.starts_with(&format!("${}", loop_ref)) {
            let reference = v.to_string().replace(&format!("${}", loop_ref), "$loop$");
            let value = resolve_loop_reference(i, &recursive_kind, doc, reference)?;
            properties.insert(k.to_string(), value);
        } else {
            new_header.push(header.to_owned());
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
            let root = if let Some(ftd::interpreter::Kind::UI { default }) =
                arguments.get(&sub.name).map(|v| v.inner())
            {
                reference = Some((
                    sub.name.to_string(),
                    ftd::interpreter::Kind::UI {
                        default: (*default).clone(),
                    },
                ));
                ftd::interpreter::Component {
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
                &Some(root.full_name),
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
        let caption_value = caption.get_value(doc.name)?.unwrap(); //TODO: throw error
        if contains_loop_ref(&loop_ref, caption_value.as_str()) {
            let reference = caption_value.replace(&format!("${}", loop_ref), "$loop$");
            let value = resolve_loop_reference(sub.line_number, &recursive_kind, doc, reference)?;
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

    return Ok(ftd::interpreter::ChildComponent {
        root: doc.resolve_name(sub.line_number, &sub.name.to_string())?,
        condition: match sub.header.str_optional(doc.name, sub.line_number, "if")? {
            Some(expr) => Some(ftd::interpreter::Boolean::from_expression(
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
        line_number: usize,
        recursive_kind: &ftd::interpreter::Kind,
        doc: &ftd::interpreter::TDoc,
        reference: String,
    ) -> ftd::p11::Result<Property> {
        let mut arguments: ftd::Map<ftd::interpreter::Kind> = Default::default();
        arguments.insert("$loop$".to_string(), recursive_kind.to_owned());
        let property = ftd::interpreter::Property::resolve_value(
            line_number,
            &format!("${}", reference),
            None,
            doc,
            &arguments,
            None,
        )?;
        Ok(ftd::interpreter::Property {
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
    !(name.starts_with("component")
        || name.starts_with("var")
        || name.starts_with("record")
        || name.starts_with("or-type")
        || name.starts_with("list")
        || name.starts_with("map")
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
    p1: &ftd::p11::Headers,
    kind: &Option<String>,
    root: &str,
    root_arguments: &ftd::Map<ftd::interpreter::Kind>,
    name: &str,
    doc: &ftd::interpreter::TDoc,
) -> ftd::p11::Result<()> {
    for header in p1.0.iter() {
        let i = header.get_line_number();
        let k = header.get_key();
        if (k.starts_with('$') && k.ends_with('$'))
            || k.eq("if")
            || ftd::variable::VariableData::get_name_kind(
                k.as_str(),
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
            || (kind.map_or(false, is_component) && universal_arguments().contains_key(key)))
        {
            return ftd::interpreter::utils::e2(
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
    doc: &ftd::interpreter::TDoc,
    line_number: usize,
) -> ftd::p11::Result<()> {
    fn get_property_default_value(
        property_name: &str,
        properties: &ftd::Map<Property>,
        doc: &ftd::interpreter::TDoc,
        line_number: usize,
    ) -> ftd::p11::Result<String> {
        if let Some(property) = properties.get(property_name) {
            return property.resolve_default_value_string(doc, line_number);
        }
        Err(ftd::p11::Error::NotFound {
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

            Err(ftd::p11::Error::ForbiddenUsage {
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
    p1: &ftd::p11::Headers,
    caption: &Option<ftd::p11::Header>,
    body: &Option<ftd::p11::Body>,
    fn_name: &str,
    root: &str,
    root_arguments: &ftd::Map<ftd::interpreter::Kind>,
    arguments: &ftd::Map<ftd::interpreter::Kind>,
    doc: &ftd::interpreter::TDoc,
    root_properties: &ftd::Map<Property>,
    is_reference: bool,
) -> ftd::p11::Result<ftd::Map<Property>> {
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
                Err(ftd::p11::Error::NotFound { .. }),
                ftd::interpreter::Kind::String {
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
                } else if matches!(kind, ftd::interpreter::Kind::Optional { .. }) {
                    continue;
                } else if let Some(d) = d {
                    (
                        vec![(None, d.to_string(), None, *r)],
                        ftd::TextSource::Default,
                    )
                } else if is_reference {
                    continue;
                } else {
                    return ftd::interpreter::utils::e2(
                        format!(
                            "{} is calling {}, without a required argument 1 `{}`",
                            fn_name, root, name
                        ),
                        doc.name,
                        line_number,
                    );
                }
            }
            (Err(ftd::p11::Error::NotFound { .. }), k) => {
                if matches!(kind, ftd::interpreter::Kind::Optional { .. }) {
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
                    return ftd::interpreter::utils::e2(
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
                return ftd::interpreter::utils::e2(
                    format!(
                        "{} is calling {}, without a referenced argument `{}`",
                        fn_name, root, value
                    ),
                    doc.name,
                    line_number,
                );
            }
            let mut property_value = match ftd::interpreter::PropertyValue::resolve_value(
                line_number,
                value.as_str(),
                Some(kind.to_owned()),
                doc,
                arguments,
                Some(source.clone()),
            ) {
                Ok(p) => p,
                _ if source.eq(&ftd::TextSource::Default) => {
                    ftd::interpreter::Property::resolve_value(
                        line_number,
                        value.as_str(),
                        Some(kind.to_owned()),
                        doc,
                        root_arguments,
                        Some(source.clone()),
                    )?
                }
                Err(e) => return Err(e),
            };

            if is_referenced {
                property_value.set_reference();
            }

            let nested_properties = match property_value {
                ftd::interpreter::Property::Reference { ref kind, .. }
                    if matches!(kind.inner(), ftd::interpreter::Kind::UI { .. }) =>
                {
                    let headers = if source.eq(&ftd::TextSource::Default) {
                        let mut headers = Default::default();
                        if let ftd::interpreter::Kind::UI {
                            default: Some((_, h)),
                        } = kind.inner()
                        {
                            headers = h.clone();
                        }
                        headers
                    } else {
                        let mut headers = vec![];
                        if let Some(idx) = idx {
                            let p1 = &p1;
                            for i in idx + 1..p1.len() {
                                let p1 = p1.get(i).unwrap();
                                if let Some(k) = p1.1.strip_prefix('>') {
                                    headers.push((p1.0, k.trim().to_string(), p1.2.to_string()));
                                } else {
                                    break;
                                }
                            }
                        }
                        ftd::p11::Header(headers)
                    };
                    ftd::interpreter::utils::structure_header_to_properties(
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
                    let condition = ftd::interpreter::Boolean::from_expression(
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
/// and the user doesn't pass this data in exactly one way
///
/// # Missing data checks
/// - This happens if any (required) argument doesn't get the data from any way it takes
///
/// # Unknown data checks
/// - This happens when there is no argument to accept the data passed from caption/body
///
fn assert_caption_body_checks(
    root: &str,
    p1: &[ftd::p11::Header],
    doc: &ftd::interpreter::TDoc,
    caption: &Option<ftd::p11::Header>,
    body: &Option<ftd::p11::Body>,
    line_number: usize,
) -> ftd::p11::Result<()> {
    // No checks on ftd#ui
    if is_it_ui(root) {
        return Ok(());
    }

    let mut has_caption = caption.is_some();
    let mut has_body = body.is_some();

    let mut properties = None;
    let mut header_list: Option<&[ftd::p11::Header]> = Some(p1);

    let mut thing = doc.get_thing(line_number, root)?;
    loop {
        // Either the component is kernel or variable/derived component
        if let ftd::interpreter::Thing::Component(c) = thing {
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
        arguments: &std::collections::BTreeMap<String, ftd::interpreter::Kind>,
        properties: Option<std::collections::BTreeMap<String, Property>>,
        p1: Option<&[ftd::p11::Header]>,
        doc: &ftd::interpreter::TDoc,
        has_caption: bool,
        has_body: bool,
        line_number: usize,
    ) -> ftd::p11::Result<()> {
        /// returns a hashset`<key>` of header keys which have non-empty values
        fn get_header_set_with_values(
            p1: Option<&[ftd::p11::Header]>,
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
                ftd::interpreter::Kind::String {
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
                                return Err(ftd::p11::Error::ForbiddenUsage {
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
                                return Err(ftd::p11::Error::MissingData {
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
                                return Err(ftd::p11::Error::ForbiddenUsage {
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
                                return Err(ftd::p11::Error::MissingData {
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
                                return Err(ftd::p11::Error::ForbiddenUsage {
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
                                return Err(ftd::p11::Error::MissingData {
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
                ftd::interpreter::Kind::Integer { default, .. }
                | ftd::interpreter::Kind::Decimal { default, .. }
                | ftd::interpreter::Kind::Boolean { default, .. }
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
                        return Err(ftd::p11::Error::ForbiddenUsage {
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
                        return Err(ftd::p11::Error::MissingData {
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
                return Err(ftd::p11::Error::UnknownData {
                    message: "caption passed with no header accepting it !!".to_string(),
                    doc_id: doc.name.to_string(),
                    line_number,
                });
            }

            // if body is passed and body not utilized then throw error
            if !body_pass && has_body {
                return Err(ftd::p11::Error::UnknownData {
                    message: "body passed with no header accepting it !!".to_string(),
                    doc_id: doc.name.to_string(),
                    line_number,
                });
            }
        }

        Ok(())
    }
}

pub(crate) fn universal_arguments() -> ftd::Map<ftd::interpreter::Kind> {
    let mut universal_arguments: ftd::Map<ftd::interpreter::Kind> = Default::default();
    universal_arguments.insert(
        "id".to_string(),
        ftd::interpreter::Kind::string().into_optional(),
    );
    universal_arguments.insert(
        "top".to_string(),
        ftd::interpreter::Kind::integer().into_optional(),
    );
    universal_arguments.insert(
        "bottom".to_string(),
        ftd::interpreter::Kind::integer().into_optional(),
    );
    universal_arguments.insert(
        "left".to_string(),
        ftd::interpreter::Kind::integer().into_optional(),
    );
    universal_arguments.insert(
        "move-up".to_string(),
        ftd::interpreter::Kind::integer().into_optional(),
    );
    universal_arguments.insert(
        "move-down".to_string(),
        ftd::interpreter::Kind::integer().into_optional(),
    );
    universal_arguments.insert(
        "move-left".to_string(),
        ftd::interpreter::Kind::integer().into_optional(),
    );
    universal_arguments.insert(
        "move-right".to_string(),
        ftd::interpreter::Kind::integer().into_optional(),
    );
    universal_arguments.insert(
        "right".to_string(),
        ftd::interpreter::Kind::integer().into_optional(),
    );

    universal_arguments.insert(
        "align".to_string(),
        ftd::interpreter::Kind::string().into_optional(),
    );
    universal_arguments.insert(
        "scale".to_string(),
        ftd::interpreter::Kind::decimal().into_optional(),
    );
    universal_arguments.insert(
        "rotate".to_string(),
        ftd::interpreter::Kind::integer().into_optional(),
    );
    universal_arguments.insert(
        "scale-x".to_string(),
        ftd::interpreter::Kind::decimal().into_optional(),
    );
    universal_arguments.insert(
        "scale-y".to_string(),
        ftd::interpreter::Kind::decimal().into_optional(),
    );
    universal_arguments.insert(
        "slot".to_string(),
        ftd::interpreter::Kind::string().into_optional(),
    );

    universal_arguments
}

fn root_properties_from_inherits(
    line_number: usize,
    arguments: &ftd::Map<ftd::interpreter::Kind>,
    inherits: Vec<String>,
    doc: &ftd::interpreter::TDoc,
) -> ftd::p11::Result<ftd::Map<Property>> {
    let mut root_properties: ftd::Map<Property> = Default::default();
    for inherit in inherits {
        let pv = ftd::interpreter::Property::resolve_value(
            line_number,
            &format!("${}", inherit),
            None,
            doc,
            arguments,
            None,
        )?;
        root_properties.insert(
            inherit,
            ftd::interpreter::Property {
                default: Some(pv),
                conditions: vec![],
                ..Default::default()
            },
        );
    }
    Ok(root_properties)
}

fn read_arguments(
    p1: &ftd::p11::Headers,
    root: &str,
    root_arguments: &ftd::Map<ftd::interpreter::Kind>,
    arguments: &ftd::Map<ftd::interpreter::Kind>,
    doc: &ftd::interpreter::TDoc,
) -> ftd::p11::Result<(ftd::Map<ftd::interpreter::Kind>, Vec<String>)> {
    let mut args: ftd::Map<ftd::interpreter::Kind> = Default::default();
    let mut inherits: Vec<String> = Default::default();

    // contains parent arguments and current arguments
    let mut all_args = arguments.clone();

    // Set of all universal arguments available to all components
    let universal_arguments_set: std::collections::HashSet<String> =
        universal_arguments().keys().cloned().collect();

    // Set of root arguments which are invoked once
    let mut root_args_set: std::collections::HashSet<String> = std::collections::HashSet::new();
    for header in p1.0.iter() {
        let (line_number, key, kind, value) = match header {
            ftd::p11::Header::KV(ftd::p11::header::KV {
                line_number,
                key,
                kind,
                value,
            }) if !(key.starts_with('$') && key.ends_with('$')) => {
                (line_number, key, kind, value.to_owned())
            }
            ftd::p11::Header::Section(ftd::p11::header::Section {
                line_number,
                key,
                kind,
                ..
            }) => (line_number, key, kind, None),
            _ => continue,
        };

        // if (k.starts_with('$') && k.ends_with('$')) || k.starts_with('>') {
        //     // event and loop matches
        //     continue;
        // }

        let var_data = match ftd::interpreter::variable::VariableData::get_name_kind(
            key,
            kind,
            doc,
            line_number.to_owned(),
            vec![].as_slice(),
        ) {
            Ok(v) => v,
            _ => {
                // Duplicate header usage check
                if root_args_set.contains(key) {
                    if let Some(kind) = root_arguments.get(key) {
                        if kind.inner().is_list() {
                            continue;
                        }
                        return Err(ftd::p11::Error::ForbiddenUsage {
                            message: format!("repeated usage of \'{}\' not allowed !!", key),
                            doc_id: doc.name.to_string(),
                            line_number: *line_number,
                        });
                    }
                } else {
                    root_args_set.insert(key.to_string());
                }

                continue;
            }
        };

        let mut kind = if var_data.kind.eq("inherit") {
            match root_arguments.get(&var_data.name) {
                Some(kind) => {
                    inherits.push(var_data.name.to_string());
                    let default = {
                        // resolve the default value
                        let mut default = value.to_owned();
                        if let Some(ref v) = default {
                            default = Some(doc.resolve_reference_name(
                                line_number.to_owned(),
                                v,
                                &all_args,
                            )?);
                        }
                        default
                    };
                    kind.clone().set_default(default)
                }
                None => {
                    return ftd::interpreter::utils::e2(
                        format!("'{}' is not an argument of {}", var_data.name, root),
                        doc.name,
                        line_number.to_owned(),
                    )
                }
            }
        } else {
            ftd::interpreter::Kind::for_variable(
                line_number.to_owned(),
                key,
                kind,
                value.to_owned(),
                doc,
                None,
                &all_args,
            )?
        };
        if let ftd::interpreter::Kind::UI {
            default: Some((ui_id, h)),
        } = &mut kind.mut_inner()
        {
            let headers = if header.is_section() {
                header
            } else {
                return ftd::interpreter::utils::e2(
                    format!(
                        "'{}' header is not a section type {}",
                        header.get_key(),
                        root
                    ),
                    doc.name,
                    line_number.to_owned(),
                );
            };
            *h = headers.to_owned();
            *ui_id = doc.resolve_name(*line_number, ui_id.as_str())?;
        }

        // Duplicate header definition check
        if args.contains_key(var_data.name.as_str()) {
            return Err(ftd::p11::Error::ForbiddenUsage {
                message: format!(
                    "\'{}\' is already used as header name/identifier !!",
                    &var_data.name
                ),
                doc_id: doc.name.to_string(),
                line_number: *line_number,
            });
        }

        // checking if any universal argument is declared by the user (forbidden)
        if universal_arguments_set.contains(&var_data.name) {
            return Err(ftd::p11::Error::ForbiddenUsage {
                message: format!(
                    "redundant declaration of universal argument \'{}\' !!",
                    &var_data.name
                ),
                doc_id: doc.name.to_string(),
                line_number: *line_number,
            });
        }

        args.insert(var_data.name.to_string(), kind.clone());
        all_args.insert(var_data.name.to_string(), kind);
    }

    Ok((args, inherits))
}
