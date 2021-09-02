pub(crate) struct Interpreter<'a> {
    lib: &'a dyn crate::p2::Library,
    pub bag: std::collections::BTreeMap<String, crate::p2::Thing>,
    pub p1: Vec<ftd::p1::Section>,
    pub aliases: std::collections::BTreeMap<String, String>,
}

impl<'a> Interpreter<'a> {
    pub(crate) fn interpret(
        &mut self,
        name: &str,
        s: &str,
    ) -> crate::p1::Result<Vec<ftd::Instruction>> {
        self.interpret_(name, s, true)
    }

    fn interpret_(
        &mut self,
        name: &str,
        s: &str,
        is_main: bool,
    ) -> crate::p1::Result<Vec<ftd::Instruction>> {
        let p1 = crate::p1::parse(s)?;
        let mut aliases = default_aliases();

        let mut instructions: Vec<ftd::Instruction> = Default::default();

        for p1 in p1.iter() {
            if p1.name == "import" {
                let (library_name, alias) = crate::p2::utils::parse_import(&p1.caption)?;
                aliases.insert(alias, library_name.clone());
                let s = self.lib.get_with_result(library_name.as_str())?;
                self.interpret_(library_name.as_str(), s.as_str(), false)?;
                continue;
            }

            // while this is a specific to entire document, we are still creating it in a loop
            // because otherwise the self.interpret() call wont compile.
            let doc = crate::p2::TDoc {
                name,
                aliases: &aliases,
                bag: &self.bag,
            };

            let mut thing = None;

            if p1.name.starts_with("component ") {
                // declare a function
                let d = crate::Component::from_p1(p1, &doc)?;
                thing = Some((d.full_name.to_string(), crate::p2::Thing::Component(d)));
            } else if p1.name.starts_with("var ") {
                // declare and instantiate a variable
                let d = crate::Variable::from_p1(p1, &doc)?;
                thing = Some((d.name.to_string(), crate::p2::Thing::Variable(d)));
            } else if p1.name.starts_with("record ") {
                // declare a record
                let d = crate::p2::Record::from_p1(p1.name.as_str(), &p1.header, &doc)?;
                thing = Some((d.name.to_string(), crate::p2::Thing::Record(d)));
            } else if p1.name.starts_with("or-type ") {
                // declare a record
                let d = crate::OrType::from_p1(p1, &doc)?;
                thing = Some((d.name.to_string(), crate::p2::Thing::OrType(d)));
            } else if p1.name.starts_with("list ") {
                let d = crate::Variable::list_from_p1(p1, &doc)?;
                thing = Some((d.name.to_string(), crate::p2::Thing::Variable(d)));
            } else if p1.name.starts_with("map ") {
                let d = crate::Variable::map_from_p1(p1, &doc)?;
                thing = Some((d.name.to_string(), crate::p2::Thing::Variable(d)));
                // } else if_two_words(p1.name.as_str() {
                //   TODO: <record-name> <variable-name>: foo can be used to create a variable/
                //         Not sure if its a good idea tho.
                // }
            } else if p1.name == "container" {
                instructions.push(ftd::Instruction::ChangeContainer {
                    name: doc
                        .resolve_name_with_instruction(p1.caption()?.as_str(), &instructions)?,
                });
            } else {
                // cloning because https://github.com/rust-lang/rust/issues/59159
                match (doc.get_thing(p1.name.as_str())?).clone() {
                    crate::p2::Thing::Variable(mut v) => {
                        v.update_from_p1(p1, &doc)?;
                        thing = Some((p1.name.to_string(), crate::p2::Thing::Variable(v)));
                    }
                    crate::p2::Thing::Component(_) => {
                        let mut children = vec![];
                        for sub in p1.sub_sections.0.iter() {
                            children.push(ftd::ChildComponent::from_p1(
                                sub.name.as_str(),
                                &sub.header,
                                &sub.caption,
                                &sub.body,
                                &doc,
                                sub.name.as_str(),
                                &Default::default(),
                            )?);
                        }
                        instructions.push(ftd::Instruction::Component {
                            children,
                            parent: ftd::ChildComponent::from_p1(
                                p1.name.as_str(),
                                &p1.header,
                                &p1.caption,
                                &p1.body,
                                &doc,
                                p1.name.as_str(),
                                &Default::default(),
                            )?,
                        })
                    }
                    crate::p2::Thing::Record(mut r) => {
                        r.add_instance(p1, &doc)?;
                        thing = Some((p1.name.to_string(), crate::p2::Thing::Record(r)));
                    }
                    crate::p2::Thing::OrType(_r) => {
                        // do we allow initialization of a record by name? nopes
                        return crate::e(format!("'{}' is an or-type", p1.name.as_str()));
                    }
                    crate::p2::Thing::OrTypeWithVariant { .. } => {
                        // do we allow initialization of a record by name? nopes
                        return crate::e(format!("'{}' is an or-type variant", p1.name.as_str()));
                    }
                };
            }

            if let Some((name, thing)) = thing {
                let name = doc.resolve_name(name.as_str())?;
                self.bag.insert(name, thing);
            }
        }

        if is_main {
            self.p1 = p1;
            self.aliases = aliases;
        }

        Ok(instructions)
    }

    pub(crate) fn new(lib: &'a dyn crate::p2::Library) -> Self {
        Self {
            lib,
            bag: default_bag(),
            p1: Default::default(),
            aliases: Default::default(),
        }
    }
}

pub fn interpret(
    name: &str,
    source: &str,
    lib: &dyn crate::p2::Library,
) -> crate::p1::Result<(
    std::collections::BTreeMap<String, crate::p2::Thing>,
    ftd_rt::Column,
)> {
    let mut interpreter = Interpreter::new(lib);
    let instructions = interpreter.interpret(name, source)?;
    let mut rt = ftd::RT::from(name, interpreter.aliases, interpreter.bag, instructions);
    let main = rt.render()?;
    Ok((rt.bag, main))
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum Thing {
    Component(ftd::Component),
    Variable(ftd::Variable),
    Record(ftd::p2::Record),
    OrType(ftd::OrType),
    OrTypeWithVariant { e: ftd::OrType, variant: String },
}

pub fn default_bag() -> std::collections::BTreeMap<String, crate::p2::Thing> {
    std::array::IntoIter::new([
        (
            "ftd#row".to_string(),
            crate::p2::Thing::Component(ftd::p2::element::row_function()),
        ),
        (
            "ftd#column".to_string(),
            crate::p2::Thing::Component(ftd::p2::element::column_function()),
        ),
        (
            "ftd#text".to_string(),
            crate::p2::Thing::Component(ftd::p2::element::text_function()),
        ),
        (
            "ftd#image".to_string(),
            crate::p2::Thing::Component(ftd::p2::element::image_function()),
        ),
        (
            "ftd#iframe".to_string(),
            crate::p2::Thing::Component(ftd::p2::element::iframe_function()),
        ),
        (
            "ftd#integer".to_string(),
            crate::p2::Thing::Component(ftd::p2::element::integer_function()),
        ),
        (
            "ftd#decimal".to_string(),
            crate::p2::Thing::Component(ftd::p2::element::decimal_function()),
        ),
        (
            "ftd#boolean".to_string(),
            crate::p2::Thing::Component(ftd::p2::element::boolean_function()),
        ),
        (
            "ftd#input".to_string(),
            crate::p2::Thing::Component(ftd::p2::element::input_function()),
        ),
        (
            "ftd#null".to_string(),
            crate::p2::Thing::Component(ftd::p2::element::null()),
        ),
    ])
    .collect()
}

pub fn default_aliases() -> std::collections::BTreeMap<String, String> {
    std::array::IntoIter::new([("ftd".to_string(), "ftd".to_string())]).collect()
}

pub fn default_column() -> ftd_rt::Column {
    ftd_rt::Column {
        common: ftd_rt::Common {
            width: Some(ftd_rt::Length::Fill),
            height: Some(ftd_rt::Length::Fill),
            ..Default::default()
        },
        container: ftd_rt::Container {
            align: ftd_rt::Align::Center,
            wrap: true,
            ..Default::default()
        },
    }
}

#[cfg(test)]
mod test {
    use crate::test::*;

    #[test]
    fn basic() {
        let mut bag = super::default_bag();
        bag.insert(
            "foo/bar#foo".to_string(),
            crate::p2::Thing::Component(crate::Component {
                root: "ftd.text".to_string(),
                full_name: s("foo/bar#foo"),
                properties: std::array::IntoIter::new([(
                    s("text"),
                    crate::component::Property {
                        default: Some(crate::PropertyValue::Value {
                            value: crate::Value::String {
                                text: s("hello"),
                                source: crate::TextSource::Header,
                            },
                        }),
                        conditions: vec![],
                    },
                )])
                .collect(),
                ..Default::default()
            }),
        );
        bag.insert(
            "foo/bar#x".to_string(),
            crate::p2::Thing::Variable(crate::Variable {
                name: "x".to_string(),
                value: crate::Value::Integer { value: 10 },
            }),
        );

        p!(
            "
            -- component foo:
            component: ftd.text
            text: hello

            -- var x: 10
            ",
            (bag, super::default_column()),
        );
    }

    #[test]
    fn conditional_attribute() {
        let mut bag = super::default_bag();
        bag.insert(
            "foo/bar#foo".to_string(),
            crate::p2::Thing::Component(crate::Component {
                full_name: s("foo/bar#foo"),
                root: "ftd.text".to_string(),
                arguments: std::array::IntoIter::new([(
                    s("name"),
                    crate::p2::Kind::String {
                        caption: true,
                        body: false,
                    },
                )])
                .collect(),
                properties: std::array::IntoIter::new([
                    (
                        s("color"),
                        crate::component::Property {
                            default: Some(crate::PropertyValue::Value {
                                value: crate::Value::String {
                                    text: "white".to_string(),
                                    source: crate::TextSource::Header,
                                },
                            }),
                            conditions: vec![
                                (
                                    crate::p2::Boolean::Equal {
                                        left: crate::PropertyValue::Reference {
                                            name: "foo/bar#present".to_string(),
                                            kind: crate::p2::Kind::Boolean,
                                        },
                                        right: crate::PropertyValue::Value {
                                            value: crate::Value::Boolean { value: true },
                                        },
                                    },
                                    crate::PropertyValue::Value {
                                        value: crate::Value::String {
                                            text: "green".to_string(),
                                            source: crate::TextSource::Header,
                                        },
                                    },
                                ),
                                (
                                    crate::p2::Boolean::Equal {
                                        left: crate::PropertyValue::Reference {
                                            name: "foo/bar#present".to_string(),
                                            kind: crate::p2::Kind::Boolean,
                                        },
                                        right: crate::PropertyValue::Value {
                                            value: crate::Value::Boolean { value: false },
                                        },
                                    },
                                    crate::PropertyValue::Value {
                                        value: crate::Value::String {
                                            text: "red".to_string(),
                                            source: crate::TextSource::Header,
                                        },
                                    },
                                ),
                            ],
                        },
                    ),
                    (
                        s("text"),
                        crate::component::Property {
                            default: Some(crate::PropertyValue::Argument {
                                name: "name".to_string(),
                                kind: crate::p2::Kind::String {
                                    caption: true,
                                    body: true,
                                },
                            }),
                            conditions: vec![],
                        },
                    ),
                ])
                .collect(),
                invocations: vec![std::array::IntoIter::new([(
                    s("name"),
                    crate::Value::String {
                        text: s("hello"),
                        source: crate::TextSource::Caption,
                    },
                )])
                .collect()],
                ..Default::default()
            }),
        );

        bag.insert(
            "foo/bar#present".to_string(),
            crate::p2::Thing::Variable(crate::Variable {
                name: "present".to_string(),
                value: crate::Value::Boolean { value: false },
            }),
        );

        let mut main = super::default_column();
        main.container
            .children
            .push(ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("hello"),
                line: true,
                common: ftd_rt::Common {
                    color: Some(ftd_rt::Color {
                        r: 255,
                        g: 0,
                        b: 0,
                        alpha: 1.0,
                    }),
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (g_bag, g_col) = crate::p2::interpreter::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- var present: false

                -- component foo:
                $name: caption
                component: ftd.text
                color: white
                color if present: green
                color if not present: red
                text: ref $name

                -- foo: hello
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");
        pretty_assertions::assert_eq!(g_bag, bag);
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn creating_a_tree() {
        let mut bag = super::default_bag();

        bag.insert(
            "foo/bar#ft_toc".to_string(),
            crate::p2::Thing::Component(crate::Component {
                root: "ftd.column".to_string(),
                full_name: "foo/bar#ft_toc".to_string(),
                arguments: Default::default(),
                properties: Default::default(),
                instructions: vec![
                    crate::component::Instruction::ChildComponent {
                        child: crate::component::ChildComponent {
                            root: "foo/bar#table-of-content".to_string(),
                            condition: None,
                            properties: std::array::IntoIter::new([(
                                s("id"),
                                crate::component::Property {
                                    default: Some(crate::PropertyValue::Value {
                                        value: crate::variable::Value::String {
                                            text: "toc_main".to_string(),
                                            source: crate::TextSource::Header,
                                        },
                                    }),
                                    conditions: vec![],
                                },
                            )])
                            .collect(),
                        },
                    },
                    crate::component::Instruction::ChildComponent {
                        child: crate::component::ChildComponent {
                            root: "foo/bar#parent".to_string(),
                            condition: None,
                            properties: std::array::IntoIter::new([
                                (
                                    s("active"),
                                    crate::component::Property {
                                        default: Some(crate::PropertyValue::Value {
                                            value: crate::variable::Value::Boolean { value: true },
                                        }),
                                        conditions: vec![],
                                    },
                                ),
                                (
                                    s("id"),
                                    crate::component::Property {
                                        default: Some(crate::PropertyValue::Value {
                                            value: crate::variable::Value::String {
                                                text: "/welcome/".to_string(),
                                                source: crate::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                    },
                                ),
                                (
                                    s("name"),
                                    crate::component::Property {
                                        default: Some(crate::PropertyValue::Value {
                                            value: crate::variable::Value::String {
                                                text: "5PM Tasks".to_string(),
                                                source: crate::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                    },
                                ),
                            ])
                            .collect(),
                        },
                    },
                    crate::component::Instruction::ChildComponent {
                        child: crate::component::ChildComponent {
                            root: "foo/bar#parent".to_string(),
                            condition: None,
                            properties: std::array::IntoIter::new([
                                (
                                    s("id"),
                                    crate::component::Property {
                                        default: Some(crate::PropertyValue::Value {
                                            value: crate::variable::Value::String {
                                                text: "/Building/".to_string(),
                                                source: crate::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                    },
                                ),
                                (
                                    s("name"),
                                    crate::component::Property {
                                        default: Some(crate::PropertyValue::Value {
                                            value: crate::variable::Value::String {
                                                text: "Log".to_string(),
                                                source: crate::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                    },
                                ),
                            ])
                            .collect(),
                        },
                    },
                    crate::component::Instruction::ChildComponent {
                        child: crate::component::ChildComponent {
                            root: "foo/bar#parent".to_string(),
                            condition: None,
                            properties: std::array::IntoIter::new([
                                (
                                    s("id"),
                                    crate::component::Property {
                                        default: Some(crate::PropertyValue::Value {
                                            value: crate::variable::Value::String {
                                                text: "/ChildBuilding/".to_string(),
                                                source: crate::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                    },
                                ),
                                (
                                    s("name"),
                                    crate::component::Property {
                                        default: Some(crate::PropertyValue::Value {
                                            value: crate::variable::Value::String {
                                                text: "ChildLog".to_string(),
                                                source: crate::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                    },
                                ),
                            ])
                            .collect(),
                        },
                    },
                    crate::component::Instruction::ChangeContainer {
                        name: "/welcome/".to_string(),
                    },
                    crate::component::Instruction::ChildComponent {
                        child: crate::component::ChildComponent {
                            root: "foo/bar#parent".to_string(),
                            condition: None,
                            properties: std::array::IntoIter::new([
                                (
                                    s("id"),
                                    crate::component::Property {
                                        default: Some(crate::PropertyValue::Value {
                                            value: crate::variable::Value::String {
                                                text: "/Building2/".to_string(),
                                                source: crate::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                    },
                                ),
                                (
                                    s("name"),
                                    crate::component::Property {
                                        default: Some(crate::PropertyValue::Value {
                                            value: crate::variable::Value::String {
                                                text: "Log2".to_string(),
                                                source: crate::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                    },
                                ),
                            ])
                            .collect(),
                        },
                    },
                ],
                kernel: false,
                invocations: vec![std::collections::BTreeMap::new()],
            }),
        );

        bag.insert(
            "foo/bar#parent".to_string(),
            crate::p2::Thing::Component(crate::Component {
                root: "ftd.column".to_string(),
                full_name: "foo/bar#parent".to_string(),
                arguments: std::array::IntoIter::new([
                    (
                        s("active"),
                        crate::p2::Kind::Optional {
                            kind: Box::new(crate::p2::Kind::Boolean),
                        },
                    ),
                    (
                        s("id"),
                        crate::p2::Kind::String {
                            caption: false,
                            body: false,
                        },
                    ),
                    (
                        s("name"),
                        crate::p2::Kind::String {
                            caption: true,
                            body: false,
                        },
                    ),
                ])
                .collect(),
                properties: std::array::IntoIter::new([
                    (
                        s("id"),
                        crate::component::Property {
                            default: Some(crate::PropertyValue::Argument {
                                name: "id".to_string(),
                                kind: crate::p2::Kind::Optional {
                                    kind: Box::new(crate::p2::Kind::String {
                                        caption: false,
                                        body: false,
                                    }),
                                },
                            }),
                            conditions: vec![],
                        },
                    ),
                    (
                        s("open"),
                        crate::component::Property {
                            default: Some(crate::PropertyValue::Value {
                                value: crate::variable::Value::String {
                                    text: "true".to_string(),
                                    source: crate::TextSource::Header,
                                },
                            }),
                            conditions: vec![],
                        },
                    ),
                    (
                        s("width"),
                        crate::component::Property {
                            default: Some(crate::PropertyValue::Value {
                                value: crate::variable::Value::String {
                                    text: "fill".to_string(),
                                    source: crate::TextSource::Header,
                                },
                            }),
                            conditions: vec![],
                        },
                    ),
                ])
                .collect(),
                instructions: vec![
                    crate::component::Instruction::ChildComponent {
                        child: crate::component::ChildComponent {
                            root: "ftd#text".to_string(),
                            condition: Some(ftd::p2::Boolean::IsNotNull {
                                value: ftd::PropertyValue::Argument {
                                    name: "active".to_string(),
                                    kind: crate::p2::Kind::Optional {
                                        kind: Box::new(crate::p2::Kind::Boolean),
                                    },
                                },
                            }),
                            properties: std::array::IntoIter::new([
                                (
                                    s("color"),
                                    crate::component::Property {
                                        default: Some(crate::PropertyValue::Value {
                                            value: crate::variable::Value::String {
                                                text: "white".to_string(),
                                                source: crate::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                    },
                                ),
                                (
                                    s("size"),
                                    crate::component::Property {
                                        default: Some(crate::PropertyValue::Value {
                                            value: crate::variable::Value::Integer { value: 14 },
                                        }),
                                        conditions: vec![],
                                    },
                                ),
                                (
                                    s("text"),
                                    crate::component::Property {
                                        default: Some(crate::PropertyValue::Argument {
                                            name: "name".to_string(),
                                            kind: crate::p2::Kind::String {
                                                caption: true,
                                                body: true,
                                            },
                                        }),
                                        conditions: vec![],
                                    },
                                ),
                            ])
                            .collect(),
                        },
                    },
                    crate::component::Instruction::ChildComponent {
                        child: crate::component::ChildComponent {
                            root: "ftd#text".to_string(),
                            condition: Some(ftd::p2::Boolean::IsNull {
                                value: ftd::PropertyValue::Argument {
                                    name: "active".to_string(),
                                    kind: crate::p2::Kind::Optional {
                                        kind: Box::new(crate::p2::Kind::Boolean),
                                    },
                                },
                            }),
                            properties: std::array::IntoIter::new([
                                (
                                    s("color"),
                                    crate::component::Property {
                                        default: Some(crate::PropertyValue::Value {
                                            value: crate::variable::Value::String {
                                                text: "#4D4D4D".to_string(),
                                                source: crate::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                    },
                                ),
                                (
                                    s("size"),
                                    crate::component::Property {
                                        default: Some(crate::PropertyValue::Value {
                                            value: crate::variable::Value::Integer { value: 14 },
                                        }),
                                        conditions: vec![],
                                    },
                                ),
                                (
                                    s("text"),
                                    crate::component::Property {
                                        default: Some(crate::PropertyValue::Argument {
                                            name: "name".to_string(),
                                            kind: crate::p2::Kind::String {
                                                caption: true,
                                                body: true,
                                            },
                                        }),
                                        conditions: vec![],
                                    },
                                ),
                            ])
                            .collect(),
                        },
                    },
                ],
                kernel: false,
                invocations: vec![
                    std::array::IntoIter::new([
                        (s("active"), crate::Value::Boolean { value: true }),
                        (
                            s("id"),
                            crate::Value::String {
                                text: "/welcome/".to_string(),
                                source: crate::TextSource::Header,
                            },
                        ),
                        (
                            s("name"),
                            crate::Value::String {
                                text: "5PM Tasks".to_string(),
                                source: crate::TextSource::Header,
                            },
                        ),
                    ])
                    .collect(),
                    std::array::IntoIter::new([
                        (
                            s("id"),
                            crate::Value::String {
                                text: "/Building/".to_string(),
                                source: crate::TextSource::Header,
                            },
                        ),
                        (
                            s("name"),
                            crate::Value::String {
                                text: "Log".to_string(),
                                source: crate::TextSource::Header,
                            },
                        ),
                    ])
                    .collect(),
                    std::array::IntoIter::new([
                        (
                            s("id"),
                            crate::Value::String {
                                text: "/ChildBuilding/".to_string(),
                                source: crate::TextSource::Header,
                            },
                        ),
                        (
                            s("name"),
                            crate::Value::String {
                                text: "ChildLog".to_string(),
                                source: crate::TextSource::Header,
                            },
                        ),
                    ])
                    .collect(),
                    std::array::IntoIter::new([
                        (
                            s("id"),
                            crate::Value::String {
                                text: "/Building2/".to_string(),
                                source: crate::TextSource::Header,
                            },
                        ),
                        (
                            s("name"),
                            crate::Value::String {
                                text: "Log2".to_string(),
                                source: crate::TextSource::Header,
                            },
                        ),
                    ])
                    .collect(),
                ],
            }),
        );

        bag.insert(
            "foo/bar#table-of-content".to_string(),
            crate::p2::Thing::Component(crate::Component {
                root: "ftd.column".to_string(),
                full_name: "foo/bar#table-of-content".to_string(),
                arguments: std::array::IntoIter::new([(
                    s("id"),
                    crate::p2::Kind::String {
                        caption: false,
                        body: false,
                    },
                )])
                .collect(),
                properties: std::array::IntoIter::new([
                    (
                        s("height"),
                        crate::component::Property {
                            default: Some(crate::PropertyValue::Value {
                                value: crate::variable::Value::String {
                                    text: "fill".to_string(),
                                    source: crate::TextSource::Header,
                                },
                            }),
                            conditions: vec![],
                        },
                    ),
                    (
                        s("id"),
                        crate::component::Property {
                            default: Some(crate::PropertyValue::Argument {
                                name: "id".to_string(),
                                kind: crate::p2::Kind::Optional {
                                    kind: Box::new(crate::p2::Kind::String {
                                        caption: false,
                                        body: false,
                                    }),
                                },
                            }),
                            conditions: vec![],
                        },
                    ),
                    (
                        s("width"),
                        crate::component::Property {
                            default: Some(crate::PropertyValue::Value {
                                value: crate::variable::Value::String {
                                    text: "300".to_string(),
                                    source: crate::TextSource::Header,
                                },
                            }),
                            conditions: vec![],
                        },
                    ),
                ])
                .collect(),
                instructions: vec![],
                kernel: false,
                invocations: vec![std::array::IntoIter::new([(
                    s("id"),
                    crate::Value::String {
                        text: "toc_main".to_string(),
                        source: crate::TextSource::Header,
                    },
                )])
                .collect()],
            }),
        );

        bag.insert(
            "foo/bar#toc-heading".to_string(),
            crate::p2::Thing::Component(crate::Component {
                root: "ftd.text".to_string(),
                full_name: "foo/bar#toc-heading".to_string(),
                arguments: std::array::IntoIter::new([(
                    s("text"),
                    crate::p2::Kind::String {
                        caption: true,
                        body: false,
                    },
                )])
                .collect(),
                properties: std::array::IntoIter::new([
                    (
                        s("size"),
                        crate::component::Property {
                            default: Some(crate::PropertyValue::Value {
                                value: crate::variable::Value::Integer { value: 16 },
                            }),
                            conditions: vec![],
                        },
                    ),
                    (
                        s("text"),
                        crate::component::Property {
                            default: Some(crate::PropertyValue::Argument {
                                name: "text".to_string(),
                                kind: crate::p2::Kind::String {
                                    caption: true,
                                    body: true,
                                },
                            }),
                            conditions: vec![],
                        },
                    ),
                ])
                .collect(),
                instructions: vec![],
                kernel: false,
                invocations: vec![],
            }),
        );

        let mut main = super::default_column();
        main.container
            .children
            .push(ftd_rt::Element::Column(ftd_rt::Column {
                container: ftd_rt::Container {
                    children: vec![ftd_rt::Element::Column(ftd_rt::Column {
                        container: ftd_rt::Container {
                            children: vec![ftd_rt::Element::Column(ftd_rt::Column {
                                container: ftd_rt::Container {
                                    children: vec![
                                        ftd_rt::Element::Text(ftd_rt::Text {
                                            text: ftd::markdown_line("5PM Tasks"),
                                            line: true,
                                            common: ftd_rt::Common {
                                                color: Some(ftd_rt::Color {
                                                    r: 255,
                                                    g: 255,
                                                    b: 255,
                                                    alpha: 1.0,
                                                }),
                                                ..Default::default()
                                            },
                                            size: Some(14),
                                            ..Default::default()
                                        }),
                                        ftd_rt::Element::Null,
                                        ftd_rt::Element::Column(ftd_rt::Column {
                                            container: ftd_rt::Container {
                                                children: vec![
                                                    ftd_rt::Element::Null,
                                                    ftd_rt::Element::Text(ftd_rt::Text {
                                                        text: ftd::markdown_line("Log"),
                                                        line: true,
                                                        common: ftd_rt::Common {
                                                            color: Some(ftd_rt::Color {
                                                                r: 77,
                                                                g: 77,
                                                                b: 77,
                                                                alpha: 1.0,
                                                            }),
                                                            ..Default::default()
                                                        },
                                                        size: Some(14),
                                                        ..Default::default()
                                                    }),
                                                    ftd_rt::Element::Column(ftd_rt::Column {
                                                        container: ftd_rt::Container {
                                                            children: vec![
                                                                ftd_rt::Element::Null,
                                                                ftd_rt::Element::Text(
                                                                    ftd_rt::Text {
                                                                        text: ftd::markdown_line(
                                                                            "ChildLog",
                                                                        ),
                                                                        line: true,
                                                                        common: ftd_rt::Common {
                                                                            color: Some(
                                                                                ftd_rt::Color {
                                                                                    r: 77,
                                                                                    g: 77,
                                                                                    b: 77,
                                                                                    alpha: 1.0,
                                                                                },
                                                                            ),
                                                                            ..Default::default()
                                                                        },
                                                                        size: Some(14),
                                                                        ..Default::default()
                                                                    },
                                                                ),
                                                            ],
                                                            open: (Some(true), None),
                                                            spacing: None,
                                                            align: Default::default(),
                                                            wrap: false,
                                                        },
                                                        common: ftd_rt::Common {
                                                            id: Some(s("/ChildBuilding/")),
                                                            width: Some(ftd_rt::Length::Fill),
                                                            ..Default::default()
                                                        },
                                                    }),
                                                ],
                                                open: (Some(true), None),
                                                spacing: None,
                                                align: Default::default(),
                                                wrap: false,
                                            },
                                            common: ftd_rt::Common {
                                                id: Some(s("/Building/")),
                                                width: Some(ftd_rt::Length::Fill),
                                                ..Default::default()
                                            },
                                        }),
                                        ftd_rt::Element::Column(ftd_rt::Column {
                                            container: ftd_rt::Container {
                                                children: vec![
                                                    ftd_rt::Element::Null,
                                                    ftd_rt::Element::Text(ftd_rt::Text {
                                                        text: ftd::markdown_line("Log2"),
                                                        line: true,
                                                        common: ftd_rt::Common {
                                                            color: Some(ftd_rt::Color {
                                                                r: 77,
                                                                g: 77,
                                                                b: 77,
                                                                alpha: 1.0,
                                                            }),
                                                            ..Default::default()
                                                        },
                                                        size: Some(14),
                                                        ..Default::default()
                                                    }),
                                                ],
                                                open: (Some(true), None),
                                                spacing: None,
                                                align: Default::default(),
                                                wrap: false,
                                            },
                                            common: ftd_rt::Common {
                                                id: Some(s("/Building2/")),
                                                width: Some(ftd_rt::Length::Fill),
                                                ..Default::default()
                                            },
                                        }),
                                    ],
                                    open: (Some(true), None),
                                    spacing: None,
                                    align: Default::default(),
                                    wrap: false,
                                },
                                common: ftd_rt::Common {
                                    id: Some(s("/welcome/")),
                                    width: Some(ftd_rt::Length::Fill),
                                    ..Default::default()
                                },
                            })],
                            ..Default::default()
                        },
                        common: ftd_rt::Common {
                            id: Some(s("toc_main")),
                            height: Some(ftd_rt::Length::Fill),
                            width: Some(ftd_rt::Length::Px { value: 300 }),
                            ..Default::default()
                        },
                    })],
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (g_bag, g_col) = crate::p2::interpreter::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- component toc-heading:
                component: ftd.text
                $text: caption
                text: ref $text
                size: 16


                -- component table-of-content:
                component: ftd.column
                $id: string
                id: ref $id
                width: 300
                height: fill


                -- component parent:
                component: ftd.column
                $id: string
                $name: caption
                $active: optional boolean
                id: ref $id
                width: fill
                open: true

                --- ftd.text:
                if: $active is not null
                text: ref $name
                size: 14
                color: white

                --- ftd.text:
                if: $active is null
                text: ref $name
                size: 14
                color: #4D4D4D


                -- component ft_toc:
                component: ftd.column

                --- table-of-content:
                id: toc_main

                --- parent:
                id: /welcome/
                name: 5PM Tasks
                active: true

                --- parent:
                id: /Building/
                name: Log

                --- parent:
                id: /ChildBuilding/
                name: ChildLog

                --- container: /welcome/

                --- parent:
                id: /Building2/
                name: Log2


                -- ft_toc:
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");
        pretty_assertions::assert_eq!(g_bag, bag);
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn creating_a_tree_using_import() {
        let mut bag = super::default_bag();

        bag.insert(
            "creating-a-tree#ft_toc".to_string(),
            crate::p2::Thing::Component(crate::Component {
                root: "ftd.column".to_string(),
                full_name: "creating-a-tree#ft_toc".to_string(),
                arguments: Default::default(),
                properties: Default::default(),
                instructions: vec![
                    crate::component::Instruction::ChildComponent {
                        child: crate::component::ChildComponent {
                            root: "creating-a-tree#table-of-content".to_string(),
                            condition: None,
                            properties: std::array::IntoIter::new([(
                                s("id"),
                                crate::component::Property {
                                    default: Some(crate::PropertyValue::Value {
                                        value: crate::variable::Value::String {
                                            text: "toc_main".to_string(),
                                            source: crate::TextSource::Header,
                                        },
                                    }),
                                    conditions: vec![],
                                },
                            )])
                            .collect(),
                        },
                    },
                    crate::component::Instruction::ChildComponent {
                        child: crate::component::ChildComponent {
                            root: "creating-a-tree#parent".to_string(),
                            condition: None,
                            properties: std::array::IntoIter::new([
                                (
                                    s("active"),
                                    crate::component::Property {
                                        default: Some(crate::PropertyValue::Value {
                                            value: crate::variable::Value::Boolean { value: true },
                                        }),
                                        conditions: vec![],
                                    },
                                ),
                                (
                                    s("id"),
                                    crate::component::Property {
                                        default: Some(crate::PropertyValue::Value {
                                            value: crate::variable::Value::String {
                                                text: "/welcome/".to_string(),
                                                source: crate::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                    },
                                ),
                                (
                                    s("name"),
                                    crate::component::Property {
                                        default: Some(crate::PropertyValue::Value {
                                            value: crate::variable::Value::String {
                                                text: "5PM Tasks".to_string(),
                                                source: crate::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                    },
                                ),
                            ])
                            .collect(),
                        },
                    },
                    crate::component::Instruction::ChildComponent {
                        child: crate::component::ChildComponent {
                            root: "creating-a-tree#parent".to_string(),
                            condition: None,
                            properties: std::array::IntoIter::new([
                                (
                                    s("id"),
                                    crate::component::Property {
                                        default: Some(crate::PropertyValue::Value {
                                            value: crate::variable::Value::String {
                                                text: "/Building/".to_string(),
                                                source: crate::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                    },
                                ),
                                (
                                    s("name"),
                                    crate::component::Property {
                                        default: Some(crate::PropertyValue::Value {
                                            value: crate::variable::Value::String {
                                                text: "Log".to_string(),
                                                source: crate::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                    },
                                ),
                            ])
                            .collect(),
                        },
                    },
                    crate::component::Instruction::ChildComponent {
                        child: crate::component::ChildComponent {
                            root: "creating-a-tree#parent".to_string(),
                            condition: None,
                            properties: std::array::IntoIter::new([
                                (
                                    s("id"),
                                    crate::component::Property {
                                        default: Some(crate::PropertyValue::Value {
                                            value: crate::variable::Value::String {
                                                text: "/ChildBuilding/".to_string(),
                                                source: crate::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                    },
                                ),
                                (
                                    s("name"),
                                    crate::component::Property {
                                        default: Some(crate::PropertyValue::Value {
                                            value: crate::variable::Value::String {
                                                text: "ChildLog".to_string(),
                                                source: crate::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                    },
                                ),
                            ])
                            .collect(),
                        },
                    },
                    crate::component::Instruction::ChangeContainer {
                        name: "/welcome/".to_string(),
                    },
                    crate::component::Instruction::ChildComponent {
                        child: crate::component::ChildComponent {
                            root: "creating-a-tree#parent".to_string(),
                            condition: None,
                            properties: std::array::IntoIter::new([
                                (
                                    s("id"),
                                    crate::component::Property {
                                        default: Some(crate::PropertyValue::Value {
                                            value: crate::variable::Value::String {
                                                text: "/Building2/".to_string(),
                                                source: crate::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                    },
                                ),
                                (
                                    s("name"),
                                    crate::component::Property {
                                        default: Some(crate::PropertyValue::Value {
                                            value: crate::variable::Value::String {
                                                text: "Log2".to_string(),
                                                source: crate::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                    },
                                ),
                            ])
                            .collect(),
                        },
                    },
                ],
                kernel: false,
                invocations: vec![std::collections::BTreeMap::new()],
            }),
        );

        bag.insert(
            "creating-a-tree#parent".to_string(),
            crate::p2::Thing::Component(crate::Component {
                root: "ftd.column".to_string(),
                full_name: "creating-a-tree#parent".to_string(),
                arguments: std::array::IntoIter::new([
                    (
                        s("active"),
                        crate::p2::Kind::Optional {
                            kind: Box::new(crate::p2::Kind::Boolean),
                        },
                    ),
                    (
                        s("id"),
                        crate::p2::Kind::String {
                            caption: false,
                            body: false,
                        },
                    ),
                    (
                        s("name"),
                        crate::p2::Kind::String {
                            caption: true,
                            body: false,
                        },
                    ),
                ])
                .collect(),
                properties: std::array::IntoIter::new([
                    (
                        s("id"),
                        crate::component::Property {
                            default: Some(crate::PropertyValue::Argument {
                                name: "id".to_string(),
                                kind: crate::p2::Kind::Optional {
                                    kind: Box::new(crate::p2::Kind::String {
                                        caption: false,
                                        body: false,
                                    }),
                                },
                            }),
                            conditions: vec![],
                        },
                    ),
                    (
                        s("open"),
                        crate::component::Property {
                            default: Some(crate::PropertyValue::Value {
                                value: crate::variable::Value::String {
                                    text: "true".to_string(),
                                    source: crate::TextSource::Header,
                                },
                            }),
                            conditions: vec![],
                        },
                    ),
                    (
                        s("width"),
                        crate::component::Property {
                            default: Some(crate::PropertyValue::Value {
                                value: crate::variable::Value::String {
                                    text: "fill".to_string(),
                                    source: crate::TextSource::Header,
                                },
                            }),
                            conditions: vec![],
                        },
                    ),
                ])
                .collect(),
                instructions: vec![
                    crate::component::Instruction::ChildComponent {
                        child: crate::component::ChildComponent {
                            root: "ftd#text".to_string(),
                            condition: Some(ftd::p2::Boolean::IsNotNull {
                                value: ftd::PropertyValue::Argument {
                                    name: "active".to_string(),
                                    kind: crate::p2::Kind::Optional {
                                        kind: Box::new(crate::p2::Kind::Boolean),
                                    },
                                },
                            }),
                            properties: std::array::IntoIter::new([
                                (
                                    s("color"),
                                    crate::component::Property {
                                        default: Some(crate::PropertyValue::Value {
                                            value: crate::variable::Value::String {
                                                text: "white".to_string(),
                                                source: crate::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                    },
                                ),
                                (
                                    s("size"),
                                    crate::component::Property {
                                        default: Some(crate::PropertyValue::Value {
                                            value: crate::variable::Value::Integer { value: 14 },
                                        }),
                                        conditions: vec![],
                                    },
                                ),
                                (
                                    s("text"),
                                    crate::component::Property {
                                        default: Some(crate::PropertyValue::Argument {
                                            name: "name".to_string(),
                                            kind: crate::p2::Kind::String {
                                                caption: true,
                                                body: true,
                                            },
                                        }),
                                        conditions: vec![],
                                    },
                                ),
                            ])
                            .collect(),
                        },
                    },
                    crate::component::Instruction::ChildComponent {
                        child: crate::component::ChildComponent {
                            root: "ftd#text".to_string(),
                            condition: Some(ftd::p2::Boolean::IsNull {
                                value: ftd::PropertyValue::Argument {
                                    name: "active".to_string(),
                                    kind: crate::p2::Kind::Optional {
                                        kind: Box::new(crate::p2::Kind::Boolean),
                                    },
                                },
                            }),
                            properties: std::array::IntoIter::new([
                                (
                                    s("color"),
                                    crate::component::Property {
                                        default: Some(crate::PropertyValue::Value {
                                            value: crate::variable::Value::String {
                                                text: "#4D4D4D".to_string(),
                                                source: crate::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                    },
                                ),
                                (
                                    s("size"),
                                    crate::component::Property {
                                        default: Some(crate::PropertyValue::Value {
                                            value: crate::variable::Value::Integer { value: 14 },
                                        }),
                                        conditions: vec![],
                                    },
                                ),
                                (
                                    s("text"),
                                    crate::component::Property {
                                        default: Some(crate::PropertyValue::Argument {
                                            name: "name".to_string(),
                                            kind: crate::p2::Kind::String {
                                                caption: true,
                                                body: true,
                                            },
                                        }),
                                        conditions: vec![],
                                    },
                                ),
                            ])
                            .collect(),
                        },
                    },
                ],
                kernel: false,
                invocations: vec![
                    std::array::IntoIter::new([
                        (s("active"), crate::Value::Boolean { value: true }),
                        (
                            s("id"),
                            crate::Value::String {
                                text: "/welcome/".to_string(),
                                source: crate::TextSource::Header,
                            },
                        ),
                        (
                            s("name"),
                            crate::Value::String {
                                text: "5PM Tasks".to_string(),
                                source: crate::TextSource::Header,
                            },
                        ),
                    ])
                    .collect(),
                    std::array::IntoIter::new([
                        (
                            s("id"),
                            crate::Value::String {
                                text: "/Building/".to_string(),
                                source: crate::TextSource::Header,
                            },
                        ),
                        (
                            s("name"),
                            crate::Value::String {
                                text: "Log".to_string(),
                                source: crate::TextSource::Header,
                            },
                        ),
                    ])
                    .collect(),
                    std::array::IntoIter::new([
                        (
                            s("id"),
                            crate::Value::String {
                                text: "/ChildBuilding/".to_string(),
                                source: crate::TextSource::Header,
                            },
                        ),
                        (
                            s("name"),
                            crate::Value::String {
                                text: "ChildLog".to_string(),
                                source: crate::TextSource::Header,
                            },
                        ),
                    ])
                    .collect(),
                    std::array::IntoIter::new([
                        (
                            s("id"),
                            crate::Value::String {
                                text: "/Building2/".to_string(),
                                source: crate::TextSource::Header,
                            },
                        ),
                        (
                            s("name"),
                            crate::Value::String {
                                text: "Log2".to_string(),
                                source: crate::TextSource::Header,
                            },
                        ),
                    ])
                    .collect(),
                ],
            }),
        );

        bag.insert(
            "creating-a-tree#table-of-content".to_string(),
            crate::p2::Thing::Component(crate::Component {
                root: "ftd.column".to_string(),
                full_name: "creating-a-tree#table-of-content".to_string(),
                arguments: std::array::IntoIter::new([(
                    s("id"),
                    crate::p2::Kind::String {
                        caption: false,
                        body: false,
                    },
                )])
                .collect(),
                properties: std::array::IntoIter::new([
                    (
                        s("height"),
                        crate::component::Property {
                            default: Some(crate::PropertyValue::Value {
                                value: crate::variable::Value::String {
                                    text: "fill".to_string(),
                                    source: crate::TextSource::Header,
                                },
                            }),
                            conditions: vec![],
                        },
                    ),
                    (
                        s("id"),
                        crate::component::Property {
                            default: Some(crate::PropertyValue::Argument {
                                name: "id".to_string(),
                                kind: crate::p2::Kind::Optional {
                                    kind: Box::new(crate::p2::Kind::String {
                                        caption: false,
                                        body: false,
                                    }),
                                },
                            }),
                            conditions: vec![],
                        },
                    ),
                    (
                        s("width"),
                        crate::component::Property {
                            default: Some(crate::PropertyValue::Value {
                                value: crate::variable::Value::String {
                                    text: "300".to_string(),
                                    source: crate::TextSource::Header,
                                },
                            }),
                            conditions: vec![],
                        },
                    ),
                ])
                .collect(),
                instructions: vec![],
                kernel: false,
                invocations: vec![std::array::IntoIter::new([(
                    s("id"),
                    crate::Value::String {
                        text: "toc_main".to_string(),
                        source: crate::TextSource::Header,
                    },
                )])
                .collect()],
            }),
        );

        bag.insert(
            "creating-a-tree#toc-heading".to_string(),
            crate::p2::Thing::Component(crate::Component {
                root: "ftd.text".to_string(),
                full_name: "creating-a-tree#toc-heading".to_string(),
                arguments: std::array::IntoIter::new([(
                    s("text"),
                    crate::p2::Kind::String {
                        caption: true,
                        body: false,
                    },
                )])
                .collect(),
                properties: std::array::IntoIter::new([
                    (
                        s("size"),
                        crate::component::Property {
                            default: Some(crate::PropertyValue::Value {
                                value: crate::variable::Value::Integer { value: 16 },
                            }),
                            conditions: vec![],
                        },
                    ),
                    (
                        s("text"),
                        crate::component::Property {
                            default: Some(crate::PropertyValue::Argument {
                                name: "text".to_string(),
                                kind: crate::p2::Kind::String {
                                    caption: true,
                                    body: true,
                                },
                            }),
                            conditions: vec![],
                        },
                    ),
                ])
                .collect(),
                instructions: vec![],
                kernel: false,
                invocations: vec![],
            }),
        );

        let mut main = super::default_column();
        main.container
            .children
            .push(ftd_rt::Element::Column(ftd_rt::Column {
                container: ftd_rt::Container {
                    children: vec![ftd_rt::Element::Column(ftd_rt::Column {
                        container: ftd_rt::Container {
                            children: vec![ftd_rt::Element::Column(ftd_rt::Column {
                                container: ftd_rt::Container {
                                    children: vec![
                                        ftd_rt::Element::Text(ftd_rt::Text {
                                            text: ftd::markdown_line("5PM Tasks"),
                                            line: true,
                                            common: ftd_rt::Common {
                                                color: Some(ftd_rt::Color {
                                                    r: 255,
                                                    g: 255,
                                                    b: 255,
                                                    alpha: 1.0,
                                                }),
                                                ..Default::default()
                                            },
                                            size: Some(14),
                                            ..Default::default()
                                        }),
                                        ftd_rt::Element::Null,
                                        ftd_rt::Element::Column(ftd_rt::Column {
                                            container: ftd_rt::Container {
                                                children: vec![
                                                    ftd_rt::Element::Null,
                                                    ftd_rt::Element::Text(ftd_rt::Text {
                                                        text: ftd::markdown_line("Log"),
                                                        line: true,
                                                        common: ftd_rt::Common {
                                                            color: Some(ftd_rt::Color {
                                                                r: 77,
                                                                g: 77,
                                                                b: 77,
                                                                alpha: 1.0,
                                                            }),
                                                            ..Default::default()
                                                        },
                                                        size: Some(14),
                                                        ..Default::default()
                                                    }),
                                                    ftd_rt::Element::Column(ftd_rt::Column {
                                                        container: ftd_rt::Container {
                                                            children: vec![
                                                                ftd_rt::Element::Null,
                                                                ftd_rt::Element::Text(
                                                                    ftd_rt::Text {
                                                                        text: ftd::markdown_line(
                                                                            "ChildLog",
                                                                        ),
                                                                        line: true,
                                                                        common: ftd_rt::Common {
                                                                            color: Some(
                                                                                ftd_rt::Color {
                                                                                    r: 77,
                                                                                    g: 77,
                                                                                    b: 77,
                                                                                    alpha: 1.0,
                                                                                },
                                                                            ),
                                                                            ..Default::default()
                                                                        },
                                                                        size: Some(14),
                                                                        ..Default::default()
                                                                    },
                                                                ),
                                                            ],
                                                            open: (Some(true), None),
                                                            spacing: None,
                                                            align: Default::default(),
                                                            wrap: false,
                                                        },
                                                        common: ftd_rt::Common {
                                                            id: Some(s("/ChildBuilding/")),
                                                            width: Some(ftd_rt::Length::Fill),
                                                            ..Default::default()
                                                        },
                                                    }),
                                                ],
                                                open: (Some(true), None),
                                                spacing: None,
                                                align: Default::default(),
                                                wrap: false,
                                            },
                                            common: ftd_rt::Common {
                                                id: Some(s("/Building/")),
                                                width: Some(ftd_rt::Length::Fill),
                                                ..Default::default()
                                            },
                                        }),
                                        ftd_rt::Element::Column(ftd_rt::Column {
                                            container: ftd_rt::Container {
                                                children: vec![
                                                    ftd_rt::Element::Null,
                                                    ftd_rt::Element::Text(ftd_rt::Text {
                                                        text: ftd::markdown_line("Log2"),
                                                        line: true,
                                                        common: ftd_rt::Common {
                                                            color: Some(ftd_rt::Color {
                                                                r: 77,
                                                                g: 77,
                                                                b: 77,
                                                                alpha: 1.0,
                                                            }),
                                                            ..Default::default()
                                                        },
                                                        size: Some(14),
                                                        ..Default::default()
                                                    }),
                                                ],
                                                open: (Some(true), None),
                                                spacing: None,
                                                align: Default::default(),
                                                wrap: false,
                                            },
                                            common: ftd_rt::Common {
                                                id: Some(s("/Building2/")),
                                                width: Some(ftd_rt::Length::Fill),
                                                ..Default::default()
                                            },
                                        }),
                                    ],
                                    open: (Some(true), None),
                                    spacing: None,
                                    align: Default::default(),
                                    wrap: false,
                                },
                                common: ftd_rt::Common {
                                    id: Some(s("/welcome/")),
                                    width: Some(ftd_rt::Length::Fill),
                                    ..Default::default()
                                },
                            })],
                            ..Default::default()
                        },
                        common: ftd_rt::Common {
                            id: Some(s("toc_main")),
                            height: Some(ftd_rt::Length::Fill),
                            width: Some(ftd_rt::Length::Px { value: 300 }),
                            ..Default::default()
                        },
                    })],
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (g_bag, g_col) = crate::p2::interpreter::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- import: creating-a-tree as ft
                -- ft.ft_toc:
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_bag, bag);
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn reference() {
        let mut bag = super::default_bag();

        bag.insert(
            "fifthtry/ft#dark-mode".to_string(),
            crate::p2::Thing::Variable(crate::Variable {
                name: "dark-mode".to_string(),
                value: crate::Value::Boolean { value: true },
            }),
        );

        bag.insert(
            "fifthtry/ft#toc".to_string(),
            crate::p2::Thing::Variable(crate::Variable {
                name: "toc".to_string(),
                value: crate::Value::String {
                    text: "not set".to_string(),
                    source: crate::TextSource::Caption,
                },
            }),
        );

        bag.insert(
            "fifthtry/ft#markdown".to_string(),
            crate::p2::Thing::Component(crate::Component {
                root: "ftd.text".to_string(),
                full_name: "fifthtry/ft#markdown".to_string(),
                arguments: std::array::IntoIter::new([(
                    s("body"),
                    crate::p2::Kind::String {
                        caption: false,
                        body: true,
                    },
                )])
                .collect(),
                properties: std::array::IntoIter::new([(
                    s("text"),
                    crate::component::Property {
                        default: Some(crate::PropertyValue::Argument {
                            name: "body".to_string(),
                            kind: crate::p2::Kind::String {
                                caption: true,
                                body: true,
                            },
                        }),
                        conditions: vec![],
                    },
                )])
                .collect(),
                instructions: Default::default(),
                kernel: false,
                invocations: Default::default(),
            }),
        );

        bag.insert(
            "reference#name".to_string(),
            crate::p2::Thing::Variable(crate::Variable {
                name: "name".to_string(),
                value: crate::Value::String {
                    text: "John smith".to_string(),
                    source: crate::TextSource::Caption,
                },
            }),
        );

        bag.insert(
            "reference#test-component".to_string(),
            crate::p2::Thing::Component(crate::Component {
                root: "ftd.column".to_string(),
                full_name: "reference#test-component".to_string(),
                arguments: Default::default(),
                properties: std::array::IntoIter::new([
                    (
                        s("background-color"),
                        crate::component::Property {
                            default: Some(crate::PropertyValue::Value {
                                value: crate::variable::Value::String {
                                    text: "#f3f3f3".to_string(),
                                    source: crate::TextSource::Header,
                                },
                            }),
                            conditions: vec![],
                        },
                    ),
                    (
                        s("width"),
                        crate::component::Property {
                            default: Some(crate::PropertyValue::Value {
                                value: crate::variable::Value::String {
                                    text: "200".to_string(),
                                    source: crate::TextSource::Header,
                                },
                            }),
                            conditions: vec![],
                        },
                    ),
                ])
                .collect(),
                instructions: vec![crate::component::Instruction::ChildComponent {
                    child: crate::component::ChildComponent {
                        root: "ftd#text".to_string(),
                        condition: None,
                        properties: std::array::IntoIter::new([(
                            s("text"),
                            crate::component::Property {
                                default: Some(crate::PropertyValue::Reference {
                                    name: "name".to_string(),
                                    kind: crate::p2::Kind::String {
                                        caption: true,
                                        body: true,
                                    },
                                }),
                                conditions: vec![],
                            },
                        )])
                        .collect(),
                    },
                }],
                kernel: false,
                invocations: vec![std::collections::BTreeMap::new()],
            }),
        );
        let title = ftd_rt::Text {
            text: ftd::markdown_line("John smith"),
            line: true,
            ..Default::default()
        };

        let mut main = super::default_column();
        main.container
            .children
            .push(ftd_rt::Element::Column(ftd_rt::Column {
                common: ftd_rt::Common {
                    width: Some(ftd_rt::Length::Px { value: 200 }),
                    background_color: Some(ftd_rt::Color {
                        r: 243,
                        g: 243,
                        b: 243,
                        alpha: 1.0,
                    }),
                    ..Default::default()
                },
                container: ftd_rt::Container {
                    children: vec![ftd_rt::Element::Text(title)],
                    ..Default::default()
                },
            }));
        let (g_bag, g_col) = crate::p2::interpreter::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- import: reference as ct
                -- ct.test-component:
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");
        pretty_assertions::assert_eq!(g_bag, bag);
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn text() {
        let mut bag = super::default_bag();

        bag.insert(
            "foo/bar#foo".to_string(),
            crate::p2::Thing::Component(crate::Component {
                full_name: s("foo/bar#foo"),
                root: "ftd.text".to_string(),
                arguments: std::array::IntoIter::new([(
                    s("name"),
                    crate::p2::Kind::String {
                        caption: true,
                        body: true,
                    },
                )])
                .collect(),
                properties: std::array::IntoIter::new([(
                    s("text"),
                    crate::component::Property {
                        default: Some(crate::PropertyValue::Argument {
                            name: "name".to_string(),
                            kind: crate::p2::Kind::String {
                                caption: true,
                                body: true,
                            },
                        }),
                        conditions: vec![],
                    },
                )])
                .collect(),
                invocations: vec![
                    std::array::IntoIter::new([(
                        s("name"),
                        crate::Value::String {
                            text: s("hello"),
                            source: crate::TextSource::Caption,
                        },
                    )])
                    .collect(),
                    std::array::IntoIter::new([(
                        s("name"),
                        crate::Value::String {
                            text: s("world"),
                            source: crate::TextSource::Header,
                        },
                    )])
                    .collect(),
                    std::array::IntoIter::new([(
                        s("name"),
                        crate::Value::String {
                            text: s("yo yo"),
                            source: crate::TextSource::Body,
                        },
                    )])
                    .collect(),
                ],
                ..Default::default()
            }),
        );

        let mut main = super::default_column();
        main.container
            .children
            .push(ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("hello"),
                line: true,
                ..Default::default()
            }));
        main.container
            .children
            .push(ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("world"),
                line: true,
                ..Default::default()
            }));
        main.container
            .children
            .push(ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown("yo yo"),
                line: false,
                ..Default::default()
            }));

        let (g_bag, g_col) = crate::p2::interpreter::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- component foo:
                $name: caption or body
                component: ftd.text
                text: ref $name

                -- foo: hello

                -- foo:
                name: world

                -- foo:

                yo yo
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_bag, bag);
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn row() {
        let mut main = super::default_column();
        let mut row = ftd_rt::Row {
            common: ftd_rt::Common {
                id: Some("the-row".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        row.container
            .children
            .push(ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("hello"),
                line: true,
                ..Default::default()
            }));
        row.container
            .children
            .push(ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("world"),
                line: true,
                ..Default::default()
            }));
        row.container
            .children
            .push(ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("row child three"),
                line: true,
                ..Default::default()
            }));
        main.container.children.push(ftd_rt::Element::Row(row));
        main.container
            .children
            .push(ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("back in main"),
                line: true,
                ..Default::default()
            }));

        p!(
            "
            -- ftd.row:
            id: the-row

            -- ftd.text:
            text: hello

            -- ftd.text:
            text: world

            -- container: ftd.main

            -- ftd.text:
            text: back in main

            -- container: the-row

            -- ftd.text:
            text: row child three
        ",
            (super::default_bag(), main),
        );
    }

    #[test]
    fn sub_function() {
        let mut main = super::default_column();
        let mut row: ftd_rt::Row = Default::default();
        row.container
            .children
            .push(ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("hello"),
                line: true,
                ..Default::default()
            }));
        row.container
            .children
            .push(ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("world"),
                line: true,
                ..Default::default()
            }));
        main.container.children.push(ftd_rt::Element::Row(row));
        main.container
            .children
            .push(ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("back in main"),
                line: true,
                ..Default::default()
            }));

        p!(
            "
            -- ftd.row:

            --- ftd.text:
            text: hello

            --- ftd.text:
            text: world

            -- ftd.text:
            text: back in main
        ",
            (super::default_bag(), main),
        );
    }

    #[test]
    fn sf1() {
        let mut bag = super::default_bag();

        bag.insert(
            "foo/bar#foo".to_string(),
            crate::p2::Thing::Component(crate::Component {
                full_name: s("foo/bar#foo"),
                root: "ftd.row".to_string(),
                instructions: vec![crate::Instruction::ChildComponent{child: crate::ChildComponent {
                    condition: None,
                    root: s("ftd#text"),
                    properties: std::array::IntoIter::new([
                        (
                            s("text"),
                            crate::component::Property {
                                default: Some(crate::PropertyValue::Value {
                                    value: crate::Value::String {
                                        text: s("hello"),
                                        source: crate::TextSource::Header,
                                    },
                                }),
                                conditions: vec![],
                            },
                        ),
                        (
                            s("size"),
                            crate::component::Property {
                                default: Some(crate::PropertyValue::Value {
                                    value: crate::Value::Integer { value: 14 },
                                }),
                                conditions: vec![],
                            },
                        ),
                        (
                            s("font"),
                            crate::component::Property {
                                default: Some(crate::PropertyValue::Value {
                                    value: crate::Value::String {
                                        text: s("Roboto"),
                                        source: crate::TextSource::Header,
                                    },
                                }),
                                conditions: vec![],
                            },
                        ),
                        (
                            s("font-url"),
                            crate::component::Property {
                                default: Some(crate::PropertyValue::Value {
                                    value: crate::Value::String {
                                        text: s("https://fonts.googleapis.com/css2?family=Roboto:wght@100&display=swap"),
                                        source: crate::TextSource::Header,
                                    },
                                }),
                                conditions: vec![],
                            },
                        ),
                        (
                            s("font-display"),
                            crate::component::Property {
                                default: Some(crate::PropertyValue::Value {
                                    value: crate::Value::String {
                                        text: s("swap"),
                                        source: crate::TextSource::Header,
                                    },
                                }),
                                conditions: vec![],
                            },
                        ),
                        (
                            s("border-width"),
                            crate::component::Property {
                                default: Some(crate::PropertyValue::Argument {
                                    name: s("x"),
                                    kind: crate::p2::Kind::Integer.into_optional(),
                                }),
                                conditions: vec![],
                            },
                        ),
                        (
                            s("overflow-x"),
                            crate::component::Property {
                                default: Some(crate::PropertyValue::Value {
                                    value: crate::Value::String {
                                        text: s("auto"),
                                        source: crate::TextSource::Header,
                                    },
                                }),
                                conditions: vec![],
                            },
                        ),
                        (
                            s("overflow-y"),
                            crate::component::Property {
                                default: Some(crate::PropertyValue::Value {
                                    value: crate::Value::String {
                                        text: s("auto"),
                                        source: crate::TextSource::Header,
                                    },
                                }),
                                conditions: vec![],
                            },
                        ),
                    ])
                    .collect(),
                }}],
                arguments: std::array::IntoIter::new([(s("x"), crate::p2::Kind::Integer)]).collect(),
                ..Default::default()
            }),
        );

        let mut main = super::default_column();
        let mut row: ftd_rt::Row = Default::default();
        row.container
            .children
            .push(ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("hello"),
                size: Some(14),
                external_font: Some(ftd_rt::ExternalFont {
                    url: "https://fonts.googleapis.com/css2?family=Roboto:wght@100&display=swap"
                        .to_string(),
                    display: ftd_rt::FontDisplay::Swap,
                    name: "Roboto".to_string(),
                }),
                font: vec![ftd_rt::NamedFont::Named {
                    value: "Roboto".to_string(),
                }],

                line: true,
                common: ftd_rt::Common {
                    border_width: 10,
                    overflow_x: Some(ftd_rt::Overflow::Auto),
                    overflow_y: Some(ftd_rt::Overflow::Auto),
                    ..Default::default()
                },
                ..Default::default()
            }));
        main.container.children.push(ftd_rt::Element::Row(row));
        p!(
            "
            -- component foo:
            component: ftd.row
            $x: integer

            --- ftd.text:
            text: hello
            size: 14
            border-width: ref $x
            font-url: https://fonts.googleapis.com/css2?family=Roboto:wght@100&display=swap
            font: Roboto
            font-display: swap
            overflow-x: auto
            overflow-y: auto

            -- foo:
            x: 10
        ",
            (bag, main),
        );
    }

    #[test]
    fn list_of_numbers() {
        let mut bag = super::default_bag();
        bag.insert(
            "foo/bar#numbers".to_string(),
            crate::p2::Thing::Variable(crate::Variable {
                name: "foo/bar#numbers".to_string(),
                value: crate::Value::List {
                    data: vec![
                        crate::Value::Integer { value: 20 },
                        crate::Value::Integer { value: 30 },
                    ],
                    kind: crate::p2::Kind::Integer,
                },
            }),
        );

        p!(
            "
            -- list numbers:
            type: integer

            -- numbers: 20
            -- numbers: 30
            ",
            (bag, super::default_column()),
        );
    }

    #[test]
    fn list_of_records() {
        let mut bag = super::default_bag();
        bag.insert(
            "foo/bar#point".to_string(),
            crate::p2::Thing::Record(crate::p2::Record {
                name: "foo/bar#point".to_string(),
                fields: std::array::IntoIter::new([
                    (s("x"), crate::p2::Kind::Integer),
                    (s("y"), crate::p2::Kind::Integer),
                ])
                .collect(),
                instances: Default::default(),
            }),
        );

        bag.insert(
            "foo/bar#points".to_string(),
            crate::p2::Thing::Variable(crate::Variable {
                name: "foo/bar#points".to_string(),
                value: crate::Value::List {
                    data: vec![
                        crate::Value::Record {
                            name: s("foo/bar#point"),
                            fields: std::array::IntoIter::new([
                                (
                                    s("x"),
                                    crate::PropertyValue::Value {
                                        value: crate::Value::Integer { value: 10 },
                                    },
                                ),
                                (
                                    s("y"),
                                    crate::PropertyValue::Value {
                                        value: crate::Value::Integer { value: 20 },
                                    },
                                ),
                            ])
                            .collect(),
                        },
                        crate::Value::Record {
                            name: s("foo/bar#point"),
                            fields: std::array::IntoIter::new([
                                (
                                    s("x"),
                                    crate::PropertyValue::Value {
                                        value: crate::Value::Integer { value: 0 },
                                    },
                                ),
                                (
                                    s("y"),
                                    crate::PropertyValue::Value {
                                        value: crate::Value::Integer { value: 0 },
                                    },
                                ),
                            ])
                            .collect(),
                        },
                    ],
                    kind: crate::p2::Kind::Record {
                        name: s("foo/bar#point"),
                    },
                },
            }),
        );

        p!(
            "
            -- record point:
            x: integer
            y: integer

            -- list points:
            type: point

            -- points:
            x: 10
            y: 20

            -- points:
            x: 0
            y: 0
            ",
            (bag, super::default_column()),
        );
    }

    #[test]
    #[ignore]
    fn list_with_reference() {
        let mut bag = super::default_bag();
        bag.insert(
            "foo/bar#numbers".to_string(),
            crate::p2::Thing::Variable(crate::Variable {
                name: "foo/bar#numbers".to_string(),
                value: crate::Value::List {
                    data: vec![
                        crate::Value::Integer { value: 20 },
                        crate::Value::Integer { value: 30 },
                        // TODO: third element
                    ],
                    kind: crate::p2::Kind::Integer,
                },
            }),
        );
        bag.insert(
            "foo/bar#x".to_string(),
            crate::p2::Thing::Variable(crate::Variable {
                name: "x".to_string(),
                value: crate::Value::Integer { value: 20 },
            }),
        );

        p!(
            "
            -- list numbers:
            type: integer

            -- numbers: 20
            -- numbers: 30

            -- var x: 20

            -- numbers: ref x
            ",
            (bag, super::default_column()),
        );
    }

    fn white_two_image_bag(
        about_optional: bool,
    ) -> std::collections::BTreeMap<String, crate::p2::Thing> {
        let mut bag = super::default_bag();
        bag.insert(
            s("foo/bar#white-two-image"),
            crate::p2::Thing::Component(crate::Component {
                invocations: Default::default(),
                full_name: "foo/bar#white-two-image".to_string(),
                root: s("ftd.column"),
                arguments: std::array::IntoIter::new([
                    (s("about"), {
                        let s = crate::p2::Kind::String {
                            caption: false,
                            body: true,
                        };
                        if about_optional {
                            s.into_optional()
                        } else {
                            s
                        }
                    }),
                    (s("src"), {
                        let s = crate::p2::Kind::string();
                        if about_optional {
                            s.into_optional()
                        } else {
                            s
                        }
                    }),
                    (
                        s("title"),
                        crate::p2::Kind::String {
                            caption: true,
                            body: false,
                        },
                    ),
                ])
                .collect(),
                properties: std::array::IntoIter::new([(
                    s("padding"),
                    crate::component::Property {
                        default: Some(crate::PropertyValue::Value {
                            value: crate::Value::Integer { value: 30 },
                        }),
                        conditions: vec![],
                    },
                )])
                .collect(),
                kernel: false,
                instructions: vec![
                    crate::Instruction::ChildComponent {
                        child: crate::ChildComponent {
                            condition: None,
                            root: s("ftd#text"),
                            properties: std::array::IntoIter::new([
                                (
                                    s("text"),
                                    crate::component::Property {
                                        default: Some(crate::PropertyValue::Argument {
                                            name: s("title"),
                                            kind: crate::p2::Kind::String {
                                                caption: true,
                                                body: true,
                                            },
                                        }),
                                        conditions: vec![],
                                    },
                                ),
                                (
                                    s("align"),
                                    crate::component::Property {
                                        default: Some(crate::PropertyValue::Value {
                                            value: crate::Value::String {
                                                source: crate::TextSource::Header,
                                                text: s("center"),
                                            },
                                        }),
                                        conditions: vec![],
                                    },
                                ),
                            ])
                            .collect(),
                        },
                    },
                    crate::Instruction::ChildComponent {
                        child: crate::ChildComponent {
                            condition: if about_optional {
                                Some(ftd::p2::Boolean::IsNotNull {
                                    value: crate::PropertyValue::Argument {
                                        name: s("about"),
                                        kind: crate::p2::Kind::body().into_optional(),
                                    },
                                })
                            } else {
                                None
                            },
                            root: s("ftd#text"),
                            properties: std::array::IntoIter::new([(
                                s("text"),
                                crate::component::Property {
                                    default: Some(crate::PropertyValue::Argument {
                                        name: s("about"),
                                        kind: crate::p2::Kind::String {
                                            caption: true,
                                            body: true,
                                        },
                                    }),
                                    conditions: vec![],
                                },
                            )])
                            .collect(),
                        },
                    },
                    crate::Instruction::ChildComponent {
                        child: crate::ChildComponent {
                            condition: if about_optional {
                                Some(ftd::p2::Boolean::IsNotNull {
                                    value: crate::PropertyValue::Argument {
                                        name: s("src"),
                                        kind: crate::p2::Kind::string().into_optional(),
                                    },
                                })
                            } else {
                                None
                            },
                            root: s("ftd#image"),
                            properties: std::array::IntoIter::new([(
                                s("src"),
                                crate::component::Property {
                                    default: Some(crate::PropertyValue::Argument {
                                        name: s("src"),
                                        kind: crate::p2::Kind::string(),
                                    }),
                                    conditions: vec![],
                                },
                            )])
                            .collect(),
                        },
                    },
                ],
            }),
        );
        bag
    }

    #[test]
    fn components() {
        let title = ftd_rt::Text {
            text: ftd::markdown_line("What kind of documentation?"),
            align: ftd_rt::TextAlign::Center,
            line: true,
            ..Default::default()
        };
        let about = ftd_rt::Text {
            text: ftd::markdown(
                indoc::indoc!(
                    "
                    UI screens, behaviour and journeys, database tables, APIs, how to
                    contribute to, deploy, or monitor microservice, everything that
                    makes web or mobile product teams productive.
                    "
                )
                .trim(),
            ),
            ..Default::default()
        };

        let image = ftd_rt::Image {
            src: s("/static/home/document-type-min.png"),
            ..Default::default()
        };

        let mut main = super::default_column();
        main.container
            .children
            .push(ftd_rt::Element::Column(ftd_rt::Column {
                common: ftd_rt::Common {
                    padding: Some(30),
                    ..Default::default()
                },
                container: ftd_rt::Container {
                    children: vec![
                        ftd_rt::Element::Text(title),
                        ftd_rt::Element::Text(about),
                        ftd_rt::Element::Image(image),
                    ],
                    ..Default::default()
                },
            }));

        p!(
            "
            -- component white-two-image:
            component: ftd.column
            $title: caption
            $about: body
            $src: string
            padding: 30

            --- ftd.text:
            text: ref $title
            align: center

            --- ftd.text:
            text: ref $about

            --- ftd.image:
            src: ref $src

            -- white-two-image: What kind of documentation?
            src: /static/home/document-type-min.png

            UI screens, behaviour and journeys, database tables, APIs, how to
            contribute to, deploy, or monitor microservice, everything that
            makes web or mobile product teams productive.
            ",
            (white_two_image_bag(false), main),
        );
    }

    #[test]
    fn conditional_body() {
        let title = ftd_rt::Text {
            text: ftd::markdown_line("What kind of documentation?"),
            align: ftd_rt::TextAlign::Center,
            line: true,
            ..Default::default()
        };
        let second_title = ftd_rt::Text {
            text: ftd::markdown_line("second call"),
            align: ftd_rt::TextAlign::Center,
            line: true,
            ..Default::default()
        };
        let about = ftd_rt::Text {
            text: ftd::markdown(
                indoc::indoc!(
                    "
                    UI screens, behaviour and journeys, database tables, APIs, how to
                    contribute to, deploy, or monitor microservice, everything that
                    makes web or mobile product teams productive.
                    "
                )
                .trim(),
            ),
            ..Default::default()
        };
        let image = ftd_rt::Image {
            src: s("/static/home/document-type-min.png"),
            ..Default::default()
        };
        let second_image = ftd_rt::Image {
            src: s("second-image.png"),
            ..Default::default()
        };

        let mut main = super::default_column();
        main.container
            .children
            .push(ftd_rt::Element::Column(ftd_rt::Column {
                common: ftd_rt::Common {
                    padding: Some(30),
                    ..Default::default()
                },
                container: ftd_rt::Container {
                    children: vec![
                        ftd_rt::Element::Text(title),
                        ftd_rt::Element::Text(about),
                        ftd_rt::Element::Image(image),
                    ],
                    ..Default::default()
                },
            }));
        main.container
            .children
            .push(ftd_rt::Element::Column(ftd_rt::Column {
                common: ftd_rt::Common {
                    padding: Some(30),
                    ..Default::default()
                },
                container: ftd_rt::Container {
                    children: vec![
                        ftd_rt::Element::Text(second_title),
                        ftd_rt::Element::Null,
                        ftd_rt::Element::Image(second_image),
                    ],
                    ..Default::default()
                },
            }));

        p!(
            "
            -- component white-two-image:
            component: ftd.column
            $title: caption
            $about: optional body
            $src: optional string
            padding: 30

            --- ftd.text:
            text: ref $title
            align: center

            --- ftd.text:
            if: $about is not null
            text: ref $about

            --- ftd.image:
            if: $src is not null
            src: ref $src

            -- white-two-image: What kind of documentation?
            src: /static/home/document-type-min.png

            UI screens, behaviour and journeys, database tables, APIs, how to
            contribute to, deploy, or monitor microservice, everything that
            makes web or mobile product teams productive.

            -- white-two-image: second call
            src: second-image.png
            ",
            (white_two_image_bag(true), main),
        );
    }

    #[test]
    fn conditional_header() {
        let title = ftd_rt::Text {
            text: ftd::markdown_line("What kind of documentation?"),
            align: ftd_rt::TextAlign::Center,
            line: true,
            ..Default::default()
        };
        let second_title = ftd_rt::Text {
            text: ftd::markdown_line("second call"),
            align: ftd_rt::TextAlign::Center,
            line: true,
            ..Default::default()
        };
        let third_title = ftd_rt::Text {
            text: ftd::markdown_line("third call"),
            align: ftd_rt::TextAlign::Center,
            line: true,
            ..Default::default()
        };
        let about = ftd_rt::Text {
            text: ftd::markdown(
                indoc::indoc!(
                    "
                    UI screens, behaviour and journeys, database tables, APIs, how to
                    contribute to, deploy, or monitor microservice, everything that
                    makes web or mobile product teams productive.
                    "
                )
                .trim(),
            ),
            ..Default::default()
        };
        let image = ftd_rt::Image {
            src: s("/static/home/document-type-min.png"),
            ..Default::default()
        };
        let second_image = ftd_rt::Image {
            src: s("second-image.png"),
            ..Default::default()
        };

        let mut main = super::default_column();
        main.container
            .children
            .push(ftd_rt::Element::Column(ftd_rt::Column {
                common: ftd_rt::Common {
                    padding: Some(30),
                    ..Default::default()
                },
                container: ftd_rt::Container {
                    children: vec![
                        ftd_rt::Element::Text(title),
                        ftd_rt::Element::Text(about),
                        ftd_rt::Element::Image(image),
                    ],
                    ..Default::default()
                },
            }));
        main.container
            .children
            .push(ftd_rt::Element::Column(ftd_rt::Column {
                common: ftd_rt::Common {
                    padding: Some(30),
                    ..Default::default()
                },
                container: ftd_rt::Container {
                    children: vec![
                        ftd_rt::Element::Text(second_title),
                        ftd_rt::Element::Null,
                        ftd_rt::Element::Image(second_image),
                    ],
                    ..Default::default()
                },
            }));
        main.container
            .children
            .push(ftd_rt::Element::Column(ftd_rt::Column {
                common: ftd_rt::Common {
                    padding: Some(30),
                    ..Default::default()
                },
                container: ftd_rt::Container {
                    children: vec![
                        ftd_rt::Element::Text(third_title),
                        ftd_rt::Element::Null,
                        ftd_rt::Element::Null,
                    ],
                    ..Default::default()
                },
            }));

        p!(
            "
            -- component white-two-image:
            component: ftd.column
            $title: caption
            $about: optional body
            $src: optional string
            padding: 30

            --- ftd.text:
            text: ref $title
            align: center

            --- ftd.text:
            if: $about is not null
            text: ref $about

            --- ftd.image:
            if: $src is not null
            src: ref $src

            -- white-two-image: What kind of documentation?
            src: /static/home/document-type-min.png

            UI screens, behaviour and journeys, database tables, APIs, how to
            contribute to, deploy, or monitor microservice, everything that
            makes web or mobile product teams productive.

            -- white-two-image: second call
            src: second-image.png

            -- white-two-image: third call
            ",
            (white_two_image_bag(true), main),
        );
    }

    #[test]
    fn markdown() {
        let mut bag = super::default_bag();
        bag.insert(
            s("fifthtry/ft#markdown"),
            crate::p2::Thing::Component(crate::Component {
                invocations: Default::default(),
                full_name: "fifthtry/ft#markdown".to_string(),
                root: s("ftd.text"),
                arguments: std::array::IntoIter::new([(s("body"), crate::p2::Kind::body())])
                    .collect(),
                properties: std::array::IntoIter::new([(
                    s("text"),
                    crate::component::Property {
                        default: Some(crate::PropertyValue::Argument {
                            name: s("body"),
                            kind: crate::p2::Kind::string().string_any(),
                        }),
                        conditions: vec![],
                    },
                )])
                .collect(),
                kernel: false,
                instructions: vec![],
            }),
        );
        bag.insert(
            s("fifthtry/ft#dark-mode"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("dark-mode"),
                value: ftd::Value::Boolean { value: true },
            }),
        );
        bag.insert(
            s("fifthtry/ft#toc"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("toc"),
                value: ftd::Value::String {
                    text: "not set".to_string(),
                    source: ftd::TextSource::Caption,
                },
            }),
        );
        bag.insert(
            s("foo/bar#h0"),
            crate::p2::Thing::Component(crate::Component {
                invocations: Default::default(),
                full_name: "foo/bar#h0".to_string(),
                root: s("ftd.column"),
                arguments: std::array::IntoIter::new([
                    (s("body"), crate::p2::Kind::body().into_optional()),
                    (
                        s("title"),
                        crate::p2::Kind::String {
                            caption: true,
                            body: false,
                        },
                    ),
                ])
                .collect(),
                properties: Default::default(),
                kernel: false,
                instructions: vec![
                    crate::Instruction::ChildComponent {
                        child: crate::ChildComponent {
                            condition: None,
                            root: s("ftd#text"),
                            properties: std::array::IntoIter::new([(
                                s("text"),
                                crate::component::Property {
                                    default: Some(crate::PropertyValue::Argument {
                                        name: s("title"),
                                        kind: crate::p2::Kind::String {
                                            caption: true,
                                            body: true,
                                        },
                                    }),
                                    conditions: vec![],
                                },
                            )])
                            .collect(),
                        },
                    },
                    crate::Instruction::ChildComponent {
                        child: crate::ChildComponent {
                            condition: Some(ftd::p2::Boolean::IsNotNull {
                                value: crate::PropertyValue::Argument {
                                    name: s("body"),
                                    kind: crate::p2::Kind::body().into_optional(),
                                },
                            }),
                            root: s("fifthtry/ft#markdown"),
                            properties: std::array::IntoIter::new([(
                                s("body"),
                                crate::component::Property {
                                    default: Some(crate::PropertyValue::Argument {
                                        name: s("body"),
                                        kind: crate::p2::Kind::String {
                                            caption: false,
                                            body: true,
                                        },
                                    }),
                                    conditions: vec![],
                                },
                            )])
                            .collect(),
                        },
                    },
                ],
            }),
        );

        let mut main = super::default_column();
        main.container
            .children
            .push(ftd_rt::Element::Column(ftd_rt::Column {
                container: ftd_rt::Container {
                    children: vec![
                        ftd_rt::Element::Text(ftd_rt::Text {
                            text: ftd::markdown_line("hello"),
                            line: true,
                            ..Default::default()
                        }),
                        ftd_rt::Element::Text(ftd_rt::Text {
                            text: ftd::markdown("what about the body?"),
                            ..Default::default()
                        }),
                    ],
                    ..Default::default()
                },
                ..Default::default()
            }));
        main.container
            .children
            .push(ftd_rt::Element::Column(ftd_rt::Column {
                container: ftd_rt::Container {
                    children: vec![
                        ftd_rt::Element::Text(ftd_rt::Text {
                            text: ftd::markdown_line("heading without body"),
                            line: true,
                            ..Default::default()
                        }),
                        ftd_rt::Element::Null,
                    ],
                    ..Default::default()
                },
                ..Default::default()
            }));

        p!(
            "
            -- import: fifthtry/ft

            -- component h0:
            component: ftd.column
            $title: caption
            $body: optional body

            --- ftd.text:
            text: ref $title

            --- ft.markdown:
            if: $body is not null
            body: ref $body

            -- h0: hello

            what about the body?

            -- h0: heading without body
            ",
            (bag, main),
        );
    }

    #[test]
    fn width() {
        let mut bag = super::default_bag();

        bag.insert(
            s("foo/bar#image"),
            crate::p2::Thing::Component(crate::Component {
                invocations: Default::default(),
                full_name: "foo/bar#image".to_string(),
                root: s("ftd.column"),
                arguments: std::array::IntoIter::new([
                    (s("width"), crate::p2::Kind::string().into_optional()),
                    (s("src"), crate::p2::Kind::string()),
                ])
                .collect(),
                properties: Default::default(),
                kernel: false,
                instructions: vec![crate::Instruction::ChildComponent {
                    child: crate::ChildComponent {
                        condition: None,
                        root: s("ftd#image"),
                        properties: std::array::IntoIter::new([
                            (
                                s("src"),
                                crate::component::Property {
                                    default: Some(crate::PropertyValue::Argument {
                                        name: s("src"),
                                        kind: crate::p2::Kind::string(),
                                    }),
                                    conditions: vec![],
                                },
                            ),
                            (
                                s("width"),
                                crate::component::Property {
                                    default: Some(crate::PropertyValue::Argument {
                                        name: s("width"),
                                        kind: crate::p2::Kind::string().into_optional(),
                                    }),
                                    conditions: vec![],
                                },
                            ),
                        ])
                        .collect(),
                    },
                }],
            }),
        );

        let mut main = super::default_column();

        main.container
            .children
            .push(ftd_rt::Element::Column(ftd_rt::Column {
                container: ftd_rt::Container {
                    children: vec![ftd_rt::Element::Image(ftd_rt::Image {
                        src: s("foo.png"),
                        ..Default::default()
                    })],
                    ..Default::default()
                },
                ..Default::default()
            }));
        main.container
            .children
            .push(ftd_rt::Element::Column(ftd_rt::Column {
                container: ftd_rt::Container {
                    children: vec![ftd_rt::Element::Image(ftd_rt::Image {
                        src: s("bar.png"),
                        common: ftd_rt::Common {
                            width: Some(ftd_rt::Length::Px { value: 300 }),
                            ..Default::default()
                        },
                        ..Default::default()
                    })],
                    ..Default::default()
                },
                ..Default::default()
            }));

        p!(
            "
            -- component image:
            component: ftd.column
            $src: string
            $width: optional string

            --- ftd.image:
            src: ref $src
            width: ref $width

            -- image:
            src: foo.png

            -- image:
            src: bar.png
            width: 300
            ",
            (bag, main),
        );
    }

    #[test]
    fn decimal() {
        let mut bag = super::default_bag();

        bag.insert(
            "foo/bar#foo".to_string(),
            crate::p2::Thing::Component(crate::Component {
                full_name: s("foo/bar#foo"),
                root: "ftd.row".to_string(),
                instructions: vec![
                    crate::Instruction::ChildComponent {
                        child: crate::ChildComponent {
                            condition: None,
                            root: s("ftd#decimal"),
                            properties: std::array::IntoIter::new([
                                (
                                    s("value"),
                                    crate::component::Property {
                                        default: Some(crate::PropertyValue::Value {
                                            value: crate::Value::Decimal { value: 0.06 },
                                        }),
                                        conditions: vec![],
                                    },
                                ),
                                (
                                    s("format"),
                                    crate::component::Property {
                                        default: Some(crate::PropertyValue::Value {
                                            value: crate::Value::String {
                                                text: s(".1f"),
                                                source: crate::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                    },
                                ),
                            ])
                            .collect(),
                        },
                    },
                    crate::Instruction::ChildComponent {
                        child: crate::ChildComponent {
                            condition: None,
                            root: s("ftd#decimal"),
                            properties: std::array::IntoIter::new([(
                                s("value"),
                                crate::component::Property {
                                    default: Some(crate::PropertyValue::Value {
                                        value: crate::Value::Decimal { value: 0.01 },
                                    }),
                                    conditions: vec![],
                                },
                            )])
                            .collect(),
                        },
                    },
                ],
                arguments: std::array::IntoIter::new([(s("x"), crate::p2::Kind::Integer)])
                    .collect(),
                ..Default::default()
            }),
        );

        let mut main = super::default_column();
        let mut row: ftd_rt::Row = Default::default();
        row.container
            .children
            .push(ftd_rt::Element::Decimal(ftd_rt::Text {
                text: ftd::markdown_line("0.1"),
                line: false,
                ..Default::default()
            }));
        row.container
            .children
            .push(ftd_rt::Element::Decimal(ftd_rt::Text {
                text: ftd::markdown_line("0.01"),
                line: false,
                ..Default::default()
            }));
        main.container.children.push(ftd_rt::Element::Row(row));

        p!(
            "
            -- component foo:
            component: ftd.row
            $x: integer

            --- ftd.decimal:
            value: 0.06
            format: .1f

            --- ftd.decimal:
            value: 0.01

            -- foo:
            x: 10
        ",
            (bag, main),
        );
    }

    #[test]
    fn integer() {
        let mut bag = super::default_bag();

        bag.insert(
            "foo/bar#foo".to_string(),
            crate::p2::Thing::Component(crate::Component {
                full_name: s("foo/bar#foo"),
                root: "ftd.row".to_string(),
                instructions: vec![
                    crate::Instruction::ChildComponent {
                        child: crate::ChildComponent {
                            condition: None,
                            root: s("ftd#integer"),
                            properties: std::array::IntoIter::new([
                                (
                                    s("value"),
                                    crate::component::Property {
                                        default: Some(crate::PropertyValue::Value {
                                            value: crate::Value::Integer { value: 3 },
                                        }),
                                        conditions: vec![],
                                    },
                                ),
                                (
                                    s("format"),
                                    crate::component::Property {
                                        default: Some(crate::PropertyValue::Value {
                                            value: crate::Value::String {
                                                text: s("b"),
                                                source: crate::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                    },
                                ),
                            ])
                            .collect(),
                        },
                    },
                    crate::Instruction::ChildComponent {
                        child: crate::ChildComponent {
                            condition: None,
                            root: s("ftd#integer"),
                            properties: std::array::IntoIter::new([(
                                s("value"),
                                crate::component::Property {
                                    default: Some(crate::PropertyValue::Value {
                                        value: crate::Value::Integer { value: 14 },
                                    }),
                                    conditions: vec![],
                                },
                            )])
                            .collect(),
                        },
                    },
                ],
                arguments: std::array::IntoIter::new([(s("x"), crate::p2::Kind::Integer)])
                    .collect(),
                ..Default::default()
            }),
        );

        let mut main = super::default_column();
        let mut row: ftd_rt::Row = Default::default();
        row.container
            .children
            .push(ftd_rt::Element::Integer(ftd_rt::Text {
                text: ftd::markdown_line("11"),
                line: false,
                ..Default::default()
            }));
        row.container
            .children
            .push(ftd_rt::Element::Integer(ftd_rt::Text {
                text: ftd::markdown_line("14"),
                line: false,
                ..Default::default()
            }));
        main.container.children.push(ftd_rt::Element::Row(row));

        p!(
            "
            -- component foo:
            component: ftd.row
            $x: integer

            --- ftd.integer:
            value: 3
            format: b

            --- ftd.integer:
            value: 14

            -- foo:
            x: 10
        ",
            (bag, main),
        );
    }

    #[test]
    fn boolean() {
        let mut bag = super::default_bag();

        bag.insert(
            "foo/bar#foo".to_string(),
            crate::p2::Thing::Component(crate::Component {
                full_name: s("foo/bar#foo"),
                root: "ftd.row".to_string(),
                instructions: vec![
                    crate::Instruction::ChildComponent {
                        child: crate::ChildComponent {
                            condition: None,
                            root: s("ftd#boolean"),
                            properties: std::array::IntoIter::new([
                                (
                                    s("value"),
                                    crate::component::Property {
                                        default: Some(crate::PropertyValue::Value {
                                            value: crate::Value::Boolean { value: true },
                                        }),
                                        conditions: vec![],
                                    },
                                ),
                                (
                                    s("true"),
                                    crate::component::Property {
                                        default: Some(crate::PropertyValue::Value {
                                            value: crate::Value::String {
                                                text: s("show this when value is true"),
                                                source: crate::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                    },
                                ),
                                (
                                    s("false"),
                                    crate::component::Property {
                                        default: Some(crate::PropertyValue::Value {
                                            value: crate::Value::String {
                                                text: s("show this when value is false"),
                                                source: crate::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                    },
                                ),
                            ])
                            .collect(),
                        },
                    },
                    crate::Instruction::ChildComponent {
                        child: crate::ChildComponent {
                            condition: None,
                            root: s("ftd#boolean"),
                            properties: std::array::IntoIter::new([
                                (
                                    s("value"),
                                    crate::component::Property {
                                        default: Some(crate::PropertyValue::Value {
                                            value: crate::Value::Boolean { value: false },
                                        }),
                                        conditions: vec![],
                                    },
                                ),
                                (
                                    s("true"),
                                    crate::component::Property {
                                        default: Some(crate::PropertyValue::Value {
                                            value: crate::Value::String {
                                                text: s("show this when value is true"),
                                                source: crate::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                    },
                                ),
                                (
                                    s("false"),
                                    crate::component::Property {
                                        default: Some(crate::PropertyValue::Value {
                                            value: crate::Value::String {
                                                text: s("show this when value is false"),
                                                source: crate::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                    },
                                ),
                            ])
                            .collect(),
                        },
                    },
                ],
                arguments: std::array::IntoIter::new([(s("x"), crate::p2::Kind::Integer)])
                    .collect(),
                ..Default::default()
            }),
        );

        let mut main = super::default_column();
        let mut row: ftd_rt::Row = Default::default();
        row.container
            .children
            .push(ftd_rt::Element::Boolean(ftd_rt::Text {
                text: ftd::markdown_line("show this when value is true"),
                line: false,
                ..Default::default()
            }));
        row.container
            .children
            .push(ftd_rt::Element::Boolean(ftd_rt::Text {
                text: ftd::markdown_line("show this when value is false"),
                line: false,
                ..Default::default()
            }));
        main.container.children.push(ftd_rt::Element::Row(row));

        p!(
            "
            -- component foo:
            component: ftd.row
            $x: integer

            --- ftd.boolean:
            value: true
            true:  show this when value is true
            false: show this when value is false

            --- ftd.boolean:
            value: false
            true:  show this when value is true
            false: show this when value is false

            -- foo:
            x: 10
        ",
            (bag, main),
        );
    }

    #[test]
    fn boolean_expression() {
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("present is true"),
                line: true,
                common: ftd_rt::Common {
                    condition: Some(ftd_rt::Condition {
                        variable: "foo/bar#present".to_string(),
                        value: "true".to_string(),
                    }),
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("present is false"),
                line: true,
                common: ftd_rt::Common {
                    condition: Some(ftd_rt::Condition {
                        variable: "foo/bar#present".to_string(),
                        value: "false".to_string(),
                    }),
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("dark-mode is true"),
                line: true,
                common: ftd_rt::Common {
                    condition: Some(ftd_rt::Condition {
                        variable: "fifthtry/ft#dark-mode".to_string(),
                        value: "true".to_string(),
                    }),
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("dark-mode is false"),
                line: true,
                common: ftd_rt::Common {
                    condition: Some(ftd_rt::Condition {
                        variable: "fifthtry/ft#dark-mode".to_string(),
                        value: "false".to_string(),
                    }),
                    ..Default::default()
                },
                ..Default::default()
            }));

        let mut column: ftd_rt::Column = Default::default();
        column
            .container
            .children
            .push(ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("inner present false"),
                line: true,
                common: ftd_rt::Common {
                    condition: Some(ftd_rt::Condition {
                        variable: "foo/bar#present".to_string(),
                        value: "false".to_string(),
                    }),
                    ..Default::default()
                },
                ..Default::default()
            }));

        column
            .container
            .children
            .push(ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("inner present true"),
                line: true,
                common: ftd_rt::Common {
                    condition: Some(ftd_rt::Condition {
                        variable: "foo/bar#present".to_string(),
                        value: "true".to_string(),
                    }),
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd_rt::Element::Column(column));

        let mut column: ftd_rt::Column = Default::default();
        column
            .container
            .children
            .push(ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("argument present false"),
                line: true,
                ..Default::default()
            }));
        column.container.children.push(ftd_rt::Element::Null);

        main.container
            .children
            .push(ftd_rt::Element::Column(column));

        let mut column: ftd_rt::Column = Default::default();
        column.container.children.push(ftd_rt::Element::Null);
        column
            .container
            .children
            .push(ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("argument present true"),
                line: true,
                ..Default::default()
            }));
        main.container
            .children
            .push(ftd_rt::Element::Column(column));

        let mut column: ftd_rt::Column = Default::default();
        column
            .container
            .children
            .push(ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("foo2 dark-mode is true"),
                line: true,
                common: ftd_rt::Common {
                    condition: Some(ftd_rt::Condition {
                        variable: "fifthtry/ft#dark-mode".to_string(),
                        value: "true".to_string(),
                    }),
                    ..Default::default()
                },
                ..Default::default()
            }));

        column
            .container
            .children
            .push(ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("foo2 dark-mode is false"),
                line: true,
                common: ftd_rt::Common {
                    condition: Some(ftd_rt::Condition {
                        variable: "fifthtry/ft#dark-mode".to_string(),
                        value: "false".to_string(),
                    }),
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd_rt::Element::Column(column));

        main.container
            .children
            .push(ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("hello literal truth"),
                line: true,
                ..Default::default()
            }));

        main.container.children.push(ftd_rt::Element::Null);

        p!(
            "
            -- import: fifthtry/ft
            -- var present: true

            -- ftd.text: present is true
            if: present

            -- ftd.text: present is false
            if: not present

            -- ftd.text: dark-mode is true
            if: ft.dark-mode

            -- ftd.text: dark-mode is false
            if: not ft.dark-mode

            -- component foo:
            component: ftd.column

            --- ftd.text: inner present false
            if: not present

            --- ftd.text: inner present true
            if: present

            -- foo:

            -- component bar:
            component: ftd.column
            $present: boolean

            --- ftd.text: argument present false
            if: not $present

            --- ftd.text: argument present true
            if: $present

            -- bar:
            present: false

            -- bar:
            present: ref ft.dark-mode

            -- component foo2:
            component: ftd.column

            --- ftd.text: foo2 dark-mode is true
            if: ft.dark-mode

            --- ftd.text: foo2 dark-mode is false
            if: not ft.dark-mode

            -- foo2:

            -- ftd.text: hello literal truth
            if: true

            -- ftd.text: never see light of the day
            if: false
        ",
            (Default::default(), main),
        );
    }

    #[test]
    fn inner_container() {
        let mut bag = super::default_bag();

        bag.insert(
            "foo/bar#foo".to_string(),
            crate::p2::Thing::Component(crate::Component {
                root: "ftd.column".to_string(),
                full_name: "foo/bar#foo".to_string(),
                arguments: Default::default(),
                properties: Default::default(),
                instructions: vec![
                    crate::component::Instruction::ChildComponent {
                        child: crate::component::ChildComponent {
                            root: "ftd#row".to_string(),
                            condition: None,
                            properties: std::array::IntoIter::new([(
                                s("id"),
                                crate::component::Property {
                                    default: Some(crate::PropertyValue::Value {
                                        value: crate::variable::Value::String {
                                            text: "r1".to_string(),
                                            source: crate::TextSource::Header,
                                        },
                                    }),
                                    conditions: vec![],
                                },
                            )])
                            .collect(),
                        },
                    },
                    crate::component::Instruction::ChildComponent {
                        child: crate::component::ChildComponent {
                            root: "ftd#row".to_string(),
                            condition: None,
                            properties: std::array::IntoIter::new([(
                                s("id"),
                                crate::component::Property {
                                    default: Some(crate::PropertyValue::Value {
                                        value: crate::variable::Value::String {
                                            text: "r2".to_string(),
                                            source: crate::TextSource::Header,
                                        },
                                    }),
                                    conditions: vec![],
                                },
                            )])
                            .collect(),
                        },
                    },
                ],
                kernel: false,
                invocations: vec![
                    std::array::IntoIter::new([(
                        s("id"),
                        crate::Value::String {
                            text: s("foo-1"),
                            source: crate::TextSource::Header,
                        },
                    )])
                    .collect(),
                    std::array::IntoIter::new([(
                        s("id"),
                        crate::Value::String {
                            text: s("foo-2"),
                            source: crate::TextSource::Header,
                        },
                    )])
                    .collect(),
                ],
            }),
        );
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd_rt::Element::Column(ftd_rt::Column {
                container: ftd_rt::Container {
                    children: vec![ftd_rt::Element::Row(ftd_rt::Row {
                        container: ftd_rt::Container {
                            children: vec![
                                ftd_rt::Element::Row(ftd_rt::Row {
                                    common: ftd_rt::Common {
                                        id: Some(s("r2")),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                }),
                                ftd_rt::Element::Text(ftd_rt::Text {
                                    text: ftd::markdown_line("hello"),
                                    line: true,
                                    ..Default::default()
                                }),
                            ],
                            ..Default::default()
                        },
                        common: ftd_rt::Common {
                            id: Some(s("r1")),
                            ..Default::default()
                        },
                    })],
                    ..Default::default()
                },
                common: ftd_rt::Common {
                    id: Some(s("foo-1")),
                    ..Default::default()
                },
            }));

        main.container
            .children
            .push(ftd_rt::Element::Column(ftd_rt::Column {
                container: ftd_rt::Container {
                    children: vec![ftd_rt::Element::Row(ftd_rt::Row {
                        container: ftd_rt::Container {
                            children: vec![ftd_rt::Element::Row(ftd_rt::Row {
                                common: ftd_rt::Common {
                                    id: Some(s("r2")),
                                    ..Default::default()
                                },
                                ..Default::default()
                            })],
                            ..Default::default()
                        },
                        common: ftd_rt::Common {
                            id: Some(s("r1")),
                            ..Default::default()
                        },
                    })],
                    ..Default::default()
                },
                common: ftd_rt::Common {
                    id: Some(s("foo-2")),
                    ..Default::default()
                },
            }));
        let (g_bag, g_col) = crate::p2::interpreter::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- component foo:
                component: ftd.column

                --- ftd.row:
                id: r1

                --- ftd.row:
                id: r2

                -- foo:
                id: foo-1

                -- foo:
                id: foo-2

                -- container: foo-1.r1

                -- ftd.text: hello
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");
        pretty_assertions::assert_eq!(g_bag, bag);
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn inner_container_using_import() {
        let mut bag = super::default_bag();

        bag.insert(
            "inner_container#foo".to_string(),
            crate::p2::Thing::Component(crate::Component {
                root: "ftd.column".to_string(),
                full_name: "inner_container#foo".to_string(),
                arguments: Default::default(),
                properties: Default::default(),
                instructions: vec![
                    crate::component::Instruction::ChildComponent {
                        child: crate::component::ChildComponent {
                            root: "ftd#row".to_string(),
                            condition: None,
                            properties: std::array::IntoIter::new([(
                                s("id"),
                                crate::component::Property {
                                    default: Some(crate::PropertyValue::Value {
                                        value: crate::variable::Value::String {
                                            text: "r1".to_string(),
                                            source: crate::TextSource::Header,
                                        },
                                    }),
                                    conditions: vec![],
                                },
                            )])
                            .collect(),
                        },
                    },
                    crate::component::Instruction::ChildComponent {
                        child: crate::component::ChildComponent {
                            root: "ftd#row".to_string(),
                            condition: None,
                            properties: std::array::IntoIter::new([(
                                s("id"),
                                crate::component::Property {
                                    default: Some(crate::PropertyValue::Value {
                                        value: crate::variable::Value::String {
                                            text: "r2".to_string(),
                                            source: crate::TextSource::Header,
                                        },
                                    }),
                                    conditions: vec![],
                                },
                            )])
                            .collect(),
                        },
                    },
                ],
                kernel: false,
                invocations: vec![
                    std::array::IntoIter::new([(
                        s("id"),
                        crate::Value::String {
                            text: s("foo-1"),
                            source: crate::TextSource::Header,
                        },
                    )])
                    .collect(),
                    std::array::IntoIter::new([(
                        s("id"),
                        crate::Value::String {
                            text: s("foo-2"),
                            source: crate::TextSource::Header,
                        },
                    )])
                    .collect(),
                ],
            }),
        );
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd_rt::Element::Column(ftd_rt::Column {
                container: ftd_rt::Container {
                    children: vec![ftd_rt::Element::Row(ftd_rt::Row {
                        container: ftd_rt::Container {
                            children: vec![
                                ftd_rt::Element::Row(ftd_rt::Row {
                                    common: ftd_rt::Common {
                                        id: Some(s("r2")),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                }),
                                ftd_rt::Element::Text(ftd_rt::Text {
                                    text: ftd::markdown_line("hello"),
                                    line: true,
                                    ..Default::default()
                                }),
                            ],
                            ..Default::default()
                        },
                        common: ftd_rt::Common {
                            id: Some(s("r1")),
                            ..Default::default()
                        },
                    })],
                    ..Default::default()
                },
                common: ftd_rt::Common {
                    id: Some(s("foo-1")),
                    ..Default::default()
                },
            }));

        main.container
            .children
            .push(ftd_rt::Element::Column(ftd_rt::Column {
                container: ftd_rt::Container {
                    children: vec![ftd_rt::Element::Row(ftd_rt::Row {
                        container: ftd_rt::Container {
                            children: vec![ftd_rt::Element::Row(ftd_rt::Row {
                                common: ftd_rt::Common {
                                    id: Some(s("r2")),
                                    ..Default::default()
                                },
                                ..Default::default()
                            })],
                            ..Default::default()
                        },
                        common: ftd_rt::Common {
                            id: Some(s("r1")),
                            ..Default::default()
                        },
                    })],
                    ..Default::default()
                },
                common: ftd_rt::Common {
                    id: Some(s("foo-2")),
                    ..Default::default()
                },
            }));

        let (g_bag, g_col) = crate::p2::interpreter::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- import: inner_container as ic

                -- ic.foo:
                id: foo-1

                -- ic.foo:
                id: foo-2

                -- container: foo-1.r1

                -- ftd.text: hello
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");
        pretty_assertions::assert_eq!(g_bag, bag);
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn open_container_with_id() {
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd_rt::Element::Column(ftd_rt::Column {
                container: ftd_rt::Container {
                    children: vec![ftd_rt::Element::Row(ftd_rt::Row {
                        container: ftd_rt::Container {
                            children: vec![
                                ftd_rt::Element::Row(ftd_rt::Row {
                                    ..Default::default()
                                }),
                                ftd_rt::Element::Text(ftd_rt::Text {
                                    text: ftd::markdown_line("hello"),
                                    line: true,
                                    ..Default::default()
                                }),
                            ],
                            ..Default::default()
                        },
                        common: ftd_rt::Common {
                            id: Some(s("some-child")),
                            ..Default::default()
                        },
                    })],
                    open: (None, Some(s("some-child"))),
                    spacing: None,
                    align: Default::default(),
                    wrap: false,
                },
                ..Default::default()
            }));

        let mut bag = super::default_bag();

        bag.insert(
            "foo/bar#foo".to_string(),
            crate::p2::Thing::Component(crate::Component {
                root: "ftd.column".to_string(),
                full_name: s("foo/bar#foo"),
                properties: std::array::IntoIter::new([(
                    s("open"),
                    crate::component::Property {
                        default: Some(crate::PropertyValue::Value {
                            value: crate::Value::String {
                                text: s("some-child"),
                                source: crate::TextSource::Header,
                            },
                        }),
                        conditions: vec![],
                    },
                )])
                .collect(),
                instructions: vec![
                    crate::component::Instruction::ChildComponent {
                        child: crate::component::ChildComponent {
                            root: "ftd#row".to_string(),
                            condition: None,
                            properties: std::array::IntoIter::new([(
                                s("id"),
                                crate::component::Property {
                                    default: Some(crate::PropertyValue::Value {
                                        value: crate::variable::Value::String {
                                            text: "some-child".to_string(),
                                            source: crate::TextSource::Header,
                                        },
                                    }),
                                    conditions: vec![],
                                },
                            )])
                            .collect(),
                        },
                    },
                    crate::component::Instruction::ChildComponent {
                        child: crate::component::ChildComponent {
                            root: "ftd#row".to_string(),
                            condition: None,
                            ..Default::default()
                        },
                    },
                ],
                invocations: vec![std::collections::BTreeMap::new()],
                ..Default::default()
            }),
        );

        let (g_bag, g_col) = crate::p2::interpreter::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- component foo:
                open: some-child
                component: ftd.column

                --- ftd.row:
                id: some-child

                --- ftd.row:

                -- foo:

                -- ftd.text: hello
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");
        pretty_assertions::assert_eq!(g_bag, bag);
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn open_container_id() {
        let mut main = self::default_column();
        main.container
            .children
            .push(ftd_rt::Element::Row(ftd_rt::Row {
                common: ftd_rt::Common {
                    id: Some(s("r1")),
                    ..Default::default()
                },
                container: ftd_rt::Container {
                    open: (Some(false), None),
                    ..Default::default()
                },
            }));
        main.container
            .children
            .push(ftd_rt::Element::Row(ftd_rt::Row {
                container: ftd_rt::Container {
                    children: vec![
                        ftd_rt::Element::Text(ftd_rt::Text {
                            text: ftd::markdown_line("hello"),
                            line: true,
                            ..Default::default()
                        }),
                        ftd_rt::Element::Row(ftd_rt::Row {
                            container: ftd_rt::Container {
                                open: (Some(false), None),
                                ..Default::default()
                            },
                            common: ftd_rt::Common {
                                id: Some(s("r3")),
                                ..Default::default()
                            },
                        }),
                    ],
                    open: (Some(true), None),
                    spacing: None,
                    align: Default::default(),
                    wrap: false,
                },
                common: ftd_rt::Common {
                    id: Some(s("r2")),
                    ..Default::default()
                },
            }));
        let (g_bag, g_col) = crate::p2::interpreter::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- ftd.row:
                id: r1
                open: false

                -- ftd.row:
                id: r2
                open: true

                --- ftd.text: hello

                -- ftd.row:
                id: r3
                open: false
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");
        pretty_assertions::assert_eq!(g_bag, super::default_bag());
        pretty_assertions::assert_eq!(g_col, main);
    }
}
