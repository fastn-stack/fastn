pub(crate) struct Interpreter<'a> {
    lib: &'a dyn crate::p2::Library,
    pub bag: std::collections::BTreeMap<String, crate::p2::Thing>,
    pub p1: Vec<ftd::p1::Section>,
    pub aliases: std::collections::BTreeMap<String, String>,
    pub parsed_libs: Vec<String>,
}

impl<'a> Interpreter<'a> {
    // #[observed(with_result, namespace = "ftd")]
    pub(crate) fn interpret(
        &mut self,
        name: &str,
        s: &str,
    ) -> crate::p1::Result<Vec<ftd::Instruction>> {
        let mut d_get = std::time::Duration::new(0, 0);
        let mut d_processor = std::time::Duration::new(0, 0);
        let v = self.interpret_(name, s, true, &mut d_get, &mut d_processor)?;
        Ok(v)
    }

    fn library_in_the_bag(&self, name: &str) -> bool {
        self.parsed_libs.contains(&name.to_string())
    }

    fn add_library_to_bag(&mut self, name: &str) {
        if !self.library_in_the_bag(name) {
            self.parsed_libs.push(name.to_string());
        }
    }

    fn interpret_(
        &mut self,
        name: &str,
        s: &str,
        is_main: bool,
        d_get: &mut std::time::Duration,
        d_processor: &mut std::time::Duration,
    ) -> crate::p1::Result<Vec<ftd::Instruction>> {
        let p1 = crate::p1::parse(s)?;
        let new_p1 = ftd::p2::utils::reorder(&p1)?;

        let mut aliases = default_aliases();
        let mut instructions: Vec<ftd::Instruction> = Default::default();

        for p1 in new_p1.iter() {
            if p1.is_commented {
                continue;
            }
            if p1.name == "import" {
                let (library_name, alias) = crate::p2::utils::parse_import(&p1.caption)?;
                aliases.insert(alias, library_name.clone());
                let start = std::time::Instant::now();
                let s = self.lib.get_with_result(library_name.as_str())?;
                *d_get = d_get.saturating_add(std::time::Instant::now() - start);
                if !self.library_in_the_bag(library_name.as_str()) {
                    self.interpret_(library_name.as_str(), s.as_str(), false, d_get, d_processor)?;
                    self.add_library_to_bag(library_name.as_str())
                }
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
                // processed_p1.push(p1.name.to_string());
            } else if p1.name.starts_with("var ") {
                // declare and instantiate a variable
                let d = if p1.header.str("$processor$").is_ok() {
                    let name = ftd_rt::get_name("var", p1.name.as_str())?.to_string();
                    let start = std::time::Instant::now();
                    let value = self.lib.process(p1, &doc)?;
                    *d_processor = d_processor.saturating_add(std::time::Instant::now() - start);
                    crate::Variable {
                        name,
                        value,
                        conditions: vec![],
                    }
                } else {
                    crate::Variable::from_p1(p1, &doc)?
                };
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
                let d = if p1.header.str("$processor$").is_ok() {
                    let name = doc.resolve_name(ftd_rt::get_name("list", p1.name.as_str())?)?;
                    let start = std::time::Instant::now();
                    let value = self.lib.process(p1, &doc)?;
                    *d_processor = d_processor.saturating_add(std::time::Instant::now() - start);
                    crate::Variable {
                        name,
                        value,
                        conditions: vec![],
                    }
                } else {
                    crate::Variable::list_from_p1(p1, &doc)?
                };
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
                        assert!(
                            !(p1.header.str_optional("if")?.is_some()
                                && p1.header.str_optional("$processor$")?.is_some())
                        );
                        if let Some(expr) = p1.header.str_optional("if")? {
                            let val = v.get_value(p1, &doc)?;
                            v.conditions.push((
                                crate::p2::Boolean::from_expression(
                                    expr,
                                    &doc,
                                    &Default::default(),
                                    &Default::default(),
                                    (None, None),
                                )?,
                                val,
                            ));
                        } else if p1.header.str_optional("$processor$")?.is_some() {
                            let start = std::time::Instant::now();
                            let value = self.lib.process(p1, &doc)?;
                            *d_processor =
                                d_processor.saturating_add(std::time::Instant::now() - start);
                            v.value = value;
                        } else {
                            v.update_from_p1(p1, &doc)?;
                        }
                        thing = Some((p1.name.to_string(), crate::p2::Thing::Variable(v)));
                    }
                    crate::p2::Thing::Component(_) => {
                        let mut children = vec![];

                        for sub in p1.sub_sections.0.iter() {
                            if sub.is_commented {
                                continue;
                            }
                            if let Ok(loop_data) = sub.header.str("$loop$") {
                                children.push(ftd::component::recursive_child_component(
                                    loop_data,
                                    sub,
                                    &doc,
                                    &Default::default(),
                                    None,
                                    &Default::default(),
                                )?);
                            } else {
                                children.push(ftd::ChildComponent::from_p1(
                                    sub.name.as_str(),
                                    &sub.header,
                                    &sub.caption,
                                    &sub.body_without_comment(),
                                    &doc,
                                    &Default::default(),
                                    &Default::default(),
                                )?);
                            }
                        }
                        if let Ok(loop_data) = p1.header.str("$loop$") {
                            let section_to_subsection = ftd::p1::SubSection {
                                name: p1.name.to_string(),
                                caption: p1.caption.to_owned(),
                                header: p1.header.to_owned(),
                                body: p1.body.to_owned(),
                                is_commented: p1.is_commented,
                            };
                            instructions.push(ftd::Instruction::RecursiveChildComponent {
                                child: ftd::component::recursive_child_component(
                                    loop_data,
                                    &section_to_subsection,
                                    &doc,
                                    &Default::default(),
                                    None,
                                    &Default::default(),
                                )?,
                            });
                        } else {
                            instructions.push(ftd::Instruction::Component {
                                children,
                                parent: ftd::ChildComponent::from_p1(
                                    p1.name.as_str(),
                                    &p1.header,
                                    &p1.caption,
                                    &p1.body_without_comment(),
                                    &doc,
                                    &Default::default(),
                                    &Default::default(),
                                )?,
                            })
                        }
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
            parsed_libs: Default::default(),
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
    let main = rt.render_()?;
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
    // Library -> Name of library successfully parsed
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

// #[cfg(test)]
// pub fn elapsed(e: std::time::Duration) -> String {
//     // NOTE: there is a copy of this function in ftd also
//     let nanos = e.subsec_nanos();
//     let fraction = match nanos {
//         t if nanos < 1000 => format!("{}ns", t),
//         t if nanos < 1_000_000 => format!("{:.*}µs", 3, f64::from(t) / 1000.0),
//         t => format!("{:.*}ms", 3, f64::from(t) / 1_000_000.0),
//     };
//     let secs = e.as_secs();
//     match secs {
//         _ if secs == 0 => fraction,
//         t if secs < 5 => format!("{}.{:06}s", t, nanos / 1000),
//         t if secs < 60 => format!("{}.{:03}s", t, nanos / 1_000_000),
//         t if secs < 3600 => format!("{}m {}s", t / 60, t % 60),
//         t if secs < 86400 => format!("{}h {}m", t / 3600, (t % 3600) / 60),
//         t => format!("{}s", t),
//     }
// }

#[cfg(test)]
mod test {
    use crate::test::*;
    use crate::{markdown_line, Instruction};

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
                conditions: vec![],
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
                arguments: std::array::IntoIter::new([(s("name"), crate::p2::Kind::caption())])
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
                                            kind: crate::p2::Kind::boolean(),
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
                                            kind: crate::p2::Kind::boolean(),
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
                                kind: crate::p2::Kind::caption_or_body(),
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
                conditions: vec![],
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
                            events: vec![],
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
                            is_recursive: false,
                        },
                    },
                    crate::component::Instruction::ChildComponent {
                        child: crate::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
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
                            is_recursive: false,
                            events: vec![],
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
                            is_recursive: false,
                            events: vec![],
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
                            is_recursive: false,
                            events: vec![],
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
                ..Default::default()
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
                            kind: Box::new(crate::p2::Kind::boolean()),
                        },
                    ),
                    (s("id"), crate::p2::Kind::string()),
                    (s("name"), crate::p2::Kind::caption()),
                ])
                .collect(),
                properties: std::array::IntoIter::new([
                    (
                        s("id"),
                        crate::component::Property {
                            default: Some(crate::PropertyValue::Argument {
                                name: "id".to_string(),
                                kind: crate::p2::Kind::Optional {
                                    kind: Box::new(crate::p2::Kind::string()),
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
                            is_recursive: false,
                            events: vec![],
                            root: "ftd#text".to_string(),
                            condition: Some(ftd::p2::Boolean::IsNotNull {
                                value: ftd::PropertyValue::Argument {
                                    name: "active".to_string(),
                                    kind: crate::p2::Kind::Optional {
                                        kind: Box::new(crate::p2::Kind::boolean()),
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
                                            kind: crate::p2::Kind::caption_or_body(),
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
                            is_recursive: false,
                            events: vec![],
                            root: "ftd#text".to_string(),
                            condition: Some(ftd::p2::Boolean::IsNull {
                                value: ftd::PropertyValue::Argument {
                                    name: "active".to_string(),
                                    kind: crate::p2::Kind::Optional {
                                        kind: Box::new(crate::p2::Kind::boolean()),
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
                                            kind: crate::p2::Kind::caption_or_body(),
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
                ..Default::default()
            }),
        );

        bag.insert(
            "foo/bar#table-of-content".to_string(),
            crate::p2::Thing::Component(crate::Component {
                root: "ftd.column".to_string(),
                full_name: "foo/bar#table-of-content".to_string(),
                arguments: std::array::IntoIter::new([(s("id"), crate::p2::Kind::string())])
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
                                    kind: Box::new(crate::p2::Kind::string()),
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
                ..Default::default()
            }),
        );

        bag.insert(
            "foo/bar#toc-heading".to_string(),
            crate::p2::Thing::Component(crate::Component {
                root: "ftd.text".to_string(),
                full_name: "foo/bar#toc-heading".to_string(),
                arguments: std::array::IntoIter::new([(s("text"), crate::p2::Kind::caption())])
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
                                kind: crate::p2::Kind::caption_or_body(),
                            }),
                            conditions: vec![],
                        },
                    ),
                ])
                .collect(),
                ..Default::default()
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
                                                            external_children: Default::default(),
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
                                                external_children: Default::default(),
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
                                                external_children: Default::default(),
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
                                    external_children: Default::default(),
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
                            is_recursive: false,
                            events: vec![],
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
                            is_recursive: false,
                            events: vec![],
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
                            is_recursive: false,
                            events: vec![],
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
                            is_recursive: false,
                            events: vec![],
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
                            is_recursive: false,
                            events: vec![],
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
                ..Default::default()
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
                            kind: Box::new(crate::p2::Kind::boolean()),
                        },
                    ),
                    (s("id"), crate::p2::Kind::string()),
                    (s("name"), crate::p2::Kind::caption()),
                ])
                .collect(),
                properties: std::array::IntoIter::new([
                    (
                        s("id"),
                        crate::component::Property {
                            default: Some(crate::PropertyValue::Argument {
                                name: "id".to_string(),
                                kind: crate::p2::Kind::Optional {
                                    kind: Box::new(crate::p2::Kind::string()),
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
                            is_recursive: false,
                            events: vec![],
                            root: "ftd#text".to_string(),
                            condition: Some(ftd::p2::Boolean::IsNotNull {
                                value: ftd::PropertyValue::Argument {
                                    name: "active".to_string(),
                                    kind: crate::p2::Kind::Optional {
                                        kind: Box::new(crate::p2::Kind::boolean()),
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
                                            kind: crate::p2::Kind::caption_or_body(),
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
                            is_recursive: false,
                            events: vec![],
                            root: "ftd#text".to_string(),
                            condition: Some(ftd::p2::Boolean::IsNull {
                                value: ftd::PropertyValue::Argument {
                                    name: "active".to_string(),
                                    kind: crate::p2::Kind::Optional {
                                        kind: Box::new(crate::p2::Kind::boolean()),
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
                                            kind: crate::p2::Kind::caption_or_body(),
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
                ..Default::default()
            }),
        );

        bag.insert(
            "creating-a-tree#table-of-content".to_string(),
            crate::p2::Thing::Component(crate::Component {
                root: "ftd.column".to_string(),
                full_name: "creating-a-tree#table-of-content".to_string(),
                arguments: std::array::IntoIter::new([(s("id"), crate::p2::Kind::string())])
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
                                    kind: Box::new(crate::p2::Kind::string()),
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
                ..Default::default()
            }),
        );

        bag.insert(
            "creating-a-tree#toc-heading".to_string(),
            crate::p2::Thing::Component(crate::Component {
                root: "ftd.text".to_string(),
                full_name: "creating-a-tree#toc-heading".to_string(),
                arguments: std::array::IntoIter::new([(s("text"), crate::p2::Kind::caption())])
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
                                kind: crate::p2::Kind::caption_or_body(),
                            }),
                            conditions: vec![],
                        },
                    ),
                ])
                .collect(),
                ..Default::default()
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
                                                external_children: Default::default(),
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
                                                            external_children: Default::default(),
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
                                                external_children: Default::default(),
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
                                    external_children: Default::default(),
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
                conditions: vec![],
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
                conditions: vec![],
            }),
        );

        bag.insert(
            "fifthtry/ft#markdown".to_string(),
            crate::p2::Thing::Component(crate::Component {
                root: "ftd.text".to_string(),
                full_name: "fifthtry/ft#markdown".to_string(),
                arguments: std::array::IntoIter::new([(s("body"), crate::p2::Kind::body())])
                    .collect(),
                properties: std::array::IntoIter::new([(
                    s("text"),
                    crate::component::Property {
                        default: Some(crate::PropertyValue::Argument {
                            name: "body".to_string(),
                            kind: crate::p2::Kind::caption_or_body(),
                        }),
                        conditions: vec![],
                    },
                )])
                .collect(),
                ..Default::default()
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
                conditions: vec![],
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
                        is_recursive: false,
                        events: vec![],
                        root: "ftd#text".to_string(),
                        condition: None,
                        properties: std::array::IntoIter::new([(
                            s("text"),
                            crate::component::Property {
                                default: Some(crate::PropertyValue::Reference {
                                    name: "reference#name".to_string(),
                                    kind: crate::p2::Kind::caption_or_body(),
                                }),
                                conditions: vec![],
                            },
                        )])
                        .collect(),
                    },
                }],
                kernel: false,
                invocations: vec![std::collections::BTreeMap::new()],
                ..Default::default()
            }),
        );
        let title = ftd_rt::Text {
            text: ftd::markdown_line("John smith"),
            line: true,
            common: ftd_rt::Common {
                reference: Some(s("reference#name")),
                ..Default::default()
            },
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
                    crate::p2::Kind::caption_or_body(),
                )])
                .collect(),
                properties: std::array::IntoIter::new([(
                    s("text"),
                    crate::component::Property {
                        default: Some(crate::PropertyValue::Argument {
                            name: "name".to_string(),
                            kind: crate::p2::Kind::caption_or_body(),
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
                    events: vec![],
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
                                    kind: crate::p2::Kind::integer().into_optional(),
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
                    ..Default::default()
                }}],
                arguments: std::array::IntoIter::new([(s("x"), crate::p2::Kind::integer())]).collect(),
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
                    kind: crate::p2::Kind::integer(),
                },
                conditions: vec![],
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
                    (s("x"), crate::p2::Kind::integer()),
                    (s("y"), crate::p2::Kind::integer()),
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
                conditions: vec![],
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
                    kind: crate::p2::Kind::integer(),
                },
                conditions: vec![],
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
                        let s = crate::p2::Kind::body();
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
                    (s("title"), crate::p2::Kind::caption()),
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
                            events: vec![],
                            condition: None,
                            root: s("ftd#text"),
                            properties: std::array::IntoIter::new([
                                (
                                    s("text"),
                                    crate::component::Property {
                                        default: Some(crate::PropertyValue::Argument {
                                            name: s("title"),
                                            kind: crate::p2::Kind::caption_or_body(),
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
                            ..Default::default()
                        },
                    },
                    crate::Instruction::ChildComponent {
                        child: crate::ChildComponent {
                            events: vec![],
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
                                        kind: crate::p2::Kind::caption_or_body(),
                                    }),
                                    conditions: vec![],
                                },
                            )])
                            .collect(),
                            ..Default::default()
                        },
                    },
                    crate::Instruction::ChildComponent {
                        child: crate::ChildComponent {
                            events: vec![],
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
                            ..Default::default()
                        },
                    },
                ],
                ..Default::default()
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
                ..Default::default()
            }),
        );
        bag.insert(
            s("fifthtry/ft#dark-mode"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("dark-mode"),
                value: ftd::Value::Boolean { value: true },
                conditions: vec![],
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
                conditions: vec![],
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
                    (s("title"), crate::p2::Kind::caption()),
                ])
                .collect(),
                instructions: vec![
                    crate::Instruction::ChildComponent {
                        child: crate::ChildComponent {
                            events: vec![],
                            condition: None,
                            root: s("ftd#text"),
                            properties: std::array::IntoIter::new([(
                                s("text"),
                                crate::component::Property {
                                    default: Some(crate::PropertyValue::Argument {
                                        name: s("title"),
                                        kind: crate::p2::Kind::caption_or_body(),
                                    }),
                                    conditions: vec![],
                                },
                            )])
                            .collect(),
                            ..Default::default()
                        },
                    },
                    crate::Instruction::ChildComponent {
                        child: crate::ChildComponent {
                            events: vec![],
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
                                        kind: crate::p2::Kind::body(),
                                    }),
                                    conditions: vec![],
                                },
                            )])
                            .collect(),
                            ..Default::default()
                        },
                    },
                ],
                ..Default::default()
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
                instructions: vec![crate::Instruction::ChildComponent {
                    child: crate::ChildComponent {
                        events: vec![],
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
                        ..Default::default()
                    },
                }],
                ..Default::default()
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
                            events: vec![],
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
                            ..Default::default()
                        },
                    },
                    crate::Instruction::ChildComponent {
                        child: crate::ChildComponent {
                            events: vec![],
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
                            ..Default::default()
                        },
                    },
                ],
                arguments: std::array::IntoIter::new([(s("x"), crate::p2::Kind::integer())])
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
                            events: vec![],
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
                            ..Default::default()
                        },
                    },
                    crate::Instruction::ChildComponent {
                        child: crate::ChildComponent {
                            events: vec![],
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
                            ..Default::default()
                        },
                    },
                ],
                arguments: std::array::IntoIter::new([(s("x"), crate::p2::Kind::integer())])
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
                            events: vec![],
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
                            ..Default::default()
                        },
                    },
                    crate::Instruction::ChildComponent {
                        child: crate::ChildComponent {
                            events: vec![],
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
                            ..Default::default()
                        },
                    },
                ],
                arguments: std::array::IntoIter::new([(s("x"), crate::p2::Kind::integer())])
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
                instructions: vec![
                    crate::component::Instruction::ChildComponent {
                        child: crate::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
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
                            is_recursive: false,
                            events: vec![],
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
                ..Default::default()
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
                instructions: vec![
                    crate::component::Instruction::ChildComponent {
                        child: crate::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
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
                            is_recursive: false,
                            events: vec![],
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
                ..Default::default()
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
        let mut external_children = super::default_column();
        external_children.container.children = vec![ftd_rt::Element::Text(ftd_rt::Text {
            text: ftd::markdown_line("hello"),
            line: true,
            ..Default::default()
        })];

        let mut main = super::default_column();
        main.container
            .children
            .push(ftd_rt::Element::Column(ftd_rt::Column {
                container: ftd_rt::Container {
                    external_children: Some((
                        s("some-child"),
                        vec![vec![0, 0]],
                        vec![ftd_rt::Element::Column(external_children)],
                    )),
                    children: vec![ftd_rt::Element::Row(ftd_rt::Row {
                        container: ftd_rt::Container {
                            children: vec![ftd_rt::Element::Row(ftd_rt::Row {
                                common: ftd_rt::Common {
                                    id: Some(s("some-child")),
                                    ..Default::default()
                                },
                                ..Default::default()
                            })],
                            ..Default::default()
                        },
                        ..Default::default()
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
                            events: vec![],
                            root: "ftd#row".to_string(),
                            condition: None,
                            ..Default::default()
                        },
                    },
                    crate::component::Instruction::ChildComponent {
                        child: crate::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
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

                --- ftd.row:
                id: some-child

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
    fn open_container_with_if() {
        let mut external_children = super::default_column();
        external_children.container.children = vec![
            ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("hello"),
                line: true,
                ..Default::default()
            }),
            ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("hello1"),
                line: true,
                ..Default::default()
            }),
        ];

        let mut main = super::default_column();
        main.container
            .children
            .push(ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("Start Browser"),
                line: true,
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd_rt::Element::Column(ftd_rt::Column {
                container: ftd_rt::Container {
                    children: vec![ftd_rt::Element::Column(ftd_rt::Column {
                        container: ftd_rt::Container {
                            children: vec![ftd_rt::Element::Column(ftd_rt::Column {
                                container: ftd_rt::Container {
                                    children: vec![
                                        ftd_rt::Element::Column(ftd_rt::Column {
                                            container: ftd_rt::Container {
                                                children: vec![ftd_rt::Element::Text(
                                                    ftd_rt::Text {
                                                        text: ftd::markdown_line("Mobile Display"),
                                                        line: true,
                                                        ..Default::default()
                                                    },
                                                )],
                                                ..Default::default()
                                            },
                                            common: ftd_rt::Common {
                                                condition: Some(ftd_rt::Condition {
                                                    variable: s("foo/bar#mobile"),
                                                    value: s("true"),
                                                }),
                                                id: Some(s("some-child")),
                                                ..Default::default()
                                            },
                                        }),
                                        ftd_rt::Element::Column(ftd_rt::Column {
                                            container: ftd_rt::Container {
                                                children: vec![ftd_rt::Element::Text(
                                                    ftd_rt::Text {
                                                        text: ftd::markdown_line("Desktop Display"),
                                                        line: true,
                                                        ..Default::default()
                                                    },
                                                )],
                                                ..Default::default()
                                            },
                                            common: ftd_rt::Common {
                                                condition: Some(ftd_rt::Condition {
                                                    variable: s("foo/bar#mobile"),
                                                    value: s("false"),
                                                }),
                                                id: Some(s("some-child")),
                                                ..Default::default()
                                            },
                                        }),
                                    ],
                                    external_children: Some((
                                        s("some-child"),
                                        vec![vec![0], vec![1]],
                                        vec![ftd_rt::Element::Column(external_children)],
                                    )),
                                    open: (None, Some(s("some-child"))),
                                    ..Default::default()
                                },
                                ..Default::default()
                            })],
                            ..Default::default()
                        },
                        common: ftd_rt::Common {
                            id: Some(s("c2")),
                            ..Default::default()
                        },
                    })],
                    ..Default::default()
                },
                common: ftd_rt::Common {
                    id: Some(s("c1")),
                    ..Default::default()
                },
            }));

        let mut bag = super::default_bag();
        bag.insert(
            s("foo/bar#desktop-display"),
            crate::p2::Thing::Component(crate::Component {
                root: "ftd.column".to_string(),
                full_name: s("foo/bar#desktop-display"),
                arguments: std::array::IntoIter::new([(
                    s("id"),
                    crate::p2::Kind::optional(ftd::p2::Kind::string()),
                )])
                .collect(),
                properties: std::array::IntoIter::new([(
                    s("id"),
                    ftd::component::Property {
                        default: Some(crate::PropertyValue::Argument {
                            name: "id".to_string(),
                            kind: crate::p2::Kind::Optional {
                                kind: Box::new(crate::p2::Kind::string()),
                            },
                        }),
                        conditions: vec![],
                    },
                )])
                .collect(),
                instructions: vec![crate::component::Instruction::ChildComponent {
                    child: crate::component::ChildComponent {
                        is_recursive: false,
                        events: vec![],
                        root: "ftd#text".to_string(),
                        condition: None,
                        properties: std::array::IntoIter::new([(
                            s("text"),
                            crate::component::Property {
                                default: Some(crate::PropertyValue::Value {
                                    value: crate::variable::Value::String {
                                        text: s("Desktop Display"),
                                        source: ftd::TextSource::Caption,
                                    },
                                }),
                                conditions: vec![],
                            },
                        )])
                        .collect(),
                    },
                }],
                invocations: vec![std::array::IntoIter::new([(
                    s("id"),
                    crate::Value::String {
                        text: s("some-child"),
                        source: crate::TextSource::Header,
                    },
                )])
                .collect()],
                ..Default::default()
            }),
        );

        bag.insert(
            s("foo/bar#foo"),
            crate::p2::Thing::Component(crate::Component {
                root: "ftd.column".to_string(),
                full_name: s("foo/bar#foo"),
                properties: std::array::IntoIter::new([(
                    s("open"),
                    ftd::component::Property {
                        default: Some(crate::PropertyValue::Value {
                            value: crate::variable::Value::String {
                                text: s("some-child"),
                                source: ftd::TextSource::Header,
                            },
                        }),
                        conditions: vec![],
                    },
                )])
                .collect(),
                instructions: vec![
                    crate::component::Instruction::ChildComponent {
                        child: crate::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "foo/bar#mobile-display".to_string(),
                            condition: Some(ftd::p2::Boolean::Equal {
                                left: ftd::PropertyValue::Reference {
                                    name: s("foo/bar#mobile"),
                                    kind: ftd::p2::Kind::Boolean { default: None },
                                },
                                right: ftd::PropertyValue::Value {
                                    value: ftd::variable::Value::Boolean { value: true },
                                },
                            }),
                            properties: std::array::IntoIter::new([(
                                s("id"),
                                crate::component::Property {
                                    default: Some(crate::PropertyValue::Value {
                                        value: crate::variable::Value::String {
                                            text: s("some-child"),
                                            source: ftd::TextSource::Header,
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
                            is_recursive: false,
                            events: vec![],
                            root: "foo/bar#desktop-display".to_string(),
                            condition: Some(ftd::p2::Boolean::Equal {
                                left: ftd::PropertyValue::Reference {
                                    name: s("foo/bar#mobile"),
                                    kind: ftd::p2::Kind::Boolean { default: None },
                                },
                                right: ftd::PropertyValue::Value {
                                    value: ftd::variable::Value::Boolean { value: false },
                                },
                            }),
                            properties: std::array::IntoIter::new([(
                                s("id"),
                                crate::component::Property {
                                    default: Some(crate::PropertyValue::Value {
                                        value: crate::variable::Value::String {
                                            text: s("some-child"),
                                            source: ftd::TextSource::Header,
                                        },
                                    }),
                                    conditions: vec![],
                                },
                            )])
                            .collect(),
                        },
                    },
                ],
                invocations: vec![std::collections::BTreeMap::new()],
                ..Default::default()
            }),
        );

        bag.insert(
            s("foo/bar#mobile"),
            crate::p2::Thing::Variable(ftd::Variable {
                name: s("mobile"),
                value: ftd::variable::Value::Boolean { value: true },
                conditions: vec![],
            }),
        );

        bag.insert(
            s("foo/bar#mobile-display"),
            crate::p2::Thing::Component(crate::Component {
                root: "ftd.column".to_string(),
                full_name: s("foo/bar#mobile-display"),
                arguments: std::array::IntoIter::new([(
                    s("id"),
                    crate::p2::Kind::optional(ftd::p2::Kind::string()),
                )])
                .collect(),
                properties: std::array::IntoIter::new([(
                    s("id"),
                    ftd::component::Property {
                        default: Some(crate::PropertyValue::Argument {
                            name: "id".to_string(),
                            kind: crate::p2::Kind::Optional {
                                kind: Box::new(crate::p2::Kind::string()),
                            },
                        }),
                        conditions: vec![],
                    },
                )])
                .collect(),
                instructions: vec![crate::component::Instruction::ChildComponent {
                    child: crate::component::ChildComponent {
                        is_recursive: false,
                        events: vec![],
                        root: "ftd#text".to_string(),
                        condition: None,
                        properties: std::array::IntoIter::new([(
                            s("text"),
                            crate::component::Property {
                                default: Some(crate::PropertyValue::Value {
                                    value: crate::variable::Value::String {
                                        text: s("Mobile Display"),
                                        source: ftd::TextSource::Caption,
                                    },
                                }),
                                conditions: vec![],
                            },
                        )])
                        .collect(),
                    },
                }],
                invocations: vec![std::array::IntoIter::new([(
                    s("id"),
                    crate::Value::String {
                        text: s("some-child"),
                        source: crate::TextSource::Header,
                    },
                )])
                .collect()],
                ..Default::default()
            }),
        );

        let (g_bag, g_col) = crate::p2::interpreter::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- component mobile-display:
                component: ftd.column
                $id: optional string
                id: ref $id

                --- ftd.text: Mobile Display

                -- component desktop-display:
                component: ftd.column
                $id: optional string
                id: ref $id

                --- ftd.text: Desktop Display

                -- var mobile: true

                -- component foo:
                open: some-child
                component: ftd.column

                --- mobile-display:
                if: mobile
                id: some-child

                --- desktop-display:
                if: not mobile
                id: some-child

                -- ftd.text: Start Browser

                -- ftd.column:
                id: c1

                -- ftd.column:
                id: c2

                -- foo:

                -- ftd.text: hello

                -- ftd.text: hello1
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_bag, bag);
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn nested_open_container() {
        let mut external_children = super::default_column();
        external_children.container.children = vec![
            ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("hello"),
                line: true,
                ..Default::default()
            }),
            ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("hello again"),
                line: true,
                ..Default::default()
            }),
        ];

        let mut main = super::default_column();
        main.container
            .children
            .push(ftd_rt::Element::Column(ftd_rt::Column {
                container: ftd_rt::Container {
                    children: vec![ftd_rt::Element::Column(ftd_rt::Column {
                        container: ftd_rt::Container {
                            children: vec![
                                ftd_rt::Element::Column(ftd_rt::Column {
                                    container: ftd_rt::Container {
                                        children: vec![ftd_rt::Element::Column(ftd_rt::Column {
                                            container: ftd_rt::Container {
                                                children: vec![],
                                                ..Default::default()
                                            },
                                            common: ftd_rt::Common {
                                                id: Some(s("desktop-container")),
                                                ..Default::default()
                                            },
                                        })],
                                        external_children: Some((
                                            s("desktop-container"),
                                            vec![vec![0]],
                                            vec![],
                                        )),
                                        open: (None, Some(s("desktop-container"))),
                                        ..Default::default()
                                    },
                                    common: ftd_rt::Common {
                                        condition: Some(ftd_rt::Condition {
                                            variable: s("foo/bar#is-mobile"),
                                            value: s("false"),
                                        }),
                                        id: Some(s("main-container")),
                                        ..Default::default()
                                    },
                                }),
                                ftd_rt::Element::Column(ftd_rt::Column {
                                    container: ftd_rt::Container {
                                        children: vec![ftd_rt::Element::Column(ftd_rt::Column {
                                            common: ftd_rt::Common {
                                                id: Some(s("mobile-container")),
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        })],
                                        external_children: Some((
                                            s("mobile-container"),
                                            vec![vec![0]],
                                            vec![],
                                        )),
                                        open: (None, Some(s("mobile-container"))),
                                        ..Default::default()
                                    },
                                    common: ftd_rt::Common {
                                        condition: Some(ftd_rt::Condition {
                                            variable: s("foo/bar#is-mobile"),
                                            value: s("true"),
                                        }),
                                        id: Some(s("main-container")),
                                        ..Default::default()
                                    },
                                }),
                            ],
                            ..Default::default()
                        },
                        common: ftd_rt::Common {
                            id: Some(s("start")),
                            ..Default::default()
                        },
                    })],
                    external_children: Some((
                        s("main-container"),
                        vec![vec![0, 0], vec![0, 1]],
                        vec![ftd_rt::Element::Column(external_children)],
                    )),
                    open: (None, Some(s("main-container"))),
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (_g_bag, g_col) = crate::p2::interpreter::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- component desktop:
                component: ftd.column
                open: desktop-container

                --- ftd.column:
                id: desktop-container

                -- component mobile:
                component: ftd.column
                open: mobile-container

                --- ftd.column:
                id: mobile-container

                -- var is-mobile: true

                -- component page:
                component: ftd.column
                open: main-container

                --- ftd.column:
                id: start

                --- desktop:
                if: not is-mobile
                id: main-container

                --- container: start

                --- mobile:
                if: is-mobile
                id: main-container

                -- page:

                -- ftd.text: hello

                -- ftd.text: hello again
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn deep_open_container_call() {
        let mut external_children = super::default_column();
        external_children.container.children = vec![
            ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("hello"),
                line: true,
                ..Default::default()
            }),
            ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("hello again"),
                line: true,
                ..Default::default()
            }),
        ];

        let mut main = super::default_column();

        main.container
            .children
            .push(ftd_rt::Element::Column(ftd_rt::Column {
                container: ftd_rt::Container {
                    children: vec![
                        ftd_rt::Element::Column(ftd_rt::Column {
                            container: ftd_rt::Container {
                                children: vec![ftd_rt::Element::Column(ftd_rt::Column {
                                    common: ftd_rt::Common {
                                        id: Some(s("foo")),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                })],
                                ..Default::default()
                            },
                            common: ftd_rt::Common {
                                condition: Some(ftd_rt::Condition {
                                    variable: s("foo/bar#is-mobile"),
                                    value: s("false"),
                                }),
                                id: Some(s("main-container")),
                                ..Default::default()
                            },
                        }),
                        ftd_rt::Element::Column(ftd_rt::Column {
                            container: ftd_rt::Container {
                                children: vec![ftd_rt::Element::Column(ftd_rt::Column {
                                    common: ftd_rt::Common {
                                        id: Some(s("foo")),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                })],
                                ..Default::default()
                            },
                            common: ftd_rt::Common {
                                condition: Some(ftd_rt::Condition {
                                    variable: s("foo/bar#is-mobile"),
                                    value: s("true"),
                                }),
                                id: Some(s("main-container")),
                                ..Default::default()
                            },
                        }),
                    ],
                    external_children: Some((
                        s("foo"),
                        vec![vec![0, 0], vec![1, 0]],
                        vec![ftd_rt::Element::Column(external_children)],
                    )),
                    open: (None, Some(s("main-container.foo"))),
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (_g_bag, g_col) = crate::p2::interpreter::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- component desktop:
                component: ftd.column
                $id: optional string
                id: ref $id

                --- ftd.column:
                id: foo

                -- component mobile:
                component: ftd.column
                $id: optional string
                id: ref $id

                --- ftd.column:
                id: foo

                -- var is-mobile: true

                -- component page:
                component: ftd.column
                open: main-container.foo

                --- desktop:
                if: not is-mobile
                id: main-container

                --- mobile:
                if: is-mobile
                id: main-container

                -- page:

                -- ftd.text: hello

                -- ftd.text: hello again
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn deep_nested_open_container_call() {
        let mut nested_external_children = super::default_column();
        nested_external_children.container.children = vec![
            ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("hello"),
                line: true,
                ..Default::default()
            }),
            ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("hello again"),
                line: true,
                ..Default::default()
            }),
        ];

        let mut external_children = super::default_column();
        external_children.container.children = vec![ftd_rt::Element::Column(ftd_rt::Column {
            container: ftd_rt::Container {
                children: vec![ftd_rt::Element::Row(ftd_rt::Row {
                    container: ftd_rt::Container {
                        children: vec![ftd_rt::Element::Column(ftd_rt::Column {
                            common: ftd_rt::Common {
                                id: Some(s("foo")),
                                ..Default::default()
                            },
                            ..Default::default()
                        })],
                        ..Default::default()
                    },
                    common: ftd_rt::Common {
                        id: Some(s("desktop-container")),
                        ..Default::default()
                    },
                })],
                external_children: Some((
                    s("desktop-container"),
                    vec![vec![0]],
                    vec![ftd_rt::Element::Column(nested_external_children)],
                )),
                open: (None, Some(s("desktop-container"))),
                ..Default::default()
            },
            ..Default::default()
        })];

        let mut main = super::default_column();
        main.container
            .children
            .push(ftd_rt::Element::Column(ftd_rt::Column {
                container: ftd_rt::Container {
                    children: vec![
                        ftd_rt::Element::Column(ftd_rt::Column {
                            container: ftd_rt::Container {
                                children: vec![ftd_rt::Element::Row(ftd_rt::Row {
                                    container: ftd_rt::Container {
                                        children: vec![ftd_rt::Element::Column(ftd_rt::Column {
                                            common: ftd_rt::Common {
                                                id: Some(s("foo")),
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        })],
                                        ..Default::default()
                                    },
                                    common: ftd_rt::Common {
                                        id: Some(s("desktop-container")),
                                        ..Default::default()
                                    },
                                })],
                                external_children: Some((
                                    s("desktop-container"),
                                    vec![vec![0]],
                                    vec![],
                                )),
                                open: (None, Some(s("desktop-container"))),
                                ..Default::default()
                            },
                            common: ftd_rt::Common {
                                condition: Some(ftd_rt::Condition {
                                    variable: s("foo/bar#is-mobile"),
                                    value: s("false"),
                                }),
                                id: Some(s("main-container")),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd_rt::Element::Column(ftd_rt::Column {
                            container: ftd_rt::Container {
                                children: vec![ftd_rt::Element::Row(ftd_rt::Row {
                                    container: ftd_rt::Container {
                                        children: vec![ftd_rt::Element::Column(ftd_rt::Column {
                                            common: ftd_rt::Common {
                                                id: Some(s("foo")),
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        })],
                                        ..Default::default()
                                    },
                                    common: ftd_rt::Common {
                                        id: Some(s("mobile-container")),
                                        ..Default::default()
                                    },
                                })],
                                external_children: Some((
                                    s("mobile-container"),
                                    vec![vec![0]],
                                    vec![],
                                )),
                                open: (None, Some(s("mobile-container"))),
                                ..Default::default()
                            },
                            common: ftd_rt::Common {
                                condition: Some(ftd_rt::Condition {
                                    variable: s("foo/bar#is-mobile"),
                                    value: s("true"),
                                }),
                                id: Some(s("main-container")),
                                ..Default::default()
                            },
                        }),
                    ],
                    external_children: Some((
                        s("foo"),
                        vec![vec![0, 0, 0], vec![1, 0, 0]],
                        vec![ftd_rt::Element::Column(external_children)],
                    )),
                    open: (None, Some(s("main-container.foo"))),
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (_g_bag, g_col) = crate::p2::interpreter::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- component ft_container:
                component: ftd.column
                $id: optional string
                id: ref $id

                -- component ft_container_mobile:
                component: ftd.column
                $id: optional string
                id: ref $id


                -- component desktop:
                component: ftd.column
                open: desktop-container
                $id: optional string
                id: ref $id

                --- ftd.row:
                id: desktop-container

                --- ft_container:
                id: foo



                -- component mobile:
                component: ftd.column
                open: mobile-container
                $id: optional string
                id: ref $id

                --- ftd.row:
                id: mobile-container

                --- ft_container_mobile:
                id: foo


                -- var is-mobile: false


                -- component page:
                component: ftd.column
                open: main-container.foo

                --- desktop:
                if: not is-mobile
                id: main-container

                --- container: ftd.main

                --- mobile:
                if: is-mobile
                id: main-container



                -- page:

                -- desktop:

                -- ftd.text: hello

                -- ftd.text: hello again

                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn invalid_deep_open_container() {
        let mut external_children = super::default_column();
        external_children.container.children = vec![
            ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("hello"),
                line: true,
                ..Default::default()
            }),
            ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("hello again"),
                line: true,
                ..Default::default()
            }),
        ];

        let mut main = super::default_column();
        main.container
            .children
            .push(ftd_rt::Element::Column(ftd_rt::Column {
                container: ftd_rt::Container {
                    children: vec![ftd_rt::Element::Column(ftd_rt::Column {
                        container: ftd_rt::Container {
                            children: vec![
                                ftd_rt::Element::Column(ftd_rt::Column {
                                    container: ftd_rt::Container {
                                        children: vec![ftd_rt::Element::Column(ftd_rt::Column {
                                            container: ftd_rt::Container {
                                                children: vec![],
                                                ..Default::default()
                                            },
                                            common: ftd_rt::Common {
                                                id: Some(s("main-container")),
                                                ..Default::default()
                                            },
                                        })],
                                        ..Default::default()
                                    },
                                    common: ftd_rt::Common {
                                        condition: Some(ftd_rt::Condition {
                                            variable: s("foo/bar#is-mobile"),
                                            value: s("false"),
                                        }),
                                        ..Default::default()
                                    },
                                }),
                                ftd_rt::Element::Column(ftd_rt::Column {
                                    container: ftd_rt::Container {
                                        children: vec![ftd_rt::Element::Column(ftd_rt::Column {
                                            common: ftd_rt::Common {
                                                id: Some(s("main-container")),
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        })],
                                        ..Default::default()
                                    },
                                    common: ftd_rt::Common {
                                        condition: Some(ftd_rt::Condition {
                                            variable: s("foo/bar#is-mobile"),
                                            value: s("true"),
                                        }),
                                        ..Default::default()
                                    },
                                }),
                            ],
                            ..Default::default()
                        },
                        common: ftd_rt::Common {
                            id: Some(s("start")),
                            ..Default::default()
                        },
                    })],
                    external_children: Some((
                        s("main-container"),
                        vec![],
                        vec![ftd_rt::Element::Column(external_children)],
                    )),
                    open: (None, Some(s("main-container"))),
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (_g_bag, g_col) = crate::p2::interpreter::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- component desktop:
                component: ftd.column
                $id: optional string
                id: ref $id

                --- ftd.column:
                id: main-container

                -- component mobile:
                component: ftd.column
                $id: optional string
                id: ref $id

                --- ftd.column:
                id: main-container

                -- var is-mobile: true

                -- component page:
                component: ftd.column
                open: main-container

                --- ftd.column:
                id: start

                --- desktop:
                if: not is-mobile

                --- container: start

                --- mobile:
                if: is-mobile

                -- page:

                -- ftd.text: hello

                -- ftd.text: hello again
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

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
                    external_children: Default::default(),
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

    #[test]
    fn submit() {
        let mut main = super::default_column();

        main.container
            .children
            .push(ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("hello"),
                line: true,
                common: ftd_rt::Common {
                    submit: Some("https://httpbin.org/post?x=10".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (g_bag, g_col) = crate::p2::interpreter::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- ftd.text: hello
                submit: https://httpbin.org/post?x=10
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");
        pretty_assertions::assert_eq!(g_bag, super::default_bag());
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn basic_loop_on_record() {
        let mut main = super::default_column();
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
                        ftd_rt::Element::Text(ftd_rt::Text {
                            text: ftd::markdown_line("world"),
                            line: true,
                            ..Default::default()
                        }),
                    ],
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd_rt::Element::Row(ftd_rt::Row {
                container: ftd_rt::Container {
                    children: vec![
                        ftd_rt::Element::Text(ftd_rt::Text {
                            text: ftd::markdown_line("Arpita Jaiswal"),
                            line: true,
                            ..Default::default()
                        }),
                        ftd_rt::Element::Text(ftd_rt::Text {
                            text: ftd::markdown("Arpita is developer at Fifthtry"),
                            ..Default::default()
                        }),
                    ],
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd_rt::Element::Row(ftd_rt::Row {
                container: ftd_rt::Container {
                    children: vec![
                        ftd_rt::Element::Text(ftd_rt::Text {
                            text: ftd::markdown_line("Amit Upadhyay"),
                            line: true,
                            ..Default::default()
                        }),
                        ftd_rt::Element::Text(ftd_rt::Text {
                            text: ftd::markdown("Amit is CEO of FifthTry."),
                            ..Default::default()
                        }),
                    ],
                    ..Default::default()
                },
                ..Default::default()
            }));

        let mut bag = super::default_bag();

        bag.insert(
            "foo/bar#foo".to_string(),
            crate::p2::Thing::Component(crate::Component {
                root: "ftd.row".to_string(),
                full_name: s("foo/bar#foo"),
                arguments: std::array::IntoIter::new([
                    (s("body"), crate::p2::Kind::string()),
                    (s("name"), crate::p2::Kind::caption()),
                ])
                .collect(),
                instructions: vec![
                    crate::component::Instruction::ChildComponent {
                        child: crate::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "ftd#text".to_string(),
                            condition: None,
                            properties: std::array::IntoIter::new([(
                                s("text"),
                                crate::component::Property {
                                    default: Some(crate::PropertyValue::Argument {
                                        name: "name".to_string(),
                                        kind: crate::p2::Kind::caption_or_body(),
                                    }),
                                    conditions: vec![],
                                },
                            )])
                            .collect(),
                        },
                    },
                    crate::component::Instruction::ChildComponent {
                        child: crate::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "ftd#text".to_string(),
                            condition: None,
                            properties: std::array::IntoIter::new([(
                                s("text"),
                                crate::component::Property {
                                    default: Some(crate::PropertyValue::Argument {
                                        name: "body".to_string(),
                                        kind: crate::p2::Kind::caption_or_body(),
                                    }),
                                    conditions: vec![],
                                },
                            )])
                            .collect(),
                        },
                    },
                ],
                invocations: vec![
                    std::array::IntoIter::new([
                        (
                            s("body"),
                            crate::Value::String {
                                text: s("world"),
                                source: crate::TextSource::Caption,
                            },
                        ),
                        (
                            s("name"),
                            crate::Value::String {
                                text: s("hello"),
                                source: crate::TextSource::Caption,
                            },
                        ),
                    ])
                    .collect(),
                    std::array::IntoIter::new([
                        (
                            s("body"),
                            crate::Value::String {
                                text: s("Arpita is developer at Fifthtry"),
                                source: crate::TextSource::Body,
                            },
                        ),
                        (
                            s("name"),
                            crate::Value::String {
                                text: s("Arpita Jaiswal"),
                                source: crate::TextSource::Caption,
                            },
                        ),
                    ])
                    .collect(),
                    std::array::IntoIter::new([
                        (
                            s("body"),
                            crate::Value::String {
                                text: s("Amit is CEO of FifthTry."),
                                source: crate::TextSource::Body,
                            },
                        ),
                        (
                            s("name"),
                            crate::Value::String {
                                text: s("Amit Upadhyay"),
                                source: crate::TextSource::Caption,
                            },
                        ),
                    ])
                    .collect(),
                ],
                ..Default::default()
            }),
        );

        bag.insert(
            "foo/bar#get".to_string(),
            crate::p2::Thing::Variable(crate::Variable {
                name: "get".to_string(),
                value: crate::Value::String {
                    text: "world".to_string(),
                    source: crate::TextSource::Caption,
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            "foo/bar#name".to_string(),
            crate::p2::Thing::Variable(crate::Variable {
                name: "name".to_string(),
                value: crate::Value::String {
                    text: "Arpita Jaiswal".to_string(),
                    source: crate::TextSource::Caption,
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            "foo/bar#people".to_string(),
            crate::p2::Thing::Variable(crate::Variable {
                name: "foo/bar#people".to_string(),
                value: crate::Value::List {
                    data: vec![
                        crate::Value::Record {
                            name: "foo/bar#person".to_string(),
                            fields: std::array::IntoIter::new([
                                (
                                    s("bio"),
                                    crate::PropertyValue::Value {
                                        value: crate::Value::String {
                                            text: "Arpita is developer at Fifthtry".to_string(),
                                            source: crate::TextSource::Body,
                                        },
                                    },
                                ),
                                (
                                    s("name"),
                                    crate::PropertyValue::Reference {
                                        name: "foo/bar#name".to_string(),
                                        kind: crate::p2::Kind::caption(),
                                    },
                                ),
                            ])
                            .collect(),
                        },
                        crate::Value::Record {
                            name: "foo/bar#person".to_string(),
                            fields: std::array::IntoIter::new([
                                (
                                    s("bio"),
                                    crate::PropertyValue::Value {
                                        value: crate::Value::String {
                                            text: "Amit is CEO of FifthTry.".to_string(),
                                            source: crate::TextSource::Body,
                                        },
                                    },
                                ),
                                (
                                    s("name"),
                                    crate::PropertyValue::Value {
                                        value: crate::Value::String {
                                            text: "Amit Upadhyay".to_string(),
                                            source: crate::TextSource::Caption,
                                        },
                                    },
                                ),
                            ])
                            .collect(),
                        },
                    ],
                    kind: crate::p2::Kind::Record {
                        name: "foo/bar#person".to_string(),
                    },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            "foo/bar#person".to_string(),
            crate::p2::Thing::Record(crate::p2::Record {
                name: "foo/bar#person".to_string(),
                fields: std::array::IntoIter::new([
                    (s("bio"), crate::p2::Kind::body()),
                    (s("name"), crate::p2::Kind::caption()),
                ])
                .collect(),
                instances: Default::default(),
            }),
        );

        let (g_bag, g_col) = crate::p2::interpreter::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- component foo:
                component: ftd.row
                $name: caption
                $body: string

                --- ftd.text: ref $name

                --- ftd.text: ref $body

                -- record person:
                name: caption
                bio: body

                -- list people:
                type: person

                -- var name: Arpita Jaiswal

                -- people: ref name

                Arpita is developer at Fifthtry

                -- people: Amit Upadhyay

                Amit is CEO of FifthTry.

                -- var get: world

                -- foo: hello
                body: ref get

                -- foo: ref obj.name
                $loop$: people as obj
                body: ref obj.bio
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");
        pretty_assertions::assert_eq!(g_bag, bag);
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn basic_loop_on_record_with_if_condition() {
        let mut main = super::default_column();
        main.container.children.push(ftd_rt::Element::Null);

        main.container
            .children
            .push(ftd_rt::Element::Row(ftd_rt::Row {
                container: ftd_rt::Container {
                    children: vec![
                        ftd_rt::Element::Text(ftd_rt::Text {
                            text: ftd::markdown_line("Amit Upadhyay"),
                            line: true,
                            ..Default::default()
                        }),
                        ftd_rt::Element::Text(ftd_rt::Text {
                            text: ftd::markdown("Amit is CEO of FifthTry."),
                            ..Default::default()
                        }),
                    ],
                    ..Default::default()
                },
                ..Default::default()
            }));

        let mut bag = super::default_bag();

        bag.insert(
            "foo/bar#foo".to_string(),
            crate::p2::Thing::Component(crate::Component {
                root: "ftd.row".to_string(),
                full_name: s("foo/bar#foo"),
                arguments: std::array::IntoIter::new([
                    (s("body"), crate::p2::Kind::string()),
                    (s("name"), crate::p2::Kind::caption()),
                ])
                .collect(),
                instructions: vec![
                    crate::component::Instruction::ChildComponent {
                        child: crate::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "ftd#text".to_string(),
                            condition: None,
                            properties: std::array::IntoIter::new([(
                                s("text"),
                                crate::component::Property {
                                    default: Some(crate::PropertyValue::Argument {
                                        name: "name".to_string(),
                                        kind: crate::p2::Kind::caption_or_body(),
                                    }),
                                    conditions: vec![],
                                },
                            )])
                            .collect(),
                        },
                    },
                    crate::component::Instruction::ChildComponent {
                        child: crate::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "ftd#text".to_string(),
                            condition: None,
                            properties: std::array::IntoIter::new([(
                                s("text"),
                                crate::component::Property {
                                    default: Some(crate::PropertyValue::Argument {
                                        name: "body".to_string(),
                                        kind: crate::p2::Kind::caption_or_body(),
                                    }),
                                    conditions: vec![],
                                },
                            )])
                            .collect(),
                        },
                    },
                ],
                invocations: vec![std::array::IntoIter::new([
                    (
                        s("body"),
                        crate::Value::String {
                            text: s("Amit is CEO of FifthTry."),
                            source: crate::TextSource::Body,
                        },
                    ),
                    (
                        s("name"),
                        crate::Value::String {
                            text: s("Amit Upadhyay"),
                            source: crate::TextSource::Caption,
                        },
                    ),
                ])
                .collect()],
                ..Default::default()
            }),
        );

        bag.insert(
            "foo/bar#people".to_string(),
            crate::p2::Thing::Variable(crate::Variable {
                name: "foo/bar#people".to_string(),
                value: crate::Value::List {
                    data: vec![
                        crate::Value::Record {
                            name: "foo/bar#person".to_string(),
                            fields: std::array::IntoIter::new([
                                (
                                    s("bio"),
                                    crate::PropertyValue::Value {
                                        value: crate::Value::String {
                                            text: "Arpita is developer at Fifthtry".to_string(),
                                            source: crate::TextSource::Body,
                                        },
                                    },
                                ),
                                (
                                    s("ceo"),
                                    crate::PropertyValue::Value {
                                        value: crate::Value::Boolean { value: false },
                                    },
                                ),
                                (
                                    s("name"),
                                    crate::PropertyValue::Value {
                                        value: crate::Value::String {
                                            text: "Arpita Jaiswal".to_string(),
                                            source: crate::TextSource::Caption,
                                        },
                                    },
                                ),
                            ])
                            .collect(),
                        },
                        crate::Value::Record {
                            name: "foo/bar#person".to_string(),
                            fields: std::array::IntoIter::new([
                                (
                                    s("bio"),
                                    crate::PropertyValue::Value {
                                        value: crate::Value::String {
                                            text: "Amit is CEO of FifthTry.".to_string(),
                                            source: crate::TextSource::Body,
                                        },
                                    },
                                ),
                                (
                                    s("ceo"),
                                    crate::PropertyValue::Value {
                                        value: crate::Value::Boolean { value: true },
                                    },
                                ),
                                (
                                    s("name"),
                                    crate::PropertyValue::Value {
                                        value: crate::Value::String {
                                            text: "Amit Upadhyay".to_string(),
                                            source: crate::TextSource::Caption,
                                        },
                                    },
                                ),
                            ])
                            .collect(),
                        },
                    ],
                    kind: crate::p2::Kind::Record {
                        name: "foo/bar#person".to_string(),
                    },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            "foo/bar#person".to_string(),
            crate::p2::Thing::Record(crate::p2::Record {
                name: "foo/bar#person".to_string(),
                fields: std::array::IntoIter::new([
                    (s("bio"), crate::p2::Kind::body()),
                    (s("name"), crate::p2::Kind::caption()),
                    (s("ceo"), crate::p2::Kind::boolean()),
                ])
                .collect(),
                instances: Default::default(),
            }),
        );

        let (g_bag, g_col) = crate::p2::interpreter::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- component foo:
                component: ftd.row
                $name: caption
                $body: string

                --- ftd.text: ref $name

                --- ftd.text: ref $body

                -- record person:
                name: caption
                bio: body
                ceo: boolean

                -- list people:
                type: person

                -- people: Arpita Jaiswal
                ceo: false

                Arpita is developer at Fifthtry

                -- people: Amit Upadhyay
                ceo: true

                Amit is CEO of FifthTry.

                -- foo: ref obj.name
                $loop$: people as obj
                if: obj.ceo
                body: ref obj.bio
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");
        pretty_assertions::assert_eq!(g_bag, bag);
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn basic_loop_on_string() {
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("Arpita"),
                line: true,
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("Asit"),
                line: true,
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("Sourabh"),
                line: true,
                ..Default::default()
            }));

        let mut bag = super::default_bag();

        bag.insert(
            "foo/bar#people".to_string(),
            crate::p2::Thing::Variable(crate::Variable {
                name: "foo/bar#people".to_string(),
                value: crate::Value::List {
                    data: vec![
                        crate::Value::String {
                            text: "Arpita".to_string(),
                            source: crate::TextSource::Caption,
                        },
                        crate::Value::String {
                            text: "Asit".to_string(),
                            source: crate::TextSource::Caption,
                        },
                        crate::Value::String {
                            text: "Sourabh".to_string(),
                            source: crate::TextSource::Caption,
                        },
                    ],
                    kind: crate::p2::Kind::string(),
                },
                conditions: vec![],
            }),
        );
        let (g_bag, g_col) = crate::p2::interpreter::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- list people:
                type: string

                -- people: Arpita

                -- people: Asit

                -- people: Sourabh

                -- ftd.text: ref obj
                $loop$: people as obj
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");
        pretty_assertions::assert_eq!(g_bag, bag);
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn loop_inside_subsection() {
        let mut main = super::default_column();
        let mut col = ftd_rt::Column {
            ..Default::default()
        };

        col.container
            .children
            .push(ftd_rt::Element::Row(ftd_rt::Row {
                container: ftd_rt::Container {
                    children: vec![
                        ftd_rt::Element::Text(ftd_rt::Text {
                            text: ftd::markdown_line("Arpita Jaiswal"),
                            line: true,
                            ..Default::default()
                        }),
                        ftd_rt::Element::Text(ftd_rt::Text {
                            text: ftd::markdown("Arpita is developer at Fifthtry"),
                            ..Default::default()
                        }),
                    ],
                    ..Default::default()
                },
                ..Default::default()
            }));

        col.container
            .children
            .push(ftd_rt::Element::Row(ftd_rt::Row {
                container: ftd_rt::Container {
                    children: vec![
                        ftd_rt::Element::Text(ftd_rt::Text {
                            text: ftd::markdown_line("Amit Upadhyay"),
                            line: true,
                            ..Default::default()
                        }),
                        ftd_rt::Element::Text(ftd_rt::Text {
                            text: ftd::markdown("Amit is CEO of FifthTry."),
                            ..Default::default()
                        }),
                    ],
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container.children.push(ftd_rt::Element::Column(col));

        let mut bag = super::default_bag();

        bag.insert(
            "foo/bar#foo".to_string(),
            crate::p2::Thing::Component(crate::Component {
                root: "ftd.row".to_string(),
                full_name: s("foo/bar#foo"),
                arguments: std::array::IntoIter::new([
                    (s("body"), crate::p2::Kind::string()),
                    (s("name"), crate::p2::Kind::caption()),
                ])
                .collect(),
                instructions: vec![
                    crate::component::Instruction::ChildComponent {
                        child: crate::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "ftd#text".to_string(),
                            condition: None,
                            properties: std::array::IntoIter::new([(
                                s("text"),
                                crate::component::Property {
                                    default: Some(crate::PropertyValue::Argument {
                                        name: "name".to_string(),
                                        kind: crate::p2::Kind::caption_or_body(),
                                    }),
                                    conditions: vec![],
                                },
                            )])
                            .collect(),
                        },
                    },
                    crate::component::Instruction::ChildComponent {
                        child: crate::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "ftd#text".to_string(),
                            condition: None,
                            properties: std::array::IntoIter::new([(
                                s("text"),
                                crate::component::Property {
                                    default: Some(crate::PropertyValue::Argument {
                                        name: "body".to_string(),
                                        kind: crate::p2::Kind::caption_or_body(),
                                    }),
                                    conditions: vec![],
                                },
                            )])
                            .collect(),
                        },
                    },
                ],
                invocations: vec![
                    std::array::IntoIter::new([
                        (
                            s("body"),
                            crate::Value::String {
                                text: s("Arpita is developer at Fifthtry"),
                                source: crate::TextSource::Body,
                            },
                        ),
                        (
                            s("name"),
                            crate::Value::String {
                                text: s("Arpita Jaiswal"),
                                source: crate::TextSource::Caption,
                            },
                        ),
                    ])
                    .collect(),
                    std::array::IntoIter::new([
                        (
                            s("body"),
                            crate::Value::String {
                                text: s("Amit is CEO of FifthTry."),
                                source: crate::TextSource::Body,
                            },
                        ),
                        (
                            s("name"),
                            crate::Value::String {
                                text: s("Amit Upadhyay"),
                                source: crate::TextSource::Caption,
                            },
                        ),
                    ])
                    .collect(),
                ],
                ..Default::default()
            }),
        );

        bag.insert(
            "foo/bar#people".to_string(),
            crate::p2::Thing::Variable(crate::Variable {
                name: "foo/bar#people".to_string(),
                value: crate::Value::List {
                    data: vec![
                        crate::Value::Record {
                            name: "foo/bar#person".to_string(),
                            fields: std::array::IntoIter::new([
                                (
                                    s("bio"),
                                    crate::PropertyValue::Value {
                                        value: crate::Value::String {
                                            text: "Arpita is developer at Fifthtry".to_string(),
                                            source: crate::TextSource::Body,
                                        },
                                    },
                                ),
                                (
                                    s("name"),
                                    crate::PropertyValue::Value {
                                        value: crate::Value::String {
                                            text: "Arpita Jaiswal".to_string(),
                                            source: crate::TextSource::Caption,
                                        },
                                    },
                                ),
                            ])
                            .collect(),
                        },
                        crate::Value::Record {
                            name: "foo/bar#person".to_string(),
                            fields: std::array::IntoIter::new([
                                (
                                    s("bio"),
                                    crate::PropertyValue::Value {
                                        value: crate::Value::String {
                                            text: "Amit is CEO of FifthTry.".to_string(),
                                            source: crate::TextSource::Body,
                                        },
                                    },
                                ),
                                (
                                    s("name"),
                                    crate::PropertyValue::Value {
                                        value: crate::Value::String {
                                            text: "Amit Upadhyay".to_string(),
                                            source: crate::TextSource::Caption,
                                        },
                                    },
                                ),
                            ])
                            .collect(),
                        },
                    ],
                    kind: crate::p2::Kind::Record {
                        name: "foo/bar#person".to_string(),
                    },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            "foo/bar#person".to_string(),
            crate::p2::Thing::Record(crate::p2::Record {
                name: "foo/bar#person".to_string(),
                fields: std::array::IntoIter::new([
                    (s("bio"), crate::p2::Kind::body()),
                    (s("name"), crate::p2::Kind::caption()),
                ])
                .collect(),
                instances: Default::default(),
            }),
        );

        let (_g_bag, g_col) = crate::p2::interpreter::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- component foo:
                component: ftd.row
                $name: caption
                $body: string

                --- ftd.text: ref $name

                --- ftd.text: ref $body

                -- record person:
                name: caption
                bio: body

                -- list people:
                type: person

                -- people: Arpita Jaiswal

                Arpita is developer at Fifthtry

                -- people: Amit Upadhyay

                Amit is CEO of FifthTry.

                -- ftd.column:

                --- foo: ref obj.name
                $loop$: people as obj
                body: ref obj.bio
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");
        // pretty_assertions::assert_eq!(g_bag, bag);
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn basic_processor() {
        let mut main = super::default_column();

        main.container
            .children
            .push(ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("\"0.1.4\""),
                line: true,
                common: ftd_rt::Common {
                    reference: Some(s("foo/bar#test")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        let mut bag = super::default_bag();

        bag.insert(
            "foo/bar#test".to_string(),
            crate::p2::Thing::Variable(crate::Variable {
                name: "test".to_string(),
                value: crate::Value::String {
                    text: "\"0.1.4\"".to_string(),
                    source: crate::TextSource::Header,
                },
                conditions: vec![],
            }),
        );

        let (g_bag, g_col) = crate::p2::interpreter::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- var test:
                $processor$: read_version_from_cargo_toml

                -- ftd.text: ref test
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");
        pretty_assertions::assert_eq!(g_bag, bag);
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn basic_processor_that_overwrites() {
        let mut main = super::default_column();

        main.container
            .children
            .push(ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("\"0.1.4\""),
                line: true,
                common: ftd_rt::Common {
                    reference: Some(s("foo/bar#test")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        let mut bag = super::default_bag();

        bag.insert(
            "foo/bar#test".to_string(),
            crate::p2::Thing::Variable(crate::Variable {
                name: "test".to_string(),
                value: crate::Value::String {
                    text: "\"0.1.4\"".to_string(),
                    source: crate::TextSource::Header,
                },
                conditions: vec![],
            }),
        );

        let (g_bag, g_col) = crate::p2::interpreter::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- var test: yo

                -- test:
                $processor$: read_version_from_cargo_toml

                -- ftd.text: ref test
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");
        pretty_assertions::assert_eq!(g_bag, bag);
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn basic_processor_for_list() {
        let mut main = super::default_column();

        main.container
            .children
            .push(ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("\"ftd\""),
                line: true,
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("\"0.1.4\""),
                line: true,
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("[\"Amit Upadhyay <upadhyay@gmail.com>\"]"),
                line: true,
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("\"2018\""),
                line: true,
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("\"ftd: FifthTry Document Format parser\""),
                line: true,
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("\"MIT\""),
                line: true,
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("\"https://github.com/fifthtry/ftd\""),
                line: true,
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("\"https://www.fifthtry.com/fifthtry/ftd/\""),
                line: true,
                ..Default::default()
            }));

        let mut bag = super::default_bag();

        bag.insert(
            "foo/bar#test".to_string(),
            crate::p2::Thing::Variable(crate::Variable {
                name: "foo/bar#test".to_string(),
                value: crate::Value::List {
                    data: vec![
                        crate::Value::String {
                            text: "\"ftd\"".to_string(),
                            source: crate::TextSource::Header,
                        },
                        crate::Value::String {
                            text: "\"0.1.4\"".to_string(),
                            source: crate::TextSource::Header,
                        },
                        crate::Value::String {
                            text: "[\"Amit Upadhyay <upadhyay@gmail.com>\"]".to_string(),
                            source: crate::TextSource::Header,
                        },
                        crate::Value::String {
                            text: "\"2018\"".to_string(),
                            source: crate::TextSource::Header,
                        },
                        crate::Value::String {
                            text: "\"ftd: FifthTry Document Format parser\"".to_string(),
                            source: crate::TextSource::Header,
                        },
                        crate::Value::String {
                            text: "\"MIT\"".to_string(),
                            source: crate::TextSource::Header,
                        },
                        crate::Value::String {
                            text: "\"https://github.com/fifthtry/ftd\"".to_string(),
                            source: crate::TextSource::Header,
                        },
                        crate::Value::String {
                            text: "\"https://www.fifthtry.com/fifthtry/ftd/\"".to_string(),
                            source: crate::TextSource::Header,
                        },
                    ],
                    kind: crate::p2::Kind::string(),
                },
                conditions: vec![],
            }),
        );

        let (g_bag, g_col) = crate::p2::interpreter::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- list test:
                type: string
                $processor$: read_package_from_cargo_toml

                -- ftd.text: ref obj
                $loop$: test as obj
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_bag, bag);
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn processor_for_list_of_record() {
        let mut main = super::default_column();

        main.container
            .children
            .push(ftd_rt::Element::Row(ftd_rt::Row {
                container: ftd_rt::Container {
                    children: vec![
                        ftd_rt::Element::Text(ftd_rt::Text {
                            text: ftd::markdown_line("\"ftd\""),
                            line: true,
                            ..Default::default()
                        }),
                        ftd_rt::Element::Text(ftd_rt::Text {
                            text: ftd::markdown_line("name"),
                            line: true,
                            ..Default::default()
                        }),
                    ],
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd_rt::Element::Row(ftd_rt::Row {
                container: ftd_rt::Container {
                    children: vec![
                        ftd_rt::Element::Text(ftd_rt::Text {
                            text: ftd::markdown_line("\"0.1.4\""),
                            line: true,
                            ..Default::default()
                        }),
                        ftd_rt::Element::Text(ftd_rt::Text {
                            text: ftd::markdown_line("version"),
                            line: true,
                            ..Default::default()
                        }),
                    ],
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd_rt::Element::Row(ftd_rt::Row {
                container: ftd_rt::Container {
                    children: vec![
                        ftd_rt::Element::Text(ftd_rt::Text {
                            text: ftd::markdown_line("[\"Amit Upadhyay <upadhyay@gmail.com>\"]"),
                            line: true,
                            ..Default::default()
                        }),
                        ftd_rt::Element::Text(ftd_rt::Text {
                            text: ftd::markdown_line("authors"),
                            line: true,
                            ..Default::default()
                        }),
                    ],
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd_rt::Element::Row(ftd_rt::Row {
                container: ftd_rt::Container {
                    children: vec![
                        ftd_rt::Element::Text(ftd_rt::Text {
                            text: ftd::markdown_line("\"2018\""),
                            line: true,
                            ..Default::default()
                        }),
                        ftd_rt::Element::Text(ftd_rt::Text {
                            text: ftd::markdown_line("edition"),
                            line: true,
                            ..Default::default()
                        }),
                    ],
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd_rt::Element::Row(ftd_rt::Row {
                container: ftd_rt::Container {
                    children: vec![
                        ftd_rt::Element::Text(ftd_rt::Text {
                            text: ftd::markdown_line("\"ftd: FifthTry Document Format parser\""),
                            line: true,
                            ..Default::default()
                        }),
                        ftd_rt::Element::Text(ftd_rt::Text {
                            text: ftd::markdown_line("description"),
                            line: true,
                            ..Default::default()
                        }),
                    ],
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd_rt::Element::Row(ftd_rt::Row {
                container: ftd_rt::Container {
                    children: vec![
                        ftd_rt::Element::Text(ftd_rt::Text {
                            text: ftd::markdown_line("\"MIT\""),
                            line: true,
                            ..Default::default()
                        }),
                        ftd_rt::Element::Text(ftd_rt::Text {
                            text: ftd::markdown_line("license"),
                            line: true,
                            ..Default::default()
                        }),
                    ],
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd_rt::Element::Row(ftd_rt::Row {
                container: ftd_rt::Container {
                    children: vec![
                        ftd_rt::Element::Text(ftd_rt::Text {
                            text: ftd::markdown_line("\"https://github.com/fifthtry/ftd\""),
                            line: true,
                            ..Default::default()
                        }),
                        ftd_rt::Element::Text(ftd_rt::Text {
                            text: ftd::markdown_line("repository"),
                            line: true,
                            ..Default::default()
                        }),
                    ],
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd_rt::Element::Row(ftd_rt::Row {
                container: ftd_rt::Container {
                    children: vec![
                        ftd_rt::Element::Text(ftd_rt::Text {
                            text: ftd::markdown_line("\"https://www.fifthtry.com/fifthtry/ftd/\""),
                            line: true,
                            ..Default::default()
                        }),
                        ftd_rt::Element::Text(ftd_rt::Text {
                            text: ftd::markdown_line("homepage"),
                            line: true,
                            ..Default::default()
                        }),
                    ],
                    ..Default::default()
                },
                ..Default::default()
            }));

        let mut bag = super::default_bag();

        bag.insert(
            "foo/bar#data".to_string(),
            crate::p2::Thing::Record(crate::p2::Record {
                name: "foo/bar#data".to_string(),
                fields: std::array::IntoIter::new([
                    (s("description"), crate::p2::Kind::string()),
                    (s("title"), crate::p2::Kind::string()),
                ])
                .collect(),
                instances: Default::default(),
            }),
        );

        bag.insert(
            "foo/bar#foo".to_string(),
            crate::p2::Thing::Component(ftd::Component {
                root: "ftd.row".to_string(),
                full_name: "foo/bar#foo".to_string(),
                arguments: std::array::IntoIter::new([
                    (s("body"), crate::p2::Kind::string()),
                    (s("name"), crate::p2::Kind::caption()),
                ])
                .collect(),
                instructions: vec![
                    crate::component::Instruction::ChildComponent {
                        child: crate::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "ftd#text".to_string(),
                            condition: None,
                            properties: std::array::IntoIter::new([(
                                s("text"),
                                crate::component::Property {
                                    default: Some(crate::PropertyValue::Argument {
                                        name: "name".to_string(),
                                        kind: crate::p2::Kind::caption_or_body(),
                                    }),
                                    conditions: vec![],
                                },
                            )])
                            .collect(),
                        },
                    },
                    crate::component::Instruction::ChildComponent {
                        child: crate::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "ftd#text".to_string(),
                            condition: None,
                            properties: std::array::IntoIter::new([(
                                s("text"),
                                crate::component::Property {
                                    default: Some(crate::PropertyValue::Argument {
                                        name: "body".to_string(),
                                        kind: crate::p2::Kind::caption_or_body(),
                                    }),
                                    conditions: vec![],
                                },
                            )])
                            .collect(),
                        },
                    },
                ],
                invocations: vec![
                    std::array::IntoIter::new([
                        (
                            s("body"),
                            crate::Value::String {
                                text: s("name"),
                                source: crate::TextSource::Header,
                            },
                        ),
                        (
                            s("name"),
                            crate::Value::String {
                                text: s("\"ftd\""),
                                source: crate::TextSource::Header,
                            },
                        ),
                    ])
                    .collect(),
                    std::array::IntoIter::new([
                        (
                            s("body"),
                            crate::Value::String {
                                text: s("version"),
                                source: crate::TextSource::Header,
                            },
                        ),
                        (
                            s("name"),
                            crate::Value::String {
                                text: s("\"0.1.4\""),
                                source: crate::TextSource::Header,
                            },
                        ),
                    ])
                    .collect(),
                    std::array::IntoIter::new([
                        (
                            s("body"),
                            crate::Value::String {
                                text: s("authors"),
                                source: crate::TextSource::Header,
                            },
                        ),
                        (
                            s("name"),
                            crate::Value::String {
                                text: s("[\"Amit Upadhyay <upadhyay@gmail.com>\"]"),
                                source: crate::TextSource::Header,
                            },
                        ),
                    ])
                    .collect(),
                    std::array::IntoIter::new([
                        (
                            s("body"),
                            crate::Value::String {
                                text: s("edition"),
                                source: crate::TextSource::Header,
                            },
                        ),
                        (
                            s("name"),
                            crate::Value::String {
                                text: s("\"2018\""),
                                source: crate::TextSource::Header,
                            },
                        ),
                    ])
                    .collect(),
                    std::array::IntoIter::new([
                        (
                            s("body"),
                            crate::Value::String {
                                text: s("description"),
                                source: crate::TextSource::Header,
                            },
                        ),
                        (
                            s("name"),
                            crate::Value::String {
                                text: s("\"ftd: FifthTry Document Format parser\""),
                                source: crate::TextSource::Header,
                            },
                        ),
                    ])
                    .collect(),
                    std::array::IntoIter::new([
                        (
                            s("body"),
                            crate::Value::String {
                                text: s("license"),
                                source: crate::TextSource::Header,
                            },
                        ),
                        (
                            s("name"),
                            crate::Value::String {
                                text: s("\"MIT\""),
                                source: crate::TextSource::Header,
                            },
                        ),
                    ])
                    .collect(),
                    std::array::IntoIter::new([
                        (
                            s("body"),
                            crate::Value::String {
                                text: s("repository"),
                                source: crate::TextSource::Header,
                            },
                        ),
                        (
                            s("name"),
                            crate::Value::String {
                                text: s("\"https://github.com/fifthtry/ftd\""),
                                source: crate::TextSource::Header,
                            },
                        ),
                    ])
                    .collect(),
                    std::array::IntoIter::new([
                        (
                            s("body"),
                            crate::Value::String {
                                text: s("homepage"),
                                source: crate::TextSource::Header,
                            },
                        ),
                        (
                            s("name"),
                            crate::Value::String {
                                text: s("\"https://www.fifthtry.com/fifthtry/ftd/\""),
                                source: crate::TextSource::Header,
                            },
                        ),
                    ])
                    .collect(),
                ],
                ..Default::default()
            }),
        );

        bag.insert(
            "foo/bar#test".to_string(),
            crate::p2::Thing::Variable(ftd::Variable {
                name: "foo/bar#test".to_string(),
                value: crate::Value::List {
                    data: vec![
                        crate::Value::Record {
                            name: "foo/bar#data".to_string(),
                            fields: std::array::IntoIter::new([
                                (
                                    s("description"),
                                    crate::PropertyValue::Value {
                                        value: crate::variable::Value::String {
                                            text: "name".to_string(),
                                            source: crate::TextSource::Header,
                                        },
                                    },
                                ),
                                (
                                    s("title"),
                                    crate::PropertyValue::Value {
                                        value: crate::variable::Value::String {
                                            text: "\"ftd\"".to_string(),
                                            source: crate::TextSource::Header,
                                        },
                                    },
                                ),
                            ])
                            .collect(),
                        },
                        crate::Value::Record {
                            name: "foo/bar#data".to_string(),
                            fields: std::array::IntoIter::new([
                                (
                                    s("description"),
                                    crate::PropertyValue::Value {
                                        value: crate::variable::Value::String {
                                            text: "version".to_string(),
                                            source: crate::TextSource::Header,
                                        },
                                    },
                                ),
                                (
                                    s("title"),
                                    crate::PropertyValue::Value {
                                        value: crate::variable::Value::String {
                                            text: "\"0.1.4\"".to_string(),
                                            source: crate::TextSource::Header,
                                        },
                                    },
                                ),
                            ])
                            .collect(),
                        },
                        crate::Value::Record {
                            name: "foo/bar#data".to_string(),
                            fields: std::array::IntoIter::new([
                                (
                                    s("description"),
                                    crate::PropertyValue::Value {
                                        value: crate::variable::Value::String {
                                            text: "authors".to_string(),
                                            source: crate::TextSource::Header,
                                        },
                                    },
                                ),
                                (
                                    s("title"),
                                    crate::PropertyValue::Value {
                                        value: crate::variable::Value::String {
                                            text: "[\"Amit Upadhyay <upadhyay@gmail.com>\"]"
                                                .to_string(),
                                            source: crate::TextSource::Header,
                                        },
                                    },
                                ),
                            ])
                            .collect(),
                        },
                        crate::Value::Record {
                            name: "foo/bar#data".to_string(),
                            fields: std::array::IntoIter::new([
                                (
                                    s("description"),
                                    crate::PropertyValue::Value {
                                        value: crate::variable::Value::String {
                                            text: "edition".to_string(),
                                            source: crate::TextSource::Header,
                                        },
                                    },
                                ),
                                (
                                    s("title"),
                                    crate::PropertyValue::Value {
                                        value: crate::variable::Value::String {
                                            text: "\"2018\"".to_string(),
                                            source: crate::TextSource::Header,
                                        },
                                    },
                                ),
                            ])
                            .collect(),
                        },
                        crate::Value::Record {
                            name: "foo/bar#data".to_string(),
                            fields: std::array::IntoIter::new([
                                (
                                    s("description"),
                                    crate::PropertyValue::Value {
                                        value: crate::variable::Value::String {
                                            text: "description".to_string(),
                                            source: crate::TextSource::Header,
                                        },
                                    },
                                ),
                                (
                                    s("title"),
                                    crate::PropertyValue::Value {
                                        value: crate::variable::Value::String {
                                            text: "\"ftd: FifthTry Document Format parser\""
                                                .to_string(),
                                            source: crate::TextSource::Header,
                                        },
                                    },
                                ),
                            ])
                            .collect(),
                        },
                        crate::Value::Record {
                            name: "foo/bar#data".to_string(),
                            fields: std::array::IntoIter::new([
                                (
                                    s("description"),
                                    crate::PropertyValue::Value {
                                        value: crate::variable::Value::String {
                                            text: "license".to_string(),
                                            source: crate::TextSource::Header,
                                        },
                                    },
                                ),
                                (
                                    s("title"),
                                    crate::PropertyValue::Value {
                                        value: crate::variable::Value::String {
                                            text: "\"MIT\"".to_string(),
                                            source: crate::TextSource::Header,
                                        },
                                    },
                                ),
                            ])
                            .collect(),
                        },
                        crate::Value::Record {
                            name: "foo/bar#data".to_string(),
                            fields: std::array::IntoIter::new([
                                (
                                    s("description"),
                                    crate::PropertyValue::Value {
                                        value: crate::variable::Value::String {
                                            text: "repository".to_string(),
                                            source: crate::TextSource::Header,
                                        },
                                    },
                                ),
                                (
                                    s("title"),
                                    crate::PropertyValue::Value {
                                        value: crate::variable::Value::String {
                                            text: "\"https://github.com/fifthtry/ftd\"".to_string(),
                                            source: crate::TextSource::Header,
                                        },
                                    },
                                ),
                            ])
                            .collect(),
                        },
                        crate::Value::Record {
                            name: "foo/bar#data".to_string(),
                            fields: std::array::IntoIter::new([
                                (
                                    s("description"),
                                    crate::PropertyValue::Value {
                                        value: crate::variable::Value::String {
                                            text: "homepage".to_string(),
                                            source: crate::TextSource::Header,
                                        },
                                    },
                                ),
                                (
                                    s("title"),
                                    crate::PropertyValue::Value {
                                        value: crate::variable::Value::String {
                                            text: "\"https://www.fifthtry.com/fifthtry/ftd/\""
                                                .to_string(),
                                            source: crate::TextSource::Header,
                                        },
                                    },
                                ),
                            ])
                            .collect(),
                        },
                    ],
                    kind: crate::p2::Kind::Record {
                        name: s("foo/bar#data"),
                    },
                },
                conditions: vec![],
            }),
        );

        let (g_bag, g_col) = crate::p2::interpreter::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- component foo:
                component: ftd.row
                $name: caption
                $body: string

                --- ftd.text: ref $name

                --- ftd.text: ref $body

                -- record data:
                title: string
                description: string

                -- list test:
                type: data
                $processor$: read_package_records_from_cargo_toml

                -- foo: ref obj.title
                $loop$: test as obj
                body: ref obj.description
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_bag, bag);
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn loop_with_tree_structure() {
        let mut main = super::default_column();
        let col = ftd_rt::Element::Column(ftd_rt::Column {
            container: ftd_rt::Container {
                children: vec![
                    ftd_rt::Element::Text(ftd_rt::Text {
                        text: ftd::markdown_line("ab title"),
                        line: true,
                        common: ftd_rt::Common {
                            link: Some(s("ab link")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                    ftd_rt::Element::Column(ftd_rt::Column {
                        container: ftd_rt::Container {
                            children: vec![ftd_rt::Element::Text(ftd_rt::Text {
                                text: ftd::markdown_line("aa title"),
                                line: true,
                                common: ftd_rt::Common {
                                    link: Some(s("aa link")),
                                    ..Default::default()
                                },
                                ..Default::default()
                            })],
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                    ftd_rt::Element::Column(ftd_rt::Column {
                        container: ftd_rt::Container {
                            children: vec![ftd_rt::Element::Text(ftd_rt::Text {
                                text: ftd::markdown_line("aaa title"),
                                line: true,
                                common: ftd_rt::Common {
                                    link: Some(s("aaa link")),
                                    ..Default::default()
                                },
                                ..Default::default()
                            })],
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                ],
                ..Default::default()
            },
            ..Default::default()
        });
        main.container.children.push(col.clone());
        main.container
            .children
            .push(ftd_rt::Element::Row(ftd_rt::Row {
                container: ftd_rt::Container {
                    children: vec![col],
                    ..Default::default()
                },
                ..Default::default()
            }));

        let mut bag = super::default_bag();

        bag.insert(
            s("foo/bar#aa"),
            crate::p2::Thing::Variable(ftd::Variable {
                name: s("foo/bar#aa"),
                value: ftd::Value::List {
                    data: vec![
                        ftd::Value::Record {
                            name: s("foo/bar#toc-record"),
                            fields: std::array::IntoIter::new([
                                (
                                    s("children"),
                                    crate::PropertyValue::Value {
                                        value: crate::variable::Value::List {
                                            data: vec![],
                                            kind: crate::p2::Kind::Record {
                                                name: s("foo/bar#toc-record"),
                                            },
                                        },
                                    },
                                ),
                                (
                                    s("link"),
                                    crate::PropertyValue::Value {
                                        value: crate::variable::Value::String {
                                            text: s("aa link"),
                                            source: crate::TextSource::Header,
                                        },
                                    },
                                ),
                                (
                                    s("title"),
                                    crate::PropertyValue::Value {
                                        value: crate::variable::Value::String {
                                            text: s("aa title"),
                                            source: crate::TextSource::Header,
                                        },
                                    },
                                ),
                            ])
                            .collect(),
                        },
                        ftd::Value::Record {
                            name: s("foo/bar#toc-record"),
                            fields: std::array::IntoIter::new([
                                (
                                    s("children"),
                                    crate::PropertyValue::Value {
                                        value: crate::variable::Value::List {
                                            data: vec![],
                                            kind: crate::p2::Kind::Record {
                                                name: s("foo/bar#toc-record"),
                                            },
                                        },
                                    },
                                ),
                                (
                                    s("link"),
                                    crate::PropertyValue::Value {
                                        value: crate::variable::Value::String {
                                            text: s("aaa link"),
                                            source: crate::TextSource::Header,
                                        },
                                    },
                                ),
                                (
                                    s("title"),
                                    crate::PropertyValue::Value {
                                        value: crate::variable::Value::String {
                                            text: s("aaa title"),
                                            source: crate::TextSource::Header,
                                        },
                                    },
                                ),
                            ])
                            .collect(),
                        },
                    ],
                    kind: crate::p2::Kind::Record {
                        name: s("foo/bar#toc-record"),
                    },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            s("foo/bar#toc"),
            crate::p2::Thing::Variable(ftd::Variable {
                name: s("foo/bar#toc"),
                value: ftd::Value::List {
                    data: vec![ftd::Value::Record {
                        name: s("foo/bar#toc-record"),
                        fields: std::array::IntoIter::new([
                            (
                                s("children"),
                                crate::PropertyValue::Value {
                                    value: crate::variable::Value::List {
                                        data: vec![
                                            ftd::Value::Record {
                                                name: s("foo/bar#toc-record"),
                                                fields: std::array::IntoIter::new([
                                                    (
                                                        s("children"),
                                                        crate::PropertyValue::Value {
                                                            value: crate::variable::Value::List {
                                                                data: vec![],
                                                                kind: crate::p2::Kind::Record {
                                                                    name: s("foo/bar#toc-record"),
                                                                },
                                                            },
                                                        },
                                                    ),
                                                    (
                                                        s("link"),
                                                        crate::PropertyValue::Value {
                                                            value: crate::variable::Value::String {
                                                                text: s("aa link"),
                                                                source: crate::TextSource::Header,
                                                            },
                                                        },
                                                    ),
                                                    (
                                                        s("title"),
                                                        crate::PropertyValue::Value {
                                                            value: crate::variable::Value::String {
                                                                text: s("aa title"),
                                                                source: crate::TextSource::Header,
                                                            },
                                                        },
                                                    ),
                                                ])
                                                .collect(),
                                            },
                                            ftd::Value::Record {
                                                name: s("foo/bar#toc-record"),
                                                fields: std::array::IntoIter::new([
                                                    (
                                                        s("children"),
                                                        crate::PropertyValue::Value {
                                                            value: crate::variable::Value::List {
                                                                data: vec![],
                                                                kind: crate::p2::Kind::Record {
                                                                    name: s("foo/bar#toc-record"),
                                                                },
                                                            },
                                                        },
                                                    ),
                                                    (
                                                        s("link"),
                                                        crate::PropertyValue::Value {
                                                            value: crate::variable::Value::String {
                                                                text: s("aaa link"),
                                                                source: crate::TextSource::Header,
                                                            },
                                                        },
                                                    ),
                                                    (
                                                        s("title"),
                                                        crate::PropertyValue::Value {
                                                            value: crate::variable::Value::String {
                                                                text: s("aaa title"),
                                                                source: crate::TextSource::Header,
                                                            },
                                                        },
                                                    ),
                                                ])
                                                .collect(),
                                            },
                                        ],
                                        kind: crate::p2::Kind::Record {
                                            name: s("foo/bar#toc-record"),
                                        },
                                    },
                                },
                            ),
                            (
                                s("link"),
                                crate::PropertyValue::Value {
                                    value: crate::variable::Value::String {
                                        text: s("ab link"),
                                        source: crate::TextSource::Header,
                                    },
                                },
                            ),
                            (
                                s("title"),
                                crate::PropertyValue::Value {
                                    value: crate::variable::Value::String {
                                        text: s("ab title"),
                                        source: crate::TextSource::Header,
                                    },
                                },
                            ),
                        ])
                        .collect(),
                    }],
                    kind: crate::p2::Kind::Record {
                        name: s("foo/bar#toc-record"),
                    },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            s("foo/bar#toc"),
            crate::p2::Thing::Component(ftd::Component {
                root: "ftd.column".to_string(),
                full_name: "foo/bar#toc-item".to_string(),
                arguments: std::array::IntoIter::new([(
                    s("toc"),
                    crate::p2::Kind::Record {
                        name: "foo/bar#toc-record".to_string(),
                    },
                )])
                .collect(),
                instructions: vec![
                    Instruction::ChildComponent {
                        child: ftd::ChildComponent {
                            events: vec![],
                            root: "ftd#text".to_string(),
                            condition: None,
                            properties: std::array::IntoIter::new([
                                (
                                    s("link"),
                                    crate::component::Property {
                                        default: Some(crate::PropertyValue::Argument {
                                            name: "toc.link".to_string(),
                                            kind: crate::p2::Kind::Optional {
                                                kind: Box::new(crate::p2::Kind::string()),
                                            },
                                        }),
                                        conditions: vec![],
                                    },
                                ),
                                (
                                    s("text"),
                                    crate::component::Property {
                                        default: Some(crate::PropertyValue::Argument {
                                            name: "toc.title".to_string(),
                                            kind: crate::p2::Kind::Optional {
                                                kind: Box::new(crate::p2::Kind::caption_or_body()),
                                            },
                                        }),
                                        conditions: vec![],
                                    },
                                ),
                            ])
                            .collect(),
                            ..Default::default()
                        },
                    },
                    Instruction::RecursiveChildComponent {
                        child: ftd::ChildComponent {
                            is_recursive: true,
                            events: vec![],
                            root: "toc-item".to_string(),
                            condition: None,
                            properties: std::array::IntoIter::new([
                                (
                                    s("$loop$"),
                                    crate::component::Property {
                                        default: Some(crate::PropertyValue::Argument {
                                            name: "toc.children".to_string(),
                                            kind: crate::p2::Kind::Record {
                                                name: s("foo/bar#toc-record"),
                                            },
                                        }),
                                        conditions: vec![],
                                    },
                                ),
                                (
                                    s("toc"),
                                    crate::component::Property {
                                        default: Some(crate::PropertyValue::Argument {
                                            name: "$loop$".to_string(),
                                            kind: crate::p2::Kind::Record {
                                                name: s("foo/bar#toc-record"),
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
                ..Default::default()
            }),
        );

        let (_g_bag, g_col) = crate::p2::interpreter::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- record toc-record:
                title: string
                link: string
                children: list toc-record

                -- component toc-item:
                component: ftd.column
                $toc: toc-record

                --- ftd.text: ref $toc.title
                link: ref $toc.link

                --- toc-item:
                $loop$: $toc.children as obj
                toc: ref obj

                -- list aa:
                type: toc-record

                -- aa:
                title: aa title
                link: aa link

                -- aa:
                title: aaa title
                link: aaa link

                -- list toc:
                type: toc-record

                -- toc:
                title: ab title
                link: ab link
                children: ref aa

                -- component foo:
                component: ftd.row

                --- toc-item:
                $loop$: toc as obj
                toc: ref obj

                -- toc-item:
                $loop$: toc as obj
                toc: ref obj

                -- foo:
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        // pretty_assertions::assert_eq!(g_bag, bag);
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn import_check() {
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd_rt::Element::Row(ftd_rt::Row {
                container: ftd_rt::Container {
                    children: vec![ftd_rt::Element::Text(ftd_rt::Text {
                        text: ftd::markdown_line("Hello World"),
                        line: true,
                        common: ftd_rt::Common {
                            reference: Some(s("hello-world-variable#hello-world")),
                            ..Default::default()
                        },
                        ..Default::default()
                    })],
                    ..Default::default()
                },
                ..Default::default()
            }));

        let mut bag = super::default_bag();
        bag.insert(
            s("hello-world#foo"),
            crate::p2::Thing::Component(ftd::Component {
                root: s("ftd.row"),
                full_name: s("hello-world#foo"),
                instructions: vec![ftd::Instruction::ChildComponent {
                    child: ftd::ChildComponent {
                        events: vec![],
                        root: s("ftd#text"),
                        condition: None,
                        properties: std::array::IntoIter::new([(
                            s("text"),
                            crate::component::Property {
                                default: Some(crate::PropertyValue::Reference {
                                    name: "hello-world-variable#hello-world".to_string(),
                                    kind: crate::p2::Kind::caption_or_body(),
                                }),
                                conditions: vec![],
                            },
                        )])
                        .collect(),
                        ..Default::default()
                    },
                }],
                invocations: vec![std::collections::BTreeMap::new()],
                ..Default::default()
            }),
        );
        bag.insert(
            s("hello-world-variable#hello-world"),
            crate::p2::Thing::Variable(ftd::Variable {
                name: s("hello-world"),
                value: ftd::Value::String {
                    text: s("Hello World"),
                    source: ftd::TextSource::Caption,
                },
                conditions: vec![],
            }),
        );

        let (g_bag, g_col) = crate::p2::interpreter::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- import: hello-world as hw

                -- hw.foo:
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_bag, bag);
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn argument_with_default_value() {
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("hello world"),
                line: true,
                size: Some(10),
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("hello"),
                line: true,
                size: Some(10),
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("this is nice"),
                line: true,
                size: Some(20),
                ..Default::default()
            }));

        let mut bag = super::default_bag();
        bag.insert(
            s("foo/bar#foo"),
            crate::p2::Thing::Component(ftd::Component {
                root: s("ftd.text"),
                full_name: s("foo/bar#foo"),
                arguments: std::array::IntoIter::new([
                    (
                        s("name"),
                        crate::p2::Kind::caption().set_default(Some(s("hello world"))),
                    ),
                    (
                        s("size"),
                        crate::p2::Kind::Integer {
                            default: Some(s("10")),
                        },
                    ),
                ])
                .collect(),
                properties: std::array::IntoIter::new([
                    (
                        s("size"),
                        crate::component::Property {
                            default: Some(crate::PropertyValue::Argument {
                                name: s("size"),
                                kind: crate::p2::Kind::Optional {
                                    kind: Box::from(crate::p2::Kind::Integer { default: None }),
                                },
                            }),
                            conditions: vec![],
                        },
                    ),
                    (
                        s("text"),
                        crate::component::Property {
                            default: Some(crate::PropertyValue::Argument {
                                name: s("name"),
                                kind: crate::p2::Kind::caption_or_body(),
                            }),
                            conditions: vec![],
                        },
                    ),
                ])
                .collect(),
                invocations: vec![
                    std::array::IntoIter::new([
                        (
                            s("name"),
                            crate::Value::String {
                                text: s("hello world"),
                                source: crate::TextSource::Default,
                            },
                        ),
                        (s("size"), crate::Value::Integer { value: 10 }),
                    ])
                    .collect(),
                    std::array::IntoIter::new([
                        (
                            s("name"),
                            crate::Value::String {
                                text: s("hello"),
                                source: crate::TextSource::Caption,
                            },
                        ),
                        (s("size"), crate::Value::Integer { value: 10 }),
                    ])
                    .collect(),
                    std::array::IntoIter::new([
                        (
                            s("name"),
                            crate::Value::String {
                                text: s("this is nice"),
                                source: crate::TextSource::Caption,
                            },
                        ),
                        (s("size"), crate::Value::Integer { value: 20 }),
                    ])
                    .collect(),
                ],
                ..Default::default()
            }),
        );

        let (g_bag, g_col) = crate::p2::interpreter::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- component foo:
                component: ftd.text
                $name: caption with default hello world
                $size: integer with default 10
                text: ref $name
                size: ref $size

                -- foo:

                -- foo: hello

                -- foo: this is nice
                size: 20
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_bag, bag);
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn record_with_default_value() {
        let mut bag = super::default_bag();
        bag.insert(
            s("foo/bar#abrar"),
            crate::p2::Thing::Variable(ftd::Variable {
                name: s("abrar"),
                value: ftd::Value::Record {
                    name: s("foo/bar#person"),
                    fields: std::array::IntoIter::new([
                        (
                            s("address"),
                            crate::PropertyValue::Value {
                                value: crate::variable::Value::String {
                                    text: s("Bihar"),
                                    source: crate::TextSource::Default,
                                },
                            },
                        ),
                        (
                            s("age"),
                            crate::PropertyValue::Reference {
                                name: s("foo/bar#default-age"),
                                kind: crate::p2::Kind::Integer {
                                    default: Some(s("ref default-age")),
                                },
                            },
                        ),
                        (
                            s("bio"),
                            crate::PropertyValue::Value {
                                value: crate::variable::Value::String {
                                    text: s("Software developer working at fifthtry."),
                                    source: crate::TextSource::Body,
                                },
                            },
                        ),
                        (
                            s("name"),
                            crate::PropertyValue::Reference {
                                name: s("foo/bar#abrar-name"),
                                kind: crate::p2::Kind::caption(),
                            },
                        ),
                        (
                            s("size"),
                            crate::PropertyValue::Value {
                                value: crate::variable::Value::Integer { value: 10 },
                            },
                        ),
                    ])
                    .collect(),
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            s("foo/bar#abrar-name"),
            crate::p2::Thing::Variable(ftd::Variable {
                name: s("abrar-name"),
                value: crate::variable::Value::String {
                    text: s("Abrar Khan"),
                    source: crate::TextSource::Caption,
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            s("foo/bar#default-age"),
            crate::p2::Thing::Variable(ftd::Variable {
                name: s("default-age"),
                value: crate::variable::Value::Integer { value: 20 },
                conditions: vec![],
            }),
        );
        bag.insert(
            s("foo/bar#person"),
            crate::p2::Thing::Record(ftd::p2::Record {
                name: s("foo/bar#person"),
                fields: std::array::IntoIter::new([
                    (
                        s("address"),
                        crate::p2::Kind::string().set_default(Some(s("Bihar"))),
                    ),
                    (
                        s("age"),
                        crate::p2::Kind::Integer {
                            default: Some(s("ref default-age")),
                        },
                    ),
                    (
                        s("bio"),
                        crate::p2::Kind::body().set_default(Some(s("Some Bio"))),
                    ),
                    (s("name"), crate::p2::Kind::caption()),
                    (
                        s("size"),
                        crate::p2::Kind::Integer {
                            default: Some(s("10")),
                        },
                    ),
                ])
                .collect(),
                instances: Default::default(),
            }),
        );

        let mut main = super::default_column();
        main.container
            .children
            .push(ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown("Software developer working at fifthtry."),
                size: Some(20),
                common: ftd_rt::Common {
                    reference: Some(s("abrar.bio")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (g_bag, g_col) = crate::p2::interpreter::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- var default-age: 20

                -- record person:
                name: caption
                address: string with default Bihar
                bio: body with default Some Bio
                age: integer with default ref default-age
                size: integer with default 10

                -- var abrar-name: Abrar Khan

                -- var abrar: ref abrar-name
                type: person

                Software developer working at fifthtry.

                -- ftd.text: ref abrar.bio
                size: ref abrar.age
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_bag, bag);
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn default_with_reference() {
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd_rt::Element::Row(ftd_rt::Row {
                container: ftd_rt::Container {
                    children: vec![ftd_rt::Element::Text(ftd_rt::Text {
                        text: ftd::markdown_line("Arpita"),
                        line: true,
                        size: Some(10),
                        ..Default::default()
                    })],
                    ..Default::default()
                },
                ..Default::default()
            }));
        main.container
            .children
            .push(ftd_rt::Element::Row(ftd_rt::Row {
                container: ftd_rt::Container {
                    children: vec![ftd_rt::Element::Text(ftd_rt::Text {
                        text: ftd::markdown_line("Amit Upadhayay"),
                        line: true,
                        size: Some(20),
                        ..Default::default()
                    })],
                    ..Default::default()
                },
                ..Default::default()
            }));

        let mut bag = super::default_bag();
        bag.insert(
            s("foo/bar#default-name"),
            crate::p2::Thing::Variable(ftd::Variable {
                name: s("default-name"),
                value: crate::Value::String {
                    text: s("Arpita"),
                    source: crate::TextSource::Caption,
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            s("foo/bar#default-size"),
            crate::p2::Thing::Variable(ftd::Variable {
                name: s("default-size"),
                value: crate::Value::Integer { value: 10 },
                conditions: vec![],
            }),
        );
        bag.insert(
            s("foo/bar#foo"),
            crate::p2::Thing::Component(ftd::Component {
                root: s("ftd.row"),
                full_name: s("foo/bar#foo"),
                arguments: std::array::IntoIter::new([
                    (
                        s("name"),
                        crate::p2::Kind::string().set_default(Some(s("ref default-name"))),
                    ),
                    (
                        s("text-size"),
                        crate::p2::Kind::Integer {
                            default: Some(s("ref default-size")),
                        },
                    ),
                ])
                .collect(),
                instructions: vec![ftd::Instruction::ChildComponent {
                    child: ftd::ChildComponent {
                        events: vec![],
                        root: s("ftd#text"),
                        condition: None,
                        properties: std::array::IntoIter::new([
                            (
                                s("size"),
                                crate::component::Property {
                                    default: Some(ftd::PropertyValue::Argument {
                                        name: s("text-size"),
                                        kind: ftd::p2::Kind::Optional {
                                            kind: Box::new(ftd::p2::Kind::Integer {
                                                default: None,
                                            }),
                                        },
                                    }),
                                    conditions: vec![],
                                },
                            ),
                            (
                                s("text"),
                                crate::component::Property {
                                    default: Some(ftd::PropertyValue::Argument {
                                        name: s("name"),
                                        kind: ftd::p2::Kind::caption_or_body(),
                                    }),
                                    conditions: vec![],
                                },
                            ),
                        ])
                        .collect(),
                        ..Default::default()
                    },
                }],
                kernel: false,
                invocations: vec![
                    std::array::IntoIter::new([
                        (
                            s("name"),
                            ftd::Value::String {
                                text: s("Arpita"),
                                source: ftd::TextSource::Caption,
                            },
                        ),
                        (s("text-size"), ftd::Value::Integer { value: 10 }),
                    ])
                    .collect(),
                    std::array::IntoIter::new([
                        (
                            s("name"),
                            ftd::Value::String {
                                text: s("Amit Upadhayay"),
                                source: ftd::TextSource::Header,
                            },
                        ),
                        (s("text-size"), ftd::Value::Integer { value: 20 }),
                    ])
                    .collect(),
                ],
                ..Default::default()
            }),
        );

        let (g_bag, g_col) = crate::p2::interpreter::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- var default-name: Arpita

                -- var default-size: 10

                -- component foo:
                component: ftd.row
                $name: string with default ref default-name
                $text-size: integer with default ref default-size

                --- ftd.text: ref $name
                size: ref $text-size

                -- foo:

                -- foo:
                name: Amit Upadhayay
                text-size: 20
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_bag, bag);
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn or_type_with_default_value() {
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("Amit Upadhyay"),
                line: true,
                common: ftd_rt::Common {
                    reference: Some(s("amitu.name")),
                    ..Default::default()
                },
                ..Default::default()
            }));
        main.container
            .children
            .push(ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("1000"),
                line: true,
                common: ftd_rt::Common {
                    reference: Some(s("amitu.phone")),
                    ..Default::default()
                },
                ..Default::default()
            }));
        main.container
            .children
            .push(ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("John Doe"),
                line: true,
                size: Some(50),
                common: ftd_rt::Common {
                    reference: Some(s("acme.contact")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        let mut bag = super::default_bag();
        bag.insert(
            s("foo/bar#acme"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("acme"),
                value: ftd::Value::OrType {
                    name: s("foo/bar#lead"),
                    variant: s("company"),
                    fields: std::array::IntoIter::new([
                        (
                            s("contact"),
                            ftd::PropertyValue::Value {
                                value: ftd::Value::String {
                                    text: s("John Doe"),
                                    source: ftd::TextSource::Header,
                                },
                            },
                        ),
                        (
                            s("fax"),
                            ftd::PropertyValue::Value {
                                value: ftd::Value::String {
                                    text: s("+1-234-567890"),
                                    source: ftd::TextSource::Header,
                                },
                            },
                        ),
                        (
                            s("name"),
                            ftd::PropertyValue::Value {
                                value: ftd::Value::String {
                                    text: s("Acme Inc."),
                                    source: ftd::TextSource::Caption,
                                },
                            },
                        ),
                        (
                            s("no-of-employees"),
                            ftd::PropertyValue::Value {
                                value: ftd::Value::Integer { value: 50 },
                            },
                        ),
                    ])
                    .collect(),
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            s("foo/bar#amitu"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("amitu"),
                value: ftd::Value::OrType {
                    name: s("foo/bar#lead"),
                    variant: s("individual"),
                    fields: std::array::IntoIter::new([
                        (
                            s("name"),
                            ftd::PropertyValue::Value {
                                value: ftd::Value::String {
                                    text: s("Amit Upadhyay"),
                                    source: ftd::TextSource::Caption,
                                },
                            },
                        ),
                        (
                            s("phone"),
                            ftd::PropertyValue::Reference {
                                name: s("foo/bar#default-phone"),
                                kind: ftd::p2::Kind::string()
                                    .set_default(Some(s("ref default-phone"))),
                            },
                        ),
                    ])
                    .collect(),
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            s("foo/bar#default-phone"),
            ftd::p2::Thing::Variable(ftd::Variable {
                name: s("default-phone"),
                value: ftd::Value::String {
                    text: s("1000"),
                    source: ftd::TextSource::Caption,
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            s("foo/bar#lead"),
            ftd::p2::Thing::OrType(ftd::OrType {
                name: s("foo/bar#lead"),
                variants: vec![
                    ftd::p2::Record {
                        name: s("foo/bar#lead.individual"),
                        fields: std::array::IntoIter::new([
                            (s("name"), ftd::p2::Kind::caption()),
                            (
                                s("phone"),
                                ftd::p2::Kind::string().set_default(Some(s("ref default-phone"))),
                            ),
                        ])
                        .collect(),
                        instances: Default::default(),
                    },
                    ftd::p2::Record {
                        name: s("foo/bar#lead.company"),
                        fields: std::array::IntoIter::new([
                            (
                                s("contact"),
                                ftd::p2::Kind::string().set_default(Some(s("1001"))),
                            ),
                            (s("fax"), ftd::p2::Kind::string()),
                            (s("name"), ftd::p2::Kind::caption()),
                            (
                                s("no-of-employees"),
                                ftd::p2::Kind::integer().set_default(Some(s("50"))),
                            ),
                        ])
                        .collect(),
                        instances: Default::default(),
                    },
                ],
            }),
        );

        let (g_bag, g_col) = crate::p2::interpreter::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- var default-phone: 1000
                type: string

                -- or-type lead:

                --- individual:
                name: caption
                phone: string with default ref default-phone

                --- company:
                name: caption
                contact: string with default 1001
                fax: string
                no-of-employees: integer with default 50

                -- var amitu: Amit Upadhyay
                type: lead.individual

                -- var acme: Acme Inc.
                type: lead.company
                contact: John Doe
                fax: +1-234-567890

                -- ftd.text: ref amitu.name

                -- ftd.text: ref amitu.phone

                -- ftd.text: ref acme.contact
                size: ref acme.no-of-employees

                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_bag, bag);
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn default_id() {
        let mut main = super::default_column();

        main.container
            .children
            .push(ftd_rt::Element::Column(ftd_rt::Column {
                container: ftd_rt::Container {
                    children: vec![
                        ftd_rt::Element::Column(ftd_rt::Column {
                            container: ftd_rt::Container {
                                children: vec![ftd_rt::Element::Row(ftd_rt::Row {
                                    container: ftd_rt::Container {
                                        children: vec![ftd_rt::Element::Column(ftd_rt::Column {
                                            container: ftd_rt::Container {
                                                children: vec![ftd_rt::Element::Text(
                                                    ftd_rt::Text {
                                                        text: ftd::markdown_line("hello"),
                                                        line: true,
                                                        ..Default::default()
                                                    },
                                                )],
                                                ..Default::default()
                                            },
                                            common: ftd_rt::Common {
                                                id: Some(s("display-text-id")),
                                                ..Default::default()
                                            },
                                        })],
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                })],
                                ..Default::default()
                            },
                            common: ftd_rt::Common {
                                id: Some(s("inside-page-id")),
                                ..Default::default()
                            },
                        }),
                        ftd_rt::Element::Row(ftd_rt::Row {
                            common: ftd_rt::Common {
                                id: Some(s("page-id-row")),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    ],
                    ..Default::default()
                },
                common: ftd_rt::Common {
                    id: Some(s("page-id")),
                    ..Default::default()
                },
            }));

        main.container
            .children
            .push(ftd_rt::Element::Row(ftd_rt::Row {
                ..Default::default()
            }));

        let (_g_bag, g_col) = crate::p2::interpreter::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- component display-text:
                component: ftd.column

                --- ftd.text: hello


                -- component inside-page:
                component: ftd.column

                --- ftd.row:

                --- display-text:
                id: display-text-id


                -- component page:
                component: ftd.column

                --- inside-page:
                id: inside-page-id


                -- page:
                id: page-id

                -- ftd.row:

                -- container: page-id

                -- ftd.row:
                id: page-id-row

                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn region_h1() {
        let mut main = super::default_column();

        main.container
            .children
            .push(ftd_rt::Element::Column(ftd_rt::Column {
                container: ftd_rt::Container {
                    children: vec![ftd_rt::Element::Text(ftd_rt::Text {
                        text: ftd::markdown_line("Heading 31"),
                        line: true,
                        common: ftd_rt::Common {
                            region: Some(ftd_rt::Region::Title),
                            ..Default::default()
                        },
                        ..Default::default()
                    })],
                    ..Default::default()
                },
                common: ftd_rt::Common {
                    region: Some(ftd_rt::Region::H3),
                    ..Default::default()
                },
            }));

        main.container
            .children
            .push(ftd_rt::Element::Column(ftd_rt::Column {
                container: ftd_rt::Container {
                    children: vec![
                        ftd_rt::Element::Text(ftd_rt::Text {
                            text: ftd::markdown_line("Heading 11"),
                            line: true,
                            common: ftd_rt::Common {
                                region: Some(ftd_rt::Region::Title),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd_rt::Element::Column(ftd_rt::Column {
                            container: ftd_rt::Container {
                                children: vec![
                                    ftd_rt::Element::Text(ftd_rt::Text {
                                        text: ftd::markdown_line("Heading 21"),
                                        line: true,
                                        common: ftd_rt::Common {
                                            region: Some(ftd_rt::Region::Title),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    }),
                                    ftd_rt::Element::Column(ftd_rt::Column {
                                        container: ftd_rt::Container {
                                            children: vec![
                                                ftd_rt::Element::Text(ftd_rt::Text {
                                                    text: ftd::markdown_line("Heading 32"),
                                                    line: true,
                                                    common: ftd_rt::Common {
                                                        region: Some(ftd_rt::Region::Title),
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
                                            region: Some(ftd_rt::Region::H3),
                                            ..Default::default()
                                        },
                                    }),
                                ],
                                ..Default::default()
                            },
                            common: ftd_rt::Common {
                                region: Some(ftd_rt::Region::H2),
                                ..Default::default()
                            },
                        }),
                        ftd_rt::Element::Column(ftd_rt::Column {
                            container: ftd_rt::Container {
                                children: vec![ftd_rt::Element::Text(ftd_rt::Text {
                                    text: ftd::markdown_line("Heading 22"),
                                    line: true,
                                    common: ftd_rt::Common {
                                        region: Some(ftd_rt::Region::Title),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                })],
                                ..Default::default()
                            },
                            common: ftd_rt::Common {
                                region: Some(ftd_rt::Region::H2),
                                ..Default::default()
                            },
                        }),
                        ftd_rt::Element::Column(ftd_rt::Column {
                            container: ftd_rt::Container {
                                children: vec![ftd_rt::Element::Text(ftd_rt::Text {
                                    text: ftd::markdown_line("Heading 23"),
                                    line: true,
                                    common: ftd_rt::Common {
                                        region: Some(ftd_rt::Region::Title),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                })],
                                ..Default::default()
                            },
                            common: ftd_rt::Common {
                                region: Some(ftd_rt::Region::H2),
                                ..Default::default()
                            },
                        }),
                    ],
                    ..Default::default()
                },
                common: ftd_rt::Common {
                    region: Some(ftd_rt::Region::H1),
                    ..Default::default()
                },
            }));

        main.container
            .children
            .push(ftd_rt::Element::Column(ftd_rt::Column {
                container: ftd_rt::Container {
                    children: vec![
                        ftd_rt::Element::Text(ftd_rt::Text {
                            text: ftd::markdown_line("Heading 12"),
                            line: true,
                            common: ftd_rt::Common {
                                region: Some(ftd_rt::Region::Title),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd_rt::Element::Column(ftd_rt::Column {
                            container: ftd_rt::Container {
                                children: vec![ftd_rt::Element::Text(ftd_rt::Text {
                                    text: ftd::markdown_line("Heading 33"),
                                    line: true,
                                    common: ftd_rt::Common {
                                        region: Some(ftd_rt::Region::Title),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                })],
                                ..Default::default()
                            },
                            common: ftd_rt::Common {
                                region: Some(ftd_rt::Region::H3),
                                ..Default::default()
                            },
                        }),
                        ftd_rt::Element::Column(ftd_rt::Column {
                            container: ftd_rt::Container {
                                children: vec![ftd_rt::Element::Text(ftd_rt::Text {
                                    text: ftd::markdown_line("Heading 24"),
                                    line: true,
                                    common: ftd_rt::Common {
                                        region: Some(ftd_rt::Region::Title),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                })],
                                ..Default::default()
                            },
                            common: ftd_rt::Common {
                                region: Some(ftd_rt::Region::H2),
                                ..Default::default()
                            },
                        }),
                    ],
                    ..Default::default()
                },
                common: ftd_rt::Common {
                    region: Some(ftd_rt::Region::H1),
                    ..Default::default()
                },
            }));

        let (_g_bag, g_col) = crate::p2::interpreter::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- component h1:
                component: ftd.column
                region: h1
                $title: caption

                --- ftd.text:
                text: ref $title
                $title: caption
                region: title

                -- component h2:
                component: ftd.column
                region: h2
                $title: caption

                --- ftd.text:
                text: ref $title
                $title: caption
                region: title

                -- component h3:
                component: ftd.column
                region: h3
                $title: caption

                --- ftd.text:
                text: ref $title
                $title: caption
                region: title

                -- h3: Heading 31

                -- h1: Heading 11

                -- h2: Heading 21

                -- h3: Heading 32

                -- ftd.text: hello

                -- h2: Heading 22

                -- h2: Heading 23

                -- h1: Heading 12

                -- h3: Heading 33

                -- h2: Heading 24

                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn event_onclick() {
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd_rt::Element::Column(ftd_rt::Column {
                container: ftd_rt::Container {
                    children: vec![
                        ftd_rt::Element::Text(ftd_rt::Text {
                            text: ftd::markdown_line("Mobile"),
                            line: true,
                            common: ftd_rt::Common {
                                condition: Some(ftd_rt::Condition {
                                    variable: s("foo/bar#mobile"),
                                    value: s("true"),
                                }),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd_rt::Element::Text(ftd_rt::Text {
                            text: ftd::markdown_line("Desktop"),
                            line: true,
                            common: ftd_rt::Common {
                                condition: Some(ftd_rt::Condition {
                                    variable: s("foo/bar#mobile"),
                                    value: s("false"),
                                }),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    ],
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("Click Here!"),
                line: true,
                common: ftd_rt::Common {
                    events: vec![ftd_rt::Event {
                        name: s("onclick"),
                        action: ftd_rt::Action {
                            action: s("toggle"),
                            target: s("foo/bar#mobile"),
                            parameters: Default::default(),
                        },
                    }],
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (_g_bag, g_col) = crate::p2::interpreter::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- var mobile: true

                -- component foo:
                component: ftd.column

                --- ftd.text: Mobile
                if: mobile

                --- ftd.text: Desktop
                if: not mobile

                -- foo:

                -- ftd.text: Click Here!
                $event-click$: toggle mobile
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn event_toggle_with_local_variable() {
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("Hello"),
                line: true,
                common: ftd_rt::Common {
                    locals: std::array::IntoIter::new([(s("open@0"), s("true"))]).collect(),
                    condition: Some(ftd_rt::Condition {
                        variable: s("@open@0"),
                        value: s("true"),
                    }),
                    events: vec![ftd_rt::Event {
                        name: s("onclick"),
                        action: ftd_rt::Action {
                            action: s("toggle"),
                            target: s("@open@0"),
                            parameters: Default::default(),
                        },
                    }],
                    ..Default::default()
                },
                ..Default::default()
            }));

        let mut bag = super::default_bag();
        bag.insert(
            s("foo/bar#foo"),
            ftd::p2::Thing::Component(ftd::Component {
                root: "ftd.text".to_string(),
                full_name: "foo/bar#foo".to_string(),
                arguments: std::array::IntoIter::new([(s("name"), ftd::p2::Kind::caption())])
                    .collect(),
                locals: std::array::IntoIter::new([(
                    s("open"),
                    ftd::p2::Kind::boolean().set_default(Some(s("true"))),
                )])
                .collect(),
                properties: std::array::IntoIter::new([(
                    s("text"),
                    ftd::component::Property {
                        default: Some(ftd::PropertyValue::Argument {
                            name: s("name"),
                            kind: ftd::p2::Kind::String {
                                caption: true,
                                body: true,
                                default: None,
                            },
                        }),
                        ..Default::default()
                    },
                )])
                .collect(),
                instructions: vec![],
                events: vec![ftd::p2::Event {
                    name: ftd::p2::EventName::OnClick,
                    action: ftd::p2::Action {
                        action: ftd::p2::ActionKind::Toggle,
                        target: s("@open"),
                        parameters: Default::default(),
                    },
                }],
                condition: Some(ftd::p2::Boolean::Equal {
                    left: ftd::PropertyValue::LocalVariable {
                        name: s("open"),
                        kind: ftd::p2::Kind::Boolean {
                            default: Some(s("true")),
                        },
                    },
                    right: ftd::PropertyValue::Value {
                        value: crate::variable::Value::Boolean { value: true },
                    },
                }),
                kernel: false,
                invocations: vec![std::array::IntoIter::new([(
                    s("name"),
                    ftd::Value::String {
                        text: s("Hello"),
                        source: ftd::TextSource::Caption,
                    },
                )])
                .collect()],
            }),
        );

        let (g_bag, g_col) = crate::p2::interpreter::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- component foo:
                component: ftd.text
                $name: caption
                @open: boolean with default true
                text: ref $name
                if: @open
                $event-click$: toggle @open

                -- foo: Hello
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
        pretty_assertions::assert_eq!(g_bag, bag);
    }

    #[test]
    fn event_toggle_with_local_variable_for_component() {
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd_rt::Element::Column(ftd_rt::Column {
                container: ftd_rt::Container {
                    children: vec![
                        ftd_rt::Element::Text(ftd_rt::Text {
                            text: ftd::markdown_line("Click here"),
                            line: true,
                            common: ftd_rt::Common {
                                events: vec![ftd_rt::Event {
                                    name: s("onclick"),
                                    action: ftd_rt::Action {
                                        action: s("toggle"),
                                        target: s("@open@0"),
                                        parameters: Default::default(),
                                    },
                                }],
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd_rt::Element::Text(ftd_rt::Text {
                            text: ftd::markdown_line("Open True"),
                            line: true,
                            common: ftd_rt::Common {
                                condition: Some(ftd_rt::Condition {
                                    variable: s("@open@0"),
                                    value: s("true"),
                                }),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd_rt::Element::Text(ftd_rt::Text {
                            text: ftd::markdown_line("Open False"),
                            line: true,
                            common: ftd_rt::Common {
                                condition: Some(ftd_rt::Condition {
                                    variable: s("@open@0"),
                                    value: s("false"),
                                }),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    ],
                    ..Default::default()
                },
                common: ftd_rt::Common {
                    locals: std::array::IntoIter::new([(s("open@0"), s("true"))]).collect(),
                    ..Default::default()
                },
            }));

        let (_g_bag, g_col) = crate::p2::interpreter::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- component foo:
                component: ftd.column
                @open: boolean with default true

                --- ftd.text: Click here
                $event-click$: toggle @open

                --- ftd.text: Open True
                if: @open

                --- ftd.text: Open False
                if: not @open

                -- foo:
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn event_toggle_for_loop() {
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd_rt::Element::Column(ftd_rt::Column {
                container: ftd_rt::Container {
                    children: vec![
                        ftd_rt::Element::Text(ftd_rt::Text {
                            text: ftd::markdown_line("ab title"),
                            line: true,
                            common: ftd_rt::Common {
                                events: vec![ftd_rt::Event {
                                    name: s("onclick"),
                                    action: ftd_rt::Action {
                                        action: s("toggle"),
                                        target: s("@open@0"),
                                        parameters: Default::default(),
                                    },
                                }],
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd_rt::Element::Column(ftd_rt::Column {
                            container: ftd_rt::Container {
                                children: vec![ftd_rt::Element::Text(ftd_rt::Text {
                                    text: ftd::markdown_line("aa title"),
                                    line: true,
                                    common: ftd_rt::Common {
                                        events: vec![ftd_rt::Event {
                                            name: s("onclick"),
                                            action: ftd_rt::Action {
                                                action: s("toggle"),
                                                target: s("@open@0,1"),
                                                parameters: Default::default(),
                                            },
                                        }],
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                })],
                                ..Default::default()
                            },
                            common: ftd_rt::Common {
                                locals: std::array::IntoIter::new([(s("open@0,1"), s("true"))])
                                    .collect(),
                                condition: Some(ftd_rt::Condition {
                                    variable: s("@open@0"),
                                    value: s("true"),
                                }),
                                ..Default::default()
                            },
                        }),
                        ftd_rt::Element::Column(ftd_rt::Column {
                            container: ftd_rt::Container {
                                children: vec![ftd_rt::Element::Text(ftd_rt::Text {
                                    text: ftd::markdown_line("aaa title"),
                                    line: true,
                                    common: ftd_rt::Common {
                                        events: vec![ftd_rt::Event {
                                            name: s("onclick"),
                                            action: ftd_rt::Action {
                                                action: s("toggle"),
                                                target: s("@open@0,2"),
                                                parameters: Default::default(),
                                            },
                                        }],
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                })],
                                ..Default::default()
                            },
                            common: ftd_rt::Common {
                                locals: std::array::IntoIter::new([(s("open@0,2"), s("true"))])
                                    .collect(),
                                condition: Some(ftd_rt::Condition {
                                    variable: s("@open@0"),
                                    value: s("true"),
                                }),
                                ..Default::default()
                            },
                        }),
                    ],
                    ..Default::default()
                },
                common: ftd_rt::Common {
                    locals: std::array::IntoIter::new([(s("open@0"), s("true"))]).collect(),
                    ..Default::default()
                },
            }));

        let (_g_bag, g_col) = crate::p2::interpreter::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- record toc-record:
                title: string
                children: list toc-record

                -- component toc-item:
                component: ftd.column
                $toc: toc-record
                @open: boolean with default true

                --- ftd.text: ref $toc.title
                $event-click$: toggle @open

                --- toc-item:
                if: @open
                $loop$: $toc.children as obj
                toc: ref obj

                -- list aa:
                type: toc-record

                -- aa:
                title: aa title

                -- aa:
                title: aaa title

                -- list toc:
                type: toc-record

                -- toc:
                title: ab title
                children: ref aa

                -- toc-item:
                $loop$: toc as obj
                toc: ref obj
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn test_local_variable() {
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd_rt::Element::Column(ftd_rt::Column {
                container: ftd_rt::Container {
                    children: vec![ftd_rt::Element::Column(ftd_rt::Column {
                        container: ftd_rt::Container {
                            children: vec![
                                ftd_rt::Element::Column(ftd_rt::Column {
                                    container: ftd_rt::Container {
                                        children: vec![
                                            ftd_rt::Element::Text(ftd_rt::Text {
                                                text: ftd::markdown_line("Click here!"),
                                                line: true,
                                                common: ftd_rt::Common {
                                                    events: vec![ftd_rt::Event {
                                                        name: s("onclick"),
                                                        action: ftd_rt::Action {
                                                            action: s("toggle"),
                                                            target: s("@open@0"),
                                                            parameters: Default::default(),
                                                        },
                                                    }],
                                                    ..Default::default()
                                                },
                                                ..Default::default()
                                            }),
                                            ftd_rt::Element::Text(ftd_rt::Text {
                                                text: ftd::markdown_line("Hello"),
                                                line: true,
                                                ..Default::default()
                                            }),
                                        ],
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                }),
                                ftd_rt::Element::Column(ftd_rt::Column {
                                    container: ftd_rt::Container {
                                        children: vec![ftd_rt::Element::Text(ftd_rt::Text {
                                            text: ftd::markdown_line("Hello Bar"),
                                            line: true,
                                            ..Default::default()
                                        })],
                                        ..Default::default()
                                    },
                                    common: ftd_rt::Common {
                                        locals: std::array::IntoIter::new([(
                                            s("open-bar@0,0,1"),
                                            s("true"),
                                        )])
                                        .collect(),
                                        condition: Some(ftd_rt::Condition {
                                            variable: s("@open@0"),
                                            value: s("true"),
                                        }),
                                        ..Default::default()
                                    },
                                }),
                            ],
                            ..Default::default()
                        },
                        common: ftd_rt::Common {
                            id: Some(s("foo-id")),
                            ..Default::default()
                        },
                    })],
                    ..Default::default()
                },
                common: ftd_rt::Common {
                    locals: std::array::IntoIter::new([(s("open@0"), s("true"))]).collect(),
                    ..Default::default()
                },
            }));

        let (_g_bag, g_col) = crate::p2::interpreter::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- component bar:
                component: ftd.column
                @open-bar: boolean with default true

                --- ftd.text: Hello Bar


                -- component foo:
                component: ftd.column
                @open: boolean with default true

                --- ftd.column:
                id: foo-id

                --- ftd.column:

                --- ftd.text: Click here!
                $event-click$: toggle @open

                --- ftd.text: Hello

                --- container: foo-id

                --- bar:
                if: @open


                -- foo:
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn if_on_var_integer() {
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd_rt::Element::Integer(ftd_rt::Text {
                text: markdown_line("20"),
                common: ftd_rt::Common {
                    reference: Some(s("foo/bar#bar")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (_g_bag, g_col) = crate::p2::interpreter::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- var foo: false

                -- var bar: 10

                -- bar: 20
                if: not foo

                -- ftd.integer:
                value: ref bar

                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn if_on_var_text() {
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd_rt::Element::Text(ftd_rt::Text {
                text: markdown_line("other-foo says hello"),
                line: true,
                common: ftd_rt::Common {
                    reference: Some(s("foo/bar#bar")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (_g_bag, g_col) = crate::p2::interpreter::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- var foo: false

                -- var other-foo: true

                -- var bar: hello

                -- bar: foo says hello
                if: not foo

                -- bar: other-foo says hello
                if: other-foo

                -- ftd.text: ref bar

                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn cursor_pointer() {
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd_rt::Element::Text(ftd_rt::Text {
                text: markdown_line("hello"),
                line: true,
                common: ftd_rt::Common {
                    cursor: Some(s("pointer")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (_g_bag, g_col) = crate::p2::interpreter::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- ftd.text: hello
                cursor: pointer

                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn comments() {
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown("hello2"),
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown("/hello3"),
                line: false,
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

        main.container
            .children
            .push(ftd_rt::Element::Row(ftd_rt::Row {
                container: ftd_rt::Container {
                    children: vec![ftd_rt::Element::Text(ftd_rt::Text {
                        text: ftd::markdown_line("hello5"),
                        line: true,
                        common: ftd_rt::Common {
                            color: Some(ftd_rt::Color {
                                r: 0,
                                g: 128,
                                b: 0,
                                alpha: 1.0,
                            }),
                            ..Default::default()
                        },
                        ..Default::default()
                    })],
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd_rt::Element::Row(ftd_rt::Row {
                container: ftd_rt::Container {
                    children: vec![ftd_rt::Element::Text(ftd_rt::Text {
                        text: ftd::markdown("/foo says hello"),
                        ..Default::default()
                    })],
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (_g_bag, g_col) = crate::p2::interpreter::interpret(
            "foo/bar",
            indoc::indoc!(
                r"
                /-- ftd.text:
                cursor: pointer

                hello1

                -- ftd.text:
                /color: red

                hello2

                -- ftd.text:
                color: red

                \/hello3

                -- ftd.row:

                /--- ftd.text: hello4

                --- ftd.text: hello5
                color: green
                /padding-left: 20

                -- component foo:
                component: ftd.row
                /color: red

                --- ftd.text:

                \/foo says hello

                /--- ftd.text: foo says hello again

                -- foo:

                /-- foo:
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn component_declaration_anywhere_2() {
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd_rt::Element::Column(ftd_rt::Column {
                container: ftd_rt::Container {
                    children: vec![
                        ftd_rt::Element::Column(ftd_rt::Column {
                            container: ftd_rt::Container {
                                children: vec![
                                    ftd_rt::Element::Text(ftd_rt::Text {
                                        text: ftd::markdown_line("Bar says hello"),
                                        line: true,
                                        ..Default::default()
                                    }),
                                    ftd_rt::Element::Text(ftd_rt::Text {
                                        text: ftd::markdown_line("Hello"),
                                        line: true,
                                        common: ftd_rt::Common {
                                            reference: Some(s("foo/bar#greeting")),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    }),
                                ],
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd_rt::Element::Text(ftd_rt::Text {
                            text: ftd::markdown_line("foo says hello"),
                            line: true,
                            ..Default::default()
                        }),
                        ftd_rt::Element::Text(ftd_rt::Text {
                            text: ftd::markdown_line("Hello"),
                            line: true,
                            common: ftd_rt::Common {
                                reference: Some(s("foo/bar#greeting")),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    ],
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (_g_bag, g_col) = crate::p2::interpreter::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- foo:

                -- component foo:
                component: ftd.column

                --- bar: Bar says hello

                --- ftd.text: foo says hello

                --- ftd.text: ref greeting

                -- var greeting: Hello

                -- component bar:
                component: ftd.column
                $name: caption

                --- ftd.text: ref $name

                --- ftd.text: ref greeting
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn action_increment_decrement_condition() {
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd_rt::Element::Integer(ftd_rt::Text {
                text: ftd::markdown_line("0"),
                common: ftd_rt::Common {
                    reference: Some(s("foo/bar#count")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("Hello on 8"),
                line: true,
                common: ftd_rt::Common {
                    condition: Some(ftd_rt::Condition {
                        variable: s("foo/bar#count"),
                        value: s("8"),
                    }),
                    is_not_visible: true,
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("increment counter"),
                line: true,
                common: ftd_rt::Common {
                    events: vec![ftd_rt::Event {
                        name: s("onclick"),
                        action: ftd_rt::Action {
                            action: s("increment"),
                            target: s("foo/bar#count"),
                            parameters: Default::default(),
                        },
                    }],
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("decrement counter"),
                line: true,
                common: ftd_rt::Common {
                    events: vec![ftd_rt::Event {
                        name: s("onclick"),
                        action: ftd_rt::Action {
                            action: s("decrement"),
                            target: s("foo/bar#count"),
                            parameters: Default::default(),
                        },
                    }],
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("increment counter"),
                line: true,
                common: ftd_rt::Common {
                    events: vec![ftd_rt::Event {
                        name: s("onclick"),
                        action: ftd_rt::Action {
                            action: s("increment"),
                            target: s("foo/bar#count"),
                            parameters: std::array::IntoIter::new([(s("by"), vec![s("2")])])
                                .collect(),
                        },
                    }],
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("increment counter by 2 clamp 2 10"),
                line: true,
                common: ftd_rt::Common {
                    events: vec![ftd_rt::Event {
                        name: s("onclick"),
                        action: ftd_rt::Action {
                            action: s("increment"),
                            target: s("foo/bar#count"),
                            parameters: std::array::IntoIter::new([
                                (s("by"), vec![s("2")]),
                                (s("clamp"), vec![s("2"), s("10")]),
                            ])
                            .collect(),
                        },
                    }],
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd_rt::Element::Text(ftd_rt::Text {
                text: ftd::markdown_line("decrement count clamp 2 10"),
                line: true,
                common: ftd_rt::Common {
                    events: vec![ftd_rt::Event {
                        name: s("onclick"),
                        action: ftd_rt::Action {
                            action: s("decrement"),
                            target: s("foo/bar#count"),
                            parameters: std::array::IntoIter::new([(
                                s("clamp"),
                                vec![s("2"), s("10")],
                            )])
                            .collect(),
                        },
                    }],
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (_g_bag, g_col) = crate::p2::interpreter::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- var count: 0

                -- ftd.integer:
                value: ref count

                -- ftd.text: Hello on 8
                if: count == 8

                -- ftd.text: increment counter
                $event-click$: increment count

                -- ftd.text: decrement counter
                $event-click$: decrement count

                -- ftd.text: increment counter
                $event-click$: increment count by 2

                -- ftd.text: increment counter by 2 clamp 2 10
                $event-click$: increment count by 2 clamp 2 10

                -- ftd.text: decrement count clamp 2 10
                $event-click$: decrement count clamp 2 10
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn action_increment_decrement_local_variable() {
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd_rt::Element::Column(ftd_rt::Column {
                container: ftd_rt::Container {
                    children: vec![
                        ftd_rt::Element::Integer(ftd_rt::Text {
                            text: ftd::markdown_line("0"),
                            common: ftd_rt::Common {
                                reference: Some(s("@count@0")),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd_rt::Element::Text(ftd_rt::Text {
                            text: ftd::markdown_line("increment counter"),
                            line: true,
                            common: ftd_rt::Common {
                                events: vec![ftd_rt::Event {
                                    name: s("onclick"),
                                    action: ftd_rt::Action {
                                        action: s("increment"),
                                        target: s("@count@0"),
                                        parameters: std::array::IntoIter::new([(
                                            s("by"),
                                            vec![s("3")],
                                        )])
                                        .collect(),
                                    },
                                }],
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd_rt::Element::Text(ftd_rt::Text {
                            text: ftd::markdown_line("decrement counter"),
                            line: true,
                            common: ftd_rt::Common {
                                events: vec![ftd_rt::Event {
                                    name: s("onclick"),
                                    action: ftd_rt::Action {
                                        action: s("decrement"),
                                        target: s("@count@0"),
                                        parameters: std::array::IntoIter::new([(
                                            s("by"),
                                            vec![s("2")],
                                        )])
                                        .collect(),
                                    },
                                }],
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    ],
                    ..Default::default()
                },
                common: ftd_rt::Common {
                    locals: std::array::IntoIter::new([(s("count@0"), s("0"))]).collect(),
                    ..Default::default()
                },
            }));

        let (_g_bag, g_col) = crate::p2::interpreter::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- var decrement-by: 2

                -- component foo:
                component: ftd.column
                $by: integer with default 4
                @count: integer with default 0

                --- ftd.integer:
                value: ref @count

                --- ftd.text: increment counter
                $event-click$: increment @count by $by

                --- ftd.text: decrement counter
                $event-click$: decrement @count by decrement-by

                -- foo:
                by: 3

                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn nested_component() {
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd_rt::Element::Row(ftd_rt::Row {
                ..Default::default()
            }));

        let (_g_bag, g_col) = crate::p2::interpreter::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- secondary-button: CTA says Hello

                -- component secondary-button:
                component: secondary-button-1
                $cta: caption
                cta: ref $cta


                -- component secondary-button-1:
                component: ftd.row
                $cta: caption

                --- ftd.text: ref $cta
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn action_increment_decrement_on_component() {
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd_rt::Element::Image(ftd_rt::Image {
                src: s("https://www.liveabout.com/thmb/YCJmu1khSJo8kMYM090QCd9W78U=/1250x0/filters:no_upscale():max_bytes(150000):strip_icc():format(webp)/powerpuff_girls-56a00bc45f9b58eba4aea61d.jpg"),
                common: ftd_rt::Common {
                    condition: Some(
                        ftd_rt::Condition {
                            variable: s("foo/bar#count"),
                            value: s("0"),
                        },
                    ),
                    is_not_visible: false,
                    events: vec![
                        ftd_rt::Event {
                            name: s("onclick"),
                            action: ftd_rt::Action {
                                action: s("increment"),
                                target: s("foo/bar#count"),
                                parameters: std::array::IntoIter::new([(s("clamp"), vec![s("0"), s("1")])])
                                    .collect(),
                            },
                        },
                    ],
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd_rt::Element::Image(ftd_rt::Image {
                src: s("https://upload.wikimedia.org/wikipedia/en/d/d4/Mickey_Mouse.png"),
                common: ftd_rt::Common {
                    condition: Some(ftd_rt::Condition {
                        variable: s("foo/bar#count"),
                        value: s("1"),
                    }),
                    is_not_visible: true,
                    events: vec![ftd_rt::Event {
                        name: s("onclick"),
                        action: ftd_rt::Action {
                            action: s("increment"),
                            target: s("foo/bar#count"),
                            parameters: std::array::IntoIter::new([(
                                s("clamp"),
                                vec![s("0"), s("1")],
                            )])
                            .collect(),
                        },
                    }],
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (_g_bag, g_col) = crate::p2::interpreter::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- var count: 0

                -- component slide:
                component: ftd.image
                $src: string
                $idx: integer
                src: ref $src
                if: count == $idx
                $event-click$: increment count clamp 0 1

                -- slide:
                src: https://www.liveabout.com/thmb/YCJmu1khSJo8kMYM090QCd9W78U=/1250x0/filters:no_upscale():max_bytes(150000):strip_icc():format(webp)/powerpuff_girls-56a00bc45f9b58eba4aea61d.jpg
                idx: 0

                -- slide:
                src: https://upload.wikimedia.org/wikipedia/en/d/d4/Mickey_Mouse.png
                idx: 1
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn loop_on_list_string() {
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd_rt::Element::Column(ftd_rt::Column {
                container: ftd_rt::Container {
                    children: vec![
                        ftd_rt::Element::Text(ftd_rt::Text {
                            text: ftd::markdown_line("Arpita"),
                            line: true,
                            ..Default::default()
                        }),
                        ftd_rt::Element::Text(ftd_rt::Text {
                            text: ftd::markdown_line("Ayushi"),
                            line: true,
                            ..Default::default()
                        }),
                        ftd_rt::Element::Text(ftd_rt::Text {
                            text: ftd::markdown_line("AmitU"),
                            line: true,
                            ..Default::default()
                        }),
                    ],
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (_g_bag, g_col) = crate::p2::interpreter::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- component foo:
                component: ftd.column
                $bar: list string

                --- ftd.text: ref obj
                $loop$: $bar as obj

                -- list names:
                type: string

                -- names: Arpita

                -- names: Ayushi

                -- names: AmitU

                -- foo:
                bar: ref names
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn open_container_with_parent_id() {
        let mut main = super::default_column();
        let beverage_external_children = vec![ftd_rt::Element::Column(ftd_rt::Column {
            container: ftd_rt::Container {
                children: vec![
                    ftd_rt::Element::Column(ftd_rt::Column {
                        container: ftd_rt::Container {
                            children: vec![
                                ftd_rt::Element::Text(ftd_rt::Text {
                                    text: ftd::markdown_line("Water"),
                                    line: true,
                                    common: ftd_rt::Common {
                                        events: vec![ftd_rt::Event {
                                            name: s("onclick"),
                                            action: ftd_rt::Action {
                                                action: s("toggle"),
                                                target: s("@visible@0,0,0"),
                                                ..Default::default()
                                            },
                                        }],
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                }),
                                ftd_rt::Element::Column(ftd_rt::Column {
                                    common: ftd_rt::Common {
                                        condition: Some(ftd_rt::Condition {
                                            variable: s("@visible@0,0,0"),
                                            value: s("true"),
                                        }),
                                        id: Some(s("some-child")),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                }),
                            ],
                            external_children: Some((s("some-child"), vec![vec![1]], vec![])),
                            open: (None, Some(s("some-child"))),
                            ..Default::default()
                        },
                        common: ftd_rt::Common {
                            locals: std::array::IntoIter::new([(s("visible@0,0,0"), s("true"))])
                                .collect(),
                            ..Default::default()
                        },
                    }),
                    ftd_rt::Element::Column(ftd_rt::Column {
                        container: ftd_rt::Container {
                            children: vec![
                                ftd_rt::Element::Text(ftd_rt::Text {
                                    text: ftd::markdown_line("Juice"),
                                    line: true,
                                    common: ftd_rt::Common {
                                        events: vec![ftd_rt::Event {
                                            name: s("onclick"),
                                            action: ftd_rt::Action {
                                                action: s("toggle"),
                                                target: s("@visible@0,0,1"),
                                                ..Default::default()
                                            },
                                        }],
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                }),
                                ftd_rt::Element::Column(ftd_rt::Column {
                                    common: ftd_rt::Common {
                                        condition: Some(ftd_rt::Condition {
                                            variable: s("@visible@0,0,1"),
                                            value: s("true"),
                                        }),
                                        id: Some(s("some-child")),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                }),
                            ],
                            external_children: Some((
                                s("some-child"),
                                vec![vec![1]],
                                vec![ftd_rt::Element::Column(ftd_rt::Column {
                                    container: ftd_rt::Container {
                                        children: vec![ftd_rt::Element::Column(ftd_rt::Column {
                                            container: ftd_rt::Container {
                                                children: vec![
                                                    ftd_rt::Element::Text(ftd_rt::Text {
                                                        text: ftd::markdown_line("Mango Juice"),
                                                        line: true,
                                                        common: ftd_rt::Common {
                                                            events: vec![ftd_rt::Event {
                                                                name: s("onclick"),
                                                                action: ftd_rt::Action {
                                                                    action: s("toggle"),
                                                                    target: s("@visible@0,0,1,0"),
                                                                    ..Default::default()
                                                                },
                                                            }],
                                                            ..Default::default()
                                                        },
                                                        ..Default::default()
                                                    }),
                                                    ftd_rt::Element::Column(ftd_rt::Column {
                                                        common: ftd_rt::Common {
                                                            condition: Some(ftd_rt::Condition {
                                                                variable: s("@visible@0,0,1,0"),
                                                                value: s("true"),
                                                            }),
                                                            id: Some(s("some-child")),
                                                            ..Default::default()
                                                        },
                                                        ..Default::default()
                                                    }),
                                                ],
                                                external_children: Some((
                                                    s("some-child"),
                                                    vec![vec![1]],
                                                    vec![],
                                                )),
                                                open: (None, Some(s("some-child"))),
                                                ..Default::default()
                                            },
                                            common: ftd_rt::Common {
                                                locals: std::array::IntoIter::new([(
                                                    s("visible@0,0,1,0"),
                                                    s("true"),
                                                )])
                                                .collect(),
                                                ..Default::default()
                                            },
                                        })],
                                        align: ftd_rt::Align::Center,
                                        wrap: true,
                                        ..Default::default()
                                    },
                                    common: ftd_rt::Common {
                                        width: Some(ftd_rt::Length::Fill),
                                        height: Some(ftd_rt::Length::Fill),
                                        ..Default::default()
                                    },
                                })],
                            )),
                            open: (None, Some(s("some-child"))),
                            ..Default::default()
                        },
                        common: ftd_rt::Common {
                            locals: std::array::IntoIter::new([(s("visible@0,0,1"), s("true"))])
                                .collect(),
                            ..Default::default()
                        },
                    }),
                ],
                align: ftd_rt::Align::Center,
                wrap: true,
                ..Default::default()
            },
            common: ftd_rt::Common {
                width: Some(ftd_rt::Length::Fill),
                height: Some(ftd_rt::Length::Fill),
                ..Default::default()
            },
        })];

        main.container
            .children
            .push(ftd_rt::Element::Column(ftd_rt::Column {
                container: ftd_rt::Container {
                    children: vec![ftd_rt::Element::Column(ftd_rt::Column {
                        container: ftd_rt::Container {
                            children: vec![
                                ftd_rt::Element::Text(ftd_rt::Text {
                                    text: ftd::markdown_line("Beverage"),
                                    line: true,
                                    common: ftd_rt::Common {
                                        events: vec![ftd_rt::Event {
                                            name: s("onclick"),
                                            action: ftd_rt::Action {
                                                action: s("toggle"),
                                                target: s("@visible@0,0"),
                                                ..Default::default()
                                            },
                                        }],
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                }),
                                ftd_rt::Element::Column(ftd_rt::Column {
                                    common: ftd_rt::Common {
                                        condition: Some(ftd_rt::Condition {
                                            variable: s("@visible@0,0"),
                                            value: s("true"),
                                        }),
                                        id: Some(s("some-child")),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                }),
                            ],
                            external_children: Some((
                                s("some-child"),
                                vec![vec![1]],
                                beverage_external_children,
                            )),
                            open: (None, Some(s("some-child"))),
                            ..Default::default()
                        },
                        common: ftd_rt::Common {
                            locals: std::array::IntoIter::new([(s("visible@0,0"), s("true"))])
                                .collect(),
                            id: Some(s("beverage")),
                            ..Default::default()
                        },
                    })],
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (_g_bag, g_col) = crate::p2::interpreter::interpret(
            "foo/bar",
            indoc::indoc!(
                "
            -- component display-item1:
            component: ftd.column
            $name: string
            open: some-child
            @visible: boolean with default true

            --- ftd.text: ref $name
            $event-click$: toggle @visible

            --- ftd.column:
            if: @visible
            id: some-child

            -- ftd.column:

            -- display-item1:
            name: Beverage
            id: beverage


            -- display-item1:
            name: Water


            -- container: beverage


            -- display-item1:
            name: Juice


            -- display-item1:
            name: Mango Juice
            "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn text_check() {
        let mut main = super::default_column();
        main.container
            .children
            .push(ftd_rt::Element::Column(ftd_rt::Column {
                container: ftd_rt::Container {
                    children: vec![
                        ftd_rt::Element::Text(ftd_rt::Text {
                            text: ftd::markdown_line("$hello"),
                            line: true,
                            ..Default::default()
                        }),
                        ftd_rt::Element::Text(ftd_rt::Text {
                            text: ftd::markdown_line("hello"),
                            line: true,
                            ..Default::default()
                        }),
                        ftd_rt::Element::Text(ftd_rt::Text {
                            text: ftd::markdown_line("hello"),
                            line: true,
                            common: ftd_rt::Common {
                                reference: Some(s("foo/bar#hello")),
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
                ..Default::default()
            }));

        let (_g_bag, g_col) = crate::p2::interpreter::interpret(
            "foo/bar",
            indoc::indoc!(
                r"
                -- var hello: hello

                -- component foo:
                component: ftd.column
                $hello: string

                --- ftd.text: $hello

                --- ftd.text: ref $hello

                --- ftd.text: ref hello

                --- ftd.text: ref 'hello'

                -- foo:
                hello: ref hello
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn caption() {
        let mut main = super::default_column();

        main.container
            .children
            .push(ftd_rt::Element::Integer(ftd_rt::Text {
                text: ftd::markdown_line("32"),
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd_rt::Element::Boolean(ftd_rt::Text {
                text: ftd::markdown_line("true"),
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd_rt::Element::Decimal(ftd_rt::Text {
                text: ftd::markdown_line("0.06"),
                ..Default::default()
            }));

        let (_g_bag, g_col) = crate::p2::interpreter::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- ftd.integer: 32

                -- ftd.boolean: true

                -- ftd.decimal: 0.06
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    /*#[test]
    fn loop_with_tree_structure_1() {
        let (g_bag, g_col) = crate::p2::interpreter::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- record toc-record:
                title: string
                link: string
                children: list toc-record

                -- component toc-item:
                component: ftd.column
                $toc: toc-record
                padding-left: 10

                --- ftd.text: ref $toc.title
                link: ref $toc.link

                --- toc-item:
                $loop$: $toc.children as obj
                toc: ref obj


                -- list toc:
                type: toc-record

                -- toc:
                title: ref ab.title
                link: ref ab.link
                children: ref ab.children

                -- var ab:
                type: toc-record
                title: ab title
                link: ab link

                -- var first_ab
                type: ab.children
                title: aa title
                link: aa link

                --- children:
                title:

                -- ab.children:
                title: aaa title
                link: aaa link



                -- toc-item:
                $loop$: toc as obj
                toc: ref obj
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");
        // pretty_assertions::assert_eq!(g_bag, bag);
        // pretty_assertions::assert_eq!(g_col, main);
        // --- toc-item:
        //                 $loop$: $toc.children as t
        //                 toc: ref t
    }

    #[test]
    fn loop_with_tree_structure_2() {
        let (g_bag, g_col) = crate::p2::interpreter::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- record toc-record:
                title: string
                link: string
                children: list toc-record

                -- component toc-item:
                component: ftd.column
                $toc: toc-record
                padding-left: 10

                --- ftd.text: ref $toc.title
                link: ref $toc.link

                --- toc-item:
                $loop$: $toc.children as obj
                toc: ref obj


                -- list toc:
                type: toc-record
                $processor$: ft.toc

                - fifthtry/ftd/p1
                  `ftd::p1`: A JSON/YML Replacement
                - fifthtry/ftd/language
                  FTD Language
                  - fifthtry/ftd/p1-grammar
                    `ftd::p1` grammar




                -- toc-item:
                $loop$: toc as obj
                toc: ref obj
                "
            ),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");
        // pretty_assertions::assert_eq!(g_bag, bag);
        // pretty_assertions::assert_eq!(g_col, main);
        // --- toc-item:
        //                 $loop$: $toc.children as t
        //                 toc: ref t
    }*/
}
