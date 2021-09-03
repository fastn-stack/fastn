#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Component {
    pub root: String,
    pub full_name: String,
    pub arguments: std::collections::BTreeMap<String, crate::p2::Kind>,
    pub properties: std::collections::BTreeMap<String, Property>,
    pub instructions: Vec<Instruction>,
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
}

#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Property {
    pub default: Option<crate::PropertyValue>,
    pub conditions: Vec<(crate::p2::Boolean, crate::PropertyValue)>,
}

pub struct ElementWithContainer {
    pub element: ftd_rt::Element,
    pub children: Vec<ftd_rt::Element>,
    pub child_container: Option<std::collections::BTreeMap<String, Vec<usize>>>,
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
    ) -> crate::p1::Result<ElementWithContainer> {
        let ElementWithContainer {
            mut element,
            child_container,
            ..
        } = self.call(doc, arguments, invocations, false, None)?;
        element.set_container_id(crate::p2::utils::string_optional(
            "id",
            &resolve_properties(&self.properties, arguments, doc, None)?,
        )?);
        match (&mut element, children.is_empty()) {
            (ftd_rt::Element::Column(c), _) => {
                for child in children.iter() {
                    c.container.children.push(
                        child
                            .call(doc, arguments, invocations, false, None)?
                            .element,
                    )
                }
            }
            (ftd_rt::Element::Row(c), _) => {
                for child in children.iter() {
                    c.container.children.push(
                        child
                            .call(doc, arguments, invocations, false, None)?
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
        )
    }

    pub fn from_p1(
        name: &str,
        p1: &crate::p1::Header,
        caption: &Option<String>,
        body: &Option<String>,
        doc: &crate::p2::TDoc,
        component: &str,
        arguments: &std::collections::BTreeMap<String, crate::p2::Kind>,
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
                doc,
            )?,
            condition: match p1.str_optional("if")? {
                Some(expr) => Some(crate::p2::Boolean::from_expression(
                    expr, doc, component, arguments,
                )?),
                None => None,
            },
            root: root.full_name.clone(),
        })
    }
}

fn resolve_properties(
    self_properties: &std::collections::BTreeMap<String, Property>,
    arguments: &std::collections::BTreeMap<String, crate::Value>,
    doc: &crate::p2::TDoc,
    root_name: Option<&str>,
) -> crate::p1::Result<std::collections::BTreeMap<String, crate::Value>> {
    let mut properties: std::collections::BTreeMap<String, crate::Value> = Default::default();
    for (name, value) in self_properties.iter() {
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
    ) -> crate::p1::Result<ElementWithContainer> {
        ftd::rt::execute(
            doc.name,
            doc.aliases,
            doc.bag,
            &self.instructions,
            arguments,
            invocations,
            root_name,
        )
    }

    pub fn from_p1(p1: &crate::p1::Section, doc: &crate::p2::TDoc) -> crate::p1::Result<Self> {
        let name = ftd_rt::get_name("component", p1.name.as_str())?.to_string();
        let root = p1.header.string("component")?;
        let mut root_arguments = doc.get_component(root.as_str())?.arguments;
        let (arguments, _inherits) =
            read_arguments(&p1.header, root.as_str(), &root_arguments, doc)?;
        assert_no_extra_properties(&p1.header, root.as_str(), &root_arguments, &p1.name)?;
        let mut instructions: Vec<Instruction> = Default::default();
        for sub in p1.sub_sections.0.iter() {
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
                )?;
                Instruction::ChildComponent { child: s }
            });
        }

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
                doc,
            )?,
            arguments,
            root,
            instructions,
            kernel: false,
            invocations: Default::default(),
        })
    }

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
    ) -> crate::p1::Result<ElementWithContainer> {
        invocations
            .entry(self.full_name.clone())
            .or_default()
            .push(arguments.to_owned());
        if self.root == "ftd.kernel" {
            let element = match self.full_name.as_str() {
                "ftd#text" => ftd_rt::Element::Text(ftd::p2::element::text_from_properties(
                    arguments, doc, condition, is_child,
                )?),
                "ftd#image" => ftd_rt::Element::Image(ftd::p2::element::image_from_properties(
                    arguments, doc, condition, is_child,
                )?),
                "ftd#row" => ftd_rt::Element::Row(ftd::p2::element::row_from_properties(
                    arguments, doc, condition, is_child,
                )?),
                "ftd#column" => ftd_rt::Element::Column(ftd::p2::element::column_from_properties(
                    arguments, doc, condition, is_child,
                )?),
                "ftd#iframe" => ftd_rt::Element::IFrame(ftd::p2::element::iframe_from_properties(
                    arguments, doc, condition, is_child,
                )?),
                "ftd#integer" => ftd_rt::Element::Integer(
                    ftd::p2::element::integer_from_properties(arguments, doc, condition, is_child)?,
                ),
                "ftd#decimal" => ftd_rt::Element::Decimal(
                    ftd::p2::element::decimal_from_properties(arguments, doc, condition, is_child)?,
                ),
                "ftd#boolean" => ftd_rt::Element::Boolean(
                    ftd::p2::element::boolean_from_properties(arguments, doc, condition, is_child)?,
                ),
                "ftd#input" => ftd_rt::Element::Input(ftd::p2::element::input_from_properties(
                    arguments, doc, condition, is_child,
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
            let root_properties = resolve_properties(&self.properties, arguments, doc, root_name)?;
            let mut element = root
                .call(
                    &root_properties,
                    doc,
                    invocations,
                    condition,
                    is_child,
                    root_name,
                )?
                .element;
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
                    } = self.call_sub_functions(arguments, doc, invocations, root_name)?;
                    containers = child_container;
                    e.container.children = children
                }
                ftd_rt::Element::Row(ref mut e) => {
                    let ElementWithContainer {
                        children,
                        child_container,
                        ..
                    } = self.call_sub_functions(arguments, doc, invocations, root_name)?;
                    containers = child_container;
                    e.container.children = children
                }
            }

            Ok(ElementWithContainer {
                element,
                children: vec![],
                child_container: containers,
            })
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
        if k == "component" || k.starts_with('$') || k == "if" {
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

fn read_value(
    name: &str,
    value: &str,
    source: crate::TextSource,
    kind: &crate::p2::Kind,
) -> crate::p1::Result<crate::Value> {
    match kind.inner() {
        crate::p2::Kind::Integer => {
            if let Ok(v) = value.parse::<i64>() {
                return Ok(crate::Value::Integer { value: v });
            }
        }
        crate::p2::Kind::Boolean => {
            if let Ok(v) = value.parse::<bool>() {
                return Ok(crate::Value::Boolean { value: v });
            }
        }
        crate::p2::Kind::Decimal => {
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
            let found_kind = match arguments.get(v) {
                Some(k) => k,
                None => {
                    return crate::e(format!("'{}' is not an argument of '{}'", v, name));
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
                name: ref_name,
                kind: kind.to_owned(),
            })
        }
    }
}

fn read_properties(
    p1: &crate::p1::Header,
    caption: &Option<String>,
    body: &Option<String>,
    fn_name: &str,
    root: &str,
    root_arguments: &mut std::collections::BTreeMap<String, crate::p2::Kind>,
    arguments: &std::collections::BTreeMap<String, crate::p2::Kind>,
    doc: &crate::p2::TDoc,
) -> crate::p1::Result<std::collections::BTreeMap<String, Property>> {
    let mut properties: std::collections::BTreeMap<String, Property> = Default::default();
    let id_already_present = root_arguments.contains_key("id");
    if !id_already_present {
        // to add "id" property by default for component as "-- component foo:"
        root_arguments.insert(
            "id".to_string(),
            crate::p2::Kind::Optional {
                kind: Box::new(crate::p2::Kind::String {
                    caption: false,
                    body: false,
                }),
            },
        );
    }
    for (name, kind) in root_arguments.iter() {
        let (conditional_vector, source) = match (p1.conditional_str(name), kind.inner()) {
            (Ok(v), _) => (v, ftd::TextSource::Header),
            (
                Err(crate::p1::Error::NotFound { .. }),
                crate::p2::Kind::String {
                    caption: c,
                    body: b,
                },
            ) => {
                if *c && caption.is_some() {
                    (
                        vec![(caption.as_ref().unwrap().as_str(), None)],
                        ftd::TextSource::Caption,
                    )
                } else if *b && body.is_some() {
                    (
                        vec![(body.as_ref().unwrap().as_str(), None)],
                        ftd::TextSource::Body,
                    )
                } else if matches!(kind, crate::p2::Kind::Optional { .. }) {
                    continue;
                } else {
                    return crate::e(format!(
                        "{} is calling {}, without a required argument `{}`",
                        fn_name, root, name,
                    ));
                }
            }
            (Err(crate::p1::Error::NotFound { .. }), _) => {
                if matches!(kind, crate::p2::Kind::Optional { .. }) {
                    continue;
                }
                return crate::e(format!(
                    "{} is calling {}, without a required argument `{}`",
                    fn_name, root, name,
                ));
            }
            (Err(e), _) => {
                return Err(e);
            }
        };
        for (value, conditional_attribute) in conditional_vector {
            let property_value = if value.starts_with("ref ") {
                read_reference(name, value, kind, doc, arguments)?
            } else {
                crate::PropertyValue::Value {
                    value: read_value(name, value, source.clone(), kind)?,
                }
            };
            let (condition_value, default_value) = if let Some(attribute) = conditional_attribute {
                let condition = crate::p2::Boolean::from_expression(attribute, doc, "", arguments)?;
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
            crate::p2::Kind::from(v, doc)?
        };
        args.insert(name.to_string(), kind);
    }

    Ok((args, inherits))
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
                        crate::p2::Kind::optional(crate::p2::Kind::Integer)
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
