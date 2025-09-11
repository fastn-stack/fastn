pub use ftd::ftd2021::p2::interpreter::{default_bag, default_column};

#[test]
fn get_name() {
    assert_eq!(
        ftd::ftd2021::p2::utils::get_name("fn", "fn foo", "test").unwrap(),
        "foo"
    )
}

/// returns the universal arguments map from component.rs as vector
fn universal_arguments_as_vec() -> Vec<(String, ftd::ftd2021::p2::Kind)> {
    ftd::ftd2021::component::universal_arguments()
        .into_iter()
        .collect::<Vec<(String, ftd::ftd2021::p2::Kind)>>()
}

/// returns the universal argumnts map from component.rs
fn universal_arguments_as_map() -> ftd::Map<ftd::ftd2021::p2::Kind> {
    ftd::ftd2021::component::universal_arguments()
}

pub fn interpret_helper(
    name: &str,
    source: &str,
    lib: &ftd::ftd2021::p2::TestLibrary,
) -> ftd::ftd2021::p1::Result<ftd::ftd2021::p2::Document> {
    let mut s = ftd::ftd2021::p2::interpreter::interpret(name, source, &None)?;
    let document;
    loop {
        match s {
            ftd::ftd2021::p2::interpreter::Interpreter::Done { document: doc } => {
                document = doc;
                break;
            }
            ftd::ftd2021::p2::interpreter::Interpreter::StuckOnProcessor { state, section } => {
                let value = lib.process(
                    &section,
                    &state.tdoc(&mut Default::default(), &mut Default::default()),
                )?;
                s = state.continue_after_processor(&section, value)?;
            }
            ftd::ftd2021::p2::interpreter::Interpreter::StuckOnImport { module, state: st } => {
                let source = lib.get_with_result(
                    module.as_str(),
                    &st.tdoc(&mut Default::default(), &mut Default::default()),
                )?;
                s = st.continue_after_import(module.as_str(), source.as_str())?;
            }
            ftd::ftd2021::p2::interpreter::Interpreter::StuckOnForeignVariable {
                state, ..
            } => {
                s = state.continue_after_variable(
                    "foo",
                    ftd::Value::String {
                        text: "This is a test".to_string(),
                        source: ftd::TextSource::Header,
                    },
                )?;
            }
            ftd::ftd2021::Interpreter::CheckID { .. } => {
                // No config in TestLibrary ignoring processing terms for now
                unimplemented!()
            }
        }
    }
    Ok(document)
}

pub fn interpret(
    name: &str,
    source: &str,
    lib: &ftd::ftd2021::p2::TestLibrary,
) -> ftd::ftd2021::p1::Result<(ftd::Map<ftd::ftd2021::p2::Thing>, ftd::Column)> {
    let doc = ftd::ftd2021::test::interpret_helper(name, source, lib)?;
    Ok((doc.data, doc.main))
}

macro_rules! p {
    ($s:expr, $t: expr,) => {
        p!($s, $t)
    };
    ($s:expr, $t: expr) => {
        let (ebag, ecol): (ftd::Map<ftd::ftd2021::p2::Thing>, _) = $t;
        let (mut bag, col) = ftd::ftd2021::test::interpret(
            "foo/bar",
            indoc::indoc!($s),
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .expect("found error");
        for v in bag.values_mut() {
            if let ftd::ftd2021::p2::Thing::Component(c) = v {
                c.invocations.clear();
                c.line_number = 0;
                for instruction in &mut c.instructions {
                    instruction.without_line_number()
                }
            }
        }
        bag.retain(|k, _| {
            !["SIBLING-INDEX", "CHILDREN-COUNT"]
                .iter()
                .any(|v| k.contains(v))
        });
        if !ebag.is_empty() {
            pretty_assertions::assert_eq!(bag, ebag);
        }
        pretty_assertions::assert_eq!(col, ecol);
    };
}

macro_rules! intf {
    ($s:expr, $m: expr,) => {
        intf!($s, $m)
    };
    ($s:expr, $m: expr) => {
        match ftd::ftd2021::test::interpret(
            "foo",
            indoc::indoc!($s),
            &ftd::ftd2021::p2::TestLibrary {},
        ) {
            Ok(some_value) => panic!("expected failure {:?}, found: {:?}", $m, some_value),
            Err(e) => {
                let expected_error = $m.trim();
                let err_found = e.to_string();
                let found = err_found.trim();
                if expected_error != found {
                    let patch = diffy::create_patch(expected_error, found);
                    let f = diffy::PatchFormatter::new().with_color();
                    print!(
                        "{}",
                        f.fmt_patch(&patch)
                            .to_string()
                            .replace("\\ No newline at end of file", "")
                    );
                    println!(
                        "expected error:\n{}\nfound:\n{}\n",
                        expected_error, err_found
                    );
                    panic!("test failed")
                }
            }
        }
    };
}

pub fn s(s: &str) -> String {
    s.to_string()
}

pub fn i(p: &str, reference: Option<String>) -> ftd::ImageSrc {
    ftd::ImageSrc {
        light: s(p),
        dark: s(p),
        reference,
    }
}

pub fn person_fields() -> ftd::Map<ftd::ftd2021::p2::Kind> {
    std::iter::IntoIterator::into_iter([
        (s("address"), ftd::ftd2021::p2::Kind::string()),
        (s("bio"), ftd::ftd2021::p2::Kind::body()),
        (s("age"), ftd::ftd2021::p2::Kind::integer()),
        (s("name"), ftd::ftd2021::p2::Kind::caption()),
    ])
    .collect()
}

pub fn abrar() -> ftd::Map<ftd::PropertyValue> {
    std::iter::IntoIterator::into_iter([
        (
            s("name"),
            ftd::PropertyValue::Value {
                value: ftd::Value::String {
                    text: "Abrar Khan2".to_string(),
                    source: ftd::TextSource::Caption,
                },
            },
        ),
        (
            s("address"),
            ftd::PropertyValue::Value {
                value: ftd::Value::String {
                    text: "Bihar2".to_string(),
                    source: ftd::TextSource::Header,
                },
            },
        ),
        (
            s("bio"),
            ftd::PropertyValue::Value {
                value: ftd::Value::String {
                    text: "Software developer working at fifthtry2.".to_string(),
                    source: ftd::TextSource::Body,
                },
            },
        ),
        (
            s("age"),
            ftd::PropertyValue::Reference {
                kind: ftd::ftd2021::p2::Kind::integer(),
                name: s("foo/bar#x"),
            },
        ),
    ])
    .collect()
}

pub fn entity() -> ftd::ftd2021::p2::Thing {
    ftd::ftd2021::p2::Thing::OrType(ftd::ftd2021::OrType {
        name: s("foo/bar#entity"),
        variants: vec![
            ftd::ftd2021::p2::Record {
                name: s("foo/bar#entity.person"),
                fields: person_fields(),
                instances: Default::default(),
                order: vec![s("name"), s("address"), s("bio"), s("age")],
            },
            ftd::ftd2021::p2::Record {
                name: s("foo/bar#entity.company"),
                fields: std::iter::IntoIterator::into_iter([
                    (s("industry"), ftd::ftd2021::p2::Kind::string()),
                    (s("name"), ftd::ftd2021::p2::Kind::caption()),
                ])
                .collect(),
                instances: Default::default(),
                order: vec![s("name"), s("industry")],
            },
        ],
    })
}

mod interpreter {
    use ftd::ftd2021::p2;
    use ftd::ftd2021::p2::interpreter;
    use ftd::ftd2021::test::*;

    /// inserts integer variable with the given value in the bag
    fn insert_update_integer_by_root(
        root: &str,
        val: i64,
        bag: &mut ftd::Map<ftd::ftd2021::p2::Thing>,
    ) {
        // root => [doc_id]#[var_name]@[level]
        // root_parts = [ doc_id , var_name, level ]
        let root_parts: Vec<&str> = root.trim().split(['#', '@']).collect();
        let var_name = root_parts[1];

        let integer_thing = ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
            name: var_name.to_string(),
            value: ftd::PropertyValue::Value {
                value: ftd::Value::Integer { value: val },
            },
            conditions: vec![],
            flags: Default::default(),
        });

        if bag.contains_key(root) {
            bag.entry(root.to_string())
                .and_modify(|e| *e = integer_thing);
        } else {
            bag.insert(root.to_string(), integer_thing);
        }
    }

    /// inserts an optional variable in the bag having the
    /// given kind with default value = None
    fn insert_update_default_optional_type_by_root(
        root: &str,
        kind: ftd::ftd2021::p2::Kind,
        bag: &mut ftd::Map<ftd::ftd2021::p2::Thing>,
    ) {
        let root_parts: Vec<&str> = root.trim().split(['#', '@']).collect();
        let var_name = root_parts[1];

        let value = ftd::Value::default_optional_value_from_kind(kind);

        let optional_thing = ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
            name: var_name.to_string(),
            value: ftd::PropertyValue::Value { value },
            conditions: vec![],
            flags: Default::default(),
        });

        if bag.contains_key(root) {
            bag.entry(root.to_string())
                .and_modify(|e| *e = optional_thing);
        } else {
            bag.insert(root.to_string(), optional_thing);
        }
    }

    fn insert_update_default_optional_list_type_by_root(
        root: &str,
        kind: ftd::ftd2021::p2::Kind,
        bag: &mut ftd::Map<ftd::ftd2021::p2::Thing>,
    ) {
        let root_parts: Vec<&str> = root.trim().split(['#', '@']).collect();
        let var_name = root_parts[1];

        let value = ftd::Value::Optional {
            data: Box::new(Some(ftd::Value::List {
                data: vec![],
                kind: kind.clone(),
            })),
            kind: ftd::ftd2021::p2::Kind::list(kind),
        };

        let optional_thing = ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
            name: var_name.to_string(),
            value: ftd::PropertyValue::Value { value },
            conditions: vec![],
            flags: Default::default(),
        });

        if bag.contains_key(root) {
            bag.entry(root.to_string())
                .and_modify(|e| *e = optional_thing);
        } else {
            bag.insert(root.to_string(), optional_thing);
        }
    }

    /// inserts decimal variable with the given value in the bag
    fn insert_update_decimal_by_root(
        root: &str,
        value: f64,
        bag: &mut ftd::Map<ftd::ftd2021::p2::Thing>,
    ) {
        let root_parts: Vec<&str> = root.trim().split(['#', '@']).collect();
        let var_name = root_parts[1];

        let decimal_thing = ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
            name: var_name.to_string(),
            value: ftd::PropertyValue::Value {
                value: ftd::Value::Decimal { value },
            },
            conditions: vec![],
            flags: Default::default(),
        });

        if bag.contains_key(root) {
            bag.entry(root.to_string())
                .and_modify(|e| *e = decimal_thing);
        } else {
            bag.insert(root.to_string(), decimal_thing);
        }
    }

    /// inserts string variable with the given value in the bag
    fn insert_update_string_by_root(
        root: &str,
        val: &str,
        source_type: &str,
        bag: &mut ftd::Map<ftd::ftd2021::p2::Thing>,
    ) {
        let root_parts: Vec<&str> = root.trim().split(['#', '@']).collect();
        let var_name = root_parts[1];

        let string_thing = ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
            name: var_name.to_string(),
            value: ftd::PropertyValue::Value {
                value: ftd::Value::String {
                    text: val.to_string(),
                    source: match source_type.to_ascii_lowercase().as_str() {
                        "header" => ftd::TextSource::Header,
                        "caption" => ftd::TextSource::Caption,
                        "body" => ftd::TextSource::Body,
                        "default" => ftd::TextSource::Default,
                        _ => panic!("invalid text source provided"),
                    },
                },
            },
            conditions: vec![],
            flags: Default::default(),
        });

        if bag.contains_key(root) {
            bag.entry(root.to_string())
                .and_modify(|e| *e = string_thing);
        } else {
            bag.insert(root.to_string(), string_thing);
        }
    }

    /// generates root bag entry and returns it as String
    ///
    /// root_id = \[doc_id\]#\[var_name\]@\[level\]
    fn make_root<T: std::fmt::Display>(var_name: &str, doc_id: &str, count: T) -> String {
        format!("{doc_id}#{var_name}@{count}")
    }

    /// inserts all default universal arguments in the bag at the mentioned levels
    fn insert_universal_variables_by_levels(
        levels: Vec<String>,
        doc_id: &str,
        bag: &mut ftd::Map<ftd::ftd2021::p2::Thing>,
    ) {
        let universal_arguments_vec = universal_arguments_as_vec();

        for (arg, kind) in universal_arguments_vec.iter() {
            for level in levels.iter() {
                if kind.is_optional() {
                    if kind.inner().is_string_list() {
                        insert_update_default_optional_list_type_by_root(
                            make_root(arg, doc_id, level).as_str(),
                            ftd::ftd2021::p2::Kind::string(),
                            bag,
                        );
                    }
                    if kind.inner().is_string() {
                        insert_update_default_optional_type_by_root(
                            make_root(arg, doc_id, level).as_str(),
                            ftd::ftd2021::p2::Kind::string(),
                            bag,
                        );
                    }
                    if kind.inner().is_integer() {
                        insert_update_default_optional_type_by_root(
                            make_root(arg, doc_id, level).as_str(),
                            ftd::ftd2021::p2::Kind::integer(),
                            bag,
                        );
                    }
                    if kind.inner().is_decimal() {
                        insert_update_default_optional_type_by_root(
                            make_root(arg, doc_id, level).as_str(),
                            ftd::ftd2021::p2::Kind::decimal(),
                            bag,
                        );
                    }
                }
            }
        }
    }

    /// insert all universal arguments in the bag by count at top level
    fn insert_universal_variables_by_count(
        lim: i32,
        doc_id: &str,
        bag: &mut ftd::Map<ftd::ftd2021::p2::Thing>,
    ) {
        let mut count: i32 = 0;
        let universal_arguments_vec = universal_arguments_as_vec();

        while count < lim {
            for (arg, kind) in universal_arguments_vec.iter() {
                if kind.is_optional() {
                    if kind.inner().is_string_list() {
                        insert_update_default_optional_list_type_by_root(
                            make_root(arg, doc_id, count).as_str(),
                            ftd::ftd2021::p2::Kind::string(),
                            bag,
                        );
                    }
                    if kind.inner().is_string() {
                        insert_update_default_optional_type_by_root(
                            make_root(arg, doc_id, count).as_str(),
                            ftd::ftd2021::p2::Kind::string(),
                            bag,
                        );
                    }
                    if kind.inner().is_integer() {
                        insert_update_default_optional_type_by_root(
                            make_root(arg, doc_id, count).as_str(),
                            ftd::ftd2021::p2::Kind::integer(),
                            bag,
                        );
                    }
                    if kind.inner().is_decimal() {
                        insert_update_default_optional_type_by_root(
                            make_root(arg, doc_id, count).as_str(),
                            ftd::ftd2021::p2::Kind::decimal(),
                            bag,
                        );
                    }
                }
            }
            count += 1
        }
    }

    #[test]
    fn basic_1() {
        let mut bag = interpreter::default_bag();
        bag.insert(
            "foo/bar#foo".to_string(),
            ftd::ftd2021::p2::Thing::Component(ftd::Component {
                root: "ftd#text".to_string(),
                full_name: s("foo/bar#foo"),
                arguments: universal_arguments_as_map(),
                properties: std::iter::IntoIterator::into_iter([(
                    s("text"),
                    ftd::ftd2021::component::Property {
                        default: Some(ftd::PropertyValue::Value {
                            value: ftd::Value::String {
                                text: s("hello"),
                                source: ftd::TextSource::Header,
                            },
                        }),
                        conditions: vec![],
                        ..Default::default()
                    },
                )])
                .collect(),
                ..Default::default()
            }),
        );
        bag.insert(
            "foo/bar#x".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "x".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Integer { value: 10 },
                },
                conditions: vec![],
            }),
        );

        p!(
            "
            -- ftd.text foo:
            text: hello

            -- integer x: 10
            ",
            (bag, super::default_column()),
        );
    }

    #[test]
    fn conditional_attribute() {
        let mut bag = interpreter::default_bag();
        bag.insert(
            "foo/bar#foo".to_string(),
            ftd::ftd2021::p2::Thing::Component(ftd::Component {
                full_name: s("foo/bar#foo"),
                root: "ftd#text".to_string(),
                arguments: [
                    vec![(s("name"), ftd::ftd2021::p2::Kind::caption())],
                    universal_arguments_as_vec(),
                ]
                .concat()
                .into_iter()
                .collect(),
                properties: std::iter::IntoIterator::into_iter([
                    (
                        s("color"),
                        ftd::ftd2021::component::Property {
                            default: Some(ftd::PropertyValue::Reference {
                                name: s("foo/bar#white"),
                                kind: ftd::ftd2021::p2::Kind::Optional {
                                    kind: Box::new(ftd::ftd2021::p2::Kind::Record {
                                        name: s("ftd#color"),
                                        default: None,
                                        is_reference: false,
                                    }),
                                    is_reference: false,
                                },
                            }),
                            conditions: vec![
                                (
                                    ftd::ftd2021::p2::Boolean::Equal {
                                        left: ftd::PropertyValue::Reference {
                                            name: "foo/bar#present".to_string(),
                                            kind: ftd::ftd2021::p2::Kind::boolean(),
                                        },
                                        right: ftd::PropertyValue::Value {
                                            value: ftd::Value::Boolean { value: true },
                                        },
                                    },
                                    ftd::PropertyValue::Reference {
                                        name: s("foo/bar#green"),
                                        kind: ftd::ftd2021::p2::Kind::Optional {
                                            kind: Box::new(ftd::ftd2021::p2::Kind::Record {
                                                name: s("ftd#color"),
                                                default: None,
                                                is_reference: false,
                                            }),
                                            is_reference: false,
                                        },
                                    },
                                ),
                                (
                                    ftd::ftd2021::p2::Boolean::Equal {
                                        left: ftd::PropertyValue::Reference {
                                            name: "foo/bar#present".to_string(),
                                            kind: ftd::ftd2021::p2::Kind::boolean(),
                                        },
                                        right: ftd::PropertyValue::Value {
                                            value: ftd::Value::Boolean { value: false },
                                        },
                                    },
                                    ftd::PropertyValue::Reference {
                                        name: s("foo/bar#red"),
                                        kind: ftd::ftd2021::p2::Kind::Optional {
                                            kind: Box::new(ftd::ftd2021::p2::Kind::Record {
                                                name: s("ftd#color"),
                                                default: None,
                                                is_reference: false,
                                            }),
                                            is_reference: false,
                                        },
                                    },
                                ),
                            ],
                            ..Default::default()
                        },
                    ),
                    (
                        s("text"),
                        ftd::ftd2021::component::Property {
                            default: Some(ftd::PropertyValue::Variable {
                                name: "name".to_string(),
                                kind: ftd::ftd2021::p2::Kind::caption_or_body(),
                            }),
                            conditions: vec![],
                            ..Default::default()
                        },
                    ),
                ])
                .collect(),
                ..Default::default()
            }),
        );

        bag.insert(
            s("foo/bar#green"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("green"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Record {
                        name: s("ftd#color"),
                        fields: std::iter::IntoIterator::into_iter([
                            (
                                s("dark"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: s("green"),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                            (
                                s("light"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: s("green"),
                                        source: ftd::TextSource::Caption,
                                    },
                                },
                            ),
                        ])
                        .collect(),
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );

        bag.insert(
            s("foo/bar#red"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("red"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Record {
                        name: s("ftd#color"),
                        fields: std::iter::IntoIterator::into_iter([
                            (
                                s("dark"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: s("red"),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                            (
                                s("light"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: s("red"),
                                        source: ftd::TextSource::Caption,
                                    },
                                },
                            ),
                        ])
                        .collect(),
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );

        bag.insert(
            s("foo/bar#white"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("white"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Record {
                        name: s("ftd#color"),
                        fields: std::iter::IntoIterator::into_iter([
                            (
                                s("dark"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: s("white"),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                            (
                                s("light"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: s("white"),
                                        source: ftd::TextSource::Caption,
                                    },
                                },
                            ),
                        ])
                        .collect(),
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );

        bag.insert(
            "foo/bar#name@0".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "name".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("hello"),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            "foo/bar#present".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "present".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Boolean { value: false },
                },
                conditions: vec![],
            }),
        );

        insert_universal_variables_by_count(1, "foo/bar", &mut bag);

        let mut main = p2::default_column();
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("hello"),
                line: true,
                common: Box::new(ftd::Common {
                    color: Some(ftd::Color {
                        light: ftd::ColorValue {
                            r: 255,
                            g: 0,
                            b: 0,
                            alpha: 1.0,
                        }),
                        dark: ftd::ColorValue {
                            r: 255,
                            g: 0,
                            b: 0,
                            alpha: 1.0,
                        }),
                        reference: Some(s("foo/bar#red")),
                    }),
                    conditional_attribute: std::iter::IntoIterator::into_iter([(
                        s("color"),
                        ftd::ConditionalAttribute {
                            attribute_type: ftd::AttributeType::Style,
                            conditions_with_value: vec![
                                (
                                    ftd::Condition {
                                        variable: s("foo/bar#present"),
                                        value: serde_json::Value::Bool(true),
                                    }),
                                    ftd::ConditionalValue {
                                        value: serde_json::from_str("{\"$kind$\":\"light\",\"dark\":\"rgba(0,128,0,1)\",\"light\":\"rgba(0,128,0,1)\"}").unwrap(),
                                        important: false,
                                        reference: Some(s("foo/bar#green")),
                                    }),
                                ),
                                (
                                    ftd::Condition {
                                        variable: s("foo/bar#present"),
                                        value: serde_json::Value::Bool(false),
                                    }),
                                    ftd::ConditionalValue {
                                        value: serde_json::from_str("{\"$kind$\":\"light\",\"dark\":\"rgba(255,0,0,1)\",\"light\":\"rgba(255,0,0,1)\"}").unwrap(),
                                        important: false,
                                        reference: Some(s("foo/bar#red")),
                                    }),
                                ),
                            ],
                            default: Some(ftd::ConditionalValue {
                                value: serde_json::from_str("{\"$kind$\":\"light\",\"dark\":\"rgba(255,255,255,1)\",\"light\":\"rgba(255,255,255,1)\"}").unwrap(),
                                important: false,
                                reference: Some(s("foo/bar#white")),
                            }),
                        }),
                    )])
                        .collect(),
                    reference: Some(s("foo/bar#name@0")),
                    ..Default::default()
                }),
                ..Default::default()
            }));

        p!(
            "
            -- boolean present: false

            -- ftd.color red: red
            dark: red

            -- ftd.color green: green
            dark: green

            -- ftd.color white: white
            dark: white

            -- ftd.text foo:
            caption name:
            color: $white
            color if $present: $green
            color if not $present: $red
            text: $name

            -- foo: hello
            ",
            (bag, main),
        );
    }

    #[test]
    fn creating_a_tree() {
        let mut bag = interpreter::default_bag();

        bag.insert(
            "foo/bar#ft_toc".to_string(),
            ftd::ftd2021::p2::Thing::Component(ftd::Component {
                root: "ftd#column".to_string(),
                full_name: "foo/bar#ft_toc".to_string(),
                arguments: universal_arguments_as_map(),
                properties: Default::default(),
                instructions: vec![
                    ftd::ftd2021::component::Instruction::ChildComponent {
                        child: ftd::ftd2021::component::ChildComponent {
                            events: vec![],
                            root: "foo/bar#table-of-content".to_string(),
                            condition: None,
                            properties: std::iter::IntoIterator::into_iter([(
                                s("id"),
                                ftd::ftd2021::component::Property {
                                    default: Some(ftd::PropertyValue::Value {
                                        value: ftd::ftd2021::variable::Value::String {
                                            text: "toc_main".to_string(),
                                            source: ftd::TextSource::Header,
                                        },
                                    }),
                                    conditions: vec![],
                                    ..Default::default()
                                },
                            )])
                            .collect(),
                            arguments: Default::default(),
                            is_recursive: false,
                            ..Default::default()
                        },
                    },
                    ftd::ftd2021::component::Instruction::ChildComponent {
                        child: ftd::ftd2021::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "foo/bar#parent".to_string(),
                            condition: None,
                            properties: std::iter::IntoIterator::into_iter([
                                (
                                    s("active"),
                                    ftd::ftd2021::component::Property {
                                        default: Some(ftd::PropertyValue::Value {
                                            value: ftd::ftd2021::variable::Value::Boolean {
                                                value: true,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                                (
                                    s("id"),
                                    ftd::ftd2021::component::Property {
                                        default: Some(ftd::PropertyValue::Value {
                                            value: ftd::ftd2021::variable::Value::String {
                                                text: "/welcome/".to_string(),
                                                source: ftd::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                                (
                                    s("name"),
                                    ftd::ftd2021::component::Property {
                                        default: Some(ftd::PropertyValue::Value {
                                            value: ftd::ftd2021::variable::Value::String {
                                                text: "5PM Tasks".to_string(),
                                                source: ftd::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                            ])
                            .collect(),
                            ..Default::default()
                        },
                    },
                    ftd::ftd2021::component::Instruction::ChildComponent {
                        child: ftd::ftd2021::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "foo/bar#parent".to_string(),
                            condition: None,
                            properties: std::iter::IntoIterator::into_iter([
                                (
                                    s("id"),
                                    ftd::ftd2021::component::Property {
                                        default: Some(ftd::PropertyValue::Value {
                                            value: ftd::ftd2021::variable::Value::String {
                                                text: "/Building/".to_string(),
                                                source: ftd::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                                (
                                    s("name"),
                                    ftd::ftd2021::component::Property {
                                        default: Some(ftd::PropertyValue::Value {
                                            value: ftd::ftd2021::variable::Value::String {
                                                text: "Log".to_string(),
                                                source: ftd::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                            ])
                            .collect(),
                            ..Default::default()
                        },
                    },
                    ftd::ftd2021::component::Instruction::ChildComponent {
                        child: ftd::ftd2021::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "foo/bar#parent".to_string(),
                            condition: None,
                            properties: std::iter::IntoIterator::into_iter([
                                (
                                    s("id"),
                                    ftd::ftd2021::component::Property {
                                        default: Some(ftd::PropertyValue::Value {
                                            value: ftd::ftd2021::variable::Value::String {
                                                text: "/ChildBuilding/".to_string(),
                                                source: ftd::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                                (
                                    s("name"),
                                    ftd::ftd2021::component::Property {
                                        default: Some(ftd::PropertyValue::Value {
                                            value: ftd::ftd2021::variable::Value::String {
                                                text: "ChildLog".to_string(),
                                                source: ftd::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                            ])
                            .collect(),
                            ..Default::default()
                        },
                    },
                    ftd::ftd2021::component::Instruction::ChangeContainer {
                        name: "/welcome/".to_string(),
                    },
                    ftd::ftd2021::component::Instruction::ChildComponent {
                        child: ftd::ftd2021::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "foo/bar#parent".to_string(),
                            condition: None,
                            properties: std::iter::IntoIterator::into_iter([
                                (
                                    s("id"),
                                    ftd::ftd2021::component::Property {
                                        default: Some(ftd::PropertyValue::Value {
                                            value: ftd::ftd2021::variable::Value::String {
                                                text: "/Building2/".to_string(),
                                                source: ftd::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                                (
                                    s("name"),
                                    ftd::ftd2021::component::Property {
                                        default: Some(ftd::PropertyValue::Value {
                                            value: ftd::ftd2021::variable::Value::String {
                                                text: "Log2".to_string(),
                                                source: ftd::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                            ])
                            .collect(),
                            ..Default::default()
                        },
                    },
                ],
                kernel: false,
                ..Default::default()
            }),
        );

        bag.insert(
            "foo/bar#parent".to_string(),
            ftd::ftd2021::p2::Thing::Component(ftd::Component {
                root: "ftd#column".to_string(),
                full_name: "foo/bar#parent".to_string(),
                arguments: [
                    vec![
                        (
                            s("active"),
                            ftd::ftd2021::p2::Kind::Optional {
                                kind: Box::new(ftd::ftd2021::p2::Kind::boolean()),
                                is_reference: false,
                            },
                        ),
                        (s("name"), ftd::ftd2021::p2::Kind::caption()),
                    ],
                    universal_arguments_as_vec(),
                ]
                .concat()
                .into_iter()
                .collect(),
                properties: std::iter::IntoIterator::into_iter([
                    (
                        s("id"),
                        ftd::ftd2021::component::Property {
                            default: Some(ftd::PropertyValue::Variable {
                                name: "id".to_string(),
                                kind: ftd::ftd2021::p2::Kind::Optional {
                                    kind: Box::new(ftd::ftd2021::p2::Kind::string()),
                                    is_reference: false,
                                },
                            }),
                            conditions: vec![],
                            ..Default::default()
                        },
                    ),
                    (
                        s("open"),
                        ftd::ftd2021::component::Property {
                            default: Some(ftd::PropertyValue::Value {
                                value: ftd::Value::Boolean { value: true },
                            }),
                            conditions: vec![],
                            ..Default::default()
                        },
                    ),
                    (
                        s("width"),
                        ftd::ftd2021::component::Property {
                            default: Some(ftd::PropertyValue::Value {
                                value: ftd::ftd2021::variable::Value::String {
                                    text: "fill".to_string(),
                                    source: ftd::TextSource::Header,
                                },
                            }),
                            conditions: vec![],
                            ..Default::default()
                        },
                    ),
                ])
                .collect(),
                instructions: vec![
                    ftd::ftd2021::component::Instruction::ChildComponent {
                        child: ftd::ftd2021::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "ftd#text".to_string(),
                            condition: Some(ftd::ftd2021::p2::Boolean::IsNotNull {
                                value: ftd::PropertyValue::Variable {
                                    name: "active".to_string(),
                                    kind: ftd::ftd2021::p2::Kind::Optional {
                                        kind: Box::new(ftd::ftd2021::p2::Kind::boolean()),
                                        is_reference: false,
                                    },
                                },
                            }),
                            properties: std::iter::IntoIterator::into_iter([
                                (
                                    s("color"),
                                    ftd::ftd2021::component::Property {
                                        default: Some(ftd::PropertyValue::Reference {
                                            name: s("foo/bar#white"),
                                            kind: ftd::ftd2021::p2::Kind::Optional {
                                                kind: Box::new(ftd::ftd2021::p2::Kind::Record {
                                                    name: s("ftd#color"),
                                                    default: None,
                                                    is_reference: false,
                                                }),
                                                is_reference: false,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                                (
                                    s("text"),
                                    ftd::ftd2021::component::Property {
                                        default: Some(ftd::PropertyValue::Variable {
                                            name: "name".to_string(),
                                            kind: ftd::ftd2021::p2::Kind::caption_or_body(),
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                            ])
                            .collect(),
                            ..Default::default()
                        },
                    },
                    ftd::ftd2021::component::Instruction::ChildComponent {
                        child: ftd::ftd2021::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "ftd#text".to_string(),
                            condition: Some(ftd::ftd2021::p2::Boolean::IsNull {
                                value: ftd::PropertyValue::Variable {
                                    name: "active".to_string(),
                                    kind: ftd::ftd2021::p2::Kind::Optional {
                                        kind: Box::new(ftd::ftd2021::p2::Kind::boolean()),
                                        is_reference: false,
                                    },
                                },
                            }),
                            properties: std::iter::IntoIterator::into_iter([
                                (
                                    s("color"),
                                    ftd::ftd2021::component::Property {
                                        default: Some(ftd::PropertyValue::Reference {
                                            name: s("foo/bar#4D4D4D"),
                                            kind: ftd::ftd2021::p2::Kind::Optional {
                                                kind: Box::new(ftd::ftd2021::p2::Kind::Record {
                                                    name: s("ftd#color"),
                                                    default: None,
                                                    is_reference: false,
                                                }),
                                                is_reference: false,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                                (
                                    s("text"),
                                    ftd::ftd2021::component::Property {
                                        default: Some(ftd::PropertyValue::Variable {
                                            name: "name".to_string(),
                                            kind: ftd::ftd2021::p2::Kind::caption_or_body(),
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                            ])
                            .collect(),
                            ..Default::default()
                        },
                    },
                ],
                kernel: false,
                ..Default::default()
            }),
        );

        bag.insert(
            "foo/bar#table-of-content".to_string(),
            ftd::ftd2021::p2::Thing::Component(ftd::Component {
                root: "ftd#column".to_string(),
                full_name: "foo/bar#table-of-content".to_string(),
                arguments: universal_arguments_as_map(),
                properties: std::iter::IntoIterator::into_iter([
                    (
                        s("height"),
                        ftd::ftd2021::component::Property {
                            default: Some(ftd::PropertyValue::Value {
                                value: ftd::ftd2021::variable::Value::String {
                                    text: "fill".to_string(),
                                    source: ftd::TextSource::Header,
                                },
                            }),
                            conditions: vec![],
                            ..Default::default()
                        },
                    ),
                    (
                        s("id"),
                        ftd::ftd2021::component::Property {
                            default: Some(ftd::PropertyValue::Variable {
                                name: "id".to_string(),
                                kind: ftd::ftd2021::p2::Kind::Optional {
                                    kind: Box::new(ftd::ftd2021::p2::Kind::string()),
                                    is_reference: false,
                                },
                            }),
                            conditions: vec![],
                            ..Default::default()
                        },
                    ),
                    (
                        s("width"),
                        ftd::ftd2021::component::Property {
                            default: Some(ftd::PropertyValue::Value {
                                value: ftd::ftd2021::variable::Value::String {
                                    text: "300".to_string(),
                                    source: ftd::TextSource::Header,
                                },
                            }),
                            conditions: vec![],
                            ..Default::default()
                        },
                    ),
                ])
                .collect(),
                instructions: vec![],
                kernel: false,
                ..Default::default()
            }),
        );

        bag.insert(
            "foo/bar#toc-heading".to_string(),
            ftd::ftd2021::p2::Thing::Component(ftd::Component {
                root: "ftd#text".to_string(),
                full_name: "foo/bar#toc-heading".to_string(),
                arguments: [
                    vec![(s("text"), ftd::ftd2021::p2::Kind::caption())],
                    universal_arguments_as_vec(),
                ]
                .concat()
                .into_iter()
                .collect(),
                properties: std::iter::IntoIterator::into_iter([
                    (
                        s("line-clamp"),
                        ftd::ftd2021::component::Property {
                            default: Some(ftd::PropertyValue::Value {
                                value: ftd::ftd2021::variable::Value::Integer { value: 16 },
                            }),
                            conditions: vec![],
                            ..Default::default()
                        },
                    ),
                    (
                        s("text"),
                        ftd::ftd2021::component::Property {
                            default: Some(ftd::PropertyValue::Variable {
                                name: "text".to_string(),
                                kind: ftd::ftd2021::p2::Kind::caption_or_body(),
                            }),
                            conditions: vec![],
                            ..Default::default()
                        },
                    ),
                ])
                .collect(),
                ..Default::default()
            }),
        );

        bag.insert(
            s("foo/bar#active@0,0,0"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("active"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Boolean { value: true },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#active@0,0,0,0,2"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("active"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Optional {
                        data: Box::new(None),
                        kind: ftd::ftd2021::p2::Kind::boolean(),
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#active@0,0,0,2"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("active"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Optional {
                        data: Box::new(None),
                        kind: ftd::ftd2021::p2::Kind::boolean(),
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#active@0,0,0,3"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("active"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Optional {
                        data: Box::new(None),
                        kind: ftd::ftd2021::p2::Kind::boolean(),
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#name@0,0,0,2"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("name"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("Log"),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#name@0,0,0,0,2"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("name"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("ChildLog"),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#name@0,0,0"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("name"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("5PM Tasks"),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#name@0,0,0,3"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("name"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("Log2"),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );

        bag.insert(
            s("foo/bar#4D4D4D"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("4D4D4D"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Record {
                        name: s("ftd#color"),
                        fields: std::iter::IntoIterator::into_iter([
                            (
                                s("dark"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: s("#4D4D4D"),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                            (
                                s("light"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: s("#4D4D4D"),
                                        source: ftd::TextSource::Caption,
                                    },
                                },
                            ),
                        ])
                        .collect(),
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );

        bag.insert(
            s("foo/bar#white"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("white"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Record {
                        name: s("ftd#color"),
                        fields: std::iter::IntoIterator::into_iter([
                            (
                                s("dark"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: s("white"),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                            (
                                s("light"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: s("white"),
                                        source: ftd::TextSource::Caption,
                                    },
                                },
                            ),
                        ])
                        .collect(),
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );

        let levels: Vec<String> = vec![
            s("0"),
            s("0,0"),
            s("0,0,0"),
            s("0,0,0,0,2"),
            s("0,0,0,2"),
            s("0,0,0,3"),
        ];
        insert_universal_variables_by_levels(levels, "foo/bar", &mut bag);
        insert_update_string_by_root("foo/bar#id@0,0", "toc_main", "header", &mut bag);
        insert_update_string_by_root("foo/bar#id@0,0,0", "/welcome/", "header", &mut bag);
        insert_update_string_by_root("foo/bar#id@0,0,0,2", "/Building/", "header", &mut bag);
        insert_update_string_by_root("foo/bar#id@0,0,0,3", "/Building2/", "header", &mut bag);
        insert_update_string_by_root(
            "foo/bar#id@0,0,0,0,2",
            "/ChildBuilding/",
            "header",
            &mut bag,
        );

        let children = vec![
            ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("5PM Tasks"),
                line: true,
                common: Box::new(ftd::Common {
                    color: Some(ftd::Color {
                        light: ftd::ColorValue {
                            r: 255,
                            g: 255,
                            b: 255,
                            alpha: 1.0,
                        }),
                        dark: ftd::ColorValue {
                            r: 255,
                            g: 255,
                            b: 255,
                            alpha: 1.0,
                        }),
                        reference: Some(s("foo/bar#white")),
                    }),
                    reference: Some(s("foo/bar#name@0,0,0")),
                    condition: Some(ftd::Condition {
                        variable: s("foo/bar#active@0,0,0"),
                        value: serde_json::Value::String(s("$IsNotNull$")),
                    }),
                    ..Default::default()
                },
                ..Default::default()
            }),
            ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("5PM Tasks"),
                line: true,
                common: Box::new(ftd::Common {
                    color: Some(ftd::Color {
                        light: ftd::ColorValue {
                            r: 77,
                            g: 77,
                            b: 77,
                            alpha: 1.0,
                        }),
                        dark: ftd::ColorValue {
                            r: 77,
                            g: 77,
                            b: 77,
                            alpha: 1.0,
                        }),
                        reference: Some(s("foo/bar#4D4D4D")),
                    }),
                    reference: Some(s("foo/bar#name@0,0,0")),
                    condition: Some(ftd::Condition {
                        variable: s("foo/bar#active@0,0,0"),
                        value: serde_json::Value::String(s("$IsNull$")),
                    }),
                    is_not_visible: true,
                    ..Default::default()
                },
                ..Default::default()
            }),
            ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::ftd2021::rendered::markup_line("Log"),
                            line: true,
                            common: Box::new(ftd::Common {
                                color: Some(ftd::Color {
                                    light: ftd::ColorValue {
                                        r: 255,
                                        g: 255,
                                        b: 255,
                                        alpha: 1.0,
                                    }),
                                    dark: ftd::ColorValue {
                                        r: 255,
                                        g: 255,
                                        b: 255,
                                        alpha: 1.0,
                                    }),
                                    reference: Some(s("foo/bar#white")),
                                }),
                                reference: Some(s("foo/bar#name@0,0,0,2")),
                                condition: Some(ftd::Condition {
                                    variable: s("foo/bar#active@0,0,0,2"),
                                    value: serde_json::Value::String(s("$IsNotNull$")),
                                }),
                                is_not_visible: true,
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::ftd2021::rendered::markup_line("Log"),
                            line: true,
                            common: Box::new(ftd::Common {
                                color: Some(ftd::Color {
                                    light: ftd::ColorValue {
                                        r: 77,
                                        g: 77,
                                        b: 77,
                                        alpha: 1.0,
                                    }),
                                    dark: ftd::ColorValue {
                                        r: 77,
                                        g: 77,
                                        b: 77,
                                        alpha: 1.0,
                                    }),
                                    reference: Some(s("foo/bar#4D4D4D")),
                                }),
                                reference: Some(s("foo/bar#name@0,0,0,2")),
                                condition: Some(ftd::Condition {
                                    variable: s("foo/bar#active@0,0,0,2"),
                                    value: serde_json::Value::String(s("$IsNull$")),
                                }),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Column(ftd::Column {
                            spacing: None,
                            container: ftd::Container {
                                external_children: Default::default(),
                                children: vec![
                                    ftd::Element::Markup(ftd::Markups {
                                        text: ftd::ftd2021::rendered::markup_line("ChildLog"),
                                        line: true,
                                        common: Box::new(ftd::Common {
                                            color: Some(ftd::Color {
                                                light: ftd::ColorValue {
                                                    r: 255,
                                                    g: 255,
                                                    b: 255,
                                                    alpha: 1.0,
                                                }),
                                                dark: ftd::ColorValue {
                                                    r: 255,
                                                    g: 255,
                                                    b: 255,
                                                    alpha: 1.0,
                                                }),
                                                reference: Some(s("foo/bar#white")),
                                            }),
                                            reference: Some(s("foo/bar#name@0,0,0,0,2")),
                                            condition: Some(ftd::Condition {
                                                variable: s("foo/bar#active@0,0,0,0,2"),
                                                value: serde_json::Value::String(s("$IsNotNull$")),
                                            }),
                                            is_not_visible: true,
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    }),
                                    ftd::Element::Markup(ftd::Markups {
                                        text: ftd::ftd2021::rendered::markup_line("ChildLog"),
                                        line: true,
                                        common: Box::new(ftd::Common {
                                            color: Some(ftd::Color {
                                                light: ftd::ColorValue {
                                                    r: 77,
                                                    g: 77,
                                                    b: 77,
                                                    alpha: 1.0,
                                                }),
                                                dark: ftd::ColorValue {
                                                    r: 77,
                                                    g: 77,
                                                    b: 77,
                                                    alpha: 1.0,
                                                }),
                                                reference: Some(s("foo/bar#4D4D4D")),
                                            }),
                                            reference: Some(s("foo/bar#name@0,0,0,0,2")),
                                            condition: Some(ftd::Condition {
                                                variable: s("foo/bar#active@0,0,0,0,2"),
                                                value: serde_json::Value::String(s("$IsNull$")),
                                            }),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    }),
                                ],
                                open: Some(true),
                                ..Default::default()
                            },
                            common: Box::new(ftd::Common {
                                data_id: Some(s("/ChildBuilding/")),
                                width: Some(ftd::Length::Fill),
                                ..Default::default()
                            },
                        }),
                    ],
                    external_children: Default::default(),
                    open: Some(true),
                    ..Default::default()
                },
                common: Box::new(ftd::Common {
                    data_id: Some(s("/Building/")),
                    width: Some(ftd::Length::Fill),
                    ..Default::default()
                },
            }),
            ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    external_children: Default::default(),
                    children: vec![
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::ftd2021::rendered::markup_line("Log2"),
                            line: true,
                            common: Box::new(ftd::Common {
                                color: Some(ftd::Color {
                                    light: ftd::ColorValue {
                                        r: 255,
                                        g: 255,
                                        b: 255,
                                        alpha: 1.0,
                                    }),
                                    dark: ftd::ColorValue {
                                        r: 255,
                                        g: 255,
                                        b: 255,
                                        alpha: 1.0,
                                    }),
                                    reference: Some(s("foo/bar#white")),
                                }),
                                reference: Some(s("foo/bar#name@0,0,0,3")),
                                condition: Some(ftd::Condition {
                                    variable: s("foo/bar#active@0,0,0,3"),
                                    value: serde_json::Value::String(s("$IsNotNull$")),
                                }),
                                is_not_visible: true,
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::ftd2021::rendered::markup_line("Log2"),
                            line: true,
                            common: Box::new(ftd::Common {
                                color: Some(ftd::Color {
                                    light: ftd::ColorValue {
                                        r: 77,
                                        g: 77,
                                        b: 77,
                                        alpha: 1.0,
                                    }),
                                    dark: ftd::ColorValue {
                                        r: 77,
                                        g: 77,
                                        b: 77,
                                        alpha: 1.0,
                                    }),
                                    reference: Some(s("foo/bar#4D4D4D")),
                                }),
                                reference: Some(s("foo/bar#name@0,0,0,3")),
                                condition: Some(ftd::Condition {
                                    variable: s("foo/bar#active@0,0,0,3"),
                                    value: serde_json::Value::String(s("$IsNull$")),
                                }),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    ],
                    open: Some(true),
                    ..Default::default()
                },
                common: Box::new(ftd::Common {
                    data_id: Some(s("/Building2/")),
                    width: Some(ftd::Length::Fill),
                    ..Default::default()
                },
            }),
        ];

        let mut main = p2::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![ftd::Element::Column(ftd::Column {
                        spacing: None,
                        container: ftd::Container {
                            children: vec![ftd::Element::Column(ftd::Column {
                                spacing: None,
                                container: ftd::Container {
                                    children,
                                    external_children: Default::default(),
                                    open: Some(true),
                                    ..Default::default()
                                },
                                common: Box::new(ftd::Common {
                                    data_id: Some(s("/welcome/")),
                                    width: Some(ftd::Length::Fill),
                                    ..Default::default()
                                },
                            })],
                            ..Default::default()
                        },
                        common: Box::new(ftd::Common {
                            data_id: Some(s("toc_main")),
                            height: Some(ftd::Length::Fill),
                            width: Some(ftd::Length::Px { value: 300 }),
                            ..Default::default()
                        },
                    })],
                    ..Default::default()
                },
                ..Default::default()
            }));

        p!(
            r"
            -- ftd.color white: white
            dark: white

            -- ftd.color 4D4D4D: #4D4D4D
            dark: #4D4D4D

            -- ftd.text toc-heading:
            caption text:
            text: $text
            line-clamp: 16


            -- ftd.column table-of-content:
            ;; id is universal argument now no repeated initialization of id allowed
            /string id:
            id: $id
            width: 300
            height: fill


            -- ftd.column parent:
            ;; id is universal argument now no repeated initialization of id allowed
            /string id:
            caption name:
            optional boolean active:
            id: $id
            width: fill
            open: true

            --- ftd.text:
            if: $active is not null
            text: $name
            color: $white

            --- ftd.text:
            if: $active is null
            text: $name
            color: $4D4D4D


            -- ftd.column ft_toc:

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
            ",
            (bag, main),
        );
    }

    #[test]
    fn creating_a_tree_using_import() {
        let mut bag = interpreter::default_bag();

        bag.insert(
            "creating-a-tree#ft_toc".to_string(),
            ftd::ftd2021::p2::Thing::Component(ftd::Component {
                root: "ftd#column".to_string(),
                full_name: "creating-a-tree#ft_toc".to_string(),
                arguments: universal_arguments_as_map(),
                properties: Default::default(),
                instructions: vec![
                    ftd::ftd2021::component::Instruction::ChildComponent {
                        child: ftd::ftd2021::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "creating-a-tree#table-of-content".to_string(),
                            condition: None,
                            properties: std::iter::IntoIterator::into_iter([(
                                s("id"),
                                ftd::ftd2021::component::Property {
                                    default: Some(ftd::PropertyValue::Value {
                                        value: ftd::ftd2021::variable::Value::String {
                                            text: "toc_main".to_string(),
                                            source: ftd::TextSource::Header,
                                        },
                                    }),
                                    conditions: vec![],
                                    ..Default::default()
                                },
                            )])
                            .collect(),
                            ..Default::default()
                        },
                    },
                    ftd::ftd2021::component::Instruction::ChildComponent {
                        child: ftd::ftd2021::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "creating-a-tree#parent".to_string(),
                            condition: None,
                            properties: std::iter::IntoIterator::into_iter([
                                (
                                    s("active"),
                                    ftd::ftd2021::component::Property {
                                        default: Some(ftd::PropertyValue::Value {
                                            value: ftd::ftd2021::variable::Value::Boolean {
                                                value: true,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                                (
                                    s("id"),
                                    ftd::ftd2021::component::Property {
                                        default: Some(ftd::PropertyValue::Value {
                                            value: ftd::ftd2021::variable::Value::String {
                                                text: "/welcome/".to_string(),
                                                source: ftd::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                                (
                                    s("name"),
                                    ftd::ftd2021::component::Property {
                                        default: Some(ftd::PropertyValue::Value {
                                            value: ftd::ftd2021::variable::Value::String {
                                                text: "5PM Tasks".to_string(),
                                                source: ftd::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                            ])
                            .collect(),
                            ..Default::default()
                        },
                    },
                    ftd::ftd2021::component::Instruction::ChildComponent {
                        child: ftd::ftd2021::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "creating-a-tree#parent".to_string(),
                            condition: None,
                            properties: std::iter::IntoIterator::into_iter([
                                (
                                    s("id"),
                                    ftd::ftd2021::component::Property {
                                        default: Some(ftd::PropertyValue::Value {
                                            value: ftd::ftd2021::variable::Value::String {
                                                text: "/Building/".to_string(),
                                                source: ftd::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                                (
                                    s("name"),
                                    ftd::ftd2021::component::Property {
                                        default: Some(ftd::PropertyValue::Value {
                                            value: ftd::ftd2021::variable::Value::String {
                                                text: "Log".to_string(),
                                                source: ftd::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                            ])
                            .collect(),
                            ..Default::default()
                        },
                    },
                    ftd::ftd2021::component::Instruction::ChildComponent {
                        child: ftd::ftd2021::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "creating-a-tree#parent".to_string(),
                            condition: None,
                            properties: std::iter::IntoIterator::into_iter([
                                (
                                    s("id"),
                                    ftd::ftd2021::component::Property {
                                        default: Some(ftd::PropertyValue::Value {
                                            value: ftd::ftd2021::variable::Value::String {
                                                text: "/ChildBuilding/".to_string(),
                                                source: ftd::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                                (
                                    s("name"),
                                    ftd::ftd2021::component::Property {
                                        default: Some(ftd::PropertyValue::Value {
                                            value: ftd::ftd2021::variable::Value::String {
                                                text: "ChildLog".to_string(),
                                                source: ftd::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                            ])
                            .collect(),
                            ..Default::default()
                        },
                    },
                    ftd::ftd2021::component::Instruction::ChangeContainer {
                        name: "/welcome/".to_string(),
                    },
                    ftd::ftd2021::component::Instruction::ChildComponent {
                        child: ftd::ftd2021::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "creating-a-tree#parent".to_string(),
                            condition: None,
                            properties: std::iter::IntoIterator::into_iter([
                                (
                                    s("id"),
                                    ftd::ftd2021::component::Property {
                                        default: Some(ftd::PropertyValue::Value {
                                            value: ftd::ftd2021::variable::Value::String {
                                                text: "/Building2/".to_string(),
                                                source: ftd::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                                (
                                    s("name"),
                                    ftd::ftd2021::component::Property {
                                        default: Some(ftd::PropertyValue::Value {
                                            value: ftd::ftd2021::variable::Value::String {
                                                text: "Log2".to_string(),
                                                source: ftd::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                            ])
                            .collect(),
                            ..Default::default()
                        },
                    },
                ],
                kernel: false,
                ..Default::default()
            }),
        );

        bag.insert(
            "creating-a-tree#parent".to_string(),
            ftd::ftd2021::p2::Thing::Component(ftd::Component {
                root: "ftd#column".to_string(),
                full_name: "creating-a-tree#parent".to_string(),
                arguments: [
                    vec![
                        (
                            s("active"),
                            ftd::ftd2021::p2::Kind::Optional {
                                kind: Box::new(ftd::ftd2021::p2::Kind::boolean()),
                                is_reference: false,
                            },
                        ),
                        (s("id"), ftd::ftd2021::p2::Kind::string()),
                        (s("name"), ftd::ftd2021::p2::Kind::caption()),
                    ],
                    universal_arguments_as_vec(),
                ]
                .concat()
                .into_iter()
                .collect(),
                properties: std::iter::IntoIterator::into_iter([
                    (
                        s("id"),
                        ftd::ftd2021::component::Property {
                            default: Some(ftd::PropertyValue::Variable {
                                name: "id".to_string(),
                                kind: ftd::ftd2021::p2::Kind::Optional {
                                    kind: Box::new(ftd::ftd2021::p2::Kind::string()),
                                    is_reference: false,
                                },
                            }),
                            conditions: vec![],
                            ..Default::default()
                        },
                    ),
                    (
                        s("open"),
                        ftd::ftd2021::component::Property {
                            default: Some(ftd::PropertyValue::Value {
                                value: ftd::Value::Boolean { value: true },
                            }),
                            conditions: vec![],
                            ..Default::default()
                        },
                    ),
                    (
                        s("width"),
                        ftd::ftd2021::component::Property {
                            default: Some(ftd::PropertyValue::Value {
                                value: ftd::ftd2021::variable::Value::String {
                                    text: "fill".to_string(),
                                    source: ftd::TextSource::Header,
                                },
                            }),
                            conditions: vec![],
                            ..Default::default()
                        },
                    ),
                ])
                .collect(),
                instructions: vec![
                    ftd::ftd2021::component::Instruction::ChildComponent {
                        child: ftd::ftd2021::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "ftd#text".to_string(),
                            condition: Some(ftd::ftd2021::p2::Boolean::IsNotNull {
                                value: ftd::PropertyValue::Variable {
                                    name: "active".to_string(),
                                    kind: ftd::ftd2021::p2::Kind::Optional {
                                        kind: Box::new(ftd::ftd2021::p2::Kind::boolean()),
                                        is_reference: false,
                                    },
                                },
                            }),
                            properties: std::iter::IntoIterator::into_iter([
                                (
                                    s("color"),
                                    ftd::ftd2021::component::Property {
                                        default: Some(ftd::PropertyValue::Reference {
                                            name: s("creating-a-tree#white"),
                                            kind: ftd::ftd2021::p2::Kind::Optional {
                                                kind: Box::new(ftd::ftd2021::p2::Kind::Record {
                                                    name: s("ftd#color"),
                                                    default: None,
                                                    is_reference: false,
                                                }),
                                                is_reference: false,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                                (
                                    s("text"),
                                    ftd::ftd2021::component::Property {
                                        default: Some(ftd::PropertyValue::Variable {
                                            name: "name".to_string(),
                                            kind: ftd::ftd2021::p2::Kind::caption_or_body(),
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                            ])
                            .collect(),
                            ..Default::default()
                        },
                    },
                    ftd::ftd2021::component::Instruction::ChildComponent {
                        child: ftd::ftd2021::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "ftd#text".to_string(),
                            condition: Some(ftd::ftd2021::p2::Boolean::IsNull {
                                value: ftd::PropertyValue::Variable {
                                    name: "active".to_string(),
                                    kind: ftd::ftd2021::p2::Kind::Optional {
                                        kind: Box::new(ftd::ftd2021::p2::Kind::boolean()),
                                        is_reference: false,
                                    },
                                },
                            }),
                            properties: std::iter::IntoIterator::into_iter([
                                (
                                    s("color"),
                                    ftd::ftd2021::component::Property {
                                        default: Some(ftd::PropertyValue::Reference {
                                            name: s("creating-a-tree#4D4D4D"),
                                            kind: ftd::ftd2021::p2::Kind::Optional {
                                                kind: Box::new(ftd::ftd2021::p2::Kind::Record {
                                                    name: s("ftd#color"),
                                                    default: None,
                                                    is_reference: false,
                                                }),
                                                is_reference: false,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                                (
                                    s("text"),
                                    ftd::ftd2021::component::Property {
                                        default: Some(ftd::PropertyValue::Variable {
                                            name: "name".to_string(),
                                            kind: ftd::ftd2021::p2::Kind::caption_or_body(),
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                            ])
                            .collect(),
                            ..Default::default()
                        },
                    },
                ],
                kernel: false,
                ..Default::default()
            }),
        );

        bag.insert(
            "creating-a-tree#table-of-content".to_string(),
            ftd::ftd2021::p2::Thing::Component(ftd::Component {
                root: "ftd#column".to_string(),
                full_name: "creating-a-tree#table-of-content".to_string(),
                arguments: [
                    vec![(s("id"), ftd::ftd2021::p2::Kind::string())],
                    universal_arguments_as_vec(),
                ]
                .concat()
                .into_iter()
                .collect(),
                properties: std::iter::IntoIterator::into_iter([
                    (
                        s("height"),
                        ftd::ftd2021::component::Property {
                            default: Some(ftd::PropertyValue::Value {
                                value: ftd::ftd2021::variable::Value::String {
                                    text: "fill".to_string(),
                                    source: ftd::TextSource::Header,
                                },
                            }),
                            conditions: vec![],
                            ..Default::default()
                        },
                    ),
                    (
                        s("id"),
                        ftd::ftd2021::component::Property {
                            default: Some(ftd::PropertyValue::Variable {
                                name: "id".to_string(),
                                kind: ftd::ftd2021::p2::Kind::Optional {
                                    kind: Box::new(ftd::ftd2021::p2::Kind::string()),
                                    is_reference: false,
                                },
                            }),
                            conditions: vec![],
                            ..Default::default()
                        },
                    ),
                    (
                        s("width"),
                        ftd::ftd2021::component::Property {
                            default: Some(ftd::PropertyValue::Value {
                                value: ftd::ftd2021::variable::Value::String {
                                    text: "300".to_string(),
                                    source: ftd::TextSource::Header,
                                },
                            }),
                            conditions: vec![],
                            ..Default::default()
                        },
                    ),
                ])
                .collect(),
                instructions: vec![],
                kernel: false,
                ..Default::default()
            }),
        );

        bag.insert(
            "creating-a-tree#toc-heading".to_string(),
            ftd::ftd2021::p2::Thing::Component(ftd::Component {
                root: "ftd#text".to_string(),
                full_name: "creating-a-tree#toc-heading".to_string(),
                arguments: [
                    vec![(s("text"), ftd::ftd2021::p2::Kind::caption())],
                    universal_arguments_as_vec(),
                ]
                .concat()
                .into_iter()
                .collect(),
                properties: std::iter::IntoIterator::into_iter([(
                    s("text"),
                    ftd::ftd2021::component::Property {
                        default: Some(ftd::PropertyValue::Variable {
                            name: "text".to_string(),
                            kind: ftd::ftd2021::p2::Kind::caption_or_body(),
                        }),
                        conditions: vec![],
                        ..Default::default()
                    },
                )])
                .collect(),
                ..Default::default()
            }),
        );
        bag.insert(
            s("foo/bar#active@0,0,0"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("active"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Boolean { value: true },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#active@0,0,0,2"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("active"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Optional {
                        data: Box::new(None),
                        kind: ftd::ftd2021::p2::Kind::boolean(),
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#active@0,0,0,0,2"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("active"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Optional {
                        data: Box::new(None),
                        kind: ftd::ftd2021::p2::Kind::boolean(),
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#active@0,0,0,3"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("active"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Optional {
                        data: Box::new(None),
                        kind: ftd::ftd2021::p2::Kind::boolean(),
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#id@0,0"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("id"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("toc_main"),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#id@0,0,0"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("id"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("/welcome/"),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#id@0,0,0,2"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("id"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("/Building/"),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#id@0,0,0,0,2"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("id"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("/ChildBuilding/"),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#id@0,0,0,3"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("id"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("/Building2/"),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#name@0,0,0,2"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("name"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("Log"),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#name@0,0,0,0,2"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("name"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("ChildLog"),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#name@0,0,0"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("name"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("5PM Tasks"),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#name@0,0,0,3"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("name"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("Log2"),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("creating-a-tree#4D4D4D"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("4D4D4D"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Record {
                        name: s("ftd#color"),
                        fields: std::iter::IntoIterator::into_iter([
                            (
                                s("dark"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: s("#4D4D4D"),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                            (
                                s("light"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: s("#4D4D4D"),
                                        source: ftd::TextSource::Caption,
                                    },
                                },
                            ),
                        ])
                        .collect(),
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );

        bag.insert(
            s("creating-a-tree#white"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("white"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Record {
                        name: s("ftd#color"),
                        fields: std::iter::IntoIterator::into_iter([
                            (
                                s("dark"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: s("white"),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                            (
                                s("light"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: s("white"),
                                        source: ftd::TextSource::Caption,
                                    },
                                },
                            ),
                        ])
                        .collect(),
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );

        let levels = vec![
            s("0"),
            s("0,0"),
            s("0,0,0"),
            s("0,0,0,2"),
            s("0,0,0,3"),
            s("0,0,0,0,2"),
        ];
        insert_universal_variables_by_levels(levels, "foo/bar", &mut bag);
        insert_update_string_by_root("foo/bar#id@0,0", "toc_main", "header", &mut bag);
        insert_update_string_by_root("foo/bar#id@0,0,0", "/welcome/", "header", &mut bag);
        insert_update_string_by_root("foo/bar#id@0,0,0,2", "/Building/", "header", &mut bag);
        insert_update_string_by_root("foo/bar#id@0,0,0,3", "/Building2/", "header", &mut bag);
        insert_update_string_by_root(
            "foo/bar#id@0,0,0,0,2",
            "/ChildBuilding/",
            "header",
            &mut bag,
        );

        let children = vec![
            ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("5PM Tasks"),
                line: true,
                common: Box::new(ftd::Common {
                    color: Some(ftd::Color {
                        light: ftd::ColorValue {
                            r: 255,
                            g: 255,
                            b: 255,
                            alpha: 1.0,
                        }),
                        dark: ftd::ColorValue {
                            r: 255,
                            g: 255,
                            b: 255,
                            alpha: 1.0,
                        }),
                        reference: Some(s("creating-a-tree#white")),
                    }),
                    reference: Some(s("foo/bar#name@0,0,0")),
                    condition: Some(ftd::Condition {
                        variable: s("foo/bar#active@0,0,0"),
                        value: serde_json::Value::String(s("$IsNotNull$")),
                    }),
                    ..Default::default()
                },
                ..Default::default()
            }),
            ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("5PM Tasks"),
                line: true,
                common: Box::new(ftd::Common {
                    color: Some(ftd::Color {
                        light: ftd::ColorValue {
                            r: 77,
                            g: 77,
                            b: 77,
                            alpha: 1.0,
                        }),
                        dark: ftd::ColorValue {
                            r: 77,
                            g: 77,
                            b: 77,
                            alpha: 1.0,
                        }),
                        reference: Some(s("creating-a-tree#4D4D4D")),
                    }),
                    reference: Some(s("foo/bar#name@0,0,0")),
                    condition: Some(ftd::Condition {
                        variable: s("foo/bar#active@0,0,0"),
                        value: serde_json::Value::String(s("$IsNull$")),
                    }),
                    is_not_visible: true,
                    ..Default::default()
                },
                ..Default::default()
            }),
            ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::ftd2021::rendered::markup_line("Log"),
                            line: true,
                            common: Box::new(ftd::Common {
                                color: Some(ftd::Color {
                                    light: ftd::ColorValue {
                                        r: 255,
                                        g: 255,
                                        b: 255,
                                        alpha: 1.0,
                                    }),
                                    dark: ftd::ColorValue {
                                        r: 255,
                                        g: 255,
                                        b: 255,
                                        alpha: 1.0,
                                    }),
                                    reference: Some(s("creating-a-tree#white")),
                                }),
                                reference: Some(s("foo/bar#name@0,0,0,2")),
                                condition: Some(ftd::Condition {
                                    variable: s("foo/bar#active@0,0,0,2"),
                                    value: serde_json::Value::String(s("$IsNotNull$")),
                                }),
                                is_not_visible: true,
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::ftd2021::rendered::markup_line("Log"),
                            line: true,
                            common: Box::new(ftd::Common {
                                color: Some(ftd::Color {
                                    light: ftd::ColorValue {
                                        r: 77,
                                        g: 77,
                                        b: 77,
                                        alpha: 1.0,
                                    }),
                                    dark: ftd::ColorValue {
                                        r: 77,
                                        g: 77,
                                        b: 77,
                                        alpha: 1.0,
                                    }),
                                    reference: Some(s("creating-a-tree#4D4D4D")),
                                }),
                                reference: Some(s("foo/bar#name@0,0,0,2")),
                                condition: Some(ftd::Condition {
                                    variable: s("foo/bar#active@0,0,0,2"),
                                    value: serde_json::Value::String(s("$IsNull$")),
                                }),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Column(ftd::Column {
                            spacing: None,
                            container: ftd::Container {
                                external_children: Default::default(),
                                children: vec![
                                    ftd::Element::Markup(ftd::Markups {
                                        text: ftd::ftd2021::rendered::markup_line("ChildLog"),
                                        line: true,
                                        common: Box::new(ftd::Common {
                                            color: Some(ftd::Color {
                                                light: ftd::ColorValue {
                                                    r: 255,
                                                    g: 255,
                                                    b: 255,
                                                    alpha: 1.0,
                                                }),
                                                dark: ftd::ColorValue {
                                                    r: 255,
                                                    g: 255,
                                                    b: 255,
                                                    alpha: 1.0,
                                                }),
                                                reference: Some(s("creating-a-tree#white")),
                                            }),
                                            reference: Some(s("foo/bar#name@0,0,0,0,2")),
                                            condition: Some(ftd::Condition {
                                                variable: s("foo/bar#active@0,0,0,0,2"),
                                                value: serde_json::Value::String(s("$IsNotNull$")),
                                            }),
                                            is_not_visible: true,
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    }),
                                    ftd::Element::Markup(ftd::Markups {
                                        text: ftd::ftd2021::rendered::markup_line("ChildLog"),
                                        line: true,
                                        common: Box::new(ftd::Common {
                                            color: Some(ftd::Color {
                                                light: ftd::ColorValue {
                                                    r: 77,
                                                    g: 77,
                                                    b: 77,
                                                    alpha: 1.0,
                                                }),
                                                dark: ftd::ColorValue {
                                                    r: 77,
                                                    g: 77,
                                                    b: 77,
                                                    alpha: 1.0,
                                                }),
                                                reference: Some(s("creating-a-tree#4D4D4D")),
                                            }),
                                            reference: Some(s("foo/bar#name@0,0,0,0,2")),
                                            condition: Some(ftd::Condition {
                                                variable: s("foo/bar#active@0,0,0,0,2"),
                                                value: serde_json::Value::String(s("$IsNull$")),
                                            }),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    }),
                                ],
                                open: Some(true),
                                ..Default::default()
                            },
                            common: Box::new(ftd::Common {
                                data_id: Some(s("/ChildBuilding/")),
                                width: Some(ftd::Length::Fill),
                                ..Default::default()
                            },
                        }),
                    ],
                    external_children: Default::default(),
                    open: Some(true),
                    ..Default::default()
                },
                common: Box::new(ftd::Common {
                    data_id: Some(s("/Building/")),
                    width: Some(ftd::Length::Fill),
                    ..Default::default()
                },
            }),
            ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    external_children: Default::default(),
                    children: vec![
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::ftd2021::rendered::markup_line("Log2"),
                            line: true,
                            common: Box::new(ftd::Common {
                                color: Some(ftd::Color {
                                    light: ftd::ColorValue {
                                        r: 255,
                                        g: 255,
                                        b: 255,
                                        alpha: 1.0,
                                    }),
                                    dark: ftd::ColorValue {
                                        r: 255,
                                        g: 255,
                                        b: 255,
                                        alpha: 1.0,
                                    }),
                                    reference: Some(s("creating-a-tree#white")),
                                }),
                                reference: Some(s("foo/bar#name@0,0,0,3")),
                                condition: Some(ftd::Condition {
                                    variable: s("foo/bar#active@0,0,0,3"),
                                    value: serde_json::Value::String(s("$IsNotNull$")),
                                }),
                                is_not_visible: true,
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::ftd2021::rendered::markup_line("Log2"),
                            line: true,
                            common: Box::new(ftd::Common {
                                color: Some(ftd::Color {
                                    light: ftd::ColorValue {
                                        r: 77,
                                        g: 77,
                                        b: 77,
                                        alpha: 1.0,
                                    }),
                                    dark: ftd::ColorValue {
                                        r: 77,
                                        g: 77,
                                        b: 77,
                                        alpha: 1.0,
                                    }),
                                    reference: Some(s("creating-a-tree#4D4D4D")),
                                }),
                                reference: Some(s("foo/bar#name@0,0,0,3")),
                                condition: Some(ftd::Condition {
                                    variable: s("foo/bar#active@0,0,0,3"),
                                    value: serde_json::Value::String(s("$IsNull$")),
                                }),
                                ..Default::default()
                            },

                            ..Default::default()
                        }),
                    ],
                    open: Some(true),
                    ..Default::default()
                },
                common: Box::new(ftd::Common {
                    data_id: Some(s("/Building2/")),
                    width: Some(ftd::Length::Fill),
                    ..Default::default()
                },
            }),
        ];

        let mut main = p2::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![ftd::Element::Column(ftd::Column {
                        spacing: None,
                        container: ftd::Container {
                            children: vec![ftd::Element::Column(ftd::Column {
                                spacing: None,
                                container: ftd::Container {
                                    children,
                                    external_children: Default::default(),
                                    open: Some(true),
                                    ..Default::default()
                                },
                                common: Box::new(ftd::Common {
                                    data_id: Some(s("/welcome/")),
                                    width: Some(ftd::Length::Fill),
                                    ..Default::default()
                                },
                            })],
                            ..Default::default()
                        },
                        common: Box::new(ftd::Common {
                            data_id: Some(s("toc_main")),
                            height: Some(ftd::Length::Fill),
                            width: Some(ftd::Length::Px { value: 300 }),
                            ..Default::default()
                        },
                    })],
                    ..Default::default()
                },
                ..Default::default()
            }));

        p!(
            "
            -- import: creating-a-tree as ft

            -- ft.ft_toc:
            ",
            (bag, main),
        );
    }

    #[test]
    fn reference() {
        let mut bag = interpreter::default_bag();

        bag.insert(
            s("reference#f3f3f3"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("f3f3f3"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Record {
                        name: s("ftd#color"),
                        fields: std::iter::IntoIterator::into_iter([
                            (
                                s("dark"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: s("#f3f3f3"),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                            (
                                s("light"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: s("#f3f3f3"),
                                        source: ftd::TextSource::Caption,
                                    },
                                },
                            ),
                        ])
                        .collect(),
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );

        bag.insert(
            "fifthtry/ft#dark-mode".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "dark-mode".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Boolean { value: true },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            "fifthtry/ft#toc".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "toc".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: "not set".to_string(),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            "fifthtry/ft#markdown".to_string(),
            ftd::ftd2021::p2::Thing::Component(ftd::Component {
                root: "ftd#text".to_string(),
                full_name: "fifthtry/ft#markdown".to_string(),
                arguments: [
                    vec![(s("body"), ftd::ftd2021::p2::Kind::body())],
                    universal_arguments_as_vec(),
                ]
                .concat()
                .into_iter()
                .collect(),
                properties: std::iter::IntoIterator::into_iter([(
                    s("text"),
                    ftd::ftd2021::component::Property {
                        default: Some(ftd::PropertyValue::Variable {
                            name: "body".to_string(),
                            kind: ftd::ftd2021::p2::Kind::caption_or_body(),
                        }),
                        conditions: vec![],
                        ..Default::default()
                    },
                )])
                .collect(),
                ..Default::default()
            }),
        );

        bag.insert(
            "reference#name".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "name".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: "John smith".to_string(),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            "reference#test-component".to_string(),
            ftd::ftd2021::p2::Thing::Component(ftd::Component {
                root: "ftd#column".to_string(),
                full_name: "reference#test-component".to_string(),
                arguments: universal_arguments_as_map(),
                properties: std::iter::IntoIterator::into_iter([
                    (
                        s("background-color"),
                        ftd::ftd2021::component::Property {
                            default: Some(ftd::PropertyValue::Reference {
                                name: s("reference#f3f3f3"),
                                kind: ftd::ftd2021::p2::Kind::Optional {
                                    kind: Box::new(ftd::ftd2021::p2::Kind::Record {
                                        name: s("ftd#color"),
                                        default: None,
                                        is_reference: false,
                                    }),
                                    is_reference: false,
                                },
                            }),
                            conditions: vec![],
                            ..Default::default()
                        },
                    ),
                    (
                        s("width"),
                        ftd::ftd2021::component::Property {
                            default: Some(ftd::PropertyValue::Value {
                                value: ftd::ftd2021::variable::Value::String {
                                    text: "200".to_string(),
                                    source: ftd::TextSource::Header,
                                },
                            }),
                            conditions: vec![],
                            ..Default::default()
                        },
                    ),
                ])
                .collect(),
                instructions: vec![ftd::ftd2021::component::Instruction::ChildComponent {
                    child: ftd::ftd2021::component::ChildComponent {
                        is_recursive: false,
                        events: vec![],
                        root: "ftd#text".to_string(),
                        condition: None,
                        properties: std::iter::IntoIterator::into_iter([(
                            s("text"),
                            ftd::ftd2021::component::Property {
                                default: Some(ftd::PropertyValue::Reference {
                                    name: "reference#name".to_string(),
                                    kind: ftd::ftd2021::p2::Kind::caption_or_body(),
                                }),
                                conditions: vec![],
                                ..Default::default()
                            },
                        )])
                        .collect(),
                        ..Default::default()
                    },
                }],
                kernel: false,
                ..Default::default()
            }),
        );

        insert_universal_variables_by_count(1, "foo/bar", &mut bag);

        let title = ftd::Markups {
            text: ftd::ftd2021::rendered::markup_line("John smith"),
            line: true,
            common: Box::new(ftd::Common {
                reference: Some(s("reference#name")),
                ..Default::default()
            },
            ..Default::default()
        };

        let mut main = p2::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                common: Box::new(ftd::Common {
                    width: Some(ftd::Length::Px { value: 200 }),
                    background_color: Some(ftd::Color {
                        light: ftd::ColorValue {
                            r: 243,
                            g: 243,
                            b: 243,
                            alpha: 1.0,
                        }),
                        dark: ftd::ColorValue {
                            r: 243,
                            g: 243,
                            b: 243,
                            alpha: 1.0,
                        }),
                        reference: Some(s("reference#f3f3f3")),
                    }),
                    ..Default::default()
                },
                container: ftd::Container {
                    children: vec![ftd::Element::Markup(title)],
                    ..Default::default()
                },
            }));

        p!(
            "
            -- import: reference as ct

            -- ct.test-component:
            ",
            (bag, main),
        );
    }

    #[test]
    fn text() {
        let mut bag = interpreter::default_bag();

        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@0", -1, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@1", -1, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@2", -1, &mut bag);

        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT@0", 0, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT@1", 0, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT@2", 0, &mut bag);

        insert_update_integer_by_root("foo/bar#SIBLING-INDEX-0@0", 0, &mut bag);
        insert_update_integer_by_root("foo/bar#SIBLING-INDEX-0@1", 1, &mut bag);
        insert_update_integer_by_root("foo/bar#SIBLING-INDEX-0@2", 2, &mut bag);

        insert_update_integer_by_root("foo/bar#SIBLING-INDEX@0", 1, &mut bag);
        insert_update_integer_by_root("foo/bar#SIBLING-INDEX@1", 2, &mut bag);
        insert_update_integer_by_root("foo/bar#SIBLING-INDEX@2", 3, &mut bag);

        insert_universal_variables_by_count(3, "foo/bar", &mut bag);

        bag.insert(
            "foo/bar#foo".to_string(),
            ftd::ftd2021::p2::Thing::Component(ftd::Component {
                full_name: s("foo/bar#foo"),
                root: "ftd#text".to_string(),
                arguments: [
                    vec![(s("name"), ftd::ftd2021::p2::Kind::caption_or_body())],
                    universal_arguments_as_vec(),
                ]
                .concat()
                .into_iter()
                .collect(),
                properties: std::iter::IntoIterator::into_iter([(
                    s("text"),
                    ftd::ftd2021::component::Property {
                        default: Some(ftd::PropertyValue::Variable {
                            name: "name".to_string(),
                            kind: ftd::ftd2021::p2::Kind::caption_or_body(),
                        }),
                        conditions: vec![],
                        ..Default::default()
                    },
                )])
                .collect(),
                invocations: vec![
                    std::iter::IntoIterator::into_iter([(
                        s("name"),
                        ftd::Value::String {
                            text: s("hello"),
                            source: ftd::TextSource::Caption,
                        },
                    )])
                    .collect(),
                    std::iter::IntoIterator::into_iter([(
                        s("name"),
                        ftd::Value::String {
                            text: s("world"),
                            source: ftd::TextSource::Header,
                        },
                    )])
                    .collect(),
                    std::iter::IntoIterator::into_iter([(
                        s("name"),
                        ftd::Value::String {
                            text: s("yo yo"),
                            source: ftd::TextSource::Body,
                        },
                    )])
                    .collect(),
                ],
                line_number: 1,
                ..Default::default()
            }),
        );
        bag.insert(
            "foo/bar#name@0".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "name".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: "hello".to_string(),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#name@1".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "name".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: "world".to_string(),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#name@2".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "name".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: "yo yo".to_string(),
                        source: ftd::TextSource::Body,
                    },
                },
                conditions: vec![],
            }),
        );

        let mut main = p2::default_column();
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("hello"),
                line: true,
                common: Box::new(ftd::Common {
                    reference: Some(s("foo/bar#name@0")),
                    ..Default::default()
                },
                ..Default::default()
            }));
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("world"),
                line: true,
                common: Box::new(ftd::Common {
                    reference: Some(s("foo/bar#name@1")),
                    ..Default::default()
                },
                ..Default::default()
            }));
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("yo yo"),
                line: true,
                common: Box::new(ftd::Common {
                    reference: Some(s("foo/bar#name@2")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (g_bag, g_col) = ftd::ftd2021::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- ftd.text foo:
                caption or body name:
                text: $name

                -- foo: hello

                -- foo:
                name: world

                -- foo:

                yo yo
                "
            ),
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_bag, bag);
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn row() {
        let mut main = p2::default_column();
        let mut row = ftd::Row {
            common: Box::new(ftd::Common {
                data_id: Some("the-row".to_string()),
                id: Some("the-row".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        row.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("hello"),
                line: true,
                ..Default::default()
            }));
        row.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("world"),
                line: true,
                ..Default::default()
            }));
        row.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("row child three"),
                line: true,
                ..Default::default()
            }));
        main.container.children.push(ftd::Element::Row(row));
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("back in main"),
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
        let mut main = p2::default_column();
        let mut row: ftd::Row = Default::default();
        row.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("hello"),
                line: true,
                ..Default::default()
            }));
        row.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("world"),
                line: true,
                ..Default::default()
            }));
        main.container.children.push(ftd::Element::Row(row));
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("back in main"),
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
    fn list_of_numbers() {
        let mut bag = interpreter::default_bag();
        bag.insert(
            "foo/bar#numbers".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "foo/bar#numbers".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::List {
                        data: vec![
                            ftd::PropertyValue::Value {
                                value: ftd::Value::Integer { value: 20 },
                            },
                            ftd::PropertyValue::Value {
                                value: ftd::Value::Integer { value: 30 },
                            },
                        ],
                        kind: ftd::ftd2021::p2::Kind::integer(),
                    },
                },
                conditions: vec![],
            }),
        );

        p!(
            "
            -- integer list numbers:

            -- numbers: 20
            -- numbers: 30
            ",
            (bag, super::default_column()),
        );
    }

    #[test]
    fn list_of_records() {
        let mut bag = interpreter::default_bag();
        bag.insert(
            "foo/bar#point".to_string(),
            ftd::ftd2021::p2::Thing::Record(ftd::ftd2021::p2::Record {
                name: "foo/bar#point".to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    (s("x"), ftd::ftd2021::p2::Kind::integer()),
                    (s("y"), ftd::ftd2021::p2::Kind::integer()),
                ])
                .collect(),
                instances: Default::default(),
                order: vec![s("x"), s("y")],
            }),
        );

        bag.insert(
            "foo/bar#points".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "foo/bar#points".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::List {
                        data: vec![
                            ftd::PropertyValue::Value {
                                value: ftd::Value::Record {
                                    name: s("foo/bar#point"),
                                    fields: std::iter::IntoIterator::into_iter([
                                        (
                                            s("x"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::Value::Integer { value: 10 },
                                            },
                                        ),
                                        (
                                            s("y"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::Value::Integer { value: 20 },
                                            },
                                        ),
                                    ])
                                    .collect(),
                                },
                            },
                            ftd::PropertyValue::Value {
                                value: ftd::Value::Record {
                                    name: s("foo/bar#point"),
                                    fields: std::iter::IntoIterator::into_iter([
                                        (
                                            s("x"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::Value::Integer { value: 0 },
                                            },
                                        ),
                                        (
                                            s("y"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::Value::Integer { value: 0 },
                                            },
                                        ),
                                    ])
                                    .collect(),
                                },
                            },
                        ],
                        kind: ftd::ftd2021::p2::Kind::Record {
                            name: s("foo/bar#point"),
                            default: None,
                            is_reference: false,
                        },
                    },
                },
                conditions: vec![],
            }),
        );

        p!(
            "
            -- record point:
            integer x:
            integer y:

            -- point list points:

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
        let mut bag = interpreter::default_bag();
        bag.insert(
            "foo/bar#numbers".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "foo/bar#numbers".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::List {
                        data: vec![
                            ftd::PropertyValue::Value {
                                value: ftd::Value::Integer { value: 20 },
                            },
                            ftd::PropertyValue::Value {
                                value: ftd::Value::Integer { value: 30 },
                            },
                            // TODO: third element
                        ],
                        kind: ftd::ftd2021::p2::Kind::integer(),
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#x".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "x".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Integer { value: 20 },
                },
                conditions: vec![],
            }),
        );

        p!(
            "
            -- integer list numbers:

            -- numbers: 20
            -- numbers: 30

            -- integer x: 20

            -- numbers: $x
            ",
            (bag, super::default_column()),
        );
    }

    fn white_two_image_bag(about_optional: bool) -> ftd::Map<ftd::ftd2021::p2::Thing> {
        let mut bag = interpreter::default_bag();
        bag.insert(
            s("foo/bar#white-two-image"),
            ftd::ftd2021::p2::Thing::Component(ftd::Component {
                invocations: Default::default(),
                full_name: "foo/bar#white-two-image".to_string(),
                root: s("ftd#column"),
                arguments: [
                    vec![
                        (s("about"), {
                            let s = ftd::ftd2021::p2::Kind::body();
                            if about_optional { s.into_optional() } else { s }
                        }),
                        (s("src"), {
                            let s = ftd::ftd2021::p2::Kind::Record {
                                name: s("ftd#image-src"),
                                default: None,
                                is_reference: false,
                            };
                            if about_optional { s.into_optional() } else { s }
                        }),
                        (s("title"), ftd::ftd2021::p2::Kind::caption()),
                    ],
                    universal_arguments_as_vec(),
                ]
                .concat()
                .into_iter()
                .collect(),
                properties: std::iter::IntoIterator::into_iter([(
                    s("padding"),
                    ftd::ftd2021::component::Property {
                        default: Some(ftd::PropertyValue::Value {
                            value: ftd::Value::Integer { value: 30 },
                        }),
                        conditions: vec![],
                        ..Default::default()
                    },
                )])
                .collect(),
                kernel: false,
                instructions: vec![
                    ftd::Instruction::ChildComponent {
                        child: ftd::ChildComponent {
                            events: vec![],
                            condition: None,
                            root: s("ftd#text"),
                            properties: std::iter::IntoIterator::into_iter([
                                (
                                    s("text"),
                                    ftd::ftd2021::component::Property {
                                        default: Some(ftd::PropertyValue::Variable {
                                            name: s("title"),
                                            kind: ftd::ftd2021::p2::Kind::caption_or_body(),
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                                (
                                    s("align"),
                                    ftd::ftd2021::component::Property {
                                        default: Some(ftd::PropertyValue::Value {
                                            value: ftd::Value::String {
                                                source: ftd::TextSource::Header,
                                                text: s("center"),
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                            ])
                            .collect(),
                            ..Default::default()
                        },
                    },
                    ftd::Instruction::ChildComponent {
                        child: ftd::ChildComponent {
                            events: vec![],
                            condition: if about_optional {
                                Some(ftd::ftd2021::p2::Boolean::IsNotNull {
                                    value: ftd::PropertyValue::Variable {
                                        name: s("about"),
                                        kind: ftd::ftd2021::p2::Kind::body().into_optional(),
                                    },
                                })
                            } else {
                                None
                            },
                            root: s("ftd#text"),
                            properties: std::iter::IntoIterator::into_iter([(
                                s("text"),
                                ftd::ftd2021::component::Property {
                                    default: Some(ftd::PropertyValue::Variable {
                                        name: s("about"),
                                        kind: ftd::ftd2021::p2::Kind::caption_or_body(),
                                    }),
                                    conditions: vec![],
                                    ..Default::default()
                                },
                            )])
                            .collect(),
                            ..Default::default()
                        },
                    },
                    ftd::Instruction::ChildComponent {
                        child: ftd::ChildComponent {
                            events: vec![],
                            condition: if about_optional {
                                Some(ftd::ftd2021::p2::Boolean::IsNotNull {
                                    value: ftd::PropertyValue::Variable {
                                        name: s("src"),
                                        kind: ftd::ftd2021::p2::Kind::Record {
                                            name: s("ftd#image-src"),
                                            default: None,
                                            is_reference: false,
                                        }
                                        .into_optional(),
                                    },
                                })
                            } else {
                                None
                            },
                            root: s("ftd#image"),
                            properties: std::iter::IntoIterator::into_iter([(
                                s("src"),
                                ftd::ftd2021::component::Property {
                                    default: Some(ftd::PropertyValue::Variable {
                                        name: s("src"),
                                        kind: ftd::ftd2021::p2::Kind::Record {
                                            name: s("ftd#image-src"),
                                            default: None,
                                            is_reference: false,
                                        },
                                    }),
                                    conditions: vec![],
                                    ..Default::default()
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
        let title = ftd::Markups {
            text: ftd::ftd2021::rendered::markup_line("What kind of documentation?"),
            line: true,
            common: Box::new(ftd::Common {
                position: Some(ftd::Position::Center),
                reference: Some(s("foo/bar#title@0")),
                ..Default::default()
            },
            ..Default::default()
        };
        let about = ftd::Markups {
            text: ftd::ftd2021::rendered::markup_line(
                indoc::indoc!(
                    "
                    UI screens, behaviour and journeys, database tables, APIs, how to
                    contribute to, deploy, or monitor microservice, everything that
                    makes web or mobile product teams productive.
                    "
                )
                .trim(),
            ),
            common: Box::new(ftd::Common {
                reference: Some(s("foo/bar#about@0")),
                ..Default::default()
            },
            line: true,
            ..Default::default()
        };

        let image = ftd::Image {
            src: i(
                "/static/home/document-type-min.png",
                Some(s("foo/bar#src@0")),
            ),
            common: Box::new(ftd::Common {
                reference: Some(s("foo/bar#src@0")),
                ..Default::default()
            },
            ..Default::default()
        };

        let mut main = p2::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                common: Box::new(ftd::Common {
                    padding: Some(30),
                    ..Default::default()
                },
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Markup(title),
                        ftd::Element::Markup(about),
                        ftd::Element::Image(image),
                    ],
                    ..Default::default()
                },
            }));

        let mut bag = white_two_image_bag(false);
        bag.insert(
            "foo/bar#about@0".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "about".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("UI screens, behaviour and journeys, database tables, APIs, how to\ncontribute to, deploy, or monitor microservice, everything that\nmakes web or mobile product teams productive."),
                        source: ftd::TextSource::Body,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#src0".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "src0".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Record {
                        name: s("ftd#image-src"),
                        fields: std::iter::IntoIterator::into_iter([
                            (
                                s("dark"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: "/static/home/document-type-min.png".to_string(),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                            (
                                s("light"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: "/static/home/document-type-min.png".to_string(),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                        ])
                        .collect(),
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#src@0".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "src".to_string(),
                value: ftd::PropertyValue::Reference {
                    name: s("foo/bar#src0"),
                    kind: ftd::ftd2021::p2::Kind::Record {
                        name: s("ftd#image-src"),
                        default: None,
                        is_reference: false,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#title@0".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "title".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("What kind of documentation?"),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
            }),
        );

        insert_universal_variables_by_count(1, "foo/bar", &mut bag);

        p!(
            "
            -- ftd.image-src src0:
            light: /static/home/document-type-min.png
            dark: /static/home/document-type-min.png

            -- ftd.column white-two-image:
            caption title:
            body about:
            ftd.image-src src:
            padding: 30

            --- ftd.text:
            text: $title
            align: center

            --- ftd.text:
            text: $about

            --- ftd.image:
            src: $src

            -- white-two-image: What kind of documentation?
            src: $src0

            UI screens, behaviour and journeys, database tables, APIs, how to
            contribute to, deploy, or monitor microservice, everything that
            makes web or mobile product teams productive.
            ",
            (bag, main),
        );
    }

    #[test]
    fn conditional_body() {
        let title = ftd::Markups {
            text: ftd::ftd2021::rendered::markup_line("What kind of documentation?"),
            common: Box::new(ftd::Common {
                position: Some(ftd::Position::Center),
                reference: Some(s("foo/bar#title@0")),
                ..Default::default()
            },
            line: true,
            ..Default::default()
        };
        let second_title = ftd::Markups {
            text: ftd::ftd2021::rendered::markup_line("second call"),
            common: Box::new(ftd::Common {
                position: Some(ftd::Position::Center),
                reference: Some(s("foo/bar#title@1")),
                ..Default::default()
            },
            line: true,
            ..Default::default()
        };
        let about = ftd::Markups {
            text: ftd::ftd2021::rendered::markup_line(
                indoc::indoc!(
                    "
                    UI screens, behaviour and journeys, database tables, APIs, how to
                    contribute to, deploy, or monitor microservice, everything that
                    makes web or mobile product teams productive.
                    "
                )
                .trim(),
            ),
            common: Box::new(ftd::Common {
                reference: Some(s("foo/bar#about@0")),
                condition: Some(ftd::Condition {
                    variable: s("foo/bar#about@0"),
                    value: serde_json::Value::String(s("$IsNotNull$")),
                }),
                ..Default::default()
            },
            line: true,
            ..Default::default()
        };
        let second_about = ftd::Markups {
            text: ftd::ftd2021::rendered::markup_line(""),
            common: Box::new(ftd::Common {
                reference: Some(s("foo/bar#about@1")),
                condition: Some(ftd::Condition {
                    variable: s("foo/bar#about@1"),
                    value: serde_json::Value::String(s("$IsNotNull$")),
                }),
                is_not_visible: true,
                ..Default::default()
            },
            line: true,
            ..Default::default()
        };
        let image = ftd::Image {
            src: i(
                "/static/home/document-type-min.png",
                Some(s("foo/bar#src@0")),
            ),
            common: Box::new(ftd::Common {
                reference: Some(s("foo/bar#src@0")),
                condition: Some(ftd::Condition {
                    variable: s("foo/bar#src@0"),
                    value: serde_json::Value::String(s("$IsNotNull$")),
                }),
                ..Default::default()
            },
            ..Default::default()
        };
        let second_image = ftd::Image {
            src: i("second-image.png", Some(s("foo/bar#src@1"))),
            common: Box::new(ftd::Common {
                reference: Some(s("foo/bar#src@1")),
                condition: Some(ftd::Condition {
                    variable: s("foo/bar#src@1"),
                    value: serde_json::Value::String(s("$IsNotNull$")),
                }),
                ..Default::default()
            },
            ..Default::default()
        };

        let mut main = p2::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                common: Box::new(ftd::Common {
                    padding: Some(30),
                    ..Default::default()
                },
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Markup(title),
                        ftd::Element::Markup(about),
                        ftd::Element::Image(image),
                    ],
                    ..Default::default()
                },
            }));
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                common: Box::new(ftd::Common {
                    padding: Some(30),
                    ..Default::default()
                },
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Markup(second_title),
                        ftd::Element::Markup(second_about),
                        ftd::Element::Image(second_image),
                    ],
                    ..Default::default()
                },
            }));

        let mut bag = white_two_image_bag(true);
        bag.insert(
            "foo/bar#src0".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "src0".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Record {
                        name: s("ftd#image-src"),
                        fields: std::iter::IntoIterator::into_iter([
                            (
                                s("dark"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: "/static/home/document-type-min.png".to_string(),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                            (
                                s("light"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: "/static/home/document-type-min.png".to_string(),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                        ])
                        .collect(),
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#src1".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "src1".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Record {
                        name: s("ftd#image-src"),
                        fields: std::iter::IntoIterator::into_iter([
                            (
                                s("dark"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: "second-image.png".to_string(),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                            (
                                s("light"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: "second-image.png".to_string(),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                        ])
                        .collect(),
                    },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            "foo/bar#about@0".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "about".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("UI screens, behaviour and journeys, database tables, APIs, how to\ncontribute to, deploy, or monitor microservice, everything that\nmakes web or mobile product teams productive."),
                        source: ftd::TextSource::Body,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#about@1".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "about".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Optional {
                        data: Box::new(None),
                        kind: ftd::ftd2021::p2::Kind::body(),
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#src@0".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "src".to_string(),
                value: ftd::PropertyValue::Reference {
                    name: s("foo/bar#src0"),
                    kind: ftd::ftd2021::p2::Kind::Optional {
                        kind: Box::new(ftd::ftd2021::p2::Kind::Record {
                            name: s("ftd#image-src"),
                            default: None,
                            is_reference: false,
                        }),
                        is_reference: false,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#src@1".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "src".to_string(),
                value: ftd::PropertyValue::Reference {
                    name: s("foo/bar#src1"),
                    kind: ftd::ftd2021::p2::Kind::Optional {
                        kind: Box::new(ftd::ftd2021::p2::Kind::Record {
                            name: s("ftd#image-src"),
                            default: None,
                            is_reference: false,
                        }),
                        is_reference: false,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#title@0".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "title".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("What kind of documentation?"),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#title@1".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "title".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("second call"),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
            }),
        );

        insert_universal_variables_by_count(2, "foo/bar", &mut bag);

        p!(
            "
            -- ftd.image-src src0:
            light: /static/home/document-type-min.png
            dark: /static/home/document-type-min.png

            -- ftd.image-src src1:
            light: second-image.png
            dark: second-image.png

            -- ftd.column white-two-image:
            caption title:
            optional body about:
            optional ftd.image-src src:
            padding: 30

            --- ftd.text:
            text: $title
            align: center

            --- ftd.text:
            if: $about is not null
            text: $about

            --- ftd.image:
            if: $src is not null
            src: $src

            -- white-two-image: What kind of documentation?
            src: $src0

            UI screens, behaviour and journeys, database tables, APIs, how to
            contribute to, deploy, or monitor microservice, everything that
            makes web or mobile product teams productive.

            -- white-two-image: second call
            src: $src1
            ",
            (bag, main),
        );
    }

    #[test]
    fn conditional_header() {
        let title = ftd::Markups {
            text: ftd::ftd2021::rendered::markup_line("What kind of documentation?"),
            common: Box::new(ftd::Common {
                position: Some(ftd::Position::Center),
                reference: Some(s("foo/bar#title@0")),
                ..Default::default()
            },
            line: true,
            ..Default::default()
        };
        let second_title = ftd::Markups {
            text: ftd::ftd2021::rendered::markup_line("second call"),
            common: Box::new(ftd::Common {
                position: Some(ftd::Position::Center),
                reference: Some(s("foo/bar#title@1")),
                ..Default::default()
            },
            line: true,
            ..Default::default()
        };
        let third_title = ftd::Markups {
            text: ftd::ftd2021::rendered::markup_line("third call"),
            common: Box::new(ftd::Common {
                position: Some(ftd::Position::Center),
                reference: Some(s("foo/bar#title@2")),
                ..Default::default()
            },
            line: true,
            ..Default::default()
        };
        let about = ftd::Markups {
            text: ftd::ftd2021::rendered::markup_line(
                indoc::indoc!(
                    "
                    UI screens, behaviour and journeys, database tables, APIs, how to
                    contribute to, deploy, or monitor microservice, everything that
                    makes web or mobile product teams productive.
                    "
                )
                .trim(),
            ),
            common: Box::new(ftd::Common {
                condition: Some(ftd::Condition {
                    variable: s("foo/bar#about@0"),
                    value: serde_json::Value::String(s("$IsNotNull$")),
                }),
                reference: Some(s("foo/bar#about@0")),
                ..Default::default()
            },
            line: true,
            ..Default::default()
        };
        let second_about = ftd::Markups {
            text: ftd::ftd2021::rendered::markup_line(""),
            common: Box::new(ftd::Common {
                condition: Some(ftd::Condition {
                    variable: s("foo/bar#about@1"),
                    value: serde_json::Value::String(s("$IsNotNull$")),
                }),
                reference: Some(s("foo/bar#about@1")),
                is_not_visible: true,
                ..Default::default()
            },
            line: true,
            ..Default::default()
        };
        let third_about = ftd::Markups {
            text: ftd::ftd2021::rendered::markup_line(""),
            common: Box::new(ftd::Common {
                condition: Some(ftd::Condition {
                    variable: s("foo/bar#about@2"),
                    value: serde_json::Value::String(s("$IsNotNull$")),
                }),
                reference: Some(s("foo/bar#about@2")),
                is_not_visible: true,
                ..Default::default()
            },
            line: true,
            ..Default::default()
        };
        let image = ftd::Image {
            src: i(
                "/static/home/document-type-min.png",
                Some(s("foo/bar#src@0")),
            ),
            common: Box::new(ftd::Common {
                reference: Some(s("foo/bar#src@0")),
                condition: Some(ftd::Condition {
                    variable: s("foo/bar#src@0"),
                    value: serde_json::Value::String(s("$IsNotNull$")),
                }),
                ..Default::default()
            },
            ..Default::default()
        };
        let second_image = ftd::Image {
            src: i("second-image.png", Some(s("foo/bar#src@1"))),
            common: Box::new(ftd::Common {
                reference: Some(s("foo/bar#src@1")),
                condition: Some(ftd::Condition {
                    variable: s("foo/bar#src@1"),
                    value: serde_json::Value::String(s("$IsNotNull$")),
                }),
                ..Default::default()
            },
            ..Default::default()
        };
        let third_image = ftd::Image {
            src: i("", Some(s("foo/bar#src@2"))),
            common: Box::new(ftd::Common {
                reference: Some(s("foo/bar#src@2")),
                condition: Some(ftd::Condition {
                    variable: s("foo/bar#src@2"),
                    value: serde_json::Value::String(s("$IsNotNull$")),
                }),
                is_not_visible: true,
                ..Default::default()
            },
            ..Default::default()
        };

        let mut main = p2::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                common: Box::new(ftd::Common {
                    padding: Some(30),
                    ..Default::default()
                },
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Markup(title),
                        ftd::Element::Markup(about),
                        ftd::Element::Image(image),
                    ],
                    ..Default::default()
                },
            }));
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                common: Box::new(ftd::Common {
                    padding: Some(30),
                    ..Default::default()
                },
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Markup(second_title),
                        ftd::Element::Markup(second_about),
                        ftd::Element::Image(second_image),
                    ],
                    ..Default::default()
                },
            }));
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                common: Box::new(ftd::Common {
                    padding: Some(30),
                    ..Default::default()
                },
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Markup(third_title),
                        ftd::Element::Markup(third_about),
                        ftd::Element::Image(third_image),
                    ],
                    ..Default::default()
                },
            }));

        let mut bag = white_two_image_bag(true);
        bag.insert(
            "foo/bar#src0".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "src0".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Record {
                        name: s("ftd#image-src"),
                        fields: std::iter::IntoIterator::into_iter([
                            (
                                s("dark"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: "/static/home/document-type-min.png".to_string(),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                            (
                                s("light"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: "/static/home/document-type-min.png".to_string(),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                        ])
                        .collect(),
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#src1".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "src1".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Record {
                        name: s("ftd#image-src"),
                        fields: std::iter::IntoIterator::into_iter([
                            (
                                s("dark"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: "second-image.png".to_string(),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                            (
                                s("light"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: "second-image.png".to_string(),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                        ])
                        .collect(),
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#about@0".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "about".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("UI screens, behaviour and journeys, database tables, APIs, how to\ncontribute to, deploy, or monitor microservice, everything that\nmakes web or mobile product teams productive."),
                        source: ftd::TextSource::Body,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#about@1".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "about".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Optional {
                        data: Box::new(None),
                        kind: ftd::ftd2021::p2::Kind::body(),
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#about@2".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "about".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Optional {
                        data: Box::new(None),
                        kind: ftd::ftd2021::p2::Kind::body(),
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#src@0".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "src".to_string(),
                value: ftd::PropertyValue::Reference {
                    name: s("foo/bar#src0"),
                    kind: ftd::ftd2021::p2::Kind::Optional {
                        kind: Box::new(ftd::ftd2021::p2::Kind::Record {
                            name: s("ftd#image-src"),
                            default: None,
                            is_reference: false,
                        }),
                        is_reference: false,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#src@1".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "src".to_string(),
                value: ftd::PropertyValue::Reference {
                    name: s("foo/bar#src1"),
                    kind: ftd::ftd2021::p2::Kind::Optional {
                        kind: Box::new(ftd::ftd2021::p2::Kind::Record {
                            name: s("ftd#image-src"),
                            default: None,
                            is_reference: false,
                        }),
                        is_reference: false,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#src@2".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "src".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Optional {
                        data: Box::new(None),
                        kind: ftd::ftd2021::p2::Kind::Record {
                            name: s("ftd#image-src"),
                            default: None,
                            is_reference: false,
                        },
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#title@0".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "title".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("What kind of documentation?"),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#title@1".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "title".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("second call"),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#title@2".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "title".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("third call"),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
            }),
        );

        insert_universal_variables_by_count(3, "foo/bar", &mut bag);

        p!(
            "
            -- ftd.image-src src0:
            light: /static/home/document-type-min.png
            dark: /static/home/document-type-min.png

            -- ftd.image-src src1:
            light: second-image.png
            dark: second-image.png

            -- ftd.column white-two-image:
            caption title:
            optional body about:
            optional ftd.image-src src:
            padding: 30

            --- ftd.text:
            text: $title
            align: center

            --- ftd.text:
            if: $about is not null
            text: $about

            --- ftd.image:
            if: $src is not null
            src: $src

            -- white-two-image: What kind of documentation?
            src: $src0

            UI screens, behaviour and journeys, database tables, APIs, how to
            contribute to, deploy, or monitor microservice, everything that
            makes web or mobile product teams productive.

            -- white-two-image: second call
            src: $src1

            -- white-two-image: third call
            ",
            (bag, main),
        );
    }

    #[test]
    fn markdown() {
        let mut bag = interpreter::default_bag();
        bag.insert(
            s("fifthtry/ft#markdown"),
            ftd::ftd2021::p2::Thing::Component(ftd::Component {
                invocations: Default::default(),
                full_name: "fifthtry/ft#markdown".to_string(),
                root: s("ftd#text"),
                arguments: [
                    vec![(s("body"), ftd::ftd2021::p2::Kind::body())],
                    universal_arguments_as_vec(),
                ]
                .concat()
                .into_iter()
                .collect(),
                properties: std::iter::IntoIterator::into_iter([(
                    s("text"),
                    ftd::ftd2021::component::Property {
                        default: Some(ftd::PropertyValue::Variable {
                            name: s("body"),
                            kind: ftd::ftd2021::p2::Kind::string().string_any(),
                        }),
                        conditions: vec![],
                        ..Default::default()
                    },
                )])
                .collect(),
                ..Default::default()
            }),
        );
        bag.insert(
            s("fifthtry/ft#dark-mode"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("dark-mode"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Boolean { value: true },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            s("fifthtry/ft#toc"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("toc"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: "not set".to_string(),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            s("foo/bar#h0"),
            ftd::ftd2021::p2::Thing::Component(ftd::Component {
                invocations: Default::default(),
                full_name: "foo/bar#h0".to_string(),
                root: s("ftd#column"),
                arguments: [
                    vec![
                        (s("body"), ftd::ftd2021::p2::Kind::body().into_optional()),
                        (s("title"), ftd::ftd2021::p2::Kind::caption()),
                    ],
                    universal_arguments_as_vec(),
                ]
                .concat()
                .into_iter()
                .collect(),
                instructions: vec![
                    ftd::Instruction::ChildComponent {
                        child: ftd::ChildComponent {
                            events: vec![],
                            condition: None,
                            root: s("ftd#text"),
                            properties: std::iter::IntoIterator::into_iter([(
                                s("text"),
                                ftd::ftd2021::component::Property {
                                    default: Some(ftd::PropertyValue::Variable {
                                        name: s("title"),
                                        kind: ftd::ftd2021::p2::Kind::caption_or_body(),
                                    }),
                                    conditions: vec![],
                                    ..Default::default()
                                },
                            )])
                            .collect(),
                            ..Default::default()
                        },
                    },
                    ftd::Instruction::ChildComponent {
                        child: ftd::ChildComponent {
                            events: vec![],
                            condition: Some(ftd::ftd2021::p2::Boolean::IsNotNull {
                                value: ftd::PropertyValue::Variable {
                                    name: s("body"),
                                    kind: ftd::ftd2021::p2::Kind::body().into_optional(),
                                },
                            }),
                            root: s("fifthtry/ft#markdown"),
                            properties: std::iter::IntoIterator::into_iter([(
                                s("body"),
                                ftd::ftd2021::component::Property {
                                    default: Some(ftd::PropertyValue::Variable {
                                        name: s("body"),
                                        kind: ftd::ftd2021::p2::Kind::body(),
                                    }),
                                    conditions: vec![],
                                    ..Default::default()
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
        bag.insert(
            s("foo/bar#body@0"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("body"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: "what about the body?".to_string(),
                        source: ftd::TextSource::Body,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            s("foo/bar#body@0,1"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("body"),
                value: ftd::PropertyValue::Variable {
                    name: "foo/bar#body@0".to_string(),
                    kind: ftd::ftd2021::p2::Kind::body(),
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            s("foo/bar#body@1"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("body"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Optional {
                        data: Box::new(None),
                        kind: ftd::ftd2021::p2::Kind::body(),
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            s("foo/bar#body@1,1"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("body"),
                value: ftd::PropertyValue::Variable {
                    name: "foo/bar#body@1".to_string(),
                    kind: ftd::ftd2021::p2::Kind::body(),
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            s("foo/bar#title@0"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("title"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: "hello".to_string(),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            s("foo/bar#title@1"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("title"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: "heading without body".to_string(),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
            }),
        );

        let levels: Vec<String> = vec![s("0"), s("0,1"), s("1"), s("1,1")];
        insert_universal_variables_by_levels(levels, "foo/bar", &mut bag);

        let mut main = p2::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::ftd2021::rendered::markup_line("hello"),
                            line: true,
                            common: Box::new(ftd::Common {
                                reference: Some(s("foo/bar#title@0")),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::ftd2021::rendered::markup_line("what about the body?"),
                            line: true,
                            common: Box::new(ftd::Common {
                                condition: Some(ftd::Condition {
                                    variable: s("foo/bar#body@0"),
                                    value: serde_json::Value::String(s("$IsNotNull$")),
                                }),
                                reference: Some(s("foo/bar#body@0,1")),
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
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::ftd2021::rendered::markup_line("heading without body"),
                            line: true,
                            common: Box::new(ftd::Common {
                                reference: Some(s("foo/bar#title@1")),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::ftd2021::rendered::markup_line(""),
                            line: true,
                            common: Box::new(ftd::Common {
                                condition: Some(ftd::Condition {
                                    variable: s("foo/bar#body@1"),
                                    value: serde_json::Value::String(s("$IsNotNull$")),
                                }),
                                reference: Some(s("foo/bar#body@1,1")),
                                is_not_visible: true,
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    ],
                    ..Default::default()
                },
                ..Default::default()
            }));

        p!(
            "
            -- import: fifthtry/ft

            -- ftd.column h0:
            caption title:
            optional body body:

            --- ftd.text:
            text: $title

            --- ft.markdown:
            if: $body is not null
            body: $body

            -- h0: hello

            what about the body?

            -- h0: heading without body
            ",
            (bag, main),
        );
    }

    #[test]
    fn width() {
        let mut bag = interpreter::default_bag();

        bag.insert(
            s("foo/bar#src@0"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("src"),
                value: ftd::PropertyValue::Reference {
                    name: s("foo/bar#src0"),
                    kind: ftd::ftd2021::p2::Kind::Record {
                        name: s("ftd#image-src"),
                        default: None,
                        is_reference: false,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            s("foo/bar#src@1"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("src"),
                value: ftd::PropertyValue::Reference {
                    name: s("foo/bar#src1"),
                    kind: ftd::ftd2021::p2::Kind::Record {
                        name: s("ftd#image-src"),
                        default: None,
                        is_reference: false,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            s("foo/bar#width@0"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("width"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Optional {
                        data: Box::new(None),
                        kind: ftd::ftd2021::p2::Kind::string(),
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            s("foo/bar#width@1"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("width"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: "300".to_string(),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#src0".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "src0".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Record {
                        name: s("ftd#image-src"),
                        fields: std::iter::IntoIterator::into_iter([
                            (
                                s("dark"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: "foo.png".to_string(),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                            (
                                s("light"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: "foo.png".to_string(),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                        ])
                        .collect(),
                    },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            "foo/bar#src1".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "src1".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Record {
                        name: s("ftd#image-src"),
                        fields: std::iter::IntoIterator::into_iter([
                            (
                                s("dark"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: "bar.png".to_string(),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                            (
                                s("light"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: "bar.png".to_string(),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                        ])
                        .collect(),
                    },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            s("foo/bar#image"),
            ftd::ftd2021::p2::Thing::Component(ftd::Component {
                invocations: Default::default(),
                full_name: "foo/bar#image".to_string(),
                root: s("ftd#column"),
                arguments: [
                    vec![
                        (s("width"), ftd::ftd2021::p2::Kind::string().into_optional()),
                        (
                            s("src"),
                            ftd::ftd2021::p2::Kind::Record {
                                name: s("ftd#image-src"),
                                default: None,
                                is_reference: false,
                            },
                        ),
                    ],
                    universal_arguments_as_vec(),
                ]
                .concat()
                .into_iter()
                .collect(),
                instructions: vec![ftd::Instruction::ChildComponent {
                    child: ftd::ChildComponent {
                        events: vec![],
                        condition: None,
                        root: s("ftd#image"),
                        properties: std::iter::IntoIterator::into_iter([
                            (
                                s("src"),
                                ftd::ftd2021::component::Property {
                                    default: Some(ftd::PropertyValue::Variable {
                                        name: s("src"),
                                        kind: ftd::ftd2021::p2::Kind::Record {
                                            name: s("ftd#image-src"),
                                            default: None,
                                            is_reference: false,
                                        },
                                    }),
                                    conditions: vec![],
                                    ..Default::default()
                                },
                            ),
                            (
                                s("width"),
                                ftd::ftd2021::component::Property {
                                    default: Some(ftd::PropertyValue::Variable {
                                        name: s("width"),
                                        kind: ftd::ftd2021::p2::Kind::string().into_optional(),
                                    }),
                                    conditions: vec![],
                                    ..Default::default()
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

        insert_universal_variables_by_count(2, "foo/bar", &mut bag);

        let mut main = p2::default_column();

        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![ftd::Element::Image(ftd::Image {
                        src: i("foo.png", Some(s("foo/bar#src@0"))),
                        common: Box::new(ftd::Common {
                            reference: Some(s("foo/bar#src@0")),
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
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![ftd::Element::Image(ftd::Image {
                        src: i("bar.png", Some(s("foo/bar#src@1"))),
                        common: Box::new(ftd::Common {
                            reference: Some(s("foo/bar#src@1")),
                            width: Some(ftd::Length::Px { value: 300 }),
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
            -- ftd.image-src src0:
            light: foo.png
            dark: foo.png

            -- ftd.image-src src1:
            light: bar.png
            dark: bar.png

            -- ftd.column image:
            ftd.image-src src:
            optional string width:

            --- ftd.image:
            src: $src
            width: $width

            -- image:
            src: $src0

            -- image:
            src: $src1
            width: 300
            ",
            (bag, main),
        );
    }

    #[test]
    fn decimal() {
        let mut bag = interpreter::default_bag();

        bag.insert(
            "foo/bar#foo".to_string(),
            ftd::ftd2021::p2::Thing::Component(ftd::Component {
                full_name: s("foo/bar#foo"),
                root: "ftd#row".to_string(),
                instructions: vec![
                    ftd::Instruction::ChildComponent {
                        child: ftd::ChildComponent {
                            events: vec![],
                            condition: None,
                            root: s("ftd#decimal"),
                            properties: std::iter::IntoIterator::into_iter([
                                (
                                    s("value"),
                                    ftd::ftd2021::component::Property {
                                        default: Some(ftd::PropertyValue::Value {
                                            value: ftd::Value::Decimal { value: 0.06 },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                                (
                                    s("format"),
                                    ftd::ftd2021::component::Property {
                                        default: Some(ftd::PropertyValue::Value {
                                            value: ftd::Value::String {
                                                text: s(".1f"),
                                                source: ftd::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                            ])
                            .collect(),
                            ..Default::default()
                        },
                    },
                    ftd::Instruction::ChildComponent {
                        child: ftd::ChildComponent {
                            events: vec![],
                            condition: None,
                            root: s("ftd#decimal"),
                            properties: std::iter::IntoIterator::into_iter([(
                                s("value"),
                                ftd::ftd2021::component::Property {
                                    default: Some(ftd::PropertyValue::Value {
                                        value: ftd::Value::Decimal { value: 0.01 },
                                    }),
                                    conditions: vec![],
                                    ..Default::default()
                                },
                            )])
                            .collect(),
                            ..Default::default()
                        },
                    },
                ],
                arguments: [
                    vec![(s("x"), ftd::ftd2021::p2::Kind::integer())],
                    universal_arguments_as_vec(),
                ]
                .concat()
                .into_iter()
                .collect(),
                ..Default::default()
            }),
        );
        bag.insert(
            "foo/bar#x@0".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "x".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Integer { value: 10 },
                },
                conditions: vec![],
            }),
        );

        insert_universal_variables_by_count(1, "foo/bar", &mut bag);

        let mut main = p2::default_column();
        let mut row: ftd::Row = Default::default();
        row.container
            .children
            .push(ftd::Element::Decimal(ftd::Text {
                text: ftd::ftd2021::rendered::markup_line("0.1"),
                line: false,
                ..Default::default()
            }));
        row.container
            .children
            .push(ftd::Element::Decimal(ftd::Text {
                text: ftd::ftd2021::rendered::markup_line("0.01"),
                line: false,
                ..Default::default()
            }));
        main.container.children.push(ftd::Element::Row(row));

        p!(
            "
            -- ftd.row foo:
            integer x:

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
        let mut bag = interpreter::default_bag();

        bag.insert(
            "foo/bar#x@0".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "x".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Integer { value: 10 },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            "foo/bar#foo".to_string(),
            ftd::ftd2021::p2::Thing::Component(ftd::Component {
                full_name: s("foo/bar#foo"),
                root: "ftd#row".to_string(),
                instructions: vec![
                    ftd::Instruction::ChildComponent {
                        child: ftd::ChildComponent {
                            events: vec![],
                            condition: None,
                            root: s("ftd#integer"),
                            properties: std::iter::IntoIterator::into_iter([
                                (
                                    s("value"),
                                    ftd::ftd2021::component::Property {
                                        default: Some(ftd::PropertyValue::Value {
                                            value: ftd::Value::Integer { value: 3 },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                                (
                                    s("format"),
                                    ftd::ftd2021::component::Property {
                                        default: Some(ftd::PropertyValue::Value {
                                            value: ftd::Value::String {
                                                text: s("b"),
                                                source: ftd::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                            ])
                            .collect(),
                            ..Default::default()
                        },
                    },
                    ftd::Instruction::ChildComponent {
                        child: ftd::ChildComponent {
                            events: vec![],
                            condition: None,
                            root: s("ftd#integer"),
                            properties: std::iter::IntoIterator::into_iter([(
                                s("value"),
                                ftd::ftd2021::component::Property {
                                    default: Some(ftd::PropertyValue::Value {
                                        value: ftd::Value::Integer { value: 14 },
                                    }),
                                    conditions: vec![],
                                    ..Default::default()
                                },
                            )])
                            .collect(),
                            ..Default::default()
                        },
                    },
                ],
                arguments: [
                    vec![(s("x"), ftd::ftd2021::p2::Kind::integer())],
                    universal_arguments_as_vec(),
                ]
                .concat()
                .into_iter()
                .collect(),
                ..Default::default()
            }),
        );

        insert_universal_variables_by_count(1, "foo/bar", &mut bag);

        let mut main = p2::default_column();
        let mut row: ftd::Row = Default::default();
        row.container
            .children
            .push(ftd::Element::Integer(ftd::Text {
                text: ftd::ftd2021::rendered::markup_line("11"),
                line: false,
                ..Default::default()
            }));
        row.container
            .children
            .push(ftd::Element::Integer(ftd::Text {
                text: ftd::ftd2021::rendered::markup_line("14"),
                line: false,
                ..Default::default()
            }));

        main.container.children.push(ftd::Element::Row(row));

        p!(
            "
            -- ftd.row foo:
            integer x:

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
        let mut bag = interpreter::default_bag();

        bag.insert(
            "foo/bar#foo".to_string(),
            ftd::ftd2021::p2::Thing::Component(ftd::Component {
                full_name: s("foo/bar#foo"),
                root: "ftd#row".to_string(),
                instructions: vec![
                    ftd::Instruction::ChildComponent {
                        child: ftd::ChildComponent {
                            events: vec![],
                            condition: None,
                            root: s("ftd#boolean"),
                            properties: std::iter::IntoIterator::into_iter([
                                (
                                    s("value"),
                                    ftd::ftd2021::component::Property {
                                        default: Some(ftd::PropertyValue::Value {
                                            value: ftd::Value::Boolean { value: true },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                                (
                                    s("true"),
                                    ftd::ftd2021::component::Property {
                                        default: Some(ftd::PropertyValue::Value {
                                            value: ftd::Value::String {
                                                text: s("show this when value is true"),
                                                source: ftd::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                                (
                                    s("false"),
                                    ftd::ftd2021::component::Property {
                                        default: Some(ftd::PropertyValue::Value {
                                            value: ftd::Value::String {
                                                text: s("show this when value is false"),
                                                source: ftd::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                            ])
                            .collect(),
                            ..Default::default()
                        },
                    },
                    ftd::Instruction::ChildComponent {
                        child: ftd::ChildComponent {
                            events: vec![],
                            condition: None,
                            root: s("ftd#boolean"),
                            properties: std::iter::IntoIterator::into_iter([
                                (
                                    s("value"),
                                    ftd::ftd2021::component::Property {
                                        default: Some(ftd::PropertyValue::Value {
                                            value: ftd::Value::Boolean { value: false },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                                (
                                    s("true"),
                                    ftd::ftd2021::component::Property {
                                        default: Some(ftd::PropertyValue::Value {
                                            value: ftd::Value::String {
                                                text: s("show this when value is true"),
                                                source: ftd::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                                (
                                    s("false"),
                                    ftd::ftd2021::component::Property {
                                        default: Some(ftd::PropertyValue::Value {
                                            value: ftd::Value::String {
                                                text: s("show this when value is false"),
                                                source: ftd::TextSource::Header,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                            ])
                            .collect(),
                            ..Default::default()
                        },
                    },
                ],
                arguments: [
                    vec![(s("x"), ftd::ftd2021::p2::Kind::integer())],
                    universal_arguments_as_vec(),
                ]
                .concat()
                .into_iter()
                .collect(),
                ..Default::default()
            }),
        );
        bag.insert(
            "foo/bar#x@0".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "x".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Integer { value: 10 },
                },
                conditions: vec![],
            }),
        );

        insert_universal_variables_by_count(1, "foo/bar", &mut bag);

        let mut main = p2::default_column();
        let mut row: ftd::Row = Default::default();
        row.container
            .children
            .push(ftd::Element::Boolean(ftd::Text {
                text: ftd::ftd2021::rendered::markup_line("show this when value is true"),
                line: false,
                ..Default::default()
            }));
        row.container
            .children
            .push(ftd::Element::Boolean(ftd::Text {
                text: ftd::ftd2021::rendered::markup_line("show this when value is false"),
                line: false,
                ..Default::default()
            }));
        main.container.children.push(ftd::Element::Row(row));

        p!(
            "
            -- ftd.row foo:
            integer x:

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
        let mut main = p2::default_column();
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("present is true"),
                line: true,
                common: Box::new(ftd::Common {
                    condition: Some(ftd::Condition {
                        variable: "foo/bar#present".to_string(),
                        value: serde_json::Value::Bool(true),
                    }),
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("present is false"),
                line: true,
                common: Box::new(ftd::Common {
                    condition: Some(ftd::Condition {
                        variable: "foo/bar#present".to_string(),
                        value: serde_json::Value::Bool(false),
                    }),
                    is_not_visible: true,
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("dark-mode is true"),
                line: true,
                common: Box::new(ftd::Common {
                    condition: Some(ftd::Condition {
                        variable: "fifthtry/ft#dark-mode".to_string(),
                        value: serde_json::Value::Bool(true),
                    }),
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("dark-mode is false"),
                line: true,
                common: Box::new(ftd::Common {
                    condition: Some(ftd::Condition {
                        variable: "fifthtry/ft#dark-mode".to_string(),
                        value: serde_json::Value::Bool(false),
                    }),
                    is_not_visible: true,
                    ..Default::default()
                },
                ..Default::default()
            }));

        let mut column: ftd::Column = Default::default();
        column
            .container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("inner present false"),
                line: true,
                common: Box::new(ftd::Common {
                    condition: Some(ftd::Condition {
                        variable: "foo/bar#present".to_string(),
                        value: serde_json::Value::Bool(false),
                    }),
                    is_not_visible: true,
                    ..Default::default()
                },
                ..Default::default()
            }));

        column
            .container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("inner present true"),
                line: true,
                common: Box::new(ftd::Common {
                    condition: Some(ftd::Condition {
                        variable: "foo/bar#present".to_string(),
                        value: serde_json::Value::Bool(true),
                    }),
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container.children.push(ftd::Element::Column(column));

        let mut column: ftd::Column = Default::default();
        column
            .container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("argument present false"),
                line: true,
                common: Box::new(ftd::Common {
                    condition: Some(ftd::Condition {
                        variable: s("foo/bar#present@5"),
                        value: serde_json::Value::Bool(false),
                    }),
                    ..Default::default()
                },
                ..Default::default()
            }));
        column
            .container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("argument present true"),
                line: true,
                common: Box::new(ftd::Common {
                    condition: Some(ftd::Condition {
                        variable: s("foo/bar#present@5"),
                        value: serde_json::Value::Bool(true),
                    }),
                    is_not_visible: true,
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container.children.push(ftd::Element::Column(column));

        let mut column: ftd::Column = Default::default();
        column
            .container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("argument present false"),
                line: true,
                common: Box::new(ftd::Common {
                    condition: Some(ftd::Condition {
                        variable: s("foo/bar#present@6"),
                        value: serde_json::Value::Bool(false),
                    }),
                    is_not_visible: true,
                    ..Default::default()
                },
                ..Default::default()
            }));
        column
            .container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("argument present true"),
                line: true,
                common: Box::new(ftd::Common {
                    condition: Some(ftd::Condition {
                        variable: s("foo/bar#present@6"),
                        value: serde_json::Value::Bool(true),
                    }),
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container.children.push(ftd::Element::Column(column));

        let mut column: ftd::Column = Default::default();
        column
            .container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("foo2 dark-mode is true"),
                line: true,
                common: Box::new(ftd::Common {
                    condition: Some(ftd::Condition {
                        variable: "fifthtry/ft#dark-mode".to_string(),
                        value: serde_json::Value::Bool(true),
                    }),
                    ..Default::default()
                },
                ..Default::default()
            }));

        column
            .container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("foo2 dark-mode is false"),
                line: true,
                common: Box::new(ftd::Common {
                    condition: Some(ftd::Condition {
                        variable: "fifthtry/ft#dark-mode".to_string(),
                        value: serde_json::Value::Bool(false),
                    }),
                    is_not_visible: true,
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container.children.push(ftd::Element::Column(column));

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("hello literal truth"),
                line: true,
                ..Default::default()
            }));

        main.container.children.push(ftd::Element::Null);

        p!(
            "
            -- import: fifthtry/ft
            -- boolean present: true

            -- ftd.text: present is true
            if: $present

            -- ftd.text: present is false
            if: not $present

            -- ftd.text: dark-mode is true
            if: $ft.dark-mode

            -- ftd.text: dark-mode is false
            if: not $ft.dark-mode

            -- ftd.column foo:

            --- ftd.text: inner present false
            if: not $present

            --- ftd.text: inner present true
            if: $present

            -- foo:

            -- ftd.column bar:
            boolean present:

            --- ftd.text: argument present false
            if: not $present

            --- ftd.text: argument present true
            if: $present

            -- bar:
            present: false

            -- bar:
            present: $ft.dark-mode

            -- ftd.column foo2:

            --- ftd.text: foo2 dark-mode is true
            if: $ft.dark-mode

            --- ftd.text: foo2 dark-mode is false
            if: not $ft.dark-mode

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
        let mut bag = interpreter::default_bag();

        bag.insert(
            "foo/bar#foo".to_string(),
            ftd::ftd2021::p2::Thing::Component(ftd::Component {
                root: "ftd#column".to_string(),
                full_name: "foo/bar#foo".to_string(),
                arguments: universal_arguments_as_map(),
                instructions: vec![
                    ftd::ftd2021::component::Instruction::ChildComponent {
                        child: ftd::ftd2021::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "ftd#row".to_string(),
                            condition: None,
                            properties: std::iter::IntoIterator::into_iter([(
                                s("id"),
                                ftd::ftd2021::component::Property {
                                    default: Some(ftd::PropertyValue::Value {
                                        value: ftd::ftd2021::variable::Value::String {
                                            text: "r1".to_string(),
                                            source: ftd::TextSource::Header,
                                        },
                                    }),
                                    conditions: vec![],
                                    ..Default::default()
                                },
                            )])
                            .collect(),
                            ..Default::default()
                        },
                    },
                    ftd::ftd2021::component::Instruction::ChildComponent {
                        child: ftd::ftd2021::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "ftd#row".to_string(),
                            condition: None,
                            properties: std::iter::IntoIterator::into_iter([(
                                s("id"),
                                ftd::ftd2021::component::Property {
                                    default: Some(ftd::PropertyValue::Value {
                                        value: ftd::ftd2021::variable::Value::String {
                                            text: "r2".to_string(),
                                            source: ftd::TextSource::Header,
                                        },
                                    }),
                                    conditions: vec![],
                                    ..Default::default()
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

        insert_universal_variables_by_count(2, "foo/bar", &mut bag);
        insert_update_string_by_root("foo/bar#id@0", "foo-1", "header", &mut bag);
        insert_update_string_by_root("foo/bar#id@1", "foo-2", "header", &mut bag);

        let mut main = p2::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![ftd::Element::Row(ftd::Row {
                        spacing: None,
                        container: ftd::Container {
                            children: vec![
                                ftd::Element::Row(ftd::Row {
                                    spacing: None,
                                    common: Box::new(ftd::Common {
                                        data_id: Some(s("r2")),
                                        id: Some(s("foo-1:r2")),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                }),
                                ftd::Element::Markup(ftd::Markups {
                                    text: ftd::ftd2021::rendered::markup_line("hello"),
                                    line: true,
                                    ..Default::default()
                                }),
                            ],
                            ..Default::default()
                        },
                        common: Box::new(ftd::Common {
                            data_id: Some(s("r1")),
                            id: Some(s("foo-1:r1")),
                            ..Default::default()
                        },
                    })],
                    ..Default::default()
                },
                common: Box::new(ftd::Common {
                    data_id: Some(s("foo-1")),
                    id: Some(s("foo-1")),
                    ..Default::default()
                },
            }));

        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![ftd::Element::Row(ftd::Row {
                        spacing: None,
                        container: ftd::Container {
                            children: vec![ftd::Element::Row(ftd::Row {
                                spacing: None,
                                common: Box::new(ftd::Common {
                                    data_id: Some(s("r2")),
                                    id: Some(s("foo-2:r2")),
                                    ..Default::default()
                                },
                                ..Default::default()
                            })],
                            ..Default::default()
                        },
                        common: Box::new(ftd::Common {
                            data_id: Some(s("r1")),
                            id: Some(s("foo-2:r1")),
                            ..Default::default()
                        },
                    })],
                    ..Default::default()
                },
                common: Box::new(ftd::Common {
                    data_id: Some(s("foo-2")),
                    id: Some(s("foo-2")),
                    ..Default::default()
                },
            }));

        p!(
            "
            -- ftd.column foo:

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
            ",
            (bag, main),
        );
    }

    #[test]
    fn inner_container_using_import() {
        let mut bag = interpreter::default_bag();

        bag.insert(
            "inner_container#foo".to_string(),
            ftd::ftd2021::p2::Thing::Component(ftd::Component {
                root: "ftd#column".to_string(),
                full_name: "inner_container#foo".to_string(),
                arguments: universal_arguments_as_map(),
                instructions: vec![
                    ftd::ftd2021::component::Instruction::ChildComponent {
                        child: ftd::ftd2021::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "ftd#row".to_string(),
                            condition: None,
                            properties: std::iter::IntoIterator::into_iter([(
                                s("id"),
                                ftd::ftd2021::component::Property {
                                    default: Some(ftd::PropertyValue::Value {
                                        value: ftd::ftd2021::variable::Value::String {
                                            text: "r1".to_string(),
                                            source: ftd::TextSource::Header,
                                        },
                                    }),
                                    conditions: vec![],
                                    ..Default::default()
                                },
                            )])
                            .collect(),
                            ..Default::default()
                        },
                    },
                    ftd::ftd2021::component::Instruction::ChildComponent {
                        child: ftd::ftd2021::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "ftd#row".to_string(),
                            condition: None,
                            properties: std::iter::IntoIterator::into_iter([(
                                s("id"),
                                ftd::ftd2021::component::Property {
                                    default: Some(ftd::PropertyValue::Value {
                                        value: ftd::ftd2021::variable::Value::String {
                                            text: "r2".to_string(),
                                            source: ftd::TextSource::Header,
                                        },
                                    }),
                                    conditions: vec![],
                                    ..Default::default()
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

        insert_universal_variables_by_count(2, "foo/bar", &mut bag);
        insert_update_string_by_root("foo/bar#id@0", "foo-1", "header", &mut bag);
        insert_update_string_by_root("foo/bar#id@1", "foo-2", "header", &mut bag);

        let mut main = p2::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![ftd::Element::Row(ftd::Row {
                        spacing: None,
                        container: ftd::Container {
                            children: vec![
                                ftd::Element::Row(ftd::Row {
                                    spacing: None,
                                    common: Box::new(ftd::Common {
                                        data_id: Some(s("r2")),
                                        id: Some(s("foo-1:r2")),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                }),
                                ftd::Element::Markup(ftd::Markups {
                                    text: ftd::ftd2021::rendered::markup_line("hello"),
                                    line: true,
                                    ..Default::default()
                                }),
                            ],
                            ..Default::default()
                        },
                        common: Box::new(ftd::Common {
                            data_id: Some(s("r1")),
                            id: Some(s("foo-1:r1")),
                            ..Default::default()
                        },
                    })],
                    ..Default::default()
                },
                common: Box::new(ftd::Common {
                    data_id: Some(s("foo-1")),
                    id: Some(s("foo-1")),
                    ..Default::default()
                },
            }));

        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![ftd::Element::Row(ftd::Row {
                        spacing: None,
                        container: ftd::Container {
                            children: vec![ftd::Element::Row(ftd::Row {
                                spacing: None,
                                common: Box::new(ftd::Common {
                                    data_id: Some(s("r2")),
                                    id: Some(s("foo-2:r2")),
                                    ..Default::default()
                                },
                                ..Default::default()
                            })],
                            ..Default::default()
                        },
                        common: Box::new(ftd::Common {
                            data_id: Some(s("r1")),
                            id: Some(s("foo-2:r1")),
                            ..Default::default()
                        },
                    })],
                    ..Default::default()
                },
                common: Box::new(ftd::Common {
                    data_id: Some(s("foo-2")),
                    id: Some(s("foo-2")),
                    ..Default::default()
                },
            }));

        p!(
            "
            -- import: inner_container as ic

            -- ic.foo:
            id: foo-1

            -- ic.foo:
            id: foo-2

            -- container: foo-1.r1

            -- ftd.text: hello
            ",
            (bag, main),
        );
    }

    #[test]
    fn open_container_with_id() {
        let mut external_children = p2::default_column();
        external_children.container.children = vec![ftd::Element::Markup(ftd::Markups {
            text: ftd::ftd2021::rendered::markup_line("hello"),
            line: true,
            ..Default::default()
        })];

        let mut main = p2::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    external_children: Some((
                        s("some-child"),
                        vec![vec![0, 0]],
                        vec![ftd::Element::Column(external_children)],
                    )),
                    children: vec![ftd::Element::Row(ftd::Row {
                        spacing: None,
                        container: ftd::Container {
                            children: vec![ftd::Element::Row(ftd::Row {
                                spacing: None,
                                common: Box::new(ftd::Common {
                                    data_id: Some(s("some-child")),
                                    ..Default::default()
                                },
                                ..Default::default()
                            })],
                            ..Default::default()
                        },
                        ..Default::default()
                    })],
                    open: Some(true),
                    append_at: Some(s("some-child")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        let mut bag = interpreter::default_bag();

        bag.insert(
            "foo/bar#foo".to_string(),
            ftd::ftd2021::p2::Thing::Component(ftd::Component {
                root: "ftd#column".to_string(),
                full_name: s("foo/bar#foo"),
                arguments: universal_arguments_as_map(),
                properties: std::iter::IntoIterator::into_iter([
                    (
                        s("append-at"),
                        ftd::ftd2021::component::Property {
                            default: Some(ftd::PropertyValue::Value {
                                value: ftd::Value::String {
                                    text: s("some-child"),
                                    source: ftd::TextSource::Header,
                                },
                            }),
                            conditions: vec![],
                            ..Default::default()
                        },
                    ),
                    (
                        s("open"),
                        ftd::ftd2021::component::Property {
                            default: Some(ftd::PropertyValue::Value {
                                value: ftd::Value::Boolean { value: true },
                            }),
                            conditions: vec![],
                            ..Default::default()
                        },
                    ),
                ])
                .collect(),
                instructions: vec![
                    ftd::ftd2021::component::Instruction::ChildComponent {
                        child: ftd::ftd2021::component::ChildComponent {
                            events: vec![],
                            root: "ftd#row".to_string(),
                            condition: None,
                            ..Default::default()
                        },
                    },
                    ftd::ftd2021::component::Instruction::ChildComponent {
                        child: ftd::ftd2021::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "ftd#row".to_string(),
                            condition: None,
                            properties: std::iter::IntoIterator::into_iter([(
                                s("id"),
                                ftd::ftd2021::component::Property {
                                    default: Some(ftd::PropertyValue::Value {
                                        value: ftd::ftd2021::variable::Value::String {
                                            text: "some-child".to_string(),
                                            source: ftd::TextSource::Header,
                                        },
                                    }),
                                    conditions: vec![],
                                    ..Default::default()
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

        insert_universal_variables_by_count(1, "foo/bar", &mut bag);

        p!(
            "
            -- ftd.column foo:
            open: true
            append-at: some-child

            --- ftd.row:

            --- ftd.row:
            id: some-child

            -- foo:

            -- ftd.text: hello
            ",
            (bag, main),
        );
    }

    #[test]
    fn open_container_with_if() {
        let mut external_children = p2::default_column();
        external_children.container.children = vec![
            ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("hello"),
                line: true,
                ..Default::default()
            }),
            ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("hello1"),
                line: true,
                ..Default::default()
            }),
        ];

        let mut main = p2::default_column();
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("Start Browser"),
                line: true,
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![ftd::Element::Column(ftd::Column {
                        spacing: None,
                        container: ftd::Container {
                            children: vec![ftd::Element::Column(ftd::Column {
                                spacing: None,
                                container: ftd::Container {
                                    children: vec![
                                        ftd::Element::Column(ftd::Column {
                                            spacing: None,
                                            container: ftd::Container {
                                                children: vec![ftd::Element::Markup(
                                                    ftd::Markups {
                                                        text: ftd::ftd2021::rendered::markup_line(
                                                            "Mobile Display",
                                                        ),
                                                        line: true,
                                                        common: Box::new(ftd::Common {
                                                            data_id: Some(s("mobile-display")),
                                                            id: Some(s(
                                                                "foo-id:some-child:mobile-display",
                                                            )),
                                                            ..Default::default()
                                                        },
                                                        ..Default::default()
                                                    },
                                                )],
                                                ..Default::default()
                                            },
                                            common: Box::new(ftd::Common {
                                                condition: Some(ftd::Condition {
                                                    variable: s("foo/bar#mobile"),
                                                    value: serde_json::Value::Bool(true),
                                                }),
                                                data_id: Some(s("some-child")),
                                                id: Some(s("foo-id:some-child")),
                                                ..Default::default()
                                            },
                                        }),
                                        ftd::Element::Column(ftd::Column {
                                            spacing: None,
                                            container: ftd::Container {
                                                children: vec![ftd::Element::Markup(
                                                    ftd::Markups {
                                                        text: ftd::ftd2021::rendered::markup_line(
                                                            "Desktop Display",
                                                        ),
                                                        line: true,
                                                        ..Default::default()
                                                    },
                                                )],
                                                ..Default::default()
                                            },
                                            common: Box::new(ftd::Common {
                                                condition: Some(ftd::Condition {
                                                    variable: s("foo/bar#mobile"),
                                                    value: serde_json::Value::Bool(false),
                                                }),
                                                is_not_visible: true,
                                                data_id: Some(s("some-child")),
                                                id: Some(s("foo-id:some-child")),
                                                ..Default::default()
                                            },
                                        }),
                                    ],
                                    external_children: Some((
                                        s("some-child"),
                                        vec![vec![0], vec![1]],
                                        vec![ftd::Element::Column(external_children)],
                                    )),
                                    open: Some(true),
                                    append_at: Some(s("some-child")),
                                    ..Default::default()
                                },
                                common: Box::new(ftd::Common {
                                    id: Some(s("foo-id")),
                                    data_id: Some(s("foo-id")),
                                    ..Default::default()
                                },
                            })],
                            ..Default::default()
                        },
                        common: Box::new(ftd::Common {
                            data_id: Some(s("c2")),
                            id: Some(s("c2")),
                            ..Default::default()
                        },
                    })],
                    ..Default::default()
                },
                common: Box::new(ftd::Common {
                    data_id: Some(s("c1")),
                    id: Some(s("c1")),
                    ..Default::default()
                },
            }));

        let mut bag = interpreter::default_bag();
        bag.insert(
            s("foo/bar#desktop-display"),
            ftd::ftd2021::p2::Thing::Component(ftd::Component {
                root: "ftd#column".to_string(),
                full_name: s("foo/bar#desktop-display"),
                arguments: universal_arguments_as_map(),
                properties: std::iter::IntoIterator::into_iter([(
                    s("id"),
                    ftd::ftd2021::component::Property {
                        default: Some(ftd::PropertyValue::Variable {
                            name: "id".to_string(),
                            kind: ftd::ftd2021::p2::Kind::Optional {
                                kind: Box::new(ftd::ftd2021::p2::Kind::string()),
                                is_reference: false,
                            },
                        }),
                        conditions: vec![],
                        ..Default::default()
                    },
                )])
                .collect(),
                instructions: vec![ftd::ftd2021::component::Instruction::ChildComponent {
                    child: ftd::ftd2021::component::ChildComponent {
                        is_recursive: false,
                        events: vec![],
                        root: "ftd#text".to_string(),
                        condition: None,
                        properties: std::iter::IntoIterator::into_iter([(
                            s("text"),
                            ftd::ftd2021::component::Property {
                                default: Some(ftd::PropertyValue::Value {
                                    value: ftd::ftd2021::variable::Value::String {
                                        text: s("Desktop Display"),
                                        source: ftd::TextSource::Caption,
                                    },
                                }),
                                conditions: vec![],
                                ..Default::default()
                            },
                        )])
                        .collect(),
                        ..Default::default()
                    },
                }],
                ..Default::default()
            }),
        );

        bag.insert(
            s("foo/bar#foo"),
            ftd::ftd2021::p2::Thing::Component(ftd::Component {
                root: "ftd#column".to_string(),
                full_name: s("foo/bar#foo"),
                arguments: universal_arguments_as_map(),
                properties: std::iter::IntoIterator::into_iter([
                    (
                        s("append-at"),
                        ftd::ftd2021::component::Property {
                            default: Some(ftd::PropertyValue::Value {
                                value: ftd::ftd2021::variable::Value::String {
                                    text: s("some-child"),
                                    source: ftd::TextSource::Header,
                                },
                            }),
                            conditions: vec![],
                            ..Default::default()
                        },
                    ),
                    (
                        s("open"),
                        ftd::ftd2021::component::Property {
                            default: Some(ftd::PropertyValue::Value {
                                value: ftd::Value::Boolean { value: true },
                            }),
                            conditions: vec![],
                            ..Default::default()
                        },
                    ),
                ])
                .collect(),
                instructions: vec![
                    ftd::ftd2021::component::Instruction::ChildComponent {
                        child: ftd::ftd2021::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "foo/bar#mobile-display".to_string(),
                            condition: Some(ftd::ftd2021::p2::Boolean::Equal {
                                left: ftd::PropertyValue::Reference {
                                    name: s("foo/bar#mobile"),
                                    kind: ftd::ftd2021::p2::Kind::Boolean {
                                        default: None,
                                        is_reference: false,
                                    },
                                },
                                right: ftd::PropertyValue::Value {
                                    value: ftd::ftd2021::variable::Value::Boolean { value: true },
                                },
                            }),
                            properties: std::iter::IntoIterator::into_iter([(
                                s("id"),
                                ftd::ftd2021::component::Property {
                                    default: Some(ftd::PropertyValue::Value {
                                        value: ftd::ftd2021::variable::Value::String {
                                            text: s("some-child"),
                                            source: ftd::TextSource::Header,
                                        },
                                    }),
                                    conditions: vec![],
                                    ..Default::default()
                                },
                            )])
                            .collect(),
                            ..Default::default()
                        },
                    },
                    ftd::ftd2021::component::Instruction::ChildComponent {
                        child: ftd::ftd2021::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "foo/bar#desktop-display".to_string(),
                            condition: Some(ftd::ftd2021::p2::Boolean::Equal {
                                left: ftd::PropertyValue::Reference {
                                    name: s("foo/bar#mobile"),
                                    kind: ftd::ftd2021::p2::Kind::Boolean {
                                        default: None,
                                        is_reference: false,
                                    },
                                },
                                right: ftd::PropertyValue::Value {
                                    value: ftd::ftd2021::variable::Value::Boolean { value: false },
                                },
                            }),
                            properties: std::iter::IntoIterator::into_iter([(
                                s("id"),
                                ftd::ftd2021::component::Property {
                                    default: Some(ftd::PropertyValue::Value {
                                        value: ftd::ftd2021::variable::Value::String {
                                            text: s("some-child"),
                                            source: ftd::TextSource::Header,
                                        },
                                    }),
                                    conditions: vec![],
                                    ..Default::default()
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

        bag.insert(
            s("foo/bar#mobile"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("mobile"),
                value: ftd::PropertyValue::Value {
                    value: ftd::ftd2021::variable::Value::Boolean { value: true },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            s("foo/bar#mobile-display"),
            ftd::ftd2021::p2::Thing::Component(ftd::Component {
                root: "ftd#column".to_string(),
                full_name: s("foo/bar#mobile-display"),
                arguments: universal_arguments_as_map(),
                properties: std::iter::IntoIterator::into_iter([(
                    s("id"),
                    ftd::ftd2021::component::Property {
                        default: Some(ftd::PropertyValue::Variable {
                            name: "id".to_string(),
                            kind: ftd::ftd2021::p2::Kind::Optional {
                                kind: Box::new(ftd::ftd2021::p2::Kind::string()),
                                is_reference: false,
                            },
                        }),
                        conditions: vec![],
                        ..Default::default()
                    },
                )])
                .collect(),
                instructions: vec![ftd::ftd2021::component::Instruction::ChildComponent {
                    child: ftd::ftd2021::component::ChildComponent {
                        is_recursive: false,
                        events: vec![],
                        root: "ftd#text".to_string(),
                        condition: None,
                        properties: std::iter::IntoIterator::into_iter([
                            (
                                s("id"),
                                ftd::ftd2021::component::Property {
                                    default: Some(ftd::PropertyValue::Value {
                                        value: ftd::ftd2021::variable::Value::String {
                                            text: s("mobile-display"),
                                            source: ftd::TextSource::Header,
                                        },
                                    }),
                                    conditions: vec![],
                                    ..Default::default()
                                },
                            ),
                            (
                                s("text"),
                                ftd::ftd2021::component::Property {
                                    default: Some(ftd::PropertyValue::Value {
                                        value: ftd::ftd2021::variable::Value::String {
                                            text: s("Mobile Display"),
                                            source: ftd::TextSource::Caption,
                                        },
                                    }),
                                    conditions: vec![],
                                    ..Default::default()
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

        bag.insert(
            s("foo/bar#id@1,0,0,0"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("id"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Optional {
                        data: Box::new(Some(ftd::Value::String {
                            text: s("some-child"),
                            source: ftd::TextSource::Header,
                        })),
                        kind: ftd::ftd2021::p2::Kind::string(),
                    },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            s("foo/bar#id@1,0,0,1"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("id"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Optional {
                        data: Box::new(Some(ftd::Value::String {
                            text: s("some-child"),
                            source: ftd::TextSource::Header,
                        })),
                        kind: ftd::ftd2021::p2::Kind::string(),
                    },
                },
                conditions: vec![],
            }),
        );

        let levels = vec![s("1,0,0"), s("1,0,0,0"), s("1,0,0,1")];
        insert_universal_variables_by_levels(levels, "foo/bar", &mut bag);
        insert_update_string_by_root("foo/bar#id@1,0,0", "foo-id", "header", &mut bag);
        insert_update_string_by_root("foo/bar#id@1,0,0,0", "some-child", "header", &mut bag);
        insert_update_string_by_root("foo/bar#id@1,0,0,1", "some-child", "header", &mut bag);

        p!(
            "
            -- ftd.column mobile-display:
            ;; id is now univeral no need to declare it again
            /optional string id:
            id: $id

            --- ftd.text: Mobile Display
            id: mobile-display

            -- ftd.column desktop-display:
            ;; redundant declaration of id
            /optional string id:
            id: $id

            --- ftd.text: Desktop Display

            -- boolean mobile: true

            -- ftd.column foo:
            open: true
            append-at: some-child

            --- mobile-display:
            if: $mobile
            id: some-child

            --- desktop-display:
            if: not $mobile
            id: some-child

            -- ftd.text: Start Browser

            -- ftd.column:
            id: c1

            -- ftd.column:
            id: c2

            -- foo:
            id: foo-id

            -- ftd.text: hello

            -- ftd.text: hello1
            ",
            (bag, main),
        );
    }

    #[test]
    fn nested_open_container() {
        let mut external_children = p2::default_column();
        external_children.container.children = vec![
            ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("hello"),
                line: true,
                ..Default::default()
            }),
            ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("hello again"),
                line: true,
                ..Default::default()
            }),
        ];

        let mut main = p2::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![ftd::Element::Column(ftd::Column {
                        spacing: None,
                        container: ftd::Container {
                            children: vec![
                                ftd::Element::Column(ftd::Column {
                                    spacing: None,
                                    container: ftd::Container {
                                        children: vec![ftd::Element::Column(ftd::Column {
                                            spacing: None,
                                            container: ftd::Container {
                                                children: vec![],
                                                ..Default::default()
                                            },
                                            common: Box::new(ftd::Common {
                                                data_id: Some(s("desktop-container")),
                                                ..Default::default()
                                            },
                                        })],
                                        external_children: Some((
                                            s("desktop-container"),
                                            vec![vec![0]],
                                            vec![],
                                        )),
                                        open: Some(true),
                                        append_at: Some(s("desktop-container")),
                                        ..Default::default()
                                    },
                                    common: Box::new(ftd::Common {
                                        condition: Some(ftd::Condition {
                                            variable: s("foo/bar#is-mobile"),
                                            value: serde_json::Value::Bool(false),
                                        }),
                                        is_not_visible: true,
                                        data_id: Some(s("main-container")),
                                        ..Default::default()
                                    },
                                }),
                                ftd::Element::Column(ftd::Column {
                                    spacing: None,
                                    container: ftd::Container {
                                        children: vec![ftd::Element::Column(ftd::Column {
                                            spacing: None,
                                            common: Box::new(ftd::Common {
                                                data_id: Some(s("mobile-container")),
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        })],
                                        external_children: Some((
                                            s("mobile-container"),
                                            vec![vec![0]],
                                            vec![],
                                        )),
                                        open: Some(true),
                                        append_at: Some(s("mobile-container")),
                                        ..Default::default()
                                    },
                                    common: Box::new(ftd::Common {
                                        condition: Some(ftd::Condition {
                                            variable: s("foo/bar#is-mobile"),
                                            value: serde_json::Value::Bool(true),
                                        }),
                                        data_id: Some(s("main-container")),
                                        ..Default::default()
                                    },
                                }),
                            ],
                            ..Default::default()
                        },
                        common: Box::new(ftd::Common {
                            data_id: Some(s("start")),
                            ..Default::default()
                        },
                    })],
                    external_children: Some((
                        s("main-container"),
                        vec![vec![0, 0], vec![0, 1]],
                        vec![ftd::Element::Column(external_children)],
                    )),
                    open: Some(true),
                    append_at: Some(s("main-container")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (_g_bag, g_col) = ftd::ftd2021::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- ftd.column desktop:
                open: true
                append-at: desktop-container

                --- ftd.column:
                id: desktop-container

                -- ftd.column mobile:
                open: true
                append-at: mobile-container

                --- ftd.column:
                id: mobile-container

                -- boolean is-mobile: true

                -- ftd.column page:
                open: true
                append-at: main-container

                --- ftd.column:
                id: start

                --- desktop:
                if: not $is-mobile
                id: main-container

                --- container: start

                --- mobile:
                if: $is-mobile
                id: main-container

                -- page:

                -- ftd.text: hello

                -- ftd.text: hello again
                "
            ),
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn deep_open_container_call() {
        let mut external_children = p2::default_column();
        external_children.container.children = vec![
            ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("hello"),
                line: true,
                ..Default::default()
            }),
            ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("hello again"),
                line: true,
                ..Default::default()
            }),
        ];

        let mut main = p2::default_column();

        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Column(ftd::Column {
                            spacing: None,
                            container: ftd::Container {
                                children: vec![ftd::Element::Column(ftd::Column {
                                    spacing: None,
                                    common: Box::new(ftd::Common {
                                        data_id: Some(s("foo")),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                })],
                                ..Default::default()
                            },
                            common: Box::new(ftd::Common {
                                condition: Some(ftd::Condition {
                                    variable: s("foo/bar#is-mobile"),
                                    value: serde_json::Value::Bool(false),
                                }),
                                is_not_visible: true,
                                data_id: Some(s("main-container")),
                                ..Default::default()
                            },
                        }),
                        ftd::Element::Column(ftd::Column {
                            spacing: None,
                            container: ftd::Container {
                                children: vec![ftd::Element::Column(ftd::Column {
                                    spacing: None,
                                    common: Box::new(ftd::Common {
                                        data_id: Some(s("foo")),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                })],
                                ..Default::default()
                            },
                            common: Box::new(ftd::Common {
                                condition: Some(ftd::Condition {
                                    variable: s("foo/bar#is-mobile"),
                                    value: serde_json::Value::Bool(true),
                                }),
                                data_id: Some(s("main-container")),
                                ..Default::default()
                            },
                        }),
                    ],
                    external_children: Some((
                        s("foo"),
                        vec![vec![0, 0], vec![1, 0]],
                        vec![ftd::Element::Column(external_children)],
                    )),
                    open: Some(true),
                    append_at: Some(s("main-container.foo")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (_g_bag, g_col) = ftd::ftd2021::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                ;; id is universal argument now no need to declare it
                -- ftd.column desktop:
                /optional string id:
                id: $id

                --- ftd.column:
                id: foo

                -- ftd.column mobile:
                /optional string id:
                id: $id

                --- ftd.column:
                id: foo

                -- boolean is-mobile: true

                -- ftd.column page:
                open: true
                append-at: main-container.foo

                --- desktop:
                if: not $is-mobile
                id: main-container

                --- mobile:
                if: $is-mobile
                id: main-container

                -- page:

                -- ftd.text: hello

                -- ftd.text: hello again
                "
            ),
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn deep_nested_open_container_call() {
        let mut nested_external_children = p2::default_column();
        nested_external_children.container.children = vec![
            ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("hello"),
                line: true,
                ..Default::default()
            }),
            ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("hello again"),
                line: true,
                ..Default::default()
            }),
        ];

        let mut external_children = p2::default_column();
        external_children.container.children = vec![ftd::Element::Column(ftd::Column {
            spacing: None,
            container: ftd::Container {
                children: vec![ftd::Element::Row(ftd::Row {
                    spacing: None,
                    container: ftd::Container {
                        children: vec![ftd::Element::Column(ftd::Column {
                            spacing: None,
                            common: Box::new(ftd::Common {
                                data_id: Some(s("foo")),
                                ..Default::default()
                            },
                            ..Default::default()
                        })],
                        ..Default::default()
                    },
                    common: Box::new(ftd::Common {
                        data_id: Some(s("desktop-container")),
                        ..Default::default()
                    },
                })],
                external_children: Some((
                    s("desktop-container"),
                    vec![vec![0]],
                    vec![ftd::Element::Column(nested_external_children)],
                )),
                open: Some(true),
                append_at: Some(s("desktop-container")),
                ..Default::default()
            },
            ..Default::default()
        })];

        let mut main = p2::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Column(ftd::Column {
                            spacing: None,
                            container: ftd::Container {
                                children: vec![ftd::Element::Row(ftd::Row {
                                    spacing: None,
                                    container: ftd::Container {
                                        children: vec![ftd::Element::Column(ftd::Column {
                                            spacing: None,
                                            common: Box::new(ftd::Common {
                                                data_id: Some(s("foo")),
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        })],
                                        ..Default::default()
                                    },
                                    common: Box::new(ftd::Common {
                                        data_id: Some(s("desktop-container")),
                                        ..Default::default()
                                    },
                                })],
                                external_children: Some((
                                    s("desktop-container"),
                                    vec![vec![0]],
                                    vec![],
                                )),
                                open: Some(true),
                                append_at: Some(s("desktop-container")),
                                ..Default::default()
                            },
                            common: Box::new(ftd::Common {
                                condition: Some(ftd::Condition {
                                    variable: s("foo/bar#is-mobile"),
                                    value: serde_json::Value::Bool(false),
                                }),
                                data_id: Some(s("main-container")),
                                ..Default::default()
                            },
                        }),
                        ftd::Element::Column(ftd::Column {
                            spacing: None,
                            container: ftd::Container {
                                children: vec![ftd::Element::Row(ftd::Row {
                                    spacing: None,
                                    container: ftd::Container {
                                        children: vec![ftd::Element::Column(ftd::Column {
                                            spacing: None,
                                            common: Box::new(ftd::Common {
                                                data_id: Some(s("foo")),
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        })],
                                        ..Default::default()
                                    },
                                    common: Box::new(ftd::Common {
                                        data_id: Some(s("mobile-container")),
                                        ..Default::default()
                                    },
                                })],
                                external_children: Some((
                                    s("mobile-container"),
                                    vec![vec![0]],
                                    vec![],
                                )),
                                open: Some(true),
                                append_at: Some(s("mobile-container")),
                                ..Default::default()
                            },
                            common: Box::new(ftd::Common {
                                condition: Some(ftd::Condition {
                                    variable: s("foo/bar#is-mobile"),
                                    value: serde_json::Value::Bool(true),
                                }),
                                is_not_visible: true,
                                data_id: Some(s("main-container")),
                                ..Default::default()
                            },
                        }),
                    ],
                    external_children: Some((
                        s("foo"),
                        vec![vec![0, 0, 0], vec![1, 0, 0]],
                        vec![ftd::Element::Column(external_children)],
                    )),
                    open: Some(true),
                    append_at: Some(s("main-container.foo")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (_g_bag, g_col) = ftd::ftd2021::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- ftd.column ft_container:
                /optional string id:
                id: $id

                -- ftd.column ft_container_mobile:
                /optional string id:
                id: $id


                -- ftd.column desktop:
                open: true
                append-at: desktop-container
                /optional string id:
                id: $id

                --- ftd.row:
                id: desktop-container

                --- ft_container:
                id: foo



                -- ftd.column mobile:
                open: true
                append-at: mobile-container
                /optional string id:
                id: $id

                --- ftd.row:
                id: mobile-container

                --- ft_container_mobile:
                id: foo


                -- boolean is-mobile: false


                -- ftd.column page:
                open: true
                append-at: main-container.foo

                --- desktop:
                if: not $is-mobile
                id: main-container

                --- container: ftd.main

                --- mobile:
                if: $is-mobile
                id: main-container



                -- page:

                -- desktop:

                -- ftd.text: hello

                -- ftd.text: hello again

                "
            ),
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    #[ignore]
    fn invalid_deep_open_container() {
        let mut external_children = p2::default_column();
        external_children.container.children = vec![
            ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("hello"),
                line: true,
                ..Default::default()
            }),
            ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("hello again"),
                line: true,
                ..Default::default()
            }),
        ];

        let mut main = p2::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![ftd::Element::Column(ftd::Column {
                        spacing: None,
                        container: ftd::Container {
                            children: vec![
                                ftd::Element::Column(ftd::Column {
                                    spacing: None,
                                    container: ftd::Container {
                                        children: vec![ftd::Element::Column(ftd::Column {
                                            spacing: None,
                                            container: ftd::Container {
                                                children: vec![],
                                                ..Default::default()
                                            },
                                            common: Box::new(ftd::Common {
                                                data_id: Some(s("main-container")),
                                                ..Default::default()
                                            },
                                        })],
                                        ..Default::default()
                                    },
                                    common: Box::new(ftd::Common {
                                        condition: Some(ftd::Condition {
                                            variable: s("foo/bar#is-mobile"),
                                            value: serde_json::Value::Bool(false),
                                        }),
                                        is_not_visible: true,
                                        ..Default::default()
                                    },
                                }),
                                ftd::Element::Column(ftd::Column {
                                    spacing: None,
                                    container: ftd::Container {
                                        children: vec![ftd::Element::Column(ftd::Column {
                                            spacing: None,
                                            common: Box::new(ftd::Common {
                                                data_id: Some(s("main-container")),
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        })],
                                        ..Default::default()
                                    },
                                    common: Box::new(ftd::Common {
                                        condition: Some(ftd::Condition {
                                            variable: s("foo/bar#is-mobile"),
                                            value: serde_json::Value::Bool(true),
                                        }),
                                        ..Default::default()
                                    },
                                }),
                            ],
                            ..Default::default()
                        },
                        common: Box::new(ftd::Common {
                            data_id: Some(s("start")),
                            ..Default::default()
                        },
                    })],
                    external_children: Some((
                        s("main-container"),
                        vec![],
                        vec![ftd::Element::Column(external_children)],
                    )),
                    open: Some(true),
                    append_at: Some(s("main-container")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (_g_bag, g_col) = ftd::ftd2021::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- ftd.column desktop:
                optional string id:
                id: $id

                --- ftd.column:
                id: main-container

                -- ftd.column mobile:
                optional string id:
                id: $id

                --- ftd.column:
                id: main-container

                -- boolean is-mobile: true

                -- ftd.column page:
                open: true
                append-at: main-container

                --- ftd.column:
                id: start

                --- desktop:
                if: not $is-mobile

                --- container: start

                --- mobile:
                if: $is-mobile

                -- page:

                -- ftd.text: hello

                -- ftd.text: hello again
                "
            ),
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn open_container_id_1() {
        let mut main = self::default_column();
        main.container.children.push(ftd::Element::Row(ftd::Row {
            spacing: None,
            common: Box::new(ftd::Common {
                data_id: Some(s("r1")),
                id: Some(s("r1")),
                ..Default::default()
            },
            container: ftd::Container {
                open: Some(false),
                ..Default::default()
            },
        }));
        main.container.children.push(ftd::Element::Row(ftd::Row {
            spacing: None,
            container: ftd::Container {
                external_children: Default::default(),
                children: vec![ftd::Element::Markup(ftd::Markups {
                    text: ftd::ftd2021::rendered::markup_line("hello"),
                    line: true,
                    ..Default::default()
                })],
                open: Some(true),
                ..Default::default()
            },
            common: Box::new(ftd::Common {
                data_id: Some(s("r2")),
                id: Some(s("r2")),
                ..Default::default()
            },
        }));

        main.container.children.push(ftd::Element::Row(ftd::Row {
            spacing: None,
            container: ftd::Container {
                open: Some(false),
                ..Default::default()
            },
            common: Box::new(ftd::Common {
                data_id: Some(s("r3")),
                id: Some(s("r3")),
                ..Default::default()
            },
        }));

        let mut bag = interpreter::default_bag();

        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@0", -1, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@1", 0, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@2", -1, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@1,0", -1, &mut bag);

        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT@0", 0, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT@1", 1, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT@2", 0, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT@1,0", 0, &mut bag);

        insert_update_integer_by_root("foo/bar#SIBLING-INDEX-0@0", 0, &mut bag);
        insert_update_integer_by_root("foo/bar#SIBLING-INDEX-0@1", 1, &mut bag);
        insert_update_integer_by_root("foo/bar#SIBLING-INDEX-0@2", 2, &mut bag);

        insert_update_integer_by_root("foo/bar#SIBLING-INDEX@0", 1, &mut bag);
        insert_update_integer_by_root("foo/bar#SIBLING-INDEX@1", 2, &mut bag);
        insert_update_integer_by_root("foo/bar#SIBLING-INDEX@2", 3, &mut bag);

        let (g_bag, g_col) = ftd::ftd2021::test::interpret(
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
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .expect("found error");
        pretty_assertions::assert_eq!(g_bag, bag);
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn submit() {
        let mut main = p2::default_column();

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("hello"),
                line: true,
                common: Box::new(ftd::Common {
                    submit: Some("https://httpbin.org/post?x=10".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (g_bag, g_col) = ftd::ftd2021::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- ftd.text: hello
                submit: https://httpbin.org/post?x=10
                "
            ),
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .expect("found error");

        let mut bag = interpreter::default_bag();

        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@0", -1, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT@0", 0, &mut bag);
        insert_update_integer_by_root("foo/bar#SIBLING-INDEX-0@0", 0, &mut bag);
        insert_update_integer_by_root("foo/bar#SIBLING-INDEX@0", 1, &mut bag);

        pretty_assertions::assert_eq!(g_bag, bag);
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn basic_loop_on_record_1() {
        let mut main = p2::default_column();
        main.container.children.push(ftd::Element::Row(ftd::Row {
            spacing: None,
            container: ftd::Container {
                children: vec![
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::ftd2021::rendered::markup_line("hello"),
                        line: true,
                        common: Box::new(ftd::Common {
                            reference: Some(s("foo/bar#name@0")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::ftd2021::rendered::markup_line("world"),
                        line: true,
                        common: Box::new(ftd::Common {
                            reference: Some(s("foo/bar#body@0")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                ],
                ..Default::default()
            },
            ..Default::default()
        }));

        main.container.children.push(ftd::Element::Row(ftd::Row {
            spacing: None,
            container: ftd::Container {
                children: vec![
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::ftd2021::rendered::markup_line("Arpita Jaiswal"),
                        line: true,
                        common: Box::new(ftd::Common {
                            reference: Some(s("foo/bar#name@1")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::ftd2021::rendered::markup_line(
                            "Arpita is developer at Fifthtry",
                        ),
                        line: true,
                        common: Box::new(ftd::Common {
                            reference: Some(s("foo/bar#body@1")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                ],
                ..Default::default()
            },
            common: Box::new(ftd::Common {
                reference: Some(s("foo/bar#people")),
                ..Default::default()
            },
        }));

        main.container.children.push(ftd::Element::Row(ftd::Row {
            spacing: None,
            container: ftd::Container {
                children: vec![
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::ftd2021::rendered::markup_line("Amit Upadhyay"),
                        line: true,
                        common: Box::new(ftd::Common {
                            reference: Some(s("foo/bar#name@2")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::ftd2021::rendered::markup_line("Amit is CEO of FifthTry."),
                        line: true,
                        common: Box::new(ftd::Common {
                            reference: Some(s("foo/bar#body@2")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                ],
                ..Default::default()
            },
            common: Box::new(ftd::Common {
                reference: Some(s("foo/bar#people")),
                ..Default::default()
            },
        }));

        let mut bag = interpreter::default_bag();

        bag.insert(
            "foo/bar#foo".to_string(),
            ftd::ftd2021::p2::Thing::Component(ftd::Component {
                root: "ftd#row".to_string(),
                full_name: s("foo/bar#foo"),
                arguments: [
                    vec![
                        (s("body"), ftd::ftd2021::p2::Kind::string()),
                        (s("name"), ftd::ftd2021::p2::Kind::caption()),
                    ],
                    universal_arguments_as_vec(),
                ]
                .concat()
                .into_iter()
                .collect(),
                instructions: vec![
                    ftd::ftd2021::component::Instruction::ChildComponent {
                        child: ftd::ftd2021::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "ftd#text".to_string(),
                            condition: None,
                            properties: std::iter::IntoIterator::into_iter([(
                                s("text"),
                                ftd::ftd2021::component::Property {
                                    default: Some(ftd::PropertyValue::Variable {
                                        name: "name".to_string(),
                                        kind: ftd::ftd2021::p2::Kind::caption_or_body(),
                                    }),
                                    conditions: vec![],
                                    ..Default::default()
                                },
                            )])
                            .collect(),
                            ..Default::default()
                        },
                    },
                    ftd::ftd2021::component::Instruction::ChildComponent {
                        child: ftd::ftd2021::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "ftd#text".to_string(),
                            condition: None,
                            properties: std::iter::IntoIterator::into_iter([(
                                s("text"),
                                ftd::ftd2021::component::Property {
                                    default: Some(ftd::PropertyValue::Variable {
                                        name: "body".to_string(),
                                        kind: ftd::ftd2021::p2::Kind::caption_or_body(),
                                    }),
                                    conditions: vec![],
                                    ..Default::default()
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

        bag.insert(
            "foo/bar#get".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "get".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: "world".to_string(),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            "foo/bar#name".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "name".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: "Arpita Jaiswal".to_string(),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            "foo/bar#people".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "foo/bar#people".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::List {
                        data: vec![
                            ftd::PropertyValue::Value {
                                value: ftd::Value::Record {
                                    name: "foo/bar#person".to_string(),
                                    fields: std::iter::IntoIterator::into_iter([
                                        (
                                            s("bio"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::Value::String {
                                                    text: "Arpita is developer at Fifthtry"
                                                        .to_string(),
                                                    source: ftd::TextSource::Body,
                                                },
                                            },
                                        ),
                                        (
                                            s("name"),
                                            ftd::PropertyValue::Reference {
                                                name: "foo/bar#name".to_string(),
                                                kind: ftd::ftd2021::p2::Kind::caption(),
                                            },
                                        ),
                                    ])
                                    .collect(),
                                },
                            },
                            ftd::PropertyValue::Value {
                                value: ftd::Value::Record {
                                    name: "foo/bar#person".to_string(),
                                    fields: std::iter::IntoIterator::into_iter([
                                        (
                                            s("bio"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::Value::String {
                                                    text: "Amit is CEO of FifthTry.".to_string(),
                                                    source: ftd::TextSource::Body,
                                                },
                                            },
                                        ),
                                        (
                                            s("name"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::Value::String {
                                                    text: "Amit Upadhyay".to_string(),
                                                    source: ftd::TextSource::Caption,
                                                },
                                            },
                                        ),
                                    ])
                                    .collect(),
                                },
                            },
                        ],
                        kind: ftd::ftd2021::p2::Kind::Record {
                            name: "foo/bar#person".to_string(),
                            default: None,
                            is_reference: false,
                        },
                    },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            "foo/bar#$loop$@1".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "$loop$".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Record {
                        name: "foo/bar#person".to_string(),
                        fields: std::iter::IntoIterator::into_iter([
                            (
                                s("bio"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: "Arpita is developer at Fifthtry".to_string(),
                                        source: ftd::TextSource::Body,
                                    },
                                },
                            ),
                            (
                                s("name"),
                                ftd::PropertyValue::Reference {
                                    name: "foo/bar#name".to_string(),
                                    kind: ftd::ftd2021::p2::Kind::caption(),
                                },
                            ),
                        ])
                        .collect(),
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#$loop$@2".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "$loop$".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Record {
                        name: "foo/bar#person".to_string(),
                        fields: std::iter::IntoIterator::into_iter([
                            (
                                s("bio"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: "Amit is CEO of FifthTry.".to_string(),
                                        source: ftd::TextSource::Body,
                                    },
                                },
                            ),
                            (
                                s("name"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: "Amit Upadhyay".to_string(),
                                        source: ftd::TextSource::Caption,
                                    },
                                },
                            ),
                        ])
                        .collect(),
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#body@0".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "body".to_string(),
                value: ftd::PropertyValue::Reference {
                    name: s("foo/bar#get"),
                    kind: ftd::ftd2021::p2::Kind::string(),
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#body@1".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "body".to_string(),
                value: ftd::PropertyValue::Variable {
                    name: s("foo/bar#$loop$@1.bio"),
                    kind: ftd::ftd2021::p2::Kind::body(),
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#body@2".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "body".to_string(),
                value: ftd::PropertyValue::Variable {
                    name: s("foo/bar#$loop$@2.bio"),
                    kind: ftd::ftd2021::p2::Kind::body(),
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#name@0".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "name".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("hello"),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#name@1".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "name".to_string(),
                value: ftd::PropertyValue::Variable {
                    name: s("foo/bar#$loop$@1.name"),
                    kind: ftd::ftd2021::p2::Kind::caption(),
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#name@2".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "name".to_string(),
                value: ftd::PropertyValue::Variable {
                    name: s("foo/bar#$loop$@2.name"),
                    kind: ftd::ftd2021::p2::Kind::caption(),
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            "foo/bar#person".to_string(),
            ftd::ftd2021::p2::Thing::Record(ftd::ftd2021::p2::Record {
                name: "foo/bar#person".to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    (s("bio"), ftd::ftd2021::p2::Kind::body()),
                    (s("name"), ftd::ftd2021::p2::Kind::caption()),
                ])
                .collect(),
                instances: Default::default(),
                order: vec![s("name"), s("bio")],
            }),
        );

        insert_universal_variables_by_count(3, "foo/bar", &mut bag);

        p!(
            "
            -- ftd.row foo:
            caption name:
            string body:

            --- ftd.text: $name

            --- ftd.text: $body

            -- record person:
            caption name:
            body bio:

            -- person list people:

            -- string name: Arpita Jaiswal

            -- people: $name

            Arpita is developer at Fifthtry

            -- people: Amit Upadhyay

            Amit is CEO of FifthTry.

            -- string get: world

            -- foo: hello
            body: $get

            -- foo: $obj.name
            $loop$: $people as $obj
            body: $obj.bio
            ",
            (bag, main),
        );
    }

    #[test]
    fn basic_loop_on_record_with_if_condition() {
        let mut main = p2::default_column();
        main.container.children.push(ftd::Element::Row(ftd::Row {
            spacing: None,
            container: ftd::Container {
                children: vec![
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::ftd2021::rendered::markup_line("Arpita Jaiswal"),
                        line: true,
                        common: Box::new(ftd::Common {
                            reference: Some(s("foo/bar#name@0")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::ftd2021::rendered::markup_line(
                            "Arpita is developer at Fifthtry",
                        ),
                        line: true,
                        common: Box::new(ftd::Common {
                            reference: Some(s("foo/bar#body@0")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                ],
                ..Default::default()
            },
            common: Box::new(ftd::Common {
                reference: Some(s("foo/bar#people")),
                condition: Some(ftd::Condition {
                    variable: s("foo/bar#$loop$@0.ceo"),
                    value: serde_json::Value::Bool(true),
                }),
                is_not_visible: true,
                ..Default::default()
            },
        }));

        main.container.children.push(ftd::Element::Row(ftd::Row {
            spacing: None,
            container: ftd::Container {
                children: vec![
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::ftd2021::rendered::markup_line("Amit Upadhyay"),
                        line: true,
                        common: Box::new(ftd::Common {
                            reference: Some(s("foo/bar#name@1")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::ftd2021::rendered::markup_line("Amit is CEO of FifthTry."),
                        line: true,
                        common: Box::new(ftd::Common {
                            reference: Some(s("foo/bar#body@1")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                ],
                ..Default::default()
            },
            common: Box::new(ftd::Common {
                condition: Some(ftd::Condition {
                    variable: s("foo/bar#$loop$@1.ceo"),
                    value: serde_json::Value::Bool(true),
                }),
                reference: Some(s("foo/bar#people")),
                ..Default::default()
            },
        }));

        let mut bag = interpreter::default_bag();

        bag.insert(
            "foo/bar#foo".to_string(),
            ftd::ftd2021::p2::Thing::Component(ftd::Component {
                root: "ftd#row".to_string(),
                full_name: s("foo/bar#foo"),
                arguments: [
                    vec![
                        (s("body"), ftd::ftd2021::p2::Kind::string()),
                        (s("name"), ftd::ftd2021::p2::Kind::caption()),
                    ],
                    universal_arguments_as_vec(),
                ]
                .concat()
                .into_iter()
                .collect(),
                instructions: vec![
                    ftd::ftd2021::component::Instruction::ChildComponent {
                        child: ftd::ftd2021::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "ftd#text".to_string(),
                            condition: None,
                            properties: std::iter::IntoIterator::into_iter([(
                                s("text"),
                                ftd::ftd2021::component::Property {
                                    default: Some(ftd::PropertyValue::Variable {
                                        name: "name".to_string(),
                                        kind: ftd::ftd2021::p2::Kind::caption_or_body(),
                                    }),
                                    conditions: vec![],
                                    ..Default::default()
                                },
                            )])
                            .collect(),
                            ..Default::default()
                        },
                    },
                    ftd::ftd2021::component::Instruction::ChildComponent {
                        child: ftd::ftd2021::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "ftd#text".to_string(),
                            condition: None,
                            properties: std::iter::IntoIterator::into_iter([(
                                s("text"),
                                ftd::ftd2021::component::Property {
                                    default: Some(ftd::PropertyValue::Variable {
                                        name: "body".to_string(),
                                        kind: ftd::ftd2021::p2::Kind::caption_or_body(),
                                    }),
                                    conditions: vec![],
                                    ..Default::default()
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

        bag.insert(
            "foo/bar#people".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "foo/bar#people".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::List {
                        data: vec![
                            ftd::PropertyValue::Value {
                                value: ftd::Value::Record {
                                    name: "foo/bar#person".to_string(),
                                    fields: std::iter::IntoIterator::into_iter([
                                        (
                                            s("bio"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::Value::String {
                                                    text: "Arpita is developer at Fifthtry"
                                                        .to_string(),
                                                    source: ftd::TextSource::Body,
                                                },
                                            },
                                        ),
                                        (
                                            s("ceo"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::Value::Boolean { value: false },
                                            },
                                        ),
                                        (
                                            s("name"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::Value::String {
                                                    text: "Arpita Jaiswal".to_string(),
                                                    source: ftd::TextSource::Caption,
                                                },
                                            },
                                        ),
                                    ])
                                    .collect(),
                                },
                            },
                            ftd::PropertyValue::Value {
                                value: ftd::Value::Record {
                                    name: "foo/bar#person".to_string(),
                                    fields: std::iter::IntoIterator::into_iter([
                                        (
                                            s("bio"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::Value::String {
                                                    text: "Amit is CEO of FifthTry.".to_string(),
                                                    source: ftd::TextSource::Body,
                                                },
                                            },
                                        ),
                                        (
                                            s("ceo"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::Value::Boolean { value: true },
                                            },
                                        ),
                                        (
                                            s("name"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::Value::String {
                                                    text: "Amit Upadhyay".to_string(),
                                                    source: ftd::TextSource::Caption,
                                                },
                                            },
                                        ),
                                    ])
                                    .collect(),
                                },
                            },
                        ],
                        kind: ftd::ftd2021::p2::Kind::Record {
                            name: "foo/bar#person".to_string(),
                            default: None,
                            is_reference: false,
                        },
                    },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            "foo/bar#person".to_string(),
            ftd::ftd2021::p2::Thing::Record(ftd::ftd2021::p2::Record {
                name: "foo/bar#person".to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    (s("bio"), ftd::ftd2021::p2::Kind::body()),
                    (s("name"), ftd::ftd2021::p2::Kind::caption()),
                    (s("ceo"), ftd::ftd2021::p2::Kind::boolean()),
                ])
                .collect(),
                instances: Default::default(),
                order: vec![s("name"), s("bio"), s("ceo")],
            }),
        );

        bag.insert(
            s("foo/bar#$loop$@0"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: "$loop$".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Record {
                        name: "foo/bar#person".to_string(),
                        fields: std::iter::IntoIterator::into_iter([
                            (
                                s("bio"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: "Arpita is developer at Fifthtry".to_string(),
                                        source: ftd::TextSource::Body,
                                    },
                                },
                            ),
                            (
                                s("ceo"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::Boolean { value: false },
                                },
                            ),
                            (
                                s("name"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: "Arpita Jaiswal".to_string(),
                                        source: ftd::TextSource::Caption,
                                    },
                                },
                            ),
                        ])
                        .collect(),
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );

        bag.insert(
            s("foo/bar#$loop$@1"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: "$loop$".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Record {
                        name: "foo/bar#person".to_string(),
                        fields: std::iter::IntoIterator::into_iter([
                            (
                                s("bio"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: "Amit is CEO of FifthTry.".to_string(),
                                        source: ftd::TextSource::Body,
                                    },
                                },
                            ),
                            (
                                s("ceo"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::Boolean { value: true },
                                },
                            ),
                            (
                                s("name"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: "Amit Upadhyay".to_string(),
                                        source: ftd::TextSource::Caption,
                                    },
                                },
                            ),
                        ])
                        .collect(),
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );

        bag.insert(
            s("foo/bar#body@0"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: "body".to_string(),
                value: ftd::PropertyValue::Variable {
                    name: "foo/bar#$loop$@0.bio".to_string(),
                    kind: ftd::ftd2021::p2::Kind::body(),
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );

        bag.insert(
            s("foo/bar#body@1"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: "body".to_string(),
                value: ftd::PropertyValue::Variable {
                    name: "foo/bar#$loop$@1.bio".to_string(),
                    kind: ftd::ftd2021::p2::Kind::body(),
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );

        bag.insert(
            s("foo/bar#name@0"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: "name".to_string(),
                value: ftd::PropertyValue::Variable {
                    name: "foo/bar#$loop$@0.name".to_string(),
                    kind: ftd::ftd2021::p2::Kind::caption(),
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );

        bag.insert(
            s("foo/bar#name@1"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: "name".to_string(),
                value: ftd::PropertyValue::Variable {
                    name: "foo/bar#$loop$@1.name".to_string(),
                    kind: ftd::ftd2021::p2::Kind::caption(),
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );

        insert_universal_variables_by_count(2, "foo/bar", &mut bag);

        p!(
            "
            -- ftd.row foo:
            caption name:
            string body:

            --- ftd.text: $name

            --- ftd.text: $body

            -- record person:
            caption name:
            body bio:
            boolean ceo:

            -- person list people:

            -- people: Arpita Jaiswal
            ceo: false

            Arpita is developer at Fifthtry

            -- people: Amit Upadhyay
            ceo: true

            Amit is CEO of FifthTry.

            -- foo: $obj.name
            $loop$: $people as $obj
            if: $obj.ceo
            body: $obj.bio
            ",
            (bag, main),
        );
    }

    #[test]
    fn basic_loop_on_string() {
        let mut main = p2::default_column();
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("Arpita"),
                line: true,
                common: Box::new(ftd::Common {
                    reference: Some(s("foo/bar#people")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("Asit"),
                line: true,
                common: Box::new(ftd::Common {
                    reference: Some(s("foo/bar#people")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("Sourabh"),
                line: true,
                common: Box::new(ftd::Common {
                    reference: Some(s("foo/bar#people")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("$loop$"),
                line: true,
                common: Box::new(ftd::Common {
                    reference: Some(s("foo/bar#people")),
                    is_dummy: true,
                    ..Default::default()
                },
                ..Default::default()
            }));

        let mut bag = interpreter::default_bag();

        bag.insert(
            "foo/bar#$loop$@0".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "$loop$".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("Arpita"),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#$loop$@1".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "$loop$".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("Asit"),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#$loop$@2".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "$loop$".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("Sourabh"),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#$loop$@3".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "$loop$".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("$loop$"),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
            }),
        );

        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@0", -1, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@1", -1, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@2", -1, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@3", -1, &mut bag);

        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT@0", 0, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT@1", 0, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT@2", 0, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT@3", 0, &mut bag);

        bag.insert(
            "foo/bar#people".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "foo/bar#people".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::List {
                        data: vec![
                            ftd::PropertyValue::Value {
                                value: ftd::Value::String {
                                    text: "Arpita".to_string(),
                                    source: ftd::TextSource::Caption,
                                },
                            },
                            ftd::PropertyValue::Value {
                                value: ftd::Value::String {
                                    text: "Asit".to_string(),
                                    source: ftd::TextSource::Caption,
                                },
                            },
                            ftd::PropertyValue::Value {
                                value: ftd::Value::String {
                                    text: "Sourabh".to_string(),
                                    source: ftd::TextSource::Caption,
                                },
                            },
                        ],
                        kind: ftd::ftd2021::p2::Kind::string(),
                    },
                },
                conditions: vec![],
            }),
        );
        let (g_bag, g_col) = ftd::ftd2021::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- string list people:

                -- people: Arpita

                -- people: Asit

                -- people: Sourabh

                -- ftd.text: $obj
                $loop$: $people as $obj
                "
            ),
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .expect("found error");
        pretty_assertions::assert_eq!(g_bag, bag);
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn loop_inside_subsection() {
        let mut main = p2::default_column();
        let mut col = ftd::Column {
            ..Default::default()
        };

        col.container.children.push(ftd::Element::Row(ftd::Row {
            spacing: None,
            container: ftd::Container {
                children: vec![
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::ftd2021::rendered::markup_line("Arpita Jaiswal"),
                        line: true,
                        common: Box::new(ftd::Common {
                            reference: Some(s("foo/bar#name@0,0")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::ftd2021::rendered::markup_line(
                            "Arpita is developer at Fifthtry",
                        ),
                        line: true,
                        common: Box::new(ftd::Common {
                            reference: Some(s("foo/bar#body@0,0")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                ],
                ..Default::default()
            },
            common: Box::new(ftd::Common {
                reference: Some(s("foo/bar#people")),
                ..Default::default()
            },
        }));

        col.container.children.push(ftd::Element::Row(ftd::Row {
            spacing: None,
            container: ftd::Container {
                children: vec![
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::ftd2021::rendered::markup_line("Amit Upadhyay"),
                        line: true,
                        common: Box::new(ftd::Common {
                            reference: Some(s("foo/bar#name@0,1")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::ftd2021::rendered::markup_line("Amit is CEO of FifthTry."),
                        line: true,
                        common: Box::new(ftd::Common {
                            reference: Some(s("foo/bar#body@0,1")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                ],
                ..Default::default()
            },
            common: Box::new(ftd::Common {
                reference: Some(s("foo/bar#people")),
                ..Default::default()
            },
        }));

        main.container.children.push(ftd::Element::Column(col));

        let mut bag = interpreter::default_bag();

        bag.insert(
            "foo/bar#foo".to_string(),
            ftd::ftd2021::p2::Thing::Component(ftd::Component {
                root: "ftd.row".to_string(),
                full_name: s("foo/bar#foo"),
                arguments: std::iter::IntoIterator::into_iter([
                    (s("body"), ftd::ftd2021::p2::Kind::string()),
                    (s("name"), ftd::ftd2021::p2::Kind::caption()),
                ])
                .collect(),
                instructions: vec![
                    ftd::ftd2021::component::Instruction::ChildComponent {
                        child: ftd::ftd2021::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "ftd#text".to_string(),
                            condition: None,
                            properties: std::iter::IntoIterator::into_iter([(
                                s("text"),
                                ftd::ftd2021::component::Property {
                                    default: Some(ftd::PropertyValue::Variable {
                                        name: "name".to_string(),
                                        kind: ftd::ftd2021::p2::Kind::caption_or_body(),
                                    }),
                                    conditions: vec![],
                                    ..Default::default()
                                },
                            )])
                            .collect(),
                            ..Default::default()
                        },
                    },
                    ftd::ftd2021::component::Instruction::ChildComponent {
                        child: ftd::ftd2021::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "ftd#text".to_string(),
                            condition: None,
                            properties: std::iter::IntoIterator::into_iter([(
                                s("text"),
                                ftd::ftd2021::component::Property {
                                    default: Some(ftd::PropertyValue::Variable {
                                        name: "body".to_string(),
                                        kind: ftd::ftd2021::p2::Kind::caption_or_body(),
                                    }),
                                    conditions: vec![],
                                    ..Default::default()
                                },
                            )])
                            .collect(),
                            ..Default::default()
                        },
                    },
                ],
                invocations: vec![
                    std::iter::IntoIterator::into_iter([
                        (
                            s("body"),
                            ftd::Value::String {
                                text: s("Arpita is developer at Fifthtry"),
                                source: ftd::TextSource::Body,
                            },
                        ),
                        (
                            s("name"),
                            ftd::Value::String {
                                text: s("Arpita Jaiswal"),
                                source: ftd::TextSource::Caption,
                            },
                        ),
                    ])
                    .collect(),
                    std::iter::IntoIterator::into_iter([
                        (
                            s("body"),
                            ftd::Value::String {
                                text: s("Amit is CEO of FifthTry."),
                                source: ftd::TextSource::Body,
                            },
                        ),
                        (
                            s("name"),
                            ftd::Value::String {
                                text: s("Amit Upadhyay"),
                                source: ftd::TextSource::Caption,
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
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "foo/bar#people".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::List {
                        data: vec![
                            ftd::PropertyValue::Value {
                                value: ftd::Value::Record {
                                    name: "foo/bar#person".to_string(),
                                    fields: std::iter::IntoIterator::into_iter([
                                        (
                                            s("bio"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::Value::String {
                                                    text: "Arpita is developer at Fifthtry"
                                                        .to_string(),
                                                    source: ftd::TextSource::Body,
                                                },
                                            },
                                        ),
                                        (
                                            s("name"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::Value::String {
                                                    text: "Arpita Jaiswal".to_string(),
                                                    source: ftd::TextSource::Caption,
                                                },
                                            },
                                        ),
                                    ])
                                    .collect(),
                                },
                            },
                            ftd::PropertyValue::Value {
                                value: ftd::Value::Record {
                                    name: "foo/bar#person".to_string(),
                                    fields: std::iter::IntoIterator::into_iter([
                                        (
                                            s("bio"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::Value::String {
                                                    text: "Amit is CEO of FifthTry.".to_string(),
                                                    source: ftd::TextSource::Body,
                                                },
                                            },
                                        ),
                                        (
                                            s("name"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::Value::String {
                                                    text: "Amit Upadhyay".to_string(),
                                                    source: ftd::TextSource::Caption,
                                                },
                                            },
                                        ),
                                    ])
                                    .collect(),
                                },
                            },
                        ],
                        kind: ftd::ftd2021::p2::Kind::Record {
                            name: "foo/bar#person".to_string(),
                            default: None,
                            is_reference: true,
                        },
                    },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            "foo/bar#person".to_string(),
            ftd::ftd2021::p2::Thing::Record(ftd::ftd2021::p2::Record {
                name: "foo/bar#person".to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    (s("bio"), ftd::ftd2021::p2::Kind::body()),
                    (s("name"), ftd::ftd2021::p2::Kind::caption()),
                ])
                .collect(),
                instances: Default::default(),
                order: vec![s("name"), s("bio")],
            }),
        );

        let (_g_bag, g_col) = ftd::ftd2021::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- ftd.row foo:
                caption name:
                string body:

                --- ftd.text: $name

                --- ftd.text: $body

                -- record person:
                caption name:
                body bio:

                -- person list people:

                -- people: Arpita Jaiswal

                Arpita is developer at Fifthtry

                -- people: Amit Upadhyay

                Amit is CEO of FifthTry.

                -- ftd.column:

                --- foo: $obj.name
                $loop$: $people as $obj
                body: $obj.bio
                "
            ),
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .expect("found error");
        // pretty_assertions::assert_eq!(g_bag, bag);
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn basic_processor() {
        let mut main = p2::default_column();

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("\"0.3.0\""),
                line: true,
                common: Box::new(ftd::Common {
                    reference: Some(s("foo/bar#test")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        let mut bag = interpreter::default_bag();

        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@0", -1, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT@0", 0, &mut bag);
        insert_update_integer_by_root("foo/bar#SIBLING-INDEX-0@0", 0, &mut bag);
        insert_update_integer_by_root("foo/bar#SIBLING-INDEX@0", 1, &mut bag);

        bag.insert(
            "foo/bar#test".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "foo/bar#test".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: "\"0.3.0\"".to_string(),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
            }),
        );

        let (g_bag, g_col) = ftd::ftd2021::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- string test:
                $processor$: read_version_from_cargo_toml

                -- ftd.text: $test
                "
            ),
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .expect("found error");
        pretty_assertions::assert_eq!(g_bag, bag);
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn basic_processor_that_overwrites() {
        let mut main = p2::default_column();

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("\"0.3.0\""),
                line: true,
                common: Box::new(ftd::Common {
                    reference: Some(s("foo/bar#test")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        let mut bag = interpreter::default_bag();

        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@0", -1, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT@0", 0, &mut bag);
        insert_update_integer_by_root("foo/bar#SIBLING-INDEX-0@0", 0, &mut bag);
        insert_update_integer_by_root("foo/bar#SIBLING-INDEX@0", 1, &mut bag);

        bag.insert(
            "foo/bar#test".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "test".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: "\"0.3.0\"".to_string(),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
            }),
        );

        let (g_bag, g_col) = ftd::ftd2021::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- string test: yo

                -- test:
                $processor$: read_version_from_cargo_toml

                -- ftd.text: $test
                "
            ),
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .expect("found error");
        pretty_assertions::assert_eq!(g_bag, bag);
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    #[ignore]
    fn basic_processor_for_list() {
        let mut main = p2::default_column();

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("\"ftd\""),
                line: true,
                common: Box::new(ftd::Common {
                    reference: Some(s("foo/bar#test")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("\"0.2.0\""),
                line: true,
                common: Box::new(ftd::Common {
                    reference: Some(s("foo/bar#test")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("["),
                line: true,
                common: Box::new(ftd::Common {
                    reference: Some(s("foo/bar#test")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("\"2021\""),
                line: true,
                common: Box::new(ftd::Common {
                    reference: Some(s("foo/bar#test")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("\"ftd: FifthTry Document Format\""),
                line: true,
                common: Box::new(ftd::Common {
                    reference: Some(s("foo/bar#test")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("\"MIT\""),
                line: true,
                common: Box::new(ftd::Common {
                    reference: Some(s("foo/bar#test")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("\"https://github.com/FifthTry/ftd\""),
                line: true,
                common: Box::new(ftd::Common {
                    reference: Some(s("foo/bar#test")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("\"https://ftd.dev\""),
                line: true,
                common: Box::new(ftd::Common {
                    reference: Some(s("foo/bar#test")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("$loop$"),
                line: true,
                common: Box::new(ftd::Common {
                    reference: Some(s("foo/bar#test")),
                    is_dummy: true,
                    ..Default::default()
                },
                ..Default::default()
            }));

        let mut bag = interpreter::default_bag();

        bag.insert(
            "foo/bar#test".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "foo/bar#test".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::List {
                        data: vec![
                            ftd::PropertyValue::Value {
                                value: ftd::Value::String {
                                    text: "\"ftd\"".to_string(),
                                    source: ftd::TextSource::Header,
                                },
                            },
                            ftd::PropertyValue::Value {
                                value: ftd::Value::String {
                                    text: "\"0.2.0\"".to_string(),
                                    source: ftd::TextSource::Header,
                                },
                            },
                            ftd::PropertyValue::Value {
                                value: ftd::Value::String {
                                    text: "[".to_string(),
                                    source: ftd::TextSource::Header,
                                },
                            },
                            ftd::PropertyValue::Value {
                                value: ftd::Value::String {
                                    text: "\"2021\"".to_string(),
                                    source: ftd::TextSource::Header,
                                },
                            },
                            ftd::PropertyValue::Value {
                                value: ftd::Value::String {
                                    text: "\"ftd: FifthTry Document Format\"".to_string(),
                                    source: ftd::TextSource::Header,
                                },
                            },
                            ftd::PropertyValue::Value {
                                value: ftd::Value::String {
                                    text: "\"MIT\"".to_string(),
                                    source: ftd::TextSource::Header,
                                },
                            },
                            ftd::PropertyValue::Value {
                                value: ftd::Value::String {
                                    text: "\"https://github.com/FifthTry/ftd\"".to_string(),
                                    source: ftd::TextSource::Header,
                                },
                            },
                            ftd::PropertyValue::Value {
                                value: ftd::Value::String {
                                    text: "\"https://ftd.dev\"".to_string(),
                                    source: ftd::TextSource::Header,
                                },
                            },
                        ],
                        kind: ftd::ftd2021::p2::Kind::string(),
                    },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            "foo/bar#$loop$@0".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "$loop$".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: "\"ftd\"".to_string(),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            "foo/bar#$loop$@1".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "$loop$".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: "\"0.2.0\"".to_string(),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            "foo/bar#$loop$@2".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "$loop$".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: "[".to_string(),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            "foo/bar#$loop$@3".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "$loop$".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: "\"2021\"".to_string(),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#$loop$@4".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "$loop$".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: "\"ftd: FifthTry Document Format\"".to_string(),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#$loop$@5".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "$loop$".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: "\"MIT\"".to_string(),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#$loop$@6".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "$loop$".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: "\"https://github.com/FifthTry/ftd\"".to_string(),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#$loop$@7".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "$loop$".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: "\"https://ftd.dev\"".to_string(),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#$loop$@8".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "$loop$".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: "$loop$".to_string(),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
            }),
        );

        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@0", -1, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@1", -1, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@2", -1, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@3", -1, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@4", -1, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@5", -1, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@6", -1, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@7", -1, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@8", -1, &mut bag);

        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT@0", 0, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT@1", 0, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT@2", 0, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT@3", 0, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT@4", 0, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT@5", 0, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT@6", 0, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT@7", 0, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT@8", 0, &mut bag);

        let (g_bag, g_col) = ftd::ftd2021::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- string list test:
                $processor$: read_package_from_cargo_toml

                -- ftd.text: $obj
                $loop$: $test as $obj
                "
            ),
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_bag, bag);
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    #[ignore]
    fn processor_for_list_of_record() {
        let mut main = p2::default_column();

        main.container.children.push(ftd::Element::Row(ftd::Row {
            spacing: None,
            container: ftd::Container {
                children: vec![
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::ftd2021::rendered::markup_line("\"ftd\""),
                        line: true,
                        common: Box::new(ftd::Common {
                            reference: Some(s("foo/bar#name@0")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::ftd2021::rendered::markup_line("name"),
                        line: true,
                        common: Box::new(ftd::Common {
                            reference: Some(s("foo/bar#body@0")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                ],
                ..Default::default()
            },
            common: Box::new(ftd::Common {
                reference: Some(s("foo/bar#test")),
                ..Default::default()
            },
        }));

        main.container.children.push(ftd::Element::Row(ftd::Row {
            spacing: None,
            container: ftd::Container {
                children: vec![
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::ftd2021::rendered::markup_line("\"0.2.0\""),
                        line: true,
                        common: Box::new(ftd::Common {
                            reference: Some(s("foo/bar#name@1")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::ftd2021::rendered::markup_line("version"),
                        line: true,
                        common: Box::new(ftd::Common {
                            reference: Some(s("foo/bar#body@1")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                ],
                ..Default::default()
            },
            common: Box::new(ftd::Common {
                reference: Some(s("foo/bar#test")),
                ..Default::default()
            },
        }));

        main.container.children.push(ftd::Element::Row(ftd::Row {
            spacing: None,
            container: ftd::Container {
                children: vec![
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::ftd2021::rendered::markup_line("["),
                        line: true,
                        common: Box::new(ftd::Common {
                            reference: Some(s("foo/bar#name@2")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::ftd2021::rendered::markup_line("authors"),
                        line: true,
                        common: Box::new(ftd::Common {
                            reference: Some(s("foo/bar#body@2")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                ],
                ..Default::default()
            },
            common: Box::new(ftd::Common {
                reference: Some(s("foo/bar#test")),
                ..Default::default()
            },
        }));

        main.container.children.push(ftd::Element::Row(ftd::Row {
            spacing: None,
            container: ftd::Container {
                children: vec![
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::ftd2021::rendered::markup_line("\"2021\""),
                        line: true,
                        common: Box::new(ftd::Common {
                            reference: Some(s("foo/bar#name@3")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::ftd2021::rendered::markup_line("edition"),
                        line: true,
                        common: Box::new(ftd::Common {
                            reference: Some(s("foo/bar#body@3")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                ],
                ..Default::default()
            },
            common: Box::new(ftd::Common {
                reference: Some(s("foo/bar#test")),
                ..Default::default()
            },
        }));

        main.container.children.push(ftd::Element::Row(ftd::Row {
            spacing: None,
            container: ftd::Container {
                children: vec![
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::ftd2021::rendered::markup_line(
                            "\"ftd: FifthTry Document Format\"",
                        ),
                        line: true,
                        common: Box::new(ftd::Common {
                            reference: Some(s("foo/bar#name@4")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::ftd2021::rendered::markup_line("description"),
                        line: true,
                        common: Box::new(ftd::Common {
                            reference: Some(s("foo/bar#body@4")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                ],
                ..Default::default()
            },
            common: Box::new(ftd::Common {
                reference: Some(s("foo/bar#test")),
                ..Default::default()
            },
        }));

        main.container.children.push(ftd::Element::Row(ftd::Row {
            spacing: None,
            container: ftd::Container {
                children: vec![
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::ftd2021::rendered::markup_line("\"MIT\""),
                        line: true,
                        common: Box::new(ftd::Common {
                            reference: Some(s("foo/bar#name@5")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::ftd2021::rendered::markup_line("license"),
                        line: true,
                        common: Box::new(ftd::Common {
                            reference: Some(s("foo/bar#body@5")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                ],
                ..Default::default()
            },
            common: Box::new(ftd::Common {
                reference: Some(s("foo/bar#test")),
                ..Default::default()
            },
        }));

        main.container.children.push(ftd::Element::Row(ftd::Row {
            spacing: None,
            container: ftd::Container {
                children: vec![
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::ftd2021::rendered::markup_line(
                            "\"https://github.com/FifthTry/ftd\"",
                        ),
                        line: true,
                        common: Box::new(ftd::Common {
                            reference: Some(s("foo/bar#name@6")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::ftd2021::rendered::markup_line("repository"),
                        line: true,
                        common: Box::new(ftd::Common {
                            reference: Some(s("foo/bar#body@6")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                ],
                ..Default::default()
            },
            common: Box::new(ftd::Common {
                reference: Some(s("foo/bar#test")),
                ..Default::default()
            },
        }));

        main.container.children.push(ftd::Element::Row(ftd::Row {
            spacing: None,
            container: ftd::Container {
                children: vec![
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::ftd2021::rendered::markup_line("\"https://ftd.dev\""),
                        line: true,
                        common: Box::new(ftd::Common {
                            reference: Some(s("foo/bar#name@7")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::ftd2021::rendered::markup_line("homepage"),
                        line: true,
                        common: Box::new(ftd::Common {
                            reference: Some(s("foo/bar#body@7")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                ],
                ..Default::default()
            },
            common: Box::new(ftd::Common {
                reference: Some(s("foo/bar#test")),
                ..Default::default()
            },
        }));

        /*let mut bag = super::default_bag();

        bag.insert(
            "foo/bar#data".to_string(),
            ftd::ftd2021::p2::Thing::Record(ftd::ftd2021::p2::Record {
                name: "foo/bar#data".to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    (s("description"), ftd::ftd2021::p2::Kind::string()),
                    (s("title"), ftd::ftd2021::p2::Kind::string()),
                ])
                .collect(),
                instances: Default::default(),
                order: vec![s("title"), s("description")],
            }),
        );

        bag.insert(
            "foo/bar#foo".to_string(),
            ftd::ftd2021::p2::Thing::Component(ftd::Component {
                root: "ftd.row".to_string(),
                full_name: "foo/bar#foo".to_string(),
                arguments: std::iter::IntoIterator::into_iter([
                    (s("body"), ftd::ftd2021::p2::Kind::string()),
                    (s("name"), ftd::ftd2021::p2::Kind::caption()),
                ])
                .collect(),
                instructions: vec![
                    ftd::component::Instruction::ChildComponent {
                        child: ftd::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "ftd#text".to_string(),
                            condition: None,
                            properties: std::iter::IntoIterator::into_iter([(
                                s("text"),
                                ftd::component::Property {
                                    default: Some(ftd::PropertyValue::Variable {
                                        name: "name".to_string(),
                                        kind: ftd::ftd2021::p2::Kind::caption_or_body(),
                                    }),
                                    conditions: vec![],
                                    ..Default::default()
                                },
                            )])
                            .collect(),
                            ..Default::default()
                        },
                    },
                    ftd::component::Instruction::ChildComponent {
                        child: ftd::component::ChildComponent {
                            is_recursive: false,
                            events: vec![],
                            root: "ftd#text".to_string(),
                            condition: None,
                            properties: std::iter::IntoIterator::into_iter([(
                                s("text"),
                                ftd::component::Property {
                                    default: Some(ftd::PropertyValue::Variable {
                                        name: "body".to_string(),
                                        kind: ftd::ftd2021::p2::Kind::caption_or_body(),
                                    }),
                                    conditions: vec![],
                                    ..Default::default()
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

        bag.insert(
            "foo/bar#test".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "foo/bar#test".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::List {
                        data: vec![
                            ftd::PropertyValue::Value {
                                value: ftd::Value::Record {
                                    name: "foo/bar#data".to_string(),
                                    fields: std::iter::IntoIterator::into_iter([
                                        (
                                            s("description"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::variable::Value::String {
                                                    text: "name".to_string(),
                                                    source: ftd::TextSource::Header,
                                                },
                                            },
                                        ),
                                        (
                                            s("title"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::variable::Value::String {
                                                    text: "\"ftd\"".to_string(),
                                                    source: ftd::TextSource::Header,
                                                },
                                            },
                                        ),
                                    ])
                                    .collect(),
                                },
                            },
                            ftd::PropertyValue::Value {
                                value: ftd::Value::Record {
                                    name: "foo/bar#data".to_string(),
                                    fields: std::iter::IntoIterator::into_iter([
                                        (
                                            s("description"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::variable::Value::String {
                                                    text: "version".to_string(),
                                                    source: ftd::TextSource::Header,
                                                },
                                            },
                                        ),
                                        (
                                            s("title"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::variable::Value::String {
                                                    text: "\"0.2.0\"".to_string(),
                                                    source: ftd::TextSource::Header,
                                                },
                                            },
                                        ),
                                    ])
                                    .collect(),
                                },
                            },
                            ftd::PropertyValue::Value {
                                value: ftd::Value::Record {
                                    name: "foo/bar#data".to_string(),
                                    fields: std::iter::IntoIterator::into_iter([
                                        (
                                            s("description"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::variable::Value::String {
                                                    text: "authors".to_string(),
                                                    source: ftd::TextSource::Header,
                                                },
                                            },
                                        ),
                                        (
                                            s("title"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::variable::Value::String {
                                                    text: "[".to_string(),
                                                    source: ftd::TextSource::Header,
                                                },
                                            },
                                        ),
                                    ])
                                    .collect(),
                                },
                            },
                            ftd::PropertyValue::Value {
                                value: ftd::Value::Record {
                                    name: "foo/bar#data".to_string(),
                                    fields: std::iter::IntoIterator::into_iter([
                                        (
                                            s("description"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::variable::Value::String {
                                                    text: "edition".to_string(),
                                                    source: ftd::TextSource::Header,
                                                },
                                            },
                                        ),
                                        (
                                            s("title"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::variable::Value::String {
                                                    text: "\"2021\"".to_string(),
                                                    source: ftd::TextSource::Header,
                                                },
                                            },
                                        ),
                                    ])
                                    .collect(),
                                },
                            },
                            ftd::PropertyValue::Value {
                                value: ftd::Value::Record {
                                    name: "foo/bar#data".to_string(),
                                    fields: std::iter::IntoIterator::into_iter([
                                        (
                                            s("description"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::variable::Value::String {
                                                    text: "description".to_string(),
                                                    source: ftd::TextSource::Header,
                                                },
                                            },
                                        ),
                                        (
                                            s("title"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::variable::Value::String {
                                                    text: "\"ftd: FifthTry Document Format\""
                                                        .to_string(),
                                                    source: ftd::TextSource::Header,
                                                },
                                            },
                                        ),
                                    ])
                                    .collect(),
                                },
                            },
                            ftd::PropertyValue::Value {
                                value: ftd::Value::Record {
                                    name: "foo/bar#data".to_string(),
                                    fields: std::iter::IntoIterator::into_iter([
                                        (
                                            s("description"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::variable::Value::String {
                                                    text: "license".to_string(),
                                                    source: ftd::TextSource::Header,
                                                },
                                            },
                                        ),
                                        (
                                            s("title"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::variable::Value::String {
                                                    text: "\"MIT\"".to_string(),
                                                    source: ftd::TextSource::Header,
                                                },
                                            },
                                        ),
                                    ])
                                    .collect(),
                                },
                            },
                            ftd::PropertyValue::Value {
                                value: ftd::Value::Record {
                                    name: "foo/bar#data".to_string(),
                                    fields: std::iter::IntoIterator::into_iter([
                                        (
                                            s("description"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::variable::Value::String {
                                                    text: "repository".to_string(),
                                                    source: ftd::TextSource::Header,
                                                },
                                            },
                                        ),
                                        (
                                            s("title"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::variable::Value::String {
                                                    text: "\"https://github.com/FifthTry/ftd\""
                                                        .to_string(),
                                                    source: ftd::TextSource::Header,
                                                },
                                            },
                                        ),
                                    ])
                                    .collect(),
                                },
                            },
                            ftd::PropertyValue::Value {
                                value: ftd::Value::Record {
                                    name: "foo/bar#data".to_string(),
                                    fields: std::iter::IntoIterator::into_iter([
                                        (
                                            s("description"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::variable::Value::String {
                                                    text: "homepage".to_string(),
                                                    source: ftd::TextSource::Header,
                                                },
                                            },
                                        ),
                                        (
                                            s("title"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::variable::Value::String {
                                                    text: "\"https://ftd.dev\"".to_string(),
                                                    source: ftd::TextSource::Header,
                                                },
                                            },
                                        ),
                                    ])
                                    .collect(),
                                },
                            },
                        ],
                        kind: ftd::ftd2021::p2::Kind::Record {
                            name: s("foo/bar#data"),
                            default: None,
                        },
                    },
                },
                conditions: vec![],
            }),
        );*/

        let (_g_bag, g_col) = ftd::ftd2021::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- ftd.row foo:
                caption name:
                string body:

                --- ftd.text: $name

                --- ftd.text: $body

                -- record data:
                string title:
                string description:

                -- data list test:
                $processor$: read_package_records_from_cargo_toml

                -- foo: $obj.title
                $loop$: $test as $obj
                body: $obj.description
                "
            ),
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn loop_with_tree_structure() {
        let mut main = p2::default_column();
        let col = ftd::Element::Column(ftd::Column {
            spacing: None,
            container: ftd::Container {
                children: vec![
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::ftd2021::rendered::markup_line("ab title"),
                        line: true,
                        common: Box::new(ftd::Common {
                            reference: Some(s("foo/bar#toc@0.title")),
                            link: Some(s("ab link")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                    ftd::Element::Column(ftd::Column {
                        spacing: None,
                        container: ftd::Container {
                            children: vec![ftd::Element::Markup(ftd::Markups {
                                text: ftd::ftd2021::rendered::markup_line("aa title"),
                                line: true,
                                common: Box::new(ftd::Common {
                                    reference: Some(s("foo/bar#toc@0,1.title")),
                                    link: Some(s("aa link")),
                                    ..Default::default()
                                },
                                ..Default::default()
                            })],
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                    ftd::Element::Column(ftd::Column {
                        spacing: None,
                        container: ftd::Container {
                            children: vec![ftd::Element::Markup(ftd::Markups {
                                text: ftd::ftd2021::rendered::markup_line("aaa title"),
                                line: true,
                                common: Box::new(ftd::Common {
                                    reference: Some(s("foo/bar#toc@0,2.title")),
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
            common: Box::new(ftd::Common {
                reference: Some(s("foo/bar#toc")),
                ..Default::default()
            },
        });
        let col1 = ftd::Element::Column(ftd::Column {
            spacing: None,
            container: ftd::Container {
                children: vec![
                    ftd::Element::Markup(ftd::Markups {
                        text: ftd::ftd2021::rendered::markup_line("ab title"),
                        line: true,
                        common: Box::new(ftd::Common {
                            reference: Some(s("foo/bar#toc@1,0.title")),
                            link: Some(s("ab link")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                    ftd::Element::Column(ftd::Column {
                        spacing: None,
                        container: ftd::Container {
                            children: vec![ftd::Element::Markup(ftd::Markups {
                                text: ftd::ftd2021::rendered::markup_line("aa title"),
                                line: true,
                                common: Box::new(ftd::Common {
                                    reference: Some(s("foo/bar#toc@1,0,1.title")),
                                    link: Some(s("aa link")),
                                    ..Default::default()
                                },
                                ..Default::default()
                            })],
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                    ftd::Element::Column(ftd::Column {
                        spacing: None,
                        container: ftd::Container {
                            children: vec![ftd::Element::Markup(ftd::Markups {
                                text: ftd::ftd2021::rendered::markup_line("aaa title"),
                                line: true,
                                common: Box::new(ftd::Common {
                                    reference: Some(s("foo/bar#toc@1,0,2.title")),
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
            common: Box::new(ftd::Common {
                reference: Some(s("foo/bar#toc")),
                ..Default::default()
            },
        });
        main.container.children.push(col.clone());
        main.container.children.push(ftd::Element::Row(ftd::Row {
            spacing: None,
            container: ftd::Container {
                children: vec![col1],
                ..Default::default()
            },
            ..Default::default()
        }));

        let mut bag = interpreter::default_bag();

        bag.insert(
            s("foo/bar#aa"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("foo/bar#aa"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::List {
                        data: vec![
                            ftd::PropertyValue::Value {
                                value: ftd::Value::Record {
                                    name: s("foo/bar#toc-record"),
                                    fields: std::iter::IntoIterator::into_iter([
                                        (
                                            s("children"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::ftd2021::variable::Value::List {
                                                    data: vec![],
                                                    kind: ftd::ftd2021::p2::Kind::Record {
                                                        name: s("foo/bar#toc-record"),
                                                        default: None,
                                                        is_reference: true,
                                                    },
                                                },
                                            },
                                        ),
                                        (
                                            s("link"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::ftd2021::variable::Value::String {
                                                    text: s("aa link"),
                                                    source: ftd::TextSource::Header,
                                                },
                                            },
                                        ),
                                        (
                                            s("title"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::ftd2021::variable::Value::String {
                                                    text: s("aa title"),
                                                    source: ftd::TextSource::Header,
                                                },
                                            },
                                        ),
                                    ])
                                    .collect(),
                                },
                            },
                            ftd::PropertyValue::Value {
                                value: ftd::Value::Record {
                                    name: s("foo/bar#toc-record"),
                                    fields: std::iter::IntoIterator::into_iter([
                                        (
                                            s("children"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::ftd2021::variable::Value::List {
                                                    data: vec![],
                                                    kind: ftd::ftd2021::p2::Kind::Record {
                                                        name: s("foo/bar#toc-record"),
                                                        default: None,
                                                        is_reference: true,
                                                    },
                                                },
                                            },
                                        ),
                                        (
                                            s("link"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::ftd2021::variable::Value::String {
                                                    text: s("aaa link"),
                                                    source: ftd::TextSource::Header,
                                                },
                                            },
                                        ),
                                        (
                                            s("title"),
                                            ftd::PropertyValue::Value {
                                                value: ftd::ftd2021::variable::Value::String {
                                                    text: s("aaa title"),
                                                    source: ftd::TextSource::Header,
                                                },
                                            },
                                        ),
                                    ])
                                    .collect(),
                                },
                            },
                        ],
                        kind: ftd::ftd2021::p2::Kind::Record {
                            name: s("foo/bar#toc-record"),
                            default: None,
                            is_reference: true,
                        },
                    },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            s("foo/bar#toc"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("foo/bar#toc"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::List {
                        data: vec![ftd::PropertyValue::Value {
                            value: ftd::Value::Record {
                                name: s("foo/bar#toc-record"),
                                fields: std::iter::IntoIterator::into_iter([
                                    (
                                        s("children"),
                                        ftd::PropertyValue::Value {
                                            value: ftd::ftd2021::variable::Value::List {
                                                data: vec![
                                                    ftd::PropertyValue::Value {value: ftd::Value::Record {
                                                        name: s("foo/bar#toc-record"),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                s("children"),
                                                                ftd::PropertyValue::Value {
                                                                    value: ftd::ftd2021::variable::Value::List {
                                                                        data: vec![],
                                                                        kind: ftd::ftd2021::p2::Kind::Record {
                                                                            name: s("foo/bar#toc-record"),
                                                                            default: None,
                                                                            is_reference: true,
                                                                        },
                                                                    },
                                                                },
                                                            ),
                                                            (
                                                                s("link"),
                                                                ftd::PropertyValue::Value {
                                                                    value: ftd::ftd2021::variable::Value::String {
                                                                        text: s("aa link"),
                                                                        source: ftd::TextSource::Header,
                                                                    },
                                                                },
                                                            ),
                                                            (
                                                                s("title"),
                                                                ftd::PropertyValue::Value {
                                                                    value: ftd::ftd2021::variable::Value::String {
                                                                        text: s("aa title"),
                                                                        source: ftd::TextSource::Header,
                                                                    },
                                                                },
                                                            ),
                                                        ])
                                                            .collect(),
                                                    }},
                                                    ftd::PropertyValue::Value {value: ftd::Value::Record {
                                                        name: s("foo/bar#toc-record"),
                                                        fields: std::iter::IntoIterator::into_iter([
                                                            (
                                                                s("children"),
                                                                ftd::PropertyValue::Value {
                                                                    value: ftd::ftd2021::variable::Value::List {
                                                                        data: vec![],
                                                                        kind: ftd::ftd2021::p2::Kind::Record {
                                                                            name: s("foo/bar#toc-record"),
                                                                            default: None,
                                                                            is_reference: true,
                                                                        },
                                                                    },
                                                                },
                                                            ),
                                                            (
                                                                s("link"),
                                                                ftd::PropertyValue::Value {
                                                                    value: ftd::ftd2021::variable::Value::String {
                                                                        text: s("aaa link"),
                                                                        source: ftd::TextSource::Header,
                                                                    },
                                                                },
                                                            ),
                                                            (
                                                                s("title"),
                                                                ftd::PropertyValue::Value {
                                                                    value: ftd::ftd2021::variable::Value::String {
                                                                        text: s("aaa title"),
                                                                        source: ftd::TextSource::Header,
                                                                    },
                                                                },
                                                            ),
                                                        ])
                                                            .collect(),
                                                    }},
                                                ],
                                                kind: ftd::ftd2021::p2::Kind::Record {
                                                    name: s("foo/bar#toc-record"),
                                                    default: None,
                                                    is_reference: true,
                                                },
                                            },
                                        },
                                    ),
                                    (
                                        s("link"),
                                        ftd::PropertyValue::Value {
                                            value: ftd::ftd2021::variable::Value::String {
                                                text: s("ab link"),
                                                source: ftd::TextSource::Header,
                                            },
                                        },
                                    ),
                                    (
                                        s("title"),
                                        ftd::PropertyValue::Value {
                                            value: ftd::ftd2021::variable::Value::String {
                                                text: s("ab title"),
                                                source: ftd::TextSource::Header,
                                            },
                                        },
                                    ),
                                ])
                                    .collect(),
                            },
                        }],
                        kind: ftd::ftd2021::p2::Kind::Record {
                            name: s("foo/bar#toc-record"),
                            default: None,
                            is_reference: true,
                        },
                    },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            s("foo/bar#toc"),
            ftd::ftd2021::p2::Thing::Component(ftd::Component {
                root: "ftd.column".to_string(),
                full_name: "foo/bar#toc-item".to_string(),
                arguments: std::iter::IntoIterator::into_iter([(
                    s("toc"),
                    ftd::ftd2021::p2::Kind::Record {
                        name: "foo/bar#toc-record".to_string(),
                        default: None,
                        is_reference: true,
                    },
                )])
                .collect(),
                instructions: vec![
                    ftd::Instruction::ChildComponent {
                        child: ftd::ChildComponent {
                            events: vec![],
                            root: "ftd#text".to_string(),
                            condition: None,
                            properties: std::iter::IntoIterator::into_iter([
                                (
                                    s("link"),
                                    ftd::ftd2021::component::Property {
                                        default: Some(ftd::PropertyValue::Variable {
                                            name: "toc.link".to_string(),
                                            kind: ftd::ftd2021::p2::Kind::Optional {
                                                kind: Box::new(ftd::ftd2021::p2::Kind::string()),
                                                is_reference: false,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                                (
                                    s("text"),
                                    ftd::ftd2021::component::Property {
                                        default: Some(ftd::PropertyValue::Variable {
                                            name: "toc.title".to_string(),
                                            kind: ftd::ftd2021::p2::Kind::Optional {
                                                kind: Box::new(
                                                    ftd::ftd2021::p2::Kind::caption_or_body(),
                                                ),
                                                is_reference: false,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                            ])
                            .collect(),
                            ..Default::default()
                        },
                    },
                    ftd::Instruction::RecursiveChildComponent {
                        child: ftd::ChildComponent {
                            is_recursive: true,
                            events: vec![],
                            root: "toc-item".to_string(),
                            condition: None,
                            properties: std::iter::IntoIterator::into_iter([
                                (
                                    s("$loop$"),
                                    ftd::ftd2021::component::Property {
                                        default: Some(ftd::PropertyValue::Variable {
                                            name: "toc.children".to_string(),
                                            kind: ftd::ftd2021::p2::Kind::Record {
                                                name: s("foo/bar#toc-record"),
                                                default: None,
                                                is_reference: true,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                                (
                                    s("toc"),
                                    ftd::ftd2021::component::Property {
                                        default: Some(ftd::PropertyValue::Variable {
                                            name: "$loop$".to_string(),
                                            kind: ftd::ftd2021::p2::Kind::Record {
                                                name: s("foo/bar#toc-record"),
                                                default: None,
                                                is_reference: true,
                                            },
                                        }),
                                        conditions: vec![],
                                        ..Default::default()
                                    },
                                ),
                            ])
                            .collect(),
                            ..Default::default()
                        },
                    },
                ],
                ..Default::default()
            }),
        );

        let (_g_bag, g_col) = ftd::ftd2021::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- record toc-record:
                string title:
                string link:
                toc-record list children:

                -- ftd.column toc-item:
                toc-record toc:

                --- ftd.text: $toc.title
                link: $toc.link

                --- toc-item:
                $loop$: $toc.children as $obj
                toc: $obj

                -- toc-record list aa:

                -- aa:
                title: aa title
                link: aa link

                -- aa:
                title: aaa title
                link: aaa link

                -- toc-record list toc:

                -- toc:
                title: ab title
                link: ab link
                children: $aa

                -- ftd.row foo:

                --- toc-item:
                $loop$: $toc as $obj
                toc: $obj

                -- toc-item:
                $loop$: $toc as $obj
                toc: $obj

                -- foo:
                "
            ),
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .expect("found error");

        // pretty_assertions::assert_eq!(g_bag, bag);
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn import_check() {
        let mut main = p2::default_column();
        main.container.children.push(ftd::Element::Row(ftd::Row {
            spacing: None,
            container: ftd::Container {
                children: vec![ftd::Element::Markup(ftd::Markups {
                    text: ftd::ftd2021::rendered::markup_line("Hello World"),
                    line: true,
                    common: Box::new(ftd::Common {
                        reference: Some(s("hello-world-variable#hello-world")),
                        ..Default::default()
                    },
                    ..Default::default()
                })],
                ..Default::default()
            },
            ..Default::default()
        }));

        let mut bag = interpreter::default_bag();
        bag.insert(
            s("hello-world#foo"),
            ftd::ftd2021::p2::Thing::Component(ftd::Component {
                root: s("ftd#row"),
                full_name: s("hello-world#foo"),
                arguments: universal_arguments_as_map(),
                instructions: vec![ftd::Instruction::ChildComponent {
                    child: ftd::ChildComponent {
                        events: vec![],
                        root: s("ftd#text"),
                        condition: None,
                        properties: std::iter::IntoIterator::into_iter([(
                            s("text"),
                            ftd::ftd2021::component::Property {
                                default: Some(ftd::PropertyValue::Reference {
                                    name: "hello-world-variable#hello-world".to_string(),
                                    kind: ftd::ftd2021::p2::Kind::caption_or_body(),
                                }),
                                conditions: vec![],
                                ..Default::default()
                            },
                        )])
                        .collect(),
                        ..Default::default()
                    },
                }],
                invocations: vec![],
                ..Default::default()
            }),
        );
        bag.insert(
            s("hello-world-variable#hello-world"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("hello-world"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("Hello World"),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
            }),
        );

        insert_universal_variables_by_count(1, "foo/bar", &mut bag);

        p!(
            "
            -- import: hello-world as hw

            -- hw.foo:
            ",
            (bag, main),
        );
    }

    #[test]
    fn argument_with_default_value() {
        let mut main = p2::default_column();
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("hello world"),
                line: true,
                line_clamp: Some(10),
                common: Box::new(ftd::Common {
                    reference: Some(s("foo/bar#name@0")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("hello"),
                line: true,
                line_clamp: Some(10),
                common: Box::new(ftd::Common {
                    reference: Some(s("foo/bar#name@1")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("this is nice"),
                line: true,
                line_clamp: Some(20),
                common: Box::new(ftd::Common {
                    reference: Some(s("foo/bar#name@2")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        let mut bag = interpreter::default_bag();

        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@0", -1, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@1", -1, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@2", -1, &mut bag);

        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT@0", 0, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT@1", 0, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT@2", 0, &mut bag);

        insert_update_integer_by_root("foo/bar#SIBLING-INDEX-0@0", 0, &mut bag);
        insert_update_integer_by_root("foo/bar#SIBLING-INDEX-0@1", 1, &mut bag);
        insert_update_integer_by_root("foo/bar#SIBLING-INDEX-0@2", 2, &mut bag);

        insert_update_integer_by_root("foo/bar#SIBLING-INDEX@0", 1, &mut bag);
        insert_update_integer_by_root("foo/bar#SIBLING-INDEX@1", 2, &mut bag);
        insert_update_integer_by_root("foo/bar#SIBLING-INDEX@2", 3, &mut bag);

        insert_universal_variables_by_count(3, "foo/bar", &mut bag);

        bag.insert(
            s("foo/bar#foo"),
            ftd::ftd2021::p2::Thing::Component(ftd::Component {
                root: s("ftd#text"),
                full_name: s("foo/bar#foo"),
                arguments: [
                    vec![
                        (
                            s("name"),
                            ftd::ftd2021::p2::Kind::caption().set_default(Some(s("hello world"))),
                        ),
                        (
                            s("line-clamp"),
                            ftd::ftd2021::p2::Kind::Integer {
                                default: Some(s("10")),
                                is_reference: false,
                            },
                        ),
                    ],
                    universal_arguments_as_vec(),
                ]
                .concat()
                .into_iter()
                .collect(),
                properties: std::iter::IntoIterator::into_iter([
                    (
                        s("line-clamp"),
                        ftd::ftd2021::component::Property {
                            default: Some(ftd::PropertyValue::Variable {
                                name: s("line-clamp"),
                                kind: ftd::ftd2021::p2::Kind::Optional {
                                    kind: Box::from(ftd::ftd2021::p2::Kind::Integer {
                                        default: Some(s("10")),
                                        is_reference: false,
                                    }),
                                    is_reference: false,
                                },
                            }),
                            conditions: vec![],
                            ..Default::default()
                        },
                    ),
                    (
                        s("text"),
                        ftd::ftd2021::component::Property {
                            default: Some(ftd::PropertyValue::Variable {
                                name: s("name"),
                                kind: ftd::ftd2021::p2::Kind::caption_or_body()
                                    .set_default(Some(s("hello world"))),
                            }),
                            conditions: vec![],
                            ..Default::default()
                        },
                    ),
                ])
                .collect(),
                invocations: vec![
                    std::iter::IntoIterator::into_iter([
                        (
                            s("name"),
                            ftd::Value::String {
                                text: s("hello world"),
                                source: ftd::TextSource::Default,
                            },
                        ),
                        (s("line-clamp"), ftd::Value::Integer { value: 10 }),
                    ])
                    .collect(),
                    std::iter::IntoIterator::into_iter([
                        (
                            s("name"),
                            ftd::Value::String {
                                text: s("hello"),
                                source: ftd::TextSource::Caption,
                            },
                        ),
                        (s("line-clamp"), ftd::Value::Integer { value: 10 }),
                    ])
                    .collect(),
                    std::iter::IntoIterator::into_iter([
                        (
                            s("name"),
                            ftd::Value::String {
                                text: s("this is nice"),
                                source: ftd::TextSource::Caption,
                            },
                        ),
                        (s("line-clamp"), ftd::Value::Integer { value: 20 }),
                    ])
                    .collect(),
                ],
                line_number: 1,
                ..Default::default()
            }),
        );
        bag.insert(
            s("foo/bar#name@0"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("name"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("hello world"),
                        source: ftd::TextSource::Default,
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#name@1"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("name"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("hello"),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#name@2"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("name"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("this is nice"),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#line-clamp@0"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("line-clamp"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Integer { value: 10 },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#line-clamp@1"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("line-clamp"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Integer { value: 10 },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#line-clamp@2"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("line-clamp"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Integer { value: 20 },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        let (g_bag, g_col) = ftd::ftd2021::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- ftd.text foo:
                caption name: hello world
                integer line-clamp: 10
                text: $name
                line-clamp: $line-clamp

                -- foo:

                -- foo: hello

                -- foo: this is nice
                line-clamp: 20
                "
            ),
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_bag, bag);
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn record_with_default_value() {
        let mut bag = interpreter::default_bag();

        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@0", -1, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT@0", 0, &mut bag);
        insert_update_integer_by_root("foo/bar#SIBLING-INDEX-0@0", 0, &mut bag);
        insert_update_integer_by_root("foo/bar#SIBLING-INDEX@0", 1, &mut bag);

        bag.insert(
            s("foo/bar#abrar"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("abrar"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Record {
                        name: s("foo/bar#person"),
                        fields: std::iter::IntoIterator::into_iter([
                            (
                                s("address"),
                                ftd::PropertyValue::Value {
                                    value: ftd::ftd2021::variable::Value::String {
                                        text: s("Bihar"),
                                        source: ftd::TextSource::Default,
                                    },
                                },
                            ),
                            (
                                s("age"),
                                ftd::PropertyValue::Reference {
                                    name: s("foo/bar#default-age"),
                                    kind: ftd::ftd2021::p2::Kind::Integer {
                                        default: Some(s("$foo/bar#default-age")),
                                        is_reference: false,
                                    },
                                },
                            ),
                            (
                                s("bio"),
                                ftd::PropertyValue::Value {
                                    value: ftd::ftd2021::variable::Value::String {
                                        text: s("Software developer working at fifthtry."),
                                        source: ftd::TextSource::Body,
                                    },
                                },
                            ),
                            (
                                s("name"),
                                ftd::PropertyValue::Reference {
                                    name: s("foo/bar#abrar-name"),
                                    kind: ftd::ftd2021::p2::Kind::caption(),
                                },
                            ),
                            (
                                s("size"),
                                ftd::PropertyValue::Value {
                                    value: ftd::ftd2021::variable::Value::Integer { value: 10 },
                                },
                            ),
                        ])
                        .collect(),
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            s("foo/bar#abrar-name"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("abrar-name"),
                value: ftd::PropertyValue::Value {
                    value: ftd::ftd2021::variable::Value::String {
                        text: s("Abrar Khan"),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            s("foo/bar#default-age"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("default-age"),
                value: ftd::PropertyValue::Value {
                    value: ftd::ftd2021::variable::Value::Integer { value: 20 },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            s("foo/bar#person"),
            ftd::ftd2021::p2::Thing::Record(ftd::ftd2021::p2::Record {
                name: s("foo/bar#person"),
                fields: std::iter::IntoIterator::into_iter([
                    (
                        s("address"),
                        ftd::ftd2021::p2::Kind::string().set_default(Some(s("Bihar"))),
                    ),
                    (
                        s("age"),
                        ftd::ftd2021::p2::Kind::Integer {
                            default: Some(s("$foo/bar#default-age")),
                            is_reference: false,
                        },
                    ),
                    (
                        s("bio"),
                        ftd::ftd2021::p2::Kind::body().set_default(Some(s("Some Bio"))),
                    ),
                    (s("name"), ftd::ftd2021::p2::Kind::caption()),
                    (
                        s("size"),
                        ftd::ftd2021::p2::Kind::Integer {
                            default: Some(s("10")),
                            is_reference: false,
                        },
                    ),
                ])
                .collect(),
                instances: Default::default(),
                order: vec![s("name"), s("address"), s("bio"), s("age"), s("size")],
            }),
        );

        let mut main = p2::default_column();
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line(
                    "Software developer working at fifthtry.",
                ),
                line: true,
                line_clamp: Some(20),
                common: Box::new(ftd::Common {
                    reference: Some(s("foo/bar#abrar.bio")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (g_bag, g_col) = ftd::ftd2021::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- integer default-age: 20

                -- record person:
                caption name:
                string address: Bihar
                body bio: Some Bio
                integer age: $default-age
                integer size: 10

                -- string abrar-name: Abrar Khan

                -- person abrar: $abrar-name

                Software developer working at fifthtry.

                -- ftd.text: $abrar.bio
                line-clamp: $abrar.age
                "
            ),
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_bag, bag);
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn default_with_reference() {
        let mut main = p2::default_column();
        main.container.children.push(ftd::Element::Row(ftd::Row {
            spacing: None,
            container: ftd::Container {
                children: vec![ftd::Element::Markup(ftd::Markups {
                    text: ftd::ftd2021::rendered::markup_line("Arpita"),
                    line: true,
                    line_clamp: Some(10),
                    common: Box::new(ftd::Common {
                        reference: Some(s("foo/bar#name@0")),
                        ..Default::default()
                    },
                    ..Default::default()
                })],
                ..Default::default()
            },
            ..Default::default()
        }));
        main.container.children.push(ftd::Element::Row(ftd::Row {
            spacing: None,
            container: ftd::Container {
                children: vec![ftd::Element::Markup(ftd::Markups {
                    text: ftd::ftd2021::rendered::markup_line("Amit Upadhyay"),
                    line: true,
                    line_clamp: Some(20),
                    common: Box::new(ftd::Common {
                        reference: Some(s("foo/bar#name@1")),
                        ..Default::default()
                    },
                    ..Default::default()
                })],
                ..Default::default()
            },
            ..Default::default()
        }));

        let mut bag = interpreter::default_bag();
        bag.insert(
            s("foo/bar#default-name"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("default-name"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("Arpita"),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            s("foo/bar#default-size"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("default-size"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Integer { value: 10 },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            s("foo/bar#foo"),
            ftd::ftd2021::p2::Thing::Component(ftd::Component {
                root: s("ftd#row"),
                full_name: s("foo/bar#foo"),
                arguments: [
                    vec![
                        (
                            s("name"),
                            ftd::ftd2021::p2::Kind::string()
                                .set_default(Some(s("$foo/bar#default-name"))),
                        ),
                        (
                            s("text-size"),
                            ftd::ftd2021::p2::Kind::Integer {
                                default: Some(s("$foo/bar#default-size")),
                                is_reference: false,
                            },
                        ),
                    ],
                    universal_arguments_as_vec(),
                ]
                .concat()
                .into_iter()
                .collect(),
                instructions: vec![ftd::Instruction::ChildComponent {
                    child: ftd::ChildComponent {
                        events: vec![],
                        root: s("ftd#text"),
                        condition: None,
                        properties: std::iter::IntoIterator::into_iter([
                            (
                                s("line-clamp"),
                                ftd::ftd2021::component::Property {
                                    default: Some(ftd::PropertyValue::Variable {
                                        name: s("text-size"),
                                        kind: ftd::ftd2021::p2::Kind::Optional {
                                            kind: Box::new(ftd::ftd2021::p2::Kind::Integer {
                                                default: Some(s("$foo/bar#default-size")),
                                                is_reference: false,
                                            }),
                                            is_reference: false,
                                        },
                                    }),
                                    conditions: vec![],
                                    ..Default::default()
                                },
                            ),
                            (
                                s("text"),
                                ftd::ftd2021::component::Property {
                                    default: Some(ftd::PropertyValue::Variable {
                                        name: s("name"),
                                        kind: ftd::ftd2021::p2::Kind::caption_or_body()
                                            .set_default(Some(s("$foo/bar#default-name"))),
                                    }),
                                    conditions: vec![],
                                    ..Default::default()
                                },
                            ),
                        ])
                        .collect(),
                        ..Default::default()
                    },
                }],
                kernel: false,
                ..Default::default()
            }),
        );
        bag.insert(
            s("foo/bar#name@0"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("name"),
                value: ftd::PropertyValue::Reference {
                    name: s("foo/bar#default-name"),
                    kind: ftd::ftd2021::p2::Kind::string()
                        .set_default(Some(s("$foo/bar#default-name"))),
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#name@1"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("name"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("Amit Upadhyay"),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#text-size@0"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("text-size"),
                value: ftd::PropertyValue::Reference {
                    name: s("foo/bar#default-size"),
                    kind: ftd::ftd2021::p2::Kind::integer()
                        .set_default(Some(s("$foo/bar#default-size"))),
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#text-size@1"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("text-size"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Integer { value: 20 },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );

        insert_universal_variables_by_count(2, "foo/bar", &mut bag);

        p!(
            "
            -- string default-name: Arpita

            -- integer default-size: 10

            -- ftd.row foo:
            string name: $default-name
            integer text-size: $default-size

            --- ftd.text: $name
            line-clamp: $text-size

            -- foo:

            -- foo:
            name: Amit Upadhyay
            text-size: 20
            ",
            (bag, main),
        );
    }

    #[test]
    fn or_type_with_default_value() {
        let mut main = p2::default_column();
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("Amit Upadhyay"),
                line: true,
                common: Box::new(ftd::Common {
                    reference: Some(s("foo/bar#amitu.name")),
                    ..Default::default()
                },
                ..Default::default()
            }));
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("1000"),
                line: true,
                common: Box::new(ftd::Common {
                    reference: Some(s("foo/bar#amitu.phone")),
                    ..Default::default()
                },
                ..Default::default()
            }));
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("John Doe"),
                line: true,
                line_clamp: Some(50),
                common: Box::new(ftd::Common {
                    reference: Some(s("foo/bar#acme.contact")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        let mut bag = interpreter::default_bag();

        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@0", -1, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@1", -1, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@2", -1, &mut bag);

        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT@0", 0, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT@1", 0, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT@2", 0, &mut bag);

        insert_update_integer_by_root("foo/bar#SIBLING-INDEX-0@0", 0, &mut bag);
        insert_update_integer_by_root("foo/bar#SIBLING-INDEX-0@1", 1, &mut bag);
        insert_update_integer_by_root("foo/bar#SIBLING-INDEX-0@2", 2, &mut bag);

        insert_update_integer_by_root("foo/bar#SIBLING-INDEX@0", 1, &mut bag);
        insert_update_integer_by_root("foo/bar#SIBLING-INDEX@1", 2, &mut bag);
        insert_update_integer_by_root("foo/bar#SIBLING-INDEX@2", 3, &mut bag);

        bag.insert(
            s("foo/bar#acme"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("acme"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::OrType {
                        name: s("foo/bar#lead"),
                        variant: s("company"),
                        fields: std::iter::IntoIterator::into_iter([
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
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            s("foo/bar#amitu"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("amitu"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::OrType {
                        name: s("foo/bar#lead"),
                        variant: s("individual"),
                        fields: std::iter::IntoIterator::into_iter([
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
                                    kind: ftd::ftd2021::p2::Kind::string()
                                        .set_default(Some(s("$foo/bar#default-phone"))),
                                },
                            ),
                        ])
                        .collect(),
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            s("foo/bar#default-phone"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("default-phone"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("1000"),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            s("foo/bar#lead"),
            ftd::ftd2021::p2::Thing::OrType(ftd::ftd2021::OrType {
                name: s("foo/bar#lead"),
                variants: vec![
                    ftd::ftd2021::p2::Record {
                        name: s("foo/bar#lead.individual"),
                        fields: std::iter::IntoIterator::into_iter([
                            (s("name"), ftd::ftd2021::p2::Kind::caption()),
                            (
                                s("phone"),
                                ftd::ftd2021::p2::Kind::string()
                                    .set_default(Some(s("$foo/bar#default-phone"))),
                            ),
                        ])
                        .collect(),
                        instances: Default::default(),
                        order: vec![s("name"), s("phone")],
                    },
                    ftd::ftd2021::p2::Record {
                        name: s("foo/bar#lead.company"),
                        fields: std::iter::IntoIterator::into_iter([
                            (
                                s("contact"),
                                ftd::ftd2021::p2::Kind::string().set_default(Some(s("1001"))),
                            ),
                            (s("fax"), ftd::ftd2021::p2::Kind::string()),
                            (s("name"), ftd::ftd2021::p2::Kind::caption()),
                            (
                                s("no-of-employees"),
                                ftd::ftd2021::p2::Kind::integer().set_default(Some(s("50"))),
                            ),
                        ])
                        .collect(),
                        instances: Default::default(),
                        order: vec![s("name"), s("contact"), s("fax"), s("no-of-employees")],
                    },
                ],
            }),
        );

        let (g_bag, g_col) = ftd::ftd2021::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- string default-phone: 1000

                -- or-type lead:

                --- individual:
                caption name:
                string phone: $default-phone

                --- company:
                caption name:
                string contact: 1001
                string fax:
                integer no-of-employees: 50

                -- lead.individual amitu: Amit Upadhyay

                -- lead.company acme: Acme Inc.
                contact: John Doe
                fax: +1-234-567890

                -- ftd.text: $amitu.name

                -- ftd.text: $amitu.phone

                -- ftd.text: $acme.contact
                line-clamp: $acme.no-of-employees

                "
            ),
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_bag, bag);
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn default_id() {
        let mut main = p2::default_column();

        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![ftd::Element::Column(ftd::Column {
                        spacing: None,
                        container: ftd::Container {
                            children: vec![ftd::Element::Row(ftd::Row {
                                spacing: None,
                                container: ftd::Container {
                                    children: vec![ftd::Element::Column(ftd::Column {
                                        spacing: None,
                                        container: ftd::Container {
                                            children: vec![ftd::Element::Markup(ftd::Markups {
                                                text: ftd::ftd2021::rendered::markup_line("hello"),
                                                line: true,
                                                ..Default::default()
                                            })],
                                            ..Default::default()
                                        },
                                        common: Box::new(ftd::Common {
                                            data_id: Some(s("display-text-id")),
                                            ..Default::default()
                                        },
                                    })],
                                    ..Default::default()
                                },
                                ..Default::default()
                            })],
                            ..Default::default()
                        },
                        common: Box::new(ftd::Common {
                            data_id: Some(s("inside-page-id")),
                            ..Default::default()
                        },
                    })],
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Column(ftd::Column {
                            spacing: None,
                            container: ftd::Container {
                                children: vec![ftd::Element::Row(ftd::Row {
                                    spacing: None,
                                    container: ftd::Container {
                                        children: vec![ftd::Element::Column(ftd::Column {
                                            spacing: None,
                                            container: ftd::Container {
                                                children: vec![ftd::Element::Markup(
                                                    ftd::Markups {
                                                        text: ftd::ftd2021::rendered::markup_line(
                                                            "hello",
                                                        ),
                                                        line: true,
                                                        ..Default::default()
                                                    },
                                                )],
                                                ..Default::default()
                                            },
                                            common: Box::new(ftd::Common {
                                                data_id: Some(s("display-text-id")),
                                                id: Some(s(
                                                    "page-id:inside-page-id:display-text-id",
                                                )),
                                                ..Default::default()
                                            },
                                        })],
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                })],
                                ..Default::default()
                            },
                            common: Box::new(ftd::Common {
                                data_id: Some(s("inside-page-id")),
                                id: Some(s("page-id:inside-page-id")),
                                ..Default::default()
                            },
                        }),
                        ftd::Element::Row(ftd::Row {
                            spacing: None,
                            common: Box::new(ftd::Common {
                                data_id: Some(s("page-id-row")),
                                id: Some(s("page-id-row")),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    ],
                    ..Default::default()
                },
                common: Box::new(ftd::Common {
                    data_id: Some(s("page-id")),
                    id: Some(s("page-id")),
                    ..Default::default()
                },
            }));

        main.container.children.push(ftd::Element::Row(ftd::Row {
            spacing: None,
            ..Default::default()
        }));

        let (_g_bag, g_col) = ftd::ftd2021::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- ftd.column display-text:

                --- ftd.text: hello


                -- ftd.column inside-page:

                --- ftd.row:

                --- display-text:
                id: display-text-id


                -- ftd.column page:

                --- inside-page:
                id: inside-page-id

                -- page:

                -- page:
                id: page-id

                -- ftd.row:

                -- container: page-id

                -- ftd.row:
                id: page-id-row

                "
            ),
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    #[ignore]
    fn region_h1() {
        let mut main = p2::default_column();

        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![ftd::Element::Markup(ftd::Markups {
                        text: ftd::ftd2021::rendered::markup_line("Heading 31"),
                        line: true,
                        common: Box::new(ftd::Common {
                            region: Some(ftd::Region::Title),
                            reference: Some(s("foo/bar#title@0")),
                            ..Default::default()
                        },
                        ..Default::default()
                    })],
                    ..Default::default()
                },
                common: Box::new(ftd::Common {
                    region: Some(ftd::Region::H3),
                    id: Some(s("heading-31")),
                    ..Default::default()
                },
            }));

        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::ftd2021::rendered::markup_line("Heading 11"),
                            line: true,
                            common: Box::new(ftd::Common {
                                region: Some(ftd::Region::Title),
                                reference: Some(s("foo/bar#title@1")),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Column(ftd::Column {
                            spacing: None,
                            container: ftd::Container {
                                children: vec![
                                    ftd::Element::Markup(ftd::Markups {
                                        text: ftd::ftd2021::rendered::markup_line("Heading 21"),
                                        line: true,
                                        common: Box::new(ftd::Common {
                                            region: Some(ftd::Region::Title),
                                            reference: Some(s("foo/bar#title@2")),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    }),
                                    ftd::Element::Column(ftd::Column {
                                        spacing: None,
                                        container: ftd::Container {
                                            children: vec![
                                                ftd::Element::Markup(ftd::Markups {
                                                    text: ftd::ftd2021::rendered::markup_line(
                                                        "Heading 32",
                                                    ),
                                                    line: true,
                                                    common: Box::new(ftd::Common {
                                                        region: Some(ftd::Region::Title),
                                                        reference: Some(s("foo/bar#title@3")),
                                                        ..Default::default()
                                                    },
                                                    ..Default::default()
                                                }),
                                                ftd::Element::Markup(ftd::Markups {
                                                    text: ftd::ftd2021::rendered::markup_line(
                                                        "hello",
                                                    ),
                                                    line: true,
                                                    ..Default::default()
                                                }),
                                            ],
                                            ..Default::default()
                                        },
                                        common: Box::new(ftd::Common {
                                            region: Some(ftd::Region::H3),
                                            id: Some(s("heading-32")),
                                            ..Default::default()
                                        },
                                    }),
                                ],
                                ..Default::default()
                            },
                            common: Box::new(ftd::Common {
                                region: Some(ftd::Region::H2),
                                id: Some(s("heading-21")),
                                ..Default::default()
                            },
                        }),
                        ftd::Element::Column(ftd::Column {
                            spacing: None,
                            container: ftd::Container {
                                children: vec![ftd::Element::Markup(ftd::Markups {
                                    text: ftd::ftd2021::rendered::markup_line("Heading 22"),
                                    line: true,
                                    common: Box::new(ftd::Common {
                                        reference: Some(s("foo/bar#title@5")),
                                        region: Some(ftd::Region::Title),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                })],
                                ..Default::default()
                            },
                            common: Box::new(ftd::Common {
                                region: Some(ftd::Region::H2),
                                id: Some(s("heading-22")),
                                ..Default::default()
                            },
                        }),
                        ftd::Element::Column(ftd::Column {
                            spacing: None,
                            container: ftd::Container {
                                children: vec![ftd::Element::Markup(ftd::Markups {
                                    text: ftd::ftd2021::rendered::markup_line("Heading 23"),
                                    line: true,
                                    common: Box::new(ftd::Common {
                                        region: Some(ftd::Region::Title),
                                        reference: Some(s("foo/bar#title@6")),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                })],
                                ..Default::default()
                            },
                            common: Box::new(ftd::Common {
                                region: Some(ftd::Region::H2),
                                id: Some(s("heading-23")),
                                ..Default::default()
                            },
                        }),
                    ],
                    ..Default::default()
                },
                common: Box::new(ftd::Common {
                    region: Some(ftd::Region::H1),
                    id: Some(s("heading-11")),
                    ..Default::default()
                },
            }));

        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::ftd2021::rendered::markup_line("Heading 12"),
                            line: true,
                            common: Box::new(ftd::Common {
                                reference: Some(s("foo/bar#title@7")),
                                region: Some(ftd::Region::Title),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Column(ftd::Column {
                            spacing: None,
                            container: ftd::Container {
                                children: vec![ftd::Element::Markup(ftd::Markups {
                                    text: ftd::ftd2021::rendered::markup_line("Heading 33"),
                                    line: true,
                                    common: Box::new(ftd::Common {
                                        reference: Some(s("foo/bar#title@8")),
                                        region: Some(ftd::Region::Title),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                })],
                                ..Default::default()
                            },
                            common: Box::new(ftd::Common {
                                region: Some(ftd::Region::H3),
                                id: Some(s("heading-33")),
                                ..Default::default()
                            },
                        }),
                        ftd::Element::Column(ftd::Column {
                            spacing: None,
                            container: ftd::Container {
                                children: vec![ftd::Element::Markup(ftd::Markups {
                                    text: ftd::ftd2021::rendered::markup_line("Heading 24"),
                                    line: true,
                                    common: Box::new(ftd::Common {
                                        reference: Some(s("foo/bar#title@9")),
                                        region: Some(ftd::Region::Title),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                })],
                                ..Default::default()
                            },
                            common: Box::new(ftd::Common {
                                region: Some(ftd::Region::H2),
                                id: Some(s("heading-24")),
                                ..Default::default()
                            },
                        }),
                    ],
                    ..Default::default()
                },
                common: Box::new(ftd::Common {
                    region: Some(ftd::Region::H1),
                    id: Some(s("heading-12")),
                    ..Default::default()
                },
            }));

        let (_g_bag, g_col) = ftd::ftd2021::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- ftd.column h1:
                region: h1
                caption title:

                --- ftd.text:
                text: $title
                caption title:
                region: title

                -- ftd.column h2:
                region: h2
                caption title:

                --- ftd.text:
                text: $title
                caption title:
                region: title

                -- ftd.column h3:
                region: h3
                caption title:

                --- ftd.text:
                text: $title
                caption title:
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
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn event_onclick() {
        let mut main = p2::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::ftd2021::rendered::markup_line("Mobile"),
                            line: true,
                            common: Box::new(ftd::Common {
                                condition: Some(ftd::Condition {
                                    variable: s("foo/bar#mobile"),
                                    value: serde_json::Value::Bool(true),
                                }),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::ftd2021::rendered::markup_line("Desktop"),
                            line: true,
                            common: Box::new(ftd::Common {
                                condition: Some(ftd::Condition {
                                    variable: s("foo/bar#mobile"),
                                    value: serde_json::Value::Bool(false),
                                }),
                                is_not_visible: true,
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
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("Click Here!"),
                line: true,
                common: Box::new(ftd::Common {
                    events: vec![ftd::Event {
                        name: s("onclick"),
                        action: ftd::Action {
                            action: s("toggle"),
                            target: s("foo/bar#mobile"),
                            parameters: Default::default(),
                        }),
                    }],
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (_g_bag, g_col) = ftd::ftd2021::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- boolean mobile: true

                -- ftd.column foo:

                --- ftd.text: Mobile
                if: $mobile

                --- ftd.text: Desktop
                if: not $mobile

                -- foo:

                -- ftd.text: Click Here!
                $on-click$: toggle $mobile
                "
            ),
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn event_toggle_with_local_variable() {
        let mut main = p2::default_column();
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("Hello"),
                line: true,
                common: Box::new(ftd::Common {
                    reference: Some(s("foo/bar#name@0")),
                    condition: Some(ftd::Condition {
                        variable: s("foo/bar#open@0"),
                        value: serde_json::Value::Bool(true),
                    }),
                    events: vec![ftd::Event {
                        name: s("onclick"),
                        action: ftd::Action {
                            action: s("toggle"),
                            target: s("foo/bar#open@0"),
                            parameters: Default::default(),
                        }),
                    }],
                    ..Default::default()
                },
                ..Default::default()
            }));

        let mut bag = interpreter::default_bag();

        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@0", -1, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT@0", 0, &mut bag);
        insert_update_integer_by_root("foo/bar#SIBLING-INDEX-0@0", 0, &mut bag);
        insert_update_integer_by_root("foo/bar#SIBLING-INDEX@0", 1, &mut bag);

        bag.insert(
            s("foo/bar#foo"),
            ftd::ftd2021::p2::Thing::Component(ftd::Component {
                root: "ftd#text".to_string(),
                full_name: "foo/bar#foo".to_string(),
                arguments: [
                    vec![
                        (s("name"), ftd::ftd2021::p2::Kind::caption()),
                        (
                            s("open"),
                            ftd::ftd2021::p2::Kind::boolean().set_default(Some(s("true"))),
                        ),
                    ],
                    universal_arguments_as_vec(),
                ]
                .concat()
                .into_iter()
                .collect(),
                properties: std::iter::IntoIterator::into_iter([(
                    s("text"),
                    ftd::ftd2021::component::Property {
                        default: Some(ftd::PropertyValue::Variable {
                            name: s("name"),
                            kind: ftd::ftd2021::p2::Kind::String {
                                caption: true,
                                body: true,
                                default: None,
                                is_reference: false,
                            },
                        }),
                        ..Default::default()
                    },
                )])
                .collect(),
                instructions: vec![],
                events: vec![ftd::ftd2021::p2::Event {
                    name: ftd::ftd2021::p2::EventName::OnClick,
                    action: ftd::ftd2021::p2::Action {
                        action: ftd::ftd2021::p2::ActionKind::Toggle,
                        target: ftd::PropertyValue::Variable {
                            name: s("open"),
                            kind: ftd::ftd2021::p2::Kind::boolean().set_default(Some(s("true"))),
                        },
                        parameters: Default::default(),
                    },
                }],
                condition: Some(ftd::ftd2021::p2::Boolean::Equal {
                    left: ftd::PropertyValue::Variable {
                        name: s("open"),
                        kind: ftd::ftd2021::p2::Kind::boolean().set_default(Some(s("true"))),
                    },
                    right: ftd::PropertyValue::Value {
                        value: ftd::ftd2021::variable::Value::Boolean { value: true },
                    },
                }),
                kernel: false,
                invocations: vec![
                    std::iter::IntoIterator::into_iter([
                        (
                            s("name"),
                            ftd::Value::String {
                                text: s("Hello"),
                                source: ftd::TextSource::Caption,
                            },
                        ),
                        (s("open"), ftd::Value::Boolean { value: true }),
                    ])
                    .collect(),
                ],
                line_number: 1,
                ..Default::default()
            }),
        );
        bag.insert(
            s("foo/bar#name@0"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("name"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("Hello"),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#open@0"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("open"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Boolean { value: true },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );

        insert_universal_variables_by_count(1, "foo/bar", &mut bag);

        let (g_bag, g_col) = ftd::ftd2021::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- ftd.text foo:
                caption name:
                boolean open: true
                text: $name
                if: $open
                $on-click$: toggle $open

                -- foo: Hello
                "
            ),
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
        pretty_assertions::assert_eq!(g_bag, bag);
    }

    #[test]
    fn event_toggle_with_local_variable_for_component() {
        let mut main = p2::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::ftd2021::rendered::markup_line("Click here"),
                            line: true,
                            common: Box::new(ftd::Common {
                                events: vec![ftd::Event {
                                    name: s("onclick"),
                                    action: ftd::Action {
                                        action: s("toggle"),
                                        target: s("foo/bar#open@0"),
                                        parameters: Default::default(),
                                    }),
                                }],
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::ftd2021::rendered::markup_line("Open True"),
                            line: true,
                            common: Box::new(ftd::Common {
                                condition: Some(ftd::Condition {
                                    variable: s("foo/bar#open@0"),
                                    value: serde_json::Value::Bool(true),
                                }),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::ftd2021::rendered::markup_line("Open False"),
                            line: true,
                            common: Box::new(ftd::Common {
                                condition: Some(ftd::Condition {
                                    variable: s("foo/bar#open@0"),
                                    value: serde_json::Value::Bool(false),
                                }),
                                is_not_visible: true,
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    ],
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (_g_bag, g_col) = ftd::ftd2021::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- ftd.column foo:
                boolean open: true

                --- ftd.text: Click here
                $on-click$: toggle $open

                --- ftd.text: Open True
                if: $open

                --- ftd.text: Open False
                if: not $open

                -- foo:
                "
            ),
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn event_toggle_for_loop() {
        let mut main = p2::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::ftd2021::rendered::markup_line("ab title"),
                            line: true,
                            common: Box::new(ftd::Common {
                                events: vec![ftd::Event {
                                    name: s("onclick"),
                                    action: ftd::Action {
                                        action: s("toggle"),
                                        target: s("foo/bar#open@0"),
                                        parameters: Default::default(),
                                    }),
                                }],
                                reference: Some(s("foo/bar#toc@0.title")),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Column(ftd::Column {
                            spacing: None,
                            container: ftd::Container {
                                children: vec![ftd::Element::Markup(ftd::Markups {
                                    text: ftd::ftd2021::rendered::markup_line("aa title"),
                                    line: true,
                                    common: Box::new(ftd::Common {
                                        events: vec![ftd::Event {
                                            name: s("onclick"),
                                            action: ftd::Action {
                                                action: s("toggle"),
                                                target: s("foo/bar#open@0,1"),
                                                parameters: Default::default(),
                                            }),
                                        }],
                                        reference: Some(s("foo/bar#toc@0,1.title")),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                })],
                                ..Default::default()
                            },
                            common: Box::new(ftd::Common {
                                condition: Some(ftd::Condition {
                                    variable: s("foo/bar#open@0"),
                                    value: serde_json::Value::Bool(true),
                                }),
                                ..Default::default()
                            },
                        }),
                        ftd::Element::Column(ftd::Column {
                            spacing: None,
                            container: ftd::Container {
                                children: vec![ftd::Element::Markup(ftd::Markups {
                                    text: ftd::ftd2021::rendered::markup_line("aaa title"),
                                    line: true,
                                    common: Box::new(ftd::Common {
                                        events: vec![ftd::Event {
                                            name: s("onclick"),
                                            action: ftd::Action {
                                                action: s("toggle"),
                                                target: s("foo/bar#open@0,2"),
                                                parameters: Default::default(),
                                            }),
                                        }],
                                        reference: Some(s("foo/bar#toc@0,2.title")),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                })],
                                ..Default::default()
                            },
                            common: Box::new(ftd::Common {
                                condition: Some(ftd::Condition {
                                    variable: s("foo/bar#open@0"),
                                    value: serde_json::Value::Bool(true),
                                }),
                                ..Default::default()
                            },
                        }),
                    ],
                    ..Default::default()
                },
                common: Box::new(ftd::Common {
                    reference: Some(s("foo/bar#toc")),
                    ..Default::default()
                },
            }));

        let (_g_bag, g_col) = ftd::ftd2021::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- record toc-record:
                string title:
                toc-record list children:

                -- ftd.column toc-item:
                toc-record toc:
                boolean open: true

                --- ftd.text: $toc.title
                $on-click$: toggle $open

                --- toc-item:
                if: $open
                $loop$: $toc.children as $obj
                toc: $obj

                -- toc-record list aa:

                -- aa:
                title: aa title

                -- aa:
                title: aaa title

                -- toc-record list toc:

                -- toc:
                title: ab title
                children: $aa

                -- toc-item:
                $loop$: $toc as $obj
                toc: $obj
                "
            ),
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn test_local_variable() {
        let mut main = p2::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![ftd::Element::Column(ftd::Column {
                        spacing: None,
                        container: ftd::Container {
                            children: vec![
                                ftd::Element::Column(ftd::Column {
                                    spacing: None,
                                    container: ftd::Container {
                                        children: vec![
                                            ftd::Element::Markup(ftd::Markups {
                                                text: ftd::ftd2021::rendered::markup_line(
                                                    "Click here!",
                                                ),
                                                line: true,
                                                common: Box::new(ftd::Common {
                                                    events: vec![ftd::Event {
                                                        name: s("onclick"),
                                                        action: ftd::Action {
                                                            action: s("toggle"),
                                                            target: s("foo/bar#open@0"),
                                                            parameters: Default::default(),
                                                        }),
                                                    }],
                                                    ..Default::default()
                                                },
                                                ..Default::default()
                                            }),
                                            ftd::Element::Markup(ftd::Markups {
                                                text: ftd::ftd2021::rendered::markup_line("Hello"),
                                                line: true,
                                                ..Default::default()
                                            }),
                                        ],
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                }),
                                ftd::Element::Column(ftd::Column {
                                    spacing: None,
                                    container: ftd::Container {
                                        children: vec![ftd::Element::Markup(ftd::Markups {
                                            text: ftd::ftd2021::rendered::markup_line("Hello Bar"),
                                            line: true,
                                            ..Default::default()
                                        })],
                                        ..Default::default()
                                    },
                                    common: Box::new(ftd::Common {
                                        condition: Some(ftd::Condition {
                                            variable: s("foo/bar#open@0"),
                                            value: serde_json::Value::Bool(true),
                                        }),
                                        ..Default::default()
                                    },
                                }),
                            ],
                            ..Default::default()
                        },
                        common: Box::new(ftd::Common {
                            data_id: Some(s("foo-id")),
                            ..Default::default()
                        },
                    })],
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (_g_bag, g_col) = ftd::ftd2021::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- ftd.column bar:
                boolean open-bar: true

                --- ftd.text: Hello Bar


                -- ftd.column foo:
                boolean open: true

                --- ftd.column:
                id: foo-id

                --- ftd.column:

                --- ftd.text: Click here!
                $on-click$: toggle $open

                --- ftd.text: Hello

                --- container: foo-id

                --- bar:
                if: $open


                -- foo:
                "
            ),
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .expect("found error");
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn if_on_var_integer() {
        let mut main = p2::default_column();
        main.container
            .children
            .push(ftd::Element::Integer(ftd::Text {
                text: ftd::ftd2021::rendered::markup_line("20"),
                common: Box::new(ftd::Common {
                    reference: Some(s("foo/bar#bar")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (_g_bag, g_col) = ftd::ftd2021::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- boolean foo: false

                -- integer bar: 10

                -- bar: 20
                if: not $foo

                -- ftd.integer:
                value: $bar

                "
            ),
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn if_on_var_text() {
        let mut main = p2::default_column();
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("other-foo says hello"),
                line: true,
                common: Box::new(ftd::Common {
                    reference: Some(s("foo/bar#bar")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (_g_bag, g_col) = ftd::ftd2021::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- boolean foo: false

                -- boolean other-foo: true

                -- string bar: hello

                -- bar: foo says hello
                if: not $foo

                -- bar: other-foo says hello
                if: $other-foo

                -- ftd.text: $bar

                "
            ),
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn cursor_pointer() {
        let mut main = p2::default_column();
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("hello"),
                line: true,
                common: Box::new(ftd::Common {
                    cursor: Some(s("pointer")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (_g_bag, g_col) = ftd::ftd2021::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- ftd.text: hello
                cursor: pointer

                "
            ),
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn comments() {
        let mut main = p2::default_column();
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("hello2"),
                line: true,
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("/hello3"),
                line: true,
                common: Box::new(ftd::Common {
                    color: Some(ftd::Color {
                        light: ftd::ColorValue {
                            r: 255,
                            g: 0,
                            b: 0,
                            alpha: 1.0,
                        }),
                        dark: ftd::ColorValue {
                            r: 255,
                            g: 0,
                            b: 0,
                            alpha: 1.0,
                        }),
                        reference: Some(s("foo/bar#red")),
                    }),
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container.children.push(ftd::Element::Row(ftd::Row {
            spacing: None,
            container: ftd::Container {
                children: vec![ftd::Element::Markup(ftd::Markups {
                    text: ftd::ftd2021::rendered::markup_line("hello5"),
                    line: true,
                    common: Box::new(ftd::Common {
                        color: Some(ftd::Color {
                            light: ftd::ColorValue {
                                r: 0,
                                g: 128,
                                b: 0,
                                alpha: 1.0,
                            }),
                            dark: ftd::ColorValue {
                                r: 0,
                                g: 128,
                                b: 0,
                                alpha: 1.0,
                            }),
                            reference: Some(s("foo/bar#green")),
                        }),
                        ..Default::default()
                    },
                    ..Default::default()
                })],
                ..Default::default()
            },
            ..Default::default()
        }));

        main.container.children.push(ftd::Element::Row(ftd::Row {
            spacing: None,
            container: ftd::Container {
                children: vec![ftd::Element::Markup(ftd::Markups {
                    text: ftd::ftd2021::rendered::markup_line("/foo says hello"),
                    line: true,
                    ..Default::default()
                })],
                ..Default::default()
            },
            ..Default::default()
        }));

        let (_g_bag, g_col) = ftd::ftd2021::test::interpret(
            "foo/bar",
            indoc::indoc!(
                r"
                -- ftd.color red: red
                dark: red

                -- ftd.color green: green
                dark: green

                /-- ftd.text:
                cursor: pointer

                hello1

                -- ftd.text:
                /color: red

                hello2

                -- ftd.text:
                color: $red

                \/hello3

                -- ftd.row:

                /--- ftd.text: hello4

                --- ftd.text: hello5
                color: $green
                /padding-left: 20

                -- ftd.row foo:
                /color: red

                --- ftd.text:

                \/foo says hello

                /--- ftd.text: foo says hello again

                -- foo:

                /-- foo:
                "
            ),
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn component_declaration_anywhere_2() {
        let mut main = p2::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Column(ftd::Column {
                            spacing: None,
                            container: ftd::Container {
                                children: vec![
                                    ftd::Element::Markup(ftd::Markups {
                                        text: ftd::ftd2021::rendered::markup_line("Bar says hello"),
                                        line: true,
                                        common: Box::new(ftd::Common {
                                            reference: Some(s("foo/bar#name@0,0")),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    }),
                                    ftd::Element::Markup(ftd::Markups {
                                        text: ftd::ftd2021::rendered::markup_line("Hello"),
                                        line: true,
                                        common: Box::new(ftd::Common {
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
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::ftd2021::rendered::markup_line("foo says hello"),
                            line: true,
                            ..Default::default()
                        }),
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::ftd2021::rendered::markup_line("Hello"),
                            line: true,
                            common: Box::new(ftd::Common {
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

        let (_g_bag, g_col) = ftd::ftd2021::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- foo:

                -- ftd.column foo:

                --- bar: Bar says hello

                --- ftd.text: foo says hello

                --- ftd.text: $greeting

                -- string greeting: Hello

                -- ftd.column bar:
                caption name:

                --- ftd.text: $name

                --- ftd.text: $greeting
                "
            ),
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn action_increment_decrement_condition_1() {
        let mut main = p2::default_column();
        main.container
            .children
            .push(ftd::Element::Integer(ftd::Text {
                text: ftd::ftd2021::rendered::markup_line("0"),
                common: Box::new(ftd::Common {
                    reference: Some(s("foo/bar#count")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("Hello on 8"),
                line: true,
                common: Box::new(ftd::Common {
                    condition: Some(ftd::Condition {
                        variable: s("foo/bar#count"),
                        value: serde_json::Value::from(8),
                    }),
                    is_not_visible: true,
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("increment counter"),
                line: true,
                common: Box::new(ftd::Common {
                    events: vec![ftd::Event {
                        name: s("onclick"),
                        action: ftd::Action {
                            action: s("increment"),
                            target: s("foo/bar#count"),
                            parameters: Default::default(),
                        }),
                    }],
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("decrement counter"),
                line: true,
                common: Box::new(ftd::Common {
                    events: vec![ftd::Event {
                        name: s("onclick"),
                        action: ftd::Action {
                            action: s("decrement"),
                            target: s("foo/bar#count"),
                            parameters: Default::default(),
                        }),
                    }],
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("increment counter"),
                line: true,
                common: Box::new(ftd::Common {
                    events: vec![ftd::Event {
                        name: s("onclick"),
                        action: ftd::Action {
                            action: s("increment"),
                            target: s("foo/bar#count"),
                            parameters: std::iter::IntoIterator::into_iter([(
                                s("by"),
                                vec![ftd::ftd2021::event::ParameterData {
                                    value: serde_json::Value::from(2),
                                    reference: None,
                                }],
                            )])
                            .collect(),
                        }),
                    }],
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("increment counter by 2 clamp 2 10"),
                line: true,
                common: Box::new(ftd::Common {
                    events: vec![ftd::Event {
                        name: s("onclick"),
                        action: ftd::Action {
                            action: s("increment"),
                            target: s("foo/bar#count"),
                            parameters: std::iter::IntoIterator::into_iter([
                                (
                                    s("by"),
                                    vec![ftd::ftd2021::event::ParameterData {
                                        value: serde_json::Value::from(2),
                                        reference: None,
                                    }],
                                ),
                                (
                                    s("clamp"),
                                    vec![
                                        ftd::ftd2021::event::ParameterData {
                                            value: serde_json::Value::from(2),
                                            reference: None,
                                        }),
                                        ftd::ftd2021::event::ParameterData {
                                            value: serde_json::Value::from(10),
                                            reference: None,
                                        }),
                                    ],
                                ),
                            ])
                            .collect(),
                        }),
                    }],
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("decrement count clamp 2 10"),
                line: true,
                common: Box::new(ftd::Common {
                    events: vec![ftd::Event {
                        name: s("onclick"),
                        action: ftd::Action {
                            action: s("decrement"),
                            target: s("foo/bar#count"),
                            parameters: std::iter::IntoIterator::into_iter([(
                                s("clamp"),
                                vec![
                                    ftd::ftd2021::event::ParameterData {
                                        value: serde_json::Value::from(2),
                                        reference: None,
                                    }),
                                    ftd::ftd2021::event::ParameterData {
                                        value: serde_json::Value::from(10),
                                        reference: None,
                                    }),
                                ],
                            )])
                            .collect(),
                        }),
                    }],
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (_g_bag, g_col) = ftd::ftd2021::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- integer count: 0

                -- ftd.integer:
                value: $count

                -- ftd.text: Hello on 8
                if: $count == 8

                -- ftd.text: increment counter
                $on-click$: increment $count

                -- ftd.text: decrement counter
                $on-click$: decrement $count

                -- ftd.text: increment counter
                $on-click$: increment $count by 2

                -- ftd.text: increment counter by 2 clamp 2 10
                $on-click$: increment $count by 2 clamp 2 10

                -- ftd.text: decrement count clamp 2 10
                $on-click$: decrement $count clamp 2 10
                "
            ),
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .expect("found error");
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn action_increment_decrement_local_variable() {
        let mut main = p2::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Integer(ftd::Text {
                            text: ftd::ftd2021::rendered::markup_line("0"),
                            common: Box::new(ftd::Common {
                                reference: Some(s("foo/bar#count@0")),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::ftd2021::rendered::markup_line("increment counter"),
                            line: true,
                            common: Box::new(ftd::Common {
                                events: vec![ftd::Event {
                                    name: s("onclick"),
                                    action: ftd::Action {
                                        action: s("increment"),
                                        target: s("foo/bar#count@0"),
                                        parameters: std::iter::IntoIterator::into_iter([(
                                            s("by"),
                                            vec![ftd::ftd2021::event::ParameterData {
                                                value: serde_json::Value::from(3),
                                                reference: Some(s("foo/bar#by@0")),
                                            }],
                                        )])
                                        .collect(),
                                    }),
                                }],
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::ftd2021::rendered::markup_line("decrement counter"),
                            line: true,
                            common: Box::new(ftd::Common {
                                events: vec![ftd::Event {
                                    name: s("onclick"),
                                    action: ftd::Action {
                                        action: s("decrement"),
                                        target: s("foo/bar#count@0"),
                                        parameters: std::iter::IntoIterator::into_iter([(
                                            s("by"),
                                            vec![ftd::ftd2021::event::ParameterData {
                                                value: serde_json::Value::from(2),
                                                reference: Some(s("foo/bar#decrement-by")),
                                            }],
                                        )])
                                        .collect(),
                                    }),
                                }],
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    ],
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (_g_bag, g_col) = ftd::ftd2021::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- integer decrement-by: 2

                -- ftd.column foo:
                integer by: 4
                integer count: 0

                --- ftd.integer:
                value: $count

                --- ftd.text: increment counter
                $on-click$: increment $count by $by

                --- ftd.text: decrement counter
                $on-click$: decrement $count by $decrement-by

                -- foo:
                by: 3

                "
            ),
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .expect("found error");
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn nested_component() {
        let mut main = p2::default_column();
        main.container.children.push(ftd::Element::Row(ftd::Row {
            spacing: None,
            container: ftd::Container {
                children: vec![ftd::Element::Markup(ftd::Markups {
                    text: ftd::ftd2021::rendered::markup_line("CTA says Hello"),
                    line: true,
                    common: Box::new(ftd::Common {
                        reference: Some(s("foo/bar#cta@0")),
                        ..Default::default()
                    },
                    ..Default::default()
                })],
                ..Default::default()
            },
            ..Default::default()
        }));

        let (_g_bag, g_col) = ftd::ftd2021::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- secondary-button: CTA says Hello

                -- secondary-button-1 secondary-button:
                caption cta:
                cta: $cta


                -- ftd.row secondary-button-1:
                caption cta:

                --- ftd.text: $cta
                "
            ),
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn action_increment_decrement_on_component() {
        let mut main = p2::default_column();
        main.container
            .children
            .push(ftd::Element::Image(ftd::Image {
                src: i("https://www.liveabout.com/thmb/YCJmu1khSJo8kMYM090QCd9W78U=/1250x0/filters:no_upscale():max_bytes(150000):strip_icc():format(webp)/powerpuff_girls-56a00bc45f9b58eba4aea61d.jpg", Some(s("foo/bar#src@0"))),
                common: Box::new(ftd::Common {
                    condition: Some(
                        ftd::Condition {
                            variable: s("foo/bar#count"),
                            value: serde_json::Value::from(0),
                        }),
                    ),
                    is_not_visible: false,
                    events: vec![
                        ftd::Event {
                            name: s("onclick"),
                            action: ftd::Action {
                                action: s("increment"),
                                target: s("foo/bar#count"),
                                parameters: std::iter::IntoIterator::into_iter([(s("clamp"), vec![ftd::ftd2021::event::ParameterData {
                                    value: serde_json::Value::from(0),
                                    reference: None,
                                }, ftd::ftd2021::event::ParameterData {
                                    value: serde_json::Value::from(1),
                                    reference: None,
                                }])])
                                    .collect(),
                            }),
                        }),
                    ],
                    reference: Some(s("foo/bar#src@0")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Image(ftd::Image {
                src: i(
                    "https://upload.wikimedia.org/wikipedia/en/d/d4/Mickey_Mouse.png",
                    Some(s("foo/bar#src@1")),
                ),
                common: Box::new(ftd::Common {
                    condition: Some(ftd::Condition {
                        variable: s("foo/bar#count"),
                        value: serde_json::Value::from(1),
                    }),
                    is_not_visible: true,
                    events: vec![ftd::Event {
                        name: s("onclick"),
                        action: ftd::Action {
                            action: s("increment"),
                            target: s("foo/bar#count"),
                            parameters: std::iter::IntoIterator::into_iter([(
                                s("clamp"),
                                vec![
                                    ftd::ftd2021::event::ParameterData {
                                        value: serde_json::Value::from(0),
                                        reference: None,
                                    }),
                                    ftd::ftd2021::event::ParameterData {
                                        value: serde_json::Value::from(1),
                                        reference: None,
                                    }),
                                ],
                            )])
                            .collect(),
                        }),
                    }],
                    reference: Some(s("foo/bar#src@1")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (_g_bag, g_col) = ftd::ftd2021::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- integer count: 0

                -- ftd.image-src src0:
                light: https://www.liveabout.com/thmb/YCJmu1khSJo8kMYM090QCd9W78U=/1250x0/filters:no_upscale():max_bytes(150000):strip_icc():format(webp)/powerpuff_girls-56a00bc45f9b58eba4aea61d.jpg
                dark: https://www.liveabout.com/thmb/YCJmu1khSJo8kMYM090QCd9W78U=/1250x0/filters:no_upscale():max_bytes(150000):strip_icc():format(webp)/powerpuff_girls-56a00bc45f9b58eba4aea61d.jpg

                -- ftd.image-src src1:
                light: https://upload.wikimedia.org/wikipedia/en/d/d4/Mickey_Mouse.png
                dark: https://upload.wikimedia.org/wikipedia/en/d/d4/Mickey_Mouse.png

                -- ftd.image slide:
                ftd.image-src src:
                integer idx:
                src: $src
                if: $count == $idx
                $on-click$: increment $count clamp 0 1

                -- slide:
                src: $src0
                idx: 0

                -- slide:
                src: $src1
                idx: 1
                "
            ),
            &ftd::ftd2021::p2::TestLibrary {},
        )
            .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn loop_on_list_string() {
        let mut main = p2::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::ftd2021::rendered::markup_line("Arpita"),
                            line: true,
                            common: Box::new(ftd::Common {
                                reference: Some(s("foo/bar#$loop$@0,0")),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::ftd2021::rendered::markup_line("Ayushi"),
                            line: true,
                            common: Box::new(ftd::Common {
                                reference: Some(s("foo/bar#$loop$@0,1")),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::ftd2021::rendered::markup_line("AmitU"),
                            line: true,
                            common: Box::new(ftd::Common {
                                reference: Some(s("foo/bar#$loop$@0,2")),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    ],
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (_g_bag, g_col) = ftd::ftd2021::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- ftd.column foo:
                string list bar:

                --- ftd.text: $obj
                $loop$: $bar as $obj

                -- string list names:

                -- names: Arpita

                -- names: Ayushi

                -- names: AmitU

                -- foo:
                bar: $names
                "
            ),
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn open_container_with_parent_id() {
        let mut main = p2::default_column();
        let beverage_external_children = vec![ftd::Element::Column(ftd::Column {
            spacing: None,
            container: ftd::Container {
                children: vec![
                    ftd::Element::Column(ftd::Column {
                        spacing: None,
                        container: ftd::Container {
                            children: vec![
                                ftd::Element::Markup(ftd::Markups {
                                    text: ftd::ftd2021::rendered::markup_line("Water"),
                                    line: true,
                                    common: Box::new(ftd::Common {
                                        events: vec![ftd::Event {
                                            name: s("onclick"),
                                            action: ftd::Action {
                                                action: s("toggle"),
                                                target: s("foo/bar#visible@0,0,2"),
                                                ..Default::default()
                                            },
                                        }],
                                        reference: Some(s("foo/bar#name@0,0,2")),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                }),
                                ftd::Element::Column(ftd::Column {
                                    spacing: None,
                                    common: Box::new(ftd::Common {
                                        condition: Some(ftd::Condition {
                                            variable: s("foo/bar#visible@0,0,2"),
                                            value: serde_json::Value::Bool(true),
                                        }),
                                        data_id: Some(s("some-child")),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                }),
                            ],
                            external_children: Some((s("some-child"), vec![vec![1]], vec![])),
                            open: Some(true),
                            append_at: Some(s("some-child")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                    ftd::Element::Column(ftd::Column {
                        spacing: None,
                        container: ftd::Container {
                            children: vec![
                                ftd::Element::Markup(ftd::Markups {
                                    text: ftd::ftd2021::rendered::markup_line("Juice"),
                                    line: true,
                                    common: Box::new(ftd::Common {
                                        events: vec![ftd::Event {
                                            name: s("onclick"),
                                            action: ftd::Action {
                                                action: s("toggle"),
                                                target: s("foo/bar#visible@0,0,3"),
                                                ..Default::default()
                                            },
                                        }],
                                        reference: Some(s("foo/bar#name@0,0,3")),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                }),
                                ftd::Element::Column(ftd::Column {
                                    spacing: None,
                                    common: Box::new(ftd::Common {
                                        condition: Some(ftd::Condition {
                                            variable: s("foo/bar#visible@0,0,3"),
                                            value: serde_json::Value::Bool(true),
                                        }),
                                        data_id: Some(s("some-child")),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                }),
                            ],
                            external_children: Some((
                                s("some-child"),
                                vec![vec![1]],
                                vec![ftd::Element::Column(ftd::Column {
                                    spacing: None,
                                    container: ftd::Container {
                                        children: vec![ftd::Element::Column(ftd::Column {
                                            spacing: None,
                                            container: ftd::Container {
                                                children: vec![
                                                    ftd::Element::Markup(ftd::Markups {
                                                        text: ftd::ftd2021::rendered::markup_line(
                                                            "Mango Juice",
                                                        ),
                                                        line: true,
                                                        common: Box::new(ftd::Common {
                                                            events: vec![ftd::Event {
                                                                name: s("onclick"),
                                                                action: ftd::Action {
                                                                    action: s("toggle"),
                                                                    target: s(
                                                                        "foo/bar#visible@0,0,1,2",
                                                                    ),
                                                                    ..Default::default()
                                                                },
                                                            }],
                                                            reference: Some(s(
                                                                "foo/bar#name@0,0,1,2",
                                                            )),
                                                            ..Default::default()
                                                        },
                                                        ..Default::default()
                                                    }),
                                                    ftd::Element::Column(ftd::Column {
                                                        spacing: None,
                                                        common: Box::new(ftd::Common {
                                                            condition: Some(ftd::Condition {
                                                                variable: s(
                                                                    "foo/bar#visible@0,0,1,2",
                                                                ),
                                                                value: serde_json::Value::Bool(
                                                                    true,
                                                                ),
                                                            }),
                                                            data_id: Some(s("some-child")),
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
                                                open: Some(true),
                                                append_at: Some(s("some-child")),
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        })],
                                        ..Default::default()
                                    },
                                    common: Box::new(ftd::Common {
                                        width: Some(ftd::Length::Fill),
                                        height: Some(ftd::Length::Fill),
                                        position: Some(ftd::Position::Center),
                                        ..Default::default()
                                    },
                                })],
                            )),
                            open: Some(true),
                            append_at: Some(s("some-child")),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                ],
                ..Default::default()
            },
            common: Box::new(ftd::Common {
                width: Some(ftd::Length::Fill),
                height: Some(ftd::Length::Fill),
                position: Some(ftd::Position::Center),
                ..Default::default()
            },
        })];

        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![ftd::Element::Column(ftd::Column {
                        spacing: None,
                        container: ftd::Container {
                            children: vec![
                                ftd::Element::Markup(ftd::Markups {
                                    text: ftd::ftd2021::rendered::markup_line("Beverage"),
                                    line: true,
                                    common: Box::new(ftd::Common {
                                        events: vec![ftd::Event {
                                            name: s("onclick"),
                                            action: ftd::Action {
                                                action: s("toggle"),
                                                target: s("foo/bar#visible@0,0"),
                                                ..Default::default()
                                            },
                                        }],
                                        reference: Some(s("foo/bar#name@0,0")),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                }),
                                ftd::Element::Column(ftd::Column {
                                    spacing: None,
                                    common: Box::new(ftd::Common {
                                        condition: Some(ftd::Condition {
                                            variable: s("foo/bar#visible@0,0"),
                                            value: serde_json::Value::Bool(true),
                                        }),
                                        data_id: Some(s("some-child")),
                                        id: Some(s("beverage:some-child")),
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
                            open: Some(true),
                            append_at: Some(s("some-child")),
                            ..Default::default()
                        },
                        common: Box::new(ftd::Common {
                            data_id: Some(s("beverage")),
                            id: Some(s("beverage")),
                            ..Default::default()
                        },
                    })],
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (_g_bag, g_col) = ftd::ftd2021::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
            -- ftd.column display-item1:
            string name:
            open: true
            append-at: some-child
            boolean visible: true

            --- ftd.text: $name
            $on-click$: toggle $visible

            --- ftd.column:
            if: $visible
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
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn text_check() {
        let mut main = p2::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::ftd2021::rendered::markup_line("$hello"),
                            line: true,
                            ..Default::default()
                        }),
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::ftd2021::rendered::markup_line("hello"),
                            line: true,
                            common: Box::new(ftd::Common {
                                reference: Some(s("foo/bar#hello2@0")),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::ftd2021::rendered::markup_line("hello"),
                            line: true,
                            common: Box::new(ftd::Common {
                                reference: Some(s("foo/bar#hello")),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::ftd2021::rendered::markup_line("hello"),
                            line: true,
                            ..Default::default()
                        }),
                    ],
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (_g_bag, g_col) = ftd::ftd2021::test::interpret(
            "foo/bar",
            indoc::indoc!(
                r"
                -- string hello: hello

                -- ftd.column foo:
                string hello2:

                --- ftd.text: \$hello

                --- ftd.text: $hello2

                --- ftd.text: $hello

                --- ftd.text: hello

                -- foo:
                hello2: $hello
                "
            ),
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn caption() {
        let mut main = p2::default_column();

        main.container
            .children
            .push(ftd::Element::Integer(ftd::Text {
                text: ftd::ftd2021::rendered::markup_line("32"),
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Boolean(ftd::Text {
                text: ftd::ftd2021::rendered::markup_line("true"),
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Decimal(ftd::Text {
                text: ftd::ftd2021::rendered::markup_line("0.06"),
                ..Default::default()
            }));

        let (_g_bag, g_col) = ftd::ftd2021::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- ftd.integer: 32

                -- ftd.boolean: true

                -- ftd.decimal: 0.06
                "
            ),
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn heading_id() {
        let mut main = p2::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::ftd2021::rendered::markup_line("Heading 00"),
                            line: true,
                            common: Box::new(ftd::Common {
                                region: Some(ftd::Region::Title),
                                reference: Some(s("foo/bar#title@0")),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::ftd2021::rendered::markup_line("Heading 00 body"),
                            line: true,
                            common: Box::new(ftd::Common {
                                id: Some(s("one:markup-id")),
                                data_id: Some(s("markup-id")),
                                reference: Some(s("foo/bar#body@0,1")),
                                condition: Some(ftd::Condition {
                                    variable: s("foo/bar#body@0"),
                                    value: serde_json::Value::String(s("$IsNotNull$")),
                                }),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    ],
                    ..Default::default()
                },
                common: Box::new(ftd::Common {
                    region: Some(ftd::Region::H0),
                    id: Some(s("one")),
                    data_id: Some(s("one")),
                    heading_number: Some(vec![s("1")]),
                    ..Default::default()
                },
            }));

        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::ftd2021::rendered::markup_line("Heading 01"),
                            line: true,
                            common: Box::new(ftd::Common {
                                region: Some(ftd::Region::Title),
                                reference: Some(s("foo/bar#title@1")),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::ftd2021::rendered::markup_line("Heading 01 body"),
                            line: true,
                            common: Box::new(ftd::Common {
                                data_id: Some(s("markup-id")),
                                condition: Some(ftd::Condition {
                                    variable: s("foo/bar#body@1"),
                                    value: serde_json::Value::String(s("$IsNotNull$")),
                                }),
                                reference: Some(s("foo/bar#body@1,1")),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    ],
                    ..Default::default()
                },
                common: Box::new(ftd::Common {
                    region: Some(ftd::Region::H0),
                    id: Some(s("heading-01")),
                    heading_number: Some(vec![s("2")]),
                    ..Default::default()
                },
            }));

        let (_g_bag, g_col) = ftd::ftd2021::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- h0: Heading 00
                id: one

                Heading 00 body

                -- h0: Heading 01

                Heading 01 body

                -- ftd.column h0:
                caption title:
                optional body body:
                region: h0

                --- ftd.text:
                text: $title
                region: title

                --- markup:
                if: $body is not null
                body: $body
                id: markup-id

                -- ftd.text markup:
                body body:
                text: $body
                "
            ),
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn new_id() {
        let mut main = p2::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![ftd::Element::Markup(ftd::Markups {
                        text: ftd::ftd2021::rendered::markup_line("hello"),
                        line: true,
                        common: Box::new(ftd::Common {
                            data_id: Some(s("hello")),
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
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![ftd::Element::Markup(ftd::Markups {
                        text: ftd::ftd2021::rendered::markup_line("hello"),
                        line: true,
                        common: Box::new(ftd::Common {
                            data_id: Some(s("hello")),
                            id: Some(s("asd:hello")),
                            ..Default::default()
                        },
                        ..Default::default()
                    })],
                    ..Default::default()
                },
                common: Box::new(ftd::Common {
                    data_id: Some(s("asd")),
                    id: Some(s("asd")),
                    ..Default::default()
                },
            }));

        let (_g_bag, g_col) = ftd::ftd2021::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
            --  ftd.column foo:

            --- ftd.text: hello
            id: hello

            -- foo:

            -- foo:
            id: asd
            "
            ),
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn list_is_empty_check() {
        let mut main = p2::default_column();
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("Hello people"),
                line: true,
                ..Default::default()
            }));

        main.container.children.push(ftd::Element::Null);

        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Null,
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::ftd2021::rendered::markup_line("Hello empty list"),
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
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::ftd2021::rendered::markup_line("Hello list"),
                            line: true,
                            ..Default::default()
                        }),
                        ftd::Element::Null,
                    ],
                    ..Default::default()
                },
                ..Default::default()
            }));
        let (_g_bag, g_col) = ftd::ftd2021::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- string list people:

                -- people: Ayushi

                -- people: Arpita

                -- ftd.text: Hello people
                if: $people is not empty

                -- ftd.text: Hello nobody
                if: $people is empty


                -- string list empty-list:


                -- ftd.column foo:
                string list string-list:

                --- ftd.text: Hello list
                if: $string-list is not empty

                --- ftd.text: Hello empty list
                if: $string-list is empty

                -- foo:
                string-list: $empty-list

                -- foo:
                string-list: $people
                "
            ),
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .expect("found error");
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn parent_with_unsatisfied_condition() {
        let mut main = p2::default_column();
        main.container.children.push(ftd::Element::Null);
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![ftd::Element::Markup(ftd::Markups {
                        text: ftd::ftd2021::rendered::markup_line("Hello"),
                        line: true,
                        ..Default::default()
                    })],
                    ..Default::default()
                },
                common: Box::new(ftd::Common {
                    is_not_visible: true,
                    ..Default::default()
                },
            }));

        let (_g_bag, g_col) = ftd::ftd2021::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- string list empty-list:

                -- ftd.column:
                if: $empty-list is not empty

                --- ftd.text: Hello

                -- foo:

                -- ftd.column foo:
                if: $empty-list is not empty

                --- ftd.text: Hello
                "
            ),
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn open_container_id_with_children() {
        let mut external_children = p2::default_column();
        external_children
            .container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("hello"),
                line: true,
                ..Default::default()
            }));
        external_children
            .container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("world"),
                line: true,
                ..Default::default()
            }));

        let mut main = p2::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![ftd::Element::Column(ftd::Column {
                        spacing: None,
                        common: Box::new(ftd::Common {
                            id: Some(s("foo-id:some-id")),
                            data_id: Some(s("some-id")),
                            ..Default::default()
                        },
                        ..Default::default()
                    })],
                    external_children: Some((
                        s("some-id"),
                        vec![vec![0]],
                        vec![ftd::Element::Column(external_children)],
                    )),
                    open: Some(true),
                    append_at: Some(s("some-id")),
                    ..Default::default()
                },
                common: Box::new(ftd::Common {
                    id: Some(s("foo-id")),
                    data_id: Some(s("foo-id")),
                    ..Default::default()
                },
            }));

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("Outside"),
                line: true,
                ..Default::default()
            }));

        let (_g_bag, g_col) = ftd::ftd2021::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- foo:
                id: foo-id

                --- ftd.text: hello

                --- ftd.text: world

                -- ftd.text: Outside


                -- ftd.column foo:
                open: true
                append-at: some-id

                --- ftd.column:
                id: some-id
                "
            ),
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn loop_record_list() {
        let mut main = p2::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Column(ftd::Column {
                            spacing: None,
                            container: ftd::Container {
                                children: vec![ftd::Element::Markup(ftd::Markups {
                                    text: ftd::ftd2021::rendered::markup_line("commit message 1"),
                                    line: true,
                                    common: Box::new(ftd::Common {
                                        reference: Some(s("foo/bar#commit@0,0.message")),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                })],
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Column(ftd::Column {
                            spacing: None,
                            container: ftd::Container {
                                children: vec![ftd::Element::Markup(ftd::Markups {
                                    text: ftd::ftd2021::rendered::markup_line("commit message 2"),
                                    line: true,
                                    common: Box::new(ftd::Common {
                                        reference: Some(s("foo/bar#commit@0,1.message")),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                })],
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Column(ftd::Column {
                            spacing: None,
                            container: ftd::Container {
                                children: vec![ftd::Element::Markup(ftd::Markups {
                                    text: ftd::ftd2021::rendered::markup_line("file filename 1"),
                                    line: true,
                                    common: Box::new(ftd::Common {
                                        reference: Some(s("foo/bar#file@0,2.filename")),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                })],
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Column(ftd::Column {
                            spacing: None,
                            container: ftd::Container {
                                children: vec![ftd::Element::Markup(ftd::Markups {
                                    text: ftd::ftd2021::rendered::markup_line("file filename 2"),
                                    line: true,
                                    common: Box::new(ftd::Common {
                                        reference: Some(s("foo/bar#file@0,3.filename")),
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
            }));

        let (_g_bag, g_col) = ftd::ftd2021::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- record commit:
                string message:

                -- record file:
                string filename:

                -- record changes:
                commit list commits:
                file list files:


                -- commit list commit-list:

                -- commit-list:
                message: commit message 1

                -- commit-list:
                message: commit message 2


                -- file list file-list:

                -- file-list:
                filename: file filename 1

                -- file-list:
                filename: file filename 2


                -- changes rec-changes:
                commits: $commit-list
                files: $file-list

                -- display:
                changes: $rec-changes


                -- ftd.column display:
                changes changes:

                --- display-commit:
                $loop$: $changes.commits as $obj
                commit: $obj

                --- display-file:
                $loop$: $changes.files as $obj
                file: $obj


                -- ftd.column display-commit:
                commit commit:

                --- ftd.text: $commit.message


                -- ftd.column display-file:
                file file:

                --- ftd.text: $file.filename
                "
            ),
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .expect("found error");
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn scene_children_with_default_position() {
        let mut main = p2::default_column();
        main.container
            .children
            .push(ftd::Element::Scene(ftd::Scene {
                spacing: None,
                container: ftd::Container {
                    children: vec![ftd::Element::Markup(ftd::Markups {
                        text: ftd::ftd2021::rendered::markup_line("Hello"),
                        line: true,
                        common: Box::new(ftd::Common {
                            top: Some(0),
                            left: Some(0),
                            ..Default::default()
                        },
                        ..Default::default()
                    }), ftd::Element::Markup(ftd::Markups {
                        text: ftd::ftd2021::rendered::markup_line("World"),
                        line: true,
                        common: Box::new(ftd::Common {
                            top: Some(10),
                            right: Some(30),
                            scale: Some(1.5),
                            scale_x: Some(-1.0),
                            scale_y: Some(-1.0),
                            rotate: Some(45),
                            position: Some(ftd::Position::Center),
                            ..Default::default()
                        },
                        ..Default::default()
                    })],
                    ..Default::default()
                },
                common: Box::new(ftd::Common {
                    width: Some(
                        ftd::Length::Px {
                            value: 1000,
                        }),
                    ),
                    background_image: Some(
                        i("https://image.shutterstock.com/z/stock-&lt;!&ndash;&ndash;&gt;vector-vector-illustration-of-a-beautiful-summer-landscape-143054302.jpg", Some(s("foo/bar#bg-src"))),
                    ),
                    ..Default::default()
                }
            }));

        let (_g_bag, g_col) = ftd::ftd2021::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- ftd.image-src bg-src: https://image.shutterstock.com/z/stock-&lt;!&ndash;&ndash;&gt;vector-vector-illustration-of-a-beautiful-summer-landscape-143054302.jpg
                dark: https://image.shutterstock.com/z/stock-&lt;!&ndash;&ndash;&gt;vector-vector-illustration-of-a-beautiful-summer-landscape-143054302.jpg

                -- ftd.scene:
                background-image: $bg-src
                width: 1000

                --- ftd.text: Hello

                --- foo:
                top: 10
                right: 30
                align: center
                scale: 1.5
                rotate: 45
                scale-x: -1
                scale-y: -1

                -- ftd.text foo:
                text: World
                "
            ),
            &ftd::ftd2021::p2::TestLibrary {},
        )
            .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn event_set() {
        let mut main = p2::default_column();
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("Start..."),
                line: true,
                common: Box::new(ftd::Common {
                    condition: Some(ftd::Condition {
                        variable: s("foo/bar#current"),
                        value: serde_json::Value::String(s("some value")),
                    }),
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("some value"),
                line: true,
                common: Box::new(ftd::Common {
                    reference: Some(s("foo/bar#current")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("change message"),
                line: true,
                common: Box::new(ftd::Common {
                    events: vec![ftd::Event {
                        name: s("onclick"),
                        action: ftd::Action {
                            action: s("set-value"),
                            target: s("foo/bar#current"),
                            parameters: std::iter::IntoIterator::into_iter([(
                                s("value"),
                                vec![
                                    ftd::ftd2021::event::ParameterData {
                                        value: serde_json::Value::String(s("hello world")),
                                        reference: None,
                                    }),
                                    ftd::ftd2021::event::ParameterData {
                                        value: serde_json::Value::String(s("string")),
                                        reference: None,
                                    }),
                                ],
                            )])
                            .collect(),
                        }),
                    }],
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("change message again"),
                line: true,
                common: Box::new(ftd::Common {
                    events: vec![ftd::Event {
                        name: s("onclick"),
                        action: ftd::Action {
                            action: s("set-value"),
                            target: s("foo/bar#current"),
                            parameters: std::iter::IntoIterator::into_iter([(
                                s("value"),
                                vec![
                                    ftd::ftd2021::event::ParameterData {
                                        value: serde_json::Value::String(s("good bye")),
                                        reference: Some(s("foo/bar#msg")),
                                    }),
                                    ftd::ftd2021::event::ParameterData {
                                        value: serde_json::Value::String(s("string")),
                                        reference: None,
                                    }),
                                ],
                            )])
                            .collect(),
                        }),
                    }],
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (_g_bag, g_col) = ftd::ftd2021::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- string current: some value

                -- ftd.text: Start...
                if: $current == some value

                -- ftd.text: $current

                -- ftd.text: change message
                $on-click$: $current = hello world

                -- string msg: good bye

                -- ftd.text: change message again
                $on-click$: $current = $msg
                "
            ),
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .expect("found error");
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn absolute_positioning() {
        let mut main = p2::default_column();
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("hello world"),
                line: true,
                common: Box::new(ftd::Common {
                    anchor: Some(ftd::Anchor::Parent),
                    right: Some(0),
                    top: Some(100),
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (_g_bag, g_col) = ftd::ftd2021::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- ftd.text: hello world
                anchor: parent
                right: 0
                top: 100
                "
            ),
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .expect("found error");
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn inherit_check() {
        let mut main = p2::default_column();
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("hello"),
                line: true,
                line_clamp: Some(50),
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("hello"),
                line: true,
                ..Default::default()
            }));

        let (_g_bag, g_col) = ftd::ftd2021::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- ftd.text foo: hello
                inherit line-clamp:

                -- foo:
                line-clamp: 50

                -- foo:
                "
            ),
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .expect("found error");
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn inner_container_check() {
        let mut main = p2::default_column();
        let col = ftd::Element::Column(ftd::Column {
            spacing: None,
            container: ftd::Container {
                children: vec![ftd::Element::Column(ftd::Column {
                    spacing: None,
                    container: ftd::Container {
                        children: vec![
                            ftd::Element::Image(ftd::Image {
                                src: i(
                                    "https://www.nilinswap.com/static/img/dp.jpeg",
                                    Some(s("foo/bar#src0")),
                                ),
                                common: Box::new(ftd::Common {
                                    reference: Some(s("foo/bar#src0")),
                                    ..Default::default()
                                },
                                ..Default::default()
                            }),
                            ftd::Element::Markup(ftd::Markups {
                                text: ftd::ftd2021::rendered::markup_line("Swapnil Sharma"),
                                line: true,
                                ..Default::default()
                            }),
                        ],
                        ..Default::default()
                    },
                    ..Default::default()
                })],
                ..Default::default()
            },
            ..Default::default()
        });
        main.container.children.push(col.clone());
        main.container.children.push(col);

        let (_g_bag, g_col) = ftd::ftd2021::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- ftd.image-src src0:
                light: https://www.nilinswap.com/static/img/dp.jpeg
                dark: https://www.nilinswap.com/static/img/dp.jpeg

                -- ftd.column:

                --- ftd.column:

                --- ftd.image:
                src: $src0

                --- ftd.text: Swapnil Sharma


                -- ftd.column foo:

                --- ftd.column:

                --- ftd.image:
                src: $src0

                --- ftd.text: Swapnil Sharma

                -- foo:
                "
            ),
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .expect("found error");
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn mouse_in() {
        let mut main = p2::default_column();
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("Hello World"),
                line: true,
                common: Box::new(ftd::Common {
                    conditional_attribute: std::iter::IntoIterator::into_iter([(
                        s("color"),
                        ftd::ConditionalAttribute {
                            attribute_type: ftd::AttributeType::Style,
                            conditions_with_value: vec![(
                                ftd::Condition {
                                    variable: s("foo/bar#MOUSE-IN@0"),
                                    value: serde_json::Value::Bool(true),
                                }),
                                ftd::ConditionalValue {
                                    value: serde_json::from_str("{\"$kind$\":\"light\",\"dark\":\"rgba(255,0,0,1)\",\"light\":\"rgba(255,0,0,1)\"}").unwrap(),
                                    important: false,
                                    reference: Some(s("foo/bar#red")),
                                }),
                            )],
                            default: None,
                        }),
                    )])
                        .collect(),
                    events: vec![
                        ftd::Event {
                            name: s("onmouseenter"),
                            action: ftd::Action {
                                action: s("set-value"),
                                target: s("foo/bar#MOUSE-IN@0"),
                                parameters: std::iter::IntoIterator::into_iter([(
                                    s("value"),
                                    vec![
                                        ftd::ftd2021::event::ParameterData {
                                            value: serde_json::Value::from(true),
                                            reference: None,
                                        }),
                                        ftd::ftd2021::event::ParameterData {
                                            value: serde_json::Value::String(s("boolean")),
                                            reference: None,
                                        }),
                                    ],
                                )])
                                    .collect(),
                            }),
                        }),
                        ftd::Event {
                            name: s("onmouseleave"),
                            action: ftd::Action {
                                action: s("set-value"),
                                target: s("foo/bar#MOUSE-IN@0"),
                                parameters: std::iter::IntoIterator::into_iter([(
                                    s("value"),
                                    vec![
                                        ftd::ftd2021::event::ParameterData {
                                            value: serde_json::Value::from(false),
                                            reference: None,
                                        }),
                                        ftd::ftd2021::event::ParameterData {
                                            value: serde_json::Value::String(s("boolean")),
                                            reference: None,
                                        }),
                                    ],
                                )])
                                    .collect(),
                            }),
                        }),
                    ],
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (_g_bag, g_col) = ftd::ftd2021::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- ftd.color red: red
                dark: red

                -- ftd.text foo:
                text: Hello World
                color if $MOUSE-IN: $red

                -- foo:
                "
            ),
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .expect("found error");
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn event_stop_propagation() {
        let mut main = p2::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::ftd2021::rendered::markup_line("Hello"),
                            line: true,
                            common: Box::new(ftd::Common {
                                condition: Some(ftd::Condition {
                                    variable: s("foo/bar#open@0"),
                                    value: serde_json::Value::Bool(true),
                                }),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Column(ftd::Column {
                            spacing: None,
                            container: ftd::Container {
                                children: vec![ftd::Element::Markup(ftd::Markups {
                                    text: ftd::ftd2021::rendered::markup_line("Hello Again"),
                                    line: true,
                                    common: Box::new(ftd::Common {
                                        condition: Some(ftd::Condition {
                                            variable: s("foo/bar#open@0,1"),
                                            value: serde_json::Value::Bool(true),
                                        }),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                })],
                                ..Default::default()
                            },
                            common: Box::new(ftd::Common {
                                events: vec![
                                    ftd::Event {
                                        name: s("onclick"),
                                        action: ftd::Action {
                                            action: s("toggle"),
                                            target: s("foo/bar#open@0,1"),
                                            parameters: Default::default(),
                                        }),
                                    }),
                                    ftd::Event {
                                        name: s("onclick"),
                                        action: ftd::Action {
                                            action: s("stop-propagation"),
                                            target: s(""),
                                            parameters: Default::default(),
                                        }),
                                    }),
                                ],
                                ..Default::default()
                            },
                        }),
                    ],
                    ..Default::default()
                },
                common: Box::new(ftd::Common {
                    events: vec![ftd::Event {
                        name: s("onclick"),
                        action: ftd::Action {
                            action: s("toggle"),
                            target: s("foo/bar#open@0"),
                            parameters: Default::default(),
                        }),
                    }],
                    ..Default::default()
                },
            }));

        let (_g_bag, g_col) = ftd::ftd2021::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- foo:

                -- ftd.column foo:
                boolean open: true
                $on-click$: toggle $open

                --- ftd.text: Hello
                if: $open

                --- bar:


                -- ftd.column bar:
                boolean open: true
                $on-click$: toggle $open
                $on-click$: stop-propagation

                --- ftd.text: Hello Again
                if: $open

                "
            ),
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .expect("found error");
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn new_syntax() {
        let mut main = p2::default_column();
        main.container.children.push(ftd::Element::Row(ftd::Row {
            spacing: None,
            container: ftd::Container {
                children: vec![ftd::Element::Integer(ftd::Text {
                    text: ftd::ftd2021::rendered::markup_line("20"),
                    common: Box::new(ftd::Common {
                        conditional_attribute: std::iter::IntoIterator::into_iter([(
                            s("color"),
                            ftd::ConditionalAttribute {
                                attribute_type: ftd::AttributeType::Style,
                                conditions_with_value: vec![
                                    (
                                        ftd::Condition {
                                            variable: s("foo/bar#b@0"),
                                            value: serde_json::Value::Bool(true),
                                        }),
                                        ftd::ConditionalValue {
                                            value: serde_json::from_str("{\"$kind$\":\"light\",\"dark\":\"rgba(0,0,0,1)\",\"light\":\"rgba(0,0,0,1)\"}").unwrap(),
                                            important: false,
                                            reference: Some(s("foo/bar#black")),
                                        }),
                                    ),
                                    (
                                        ftd::Condition {
                                            variable: s("foo/bar#a@0"),
                                            value: serde_json::Value::from(30),
                                        }),
                                        ftd::ConditionalValue {
                                            value: serde_json::from_str("{\"$kind$\":\"light\",\"dark\":\"rgba(255,0,0,1)\",\"light\":\"rgba(255,0,0,1)\"}").unwrap(),
                                            important: false,
                                            reference: Some(s("foo/bar#red")),
                                        }),
                                    ),
                                ],
                                default: None,
                            }),
                        )])
                            .collect(),
                        reference: Some(s("foo/bar#a@0")),
                        ..Default::default()
                    },
                    ..Default::default()
                })],
                ..Default::default()
            },
            common: Box::new(ftd::Common {
                events: vec![
                    ftd::Event {
                        name: s("onclick"),
                        action: ftd::Action {
                            action: s("toggle"),
                            target: s("foo/bar#b@0"),
                            parameters: Default::default(),
                        }),
                    }),
                    ftd::Event {
                        name: s("onclick"),
                        action: ftd::Action {
                            action: s("increment"),
                            target: s("foo/bar#a@0"),
                            parameters: std::iter::IntoIterator::into_iter([(
                                s("by"),
                                vec![ftd::ftd2021::event::ParameterData {
                                    value: serde_json::Value::from(2),
                                    reference: None,
                                }],
                            )])
                                .collect(),
                        }),
                    }),
                ],
                ..Default::default()
            },
        }));

        let (_g_bag, g_col) = ftd::ftd2021::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- ftd.color black: black
                dark: black

                -- ftd.color red: red
                dark: red

                -- ftd.row foo:
                integer a:
                boolean b: false
                $on-click$: toggle $b
                $on-click$: increment $a by 2

                --- ftd.integer:
                value: $a
                color if $b: $black
                color if $a == 30: $red

                -- foo:
                a: 20
                "
            ),
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .expect("found error");
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn condition_check() {
        let mut main = p2::default_column();
        main.container.children.push(ftd::Element::Row(ftd::Row {
            spacing: None,
            container: ftd::Container {
                children: vec![ftd::Element::Column(ftd::Column {
                    spacing: None,
                    container: ftd::Container {
                        children: vec![ftd::Element::Markup(ftd::Markups {
                            text: ftd::ftd2021::rendered::markup_line("Hello"),
                            line: true,
                            common: Box::new(ftd::Common {
                                condition: Some(ftd::Condition {
                                    variable: s("foo/bar#b@0,0"),
                                    value: serde_json::Value::Bool(true),
                                }),
                                is_not_visible: true,
                                ..Default::default()
                            },
                            ..Default::default()
                        })],
                        ..Default::default()
                    },
                    common: Box::new(ftd::Common {
                        condition: Some(ftd::Condition {
                            variable: s("foo/bar#b@0"),
                            value: serde_json::Value::Bool(true),
                        }),
                        ..Default::default()
                    },
                })],
                ..Default::default()
            },
            ..Default::default()
        }));

        let (_g_bag, g_col) = ftd::ftd2021::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- boolean present: true

                -- ftd.column bar:
                boolean a: true
                if: $a
                boolean b: false

                --- ftd.text: Hello
                if: $b

                -- ftd.row foo:
                boolean b: true

                --- bar:
                if: $b

                -- foo:
                "
            ),
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .expect("found error");
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn external_variable() {
        let mut main = p2::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Integer(ftd::Text {
                            text: ftd::ftd2021::rendered::markup_line("20"),
                            common: Box::new(ftd::Common {
                                conditional_attribute: std::iter::IntoIterator::into_iter([(
                                    s("color"),
                                    ftd::ConditionalAttribute {
                                        attribute_type: ftd::AttributeType::Style,
                                        conditions_with_value: vec![(
                                            ftd::Condition {
                                                variable: s("foo/bar#b@0"),
                                                value: serde_json::Value::Bool(true),
                                            }),
                                            ftd::ConditionalValue {
                                                value: serde_json::from_str("{\"$kind$\":\"light\",\"dark\":\"rgba(0,0,0,1)\",\"light\":\"rgba(0,0,0,1)\"}").unwrap(),
                                                important: false,
                                                reference: Some(s("foo/bar#black")),
                                            }),
                                        )],
                                        default: None,
                                    }),
                                )])
                                    .collect(),
                                reference: Some(s("foo/bar#a@0")),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::ftd2021::rendered::markup_line("whatever"),
                            line: true,
                            common: Box::new(ftd::Common {
                                reference: Some(s("foo/bar#some-text@0")),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    ],
                    ..Default::default()
                },
                common: Box::new(ftd::Common {
                    events: vec![
                        ftd::Event {
                            name: s("onclick"),
                            action: ftd::Action {
                                action: s("toggle"),
                                target: s("foo/bar#b@0"),
                                parameters: Default::default(),
                            }),
                        }),
                        ftd::Event {
                            name: s("onclick"),
                            action: ftd::Action {
                                action: s("increment"),
                                target: s("foo/bar#a@0"),
                                parameters: Default::default(),
                            }),
                        }),
                        ftd::Event {
                            name: s("onclick"),
                            action: ftd::Action {
                                action: s("set-value"),
                                target: s("foo/bar#some-text@0"),
                                parameters: std::iter::IntoIterator::into_iter([(
                                    "value".to_string(),
                                    vec![
                                        ftd::ftd2021::event::ParameterData {
                                            value: serde_json::Value::String(s("hello")),
                                            reference: Some(s("foo/bar#current")),
                                        }),
                                        ftd::ftd2021::event::ParameterData {
                                            value: serde_json::Value::String(s("string")),
                                            reference: None,
                                        }),
                                    ],
                                )])
                                    .collect(),
                            }),
                        }),
                    ],
                    ..Default::default()
                },
            }));

        main.container.children.push(ftd::Element::Row(ftd::Row {
            spacing: None,
            container: ftd::Container {
                children: vec![ftd::Element::Markup(ftd::Markups {
                    text: ftd::ftd2021::rendered::markup_line("hello"),
                    line: true,
                    common: Box::new(ftd::Common {
                        conditional_attribute: std::iter::IntoIterator::into_iter([(
                            s("color"),
                            ftd::ConditionalAttribute {
                                attribute_type: ftd::AttributeType::Style,
                                conditions_with_value: vec![(
                                    ftd::Condition {
                                        variable: s("foo/bar#foo@1"),
                                        value: serde_json::Value::Bool(true),
                                    }),
                                    ftd::ConditionalValue {
                                        value: serde_json::from_str("{\"$kind$\":\"light\",\"dark\":\"rgba(255,0,0,1)\",\"light\":\"rgba(255,0,0,1)\"}").unwrap(),
                                        important: false,
                                        reference: Some(s("foo/bar#red")),
                                    }),
                                )],
                                default: None,
                            }),
                        )])
                            .collect(),
                        ..Default::default()
                    },
                    ..Default::default()
                })],
                ..Default::default()
            },
            common: Box::new(ftd::Common {
                events: vec![ftd::Event {
                    name: s("onclick"),
                    action: ftd::Action {
                        action: s("toggle"),
                        target: s("foo/bar#foo@1"),
                        parameters: Default::default(),
                    }),
                }],
                ..Default::default()
            },
        }));

        let (_g_bag, g_col) = ftd::ftd2021::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- ftd.color black: black
                dark: black

                -- ftd.color red: red
                dark: red

                -- ftd.column foo:
                integer a:
                boolean b: false
                $on-click$: toggle $b
                $on-click$: increment $a

                --- ftd.integer:
                value: $a
                color if $b: $black

                -- string current: hello

                -- foo:
                a: 20
                string some-text: whatever
                $on-click$: $some-text = $current

                --- ftd.text: $some-text

                -- ftd.row:
                boolean foo: false
                $on-click$: toggle $foo

                --- ftd.text: hello
                color if $foo: $red
                "
            ),
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .expect("found error");
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn new_var_syntax() {
        let mut main = p2::default_column();
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("hello"),
                line: true,
                line_clamp: Some(30),
                common: Box::new(ftd::Common {
                    conditional_attribute: std::iter::IntoIterator::into_iter([(
                        s("color"),
                        ftd::ConditionalAttribute {
                            attribute_type: ftd::AttributeType::Style,
                            conditions_with_value: vec![(
                                ftd::Condition {
                                    variable: s("foo/bar#t@0"),
                                    value: serde_json::Value::Bool(true),
                                }),
                                ftd::ConditionalValue {
                                    value: serde_json::from_str("{\"$kind$\":\"light\",\"dark\":\"rgba(255,0,0,1)\",\"light\":\"rgba(255,0,0,1)\"}").unwrap(),
                                    important: false,
                                    reference: Some(s("foo/bar#red")),
                                }),
                            )],
                            default: None,
                        }),
                    )])
                        .collect(),
                    reference: Some(s("foo/bar#bar")),
                    color: Some(ftd::Color {
                        light: ftd::ColorValue {
                            r: 255,
                            g: 0,
                            b: 0,
                            alpha: 1.0,
                        }),
                        dark: ftd::ColorValue {
                            r: 255,
                            g: 0,
                            b: 0,
                            alpha: 1.0,
                        }),
                        reference: Some(s("foo/bar#red")),
                    }),
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::ftd2021::rendered::markup_line("hello"),
                            line: true,
                            common: Box::new(ftd::Common {
                                reference: Some(s("foo/bar#ff@1")),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Integer(ftd::Text {
                            text: ftd::ftd2021::rendered::markup_line("20"),
                            common: Box::new(ftd::Common {
                                reference: Some(s("foo/bar#i@1")),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    ],
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (_g_bag, g_col) = ftd::ftd2021::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- ftd.color red: red
                dark: red

                -- ftd.column col:
                integer i:
                string ff: hello

                --- ftd.text: $ff

                --- ftd.integer: $i

                -- integer foo: 20

                -- foo: 30

                -- string bar: hello

                -- ftd.text: $bar
                boolean t: true
                string f: hello
                line-clamp: $foo
                color if $t: $red

                -- col:
                i: 20
                "
            ),
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .expect("found error");
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn text_block() {
        let mut main = p2::default_column();
        main.container
            .children
            .push(ftd::Element::TextBlock(ftd::TextBlock {
                text: ftd::ftd2021::rendered::markup_line("hello"),
                line: true,
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::TextBlock(ftd::TextBlock {
                text: ftd::ftd2021::rendered::markup_line("hello"),
                line: true,
                ..Default::default()
            }));

        main.container.children.push(ftd::Element::Code(ftd::Code {
            text: ftd::ftd2021::rendered::code_with_theme(
                "This is text",
                "txt",
                ftd::ftd2021::code::DEFAULT_THEME,
                "foo/bar",
            )
            .unwrap(),
            ..Default::default()
        }));

        let (_g_bag, g_col) = ftd::ftd2021::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- ftd.text-block: hello

                -- ftd.text-block b: hello

                -- b:

                -- ftd.code:

                This is text
                "
            ),
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .expect("found error");
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn variable_component() {
        let mut main = p2::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                spacing: None,
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::ftd2021::rendered::markup_line("amitu"),
                            line: true,
                            ..Default::default()
                        }),
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::ftd2021::rendered::markup_line("hello"),
                            line: true,
                            common: Box::new(ftd::Common {
                                color: Some(ftd::Color {
                                    light: ftd::ColorValue {
                                        r: 255,
                                        g: 0,
                                        b: 0,
                                        alpha: 1.0,
                                    }),
                                    dark: ftd::ColorValue {
                                        r: 255,
                                        g: 0,
                                        b: 0,
                                        alpha: 1.0,
                                    }),
                                    reference: Some(s("foo/bar#red")),
                                }),
                                ..Default::default()
                            },
                            line_clamp: Some(10),
                            ..Default::default()
                        }),
                        ftd::Element::Column(ftd::Column {
                            spacing: None,
                            container: ftd::Container {
                                children: vec![
                                    ftd::Element::Markup(ftd::Markups {
                                        text: ftd::ftd2021::rendered::markup_line("hello again"),
                                        line: true,
                                        common: Box::new(ftd::Common {
                                            reference: Some(s("foo/bar#msg@0,2")),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    }),
                                    ftd::Element::Markup(ftd::Markups {
                                        text: ftd::ftd2021::rendered::markup_line("hello world!"),
                                        line: true,
                                        common: Box::new(ftd::Common {
                                            reference: Some(s("foo/bar#other-msg@0,2")),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    }),
                                    ftd::Element::Markup(ftd::Markups {
                                        text: ftd::ftd2021::rendered::markup_line("hello"),
                                        line: true,
                                        common: Box::new(ftd::Common {
                                            color: Some(ftd::Color {
                                                light: ftd::ColorValue {
                                                    r: 255,
                                                    g: 0,
                                                    b: 0,
                                                    alpha: 1.0,
                                                }),
                                                dark: ftd::ColorValue {
                                                    r: 255,
                                                    g: 0,
                                                    b: 0,
                                                    alpha: 1.0,
                                                }),
                                                reference: Some(s("foo/bar#red")),
                                            }),
                                            ..Default::default()
                                        },
                                        line_clamp: Some(20),
                                        ..Default::default()
                                    }),
                                    ftd::Element::Markup(ftd::Markups {
                                        text: ftd::ftd2021::rendered::markup_line("hello amitu!"),
                                        line: true,
                                        common: Box::new(ftd::Common {
                                            color: Some(ftd::Color {
                                                light: ftd::ColorValue {
                                                    r: 255,
                                                    g: 0,
                                                    b: 0,
                                                    alpha: 1.0,
                                                }),
                                                dark: ftd::ColorValue {
                                                    r: 255,
                                                    g: 0,
                                                    b: 0,
                                                    alpha: 1.0,
                                                }),
                                                reference: Some(s("foo/bar#red")),
                                            }),
                                            ..Default::default()
                                        },
                                        line_clamp: Some(10),
                                        ..Default::default()
                                    }),
                                ],
                                ..Default::default()
                            },
                            common: Box::new(ftd::Common {
                                ..Default::default()
                            },
                        }),
                    ],
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (_g_bag, g_col) = ftd::ftd2021::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- ftd.color red: red
                dark: red

                -- ftd.text foo: hello
                integer line-clamp: 10
                color: $red
                line-clamp: $line-clamp

                -- ftd.column moo:
                caption msg: world
                string other-msg: world again
                ftd.ui t:
                ftd.ui k:

                --- ftd.text: $msg

                --- ftd.text: $other-msg

                --- t:

                --- k:

                -- ftd.column bar:
                ftd.ui t: foo:
                > line-clamp: 30
                ftd.ui g:

                --- ftd.text: amitu

                --- t:

                --- g:

                -- bar:
                g: moo: hello again
                > other-msg: hello world!
                > t: foo:
                >> line-clamp: 20
                > k: ftd.text: hello amitu!
                >> color: $red
                >> line-clamp: 10
                "
            ),
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .expect("found error");
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn optional_global_variable() {
        let mut main = p2::default_column();
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("hello"),
                line: true,
                common: Box::new(ftd::Common {
                    reference: Some(s("foo/bar#active")),
                    condition: Some(ftd::Condition {
                        variable: s("foo/bar#active"),
                        value: serde_json::Value::String(s("$IsNotNull$")),
                    }),
                    ..Default::default()
                },
                ..Default::default()
            }));
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("Not Active"),
                line: true,
                common: Box::new(ftd::Common {
                    condition: Some(ftd::Condition {
                        variable: s("foo/bar#active"),
                        value: serde_json::Value::String(s("$IsNull$")),
                    }),
                    is_not_visible: true,
                    ..Default::default()
                },
                ..Default::default()
            }));
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line(""),
                line: true,
                common: Box::new(ftd::Common {
                    reference: Some(s("foo/bar#flags")),
                    condition: Some(ftd::Condition {
                        variable: s("foo/bar#flags"),
                        value: serde_json::Value::String(s("$IsNotNull$")),
                    }),
                    is_not_visible: true,
                    ..Default::default()
                },
                ..Default::default()
            }));
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("No Flag Available"),
                line: true,
                common: Box::new(ftd::Common {
                    condition: Some(ftd::Condition {
                        variable: s("foo/bar#flags"),
                        value: serde_json::Value::String(s("$IsNull$")),
                    }),
                    ..Default::default()
                },
                ..Default::default()
            }));

        let (_g_bag, g_col) = ftd::ftd2021::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- optional string active:

                -- active: hello

                -- ftd.text: $active
                if: $active is not null

                -- ftd.text: Not Active
                if: $active is null

                -- optional string flags:

                -- ftd.text: $flags
                if: $flags is not null

                -- ftd.text: No Flag Available
                if: $flags is null
                "
            ),
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .expect("found error");
        pretty_assertions::assert_eq!(g_col, main);
    }

    #[test]
    fn object() {
        let mut main = p2::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                container: ftd::Container {
                    children: vec![ftd::Element::Markup(ftd::Markups {
                        text: ftd::ftd2021::rendered::markup_line("Data"),
                        line: true,
                        ..Default::default()
                    })],
                    ..Default::default()
                },
                ..Default::default()
            }));

        let mut bag = interpreter::default_bag();
        bag.insert(
            s("foo/bar#aa"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("aa"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("Madhav"),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            s("foo/bar#foo"),
            ftd::ftd2021::p2::Thing::Component(ftd::Component {
                root: s("ftd#column"),
                full_name: s("foo/bar#foo"),
                arguments: [
                    vec![(s("o"), ftd::ftd2021::p2::Kind::object())],
                    universal_arguments_as_vec(),
                ]
                .concat()
                .into_iter()
                .collect(),
                instructions: vec![ftd::Instruction::ChildComponent {
                    child: ftd::ChildComponent {
                        root: s("ftd#text"),
                        properties: std::iter::IntoIterator::into_iter([(
                            s("text"),
                            ftd::ftd2021::component::Property {
                                default: Some(ftd::PropertyValue::Value {
                                    value: ftd::ftd2021::variable::Value::String {
                                        text: s("Data"),
                                        source: ftd::TextSource::Caption,
                                    },
                                }),
                                conditions: vec![],
                                nested_properties: Default::default(),
                            },
                        )])
                        .collect(),
                        ..Default::default()
                    },
                }],
                ..Default::default()
            }),
        );
        bag.insert(
            s("foo/bar#obj"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("obj"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Object {
                        values: std::iter::IntoIterator::into_iter([
                            (
                                s("a"),
                                ftd::PropertyValue::Reference {
                                    name: s("foo/bar#aa"),
                                    kind: ftd::ftd2021::p2::Kind::String {
                                        caption: true,
                                        body: false,
                                        default: None,
                                        is_reference: false,
                                    },
                                },
                            ),
                            (
                                s("b"),
                                ftd::PropertyValue::Value {
                                    value: ftd::ftd2021::variable::Value::String {
                                        text: s("bb"),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                        ])
                        .collect(),
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            s("foo/bar#o@0"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("o"),
                value: ftd::PropertyValue::Reference {
                    name: s("foo/bar#obj"),
                    kind: ftd::ftd2021::p2::Kind::object(),
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );

        insert_universal_variables_by_count(1, "foo/bar", &mut bag);

        p!(
            "
            -- string aa: Madhav

            -- object obj:
            a: $aa
            b: bb

            -- ftd.column foo:
            object o:

            --- ftd.text: Data

            -- foo:
            o: $obj
            ",
            (bag, main),
        );
    }

    #[test]
    fn event_change() {
        let mut main = p2::default_column();
        let mut bag = interpreter::default_bag();
        bag.insert(
            s("foo/bar#input-data"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("input-data"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("Nothing"),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            s("foo/bar#obj"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("obj"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Object {
                        values: std::iter::IntoIterator::into_iter([
                            (
                                s("function"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: s("some-function"),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                            (
                                s("value"),
                                ftd::PropertyValue::Reference {
                                    name: s("foo/bar#input-data"),
                                    kind: ftd::ftd2021::p2::Kind::String {
                                        caption: true,
                                        body: false,
                                        default: None,
                                        is_reference: false,
                                    },
                                },
                            ),
                        ])
                        .collect(),
                    },
                },
                conditions: vec![],
            }),
        );

        main.container
            .children
            .push(ftd::Element::Input(ftd::Input {
                common: Box::new(ftd::Common {
                    events: vec![
                        ftd::Event {
                            name: s("onchange"),
                            action: ftd::Action {
                                action: s("set-value"),
                                target: s("foo/bar#input-data"),
                                parameters: std::iter::IntoIterator::into_iter([(
                                    s("value"),
                                    vec![
                                        ftd::ftd2021::event::ParameterData {
                                            value: serde_json::Value::String(s("$VALUE")),
                                            reference: None,
                                        }),
                                        ftd::ftd2021::event::ParameterData {
                                            value: serde_json::Value::String(s("string")),
                                            reference: None,
                                        }),
                                    ],
                                )])
                                .collect(),
                            }),
                        }),
                        ftd::Event {
                            name: s("onchange"),
                            action: ftd::Action {
                                action: s("message-host"),
                                target: s("$obj"),
                                parameters: std::iter::IntoIterator::into_iter([(
                                    "data".to_string(),
                                    vec![ftd::ftd2021::event::ParameterData {
                                    value: serde_json::from_str(
                                        "{\"function\":\"some-function\",\"value\":\"Nothing\"}",
                                    )
                                    .unwrap(),
                                    reference: Some(s("{\"value\":\"foo/bar#input-data\"}")),
                                }],
                                )])
                                .collect(),
                            }),
                        }),
                    ],
                    ..Default::default()
                },
                placeholder: None,
                ..Default::default()
            }));

        p!(
            "
            -- string input-data: Nothing

            -- object obj:
            function: some-function
            value: $input-data

            -- ftd.input:
            $on-change$: $input-data=$VALUE
            $on-change$: message-host $obj
            ",
            (bag, main),
        );
    }

    #[test]
    fn component_processor() {
        let mut main = p2::default_column();

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("Hello from text-component processor"),
                line: true,
                line_clamp: Some(40),
                ..Default::default()
            }));

        p!(
            "
            -- ftd.text: hello
            $processor$: text-component-processor
            ",
            (super::default_bag(), main),
        );
    }

    #[test]
    fn global_variable_pass_as_reference() {
        let mut main = p2::default_column();
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("Arpita"),
                line: true,
                common: Box::new(ftd::Common {
                    reference: Some(s("foo/bar#bar")),
                    ..Default::default()
                },
                ..Default::default()
            }));
        main.container
            .children
            .push(ftd::Element::Integer(ftd::Text {
                text: ftd::ftd2021::rendered::markup_line("1"),
                common: Box::new(ftd::Common {
                    reference: Some(s("foo/bar#ibar")),
                    ..Default::default()
                },
                ..Default::default()
            }));
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("Arpita"),
                line: true,
                common: Box::new(ftd::Common {
                    reference: Some(s("foo/bar#lfoo")),
                    ..Default::default()
                },
                ..Default::default()
            }));
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("Arpita"),
                line: true,
                common: Box::new(ftd::Common {
                    reference: Some(s("foo/bar#lfoo")),
                    ..Default::default()
                },
                ..Default::default()
            }));
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("Ayushi"),
                line: true,
                common: Box::new(ftd::Common {
                    reference: Some(s("foo/bar#lfoo")),
                    ..Default::default()
                },
                ..Default::default()
            }));
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("$loop$"),
                line: true,
                common: Box::new(ftd::Common {
                    is_dummy: true,
                    reference: Some(s("foo/bar#lfoo")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("Arpita"),
                line: true,
                common: Box::new(ftd::Common {
                    reference: Some(s("foo/bar#lbar")),
                    ..Default::default()
                },
                ..Default::default()
            }));
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("Arpita"),
                line: true,
                common: Box::new(ftd::Common {
                    reference: Some(s("foo/bar#lbar")),
                    ..Default::default()
                },
                ..Default::default()
            }));
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("Ayushi"),
                line: true,
                common: Box::new(ftd::Common {
                    reference: Some(s("foo/bar#lbar")),
                    ..Default::default()
                },
                ..Default::default()
            }));
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("$loop$"),
                line: true,
                common: Box::new(ftd::Common {
                    is_dummy: true,
                    reference: Some(s("foo/bar#lbar")),
                    ..Default::default()
                },
                ..Default::default()
            }));
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("Arpita"),
                line: true,
                common: Box::new(ftd::Common {
                    reference: Some(s("foo/bar#arpita.name")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        let mut bag = interpreter::default_bag();
        bag.insert(
            s("foo/bar#arpita"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("arpita"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Record {
                        name: s("foo/bar#person"),
                        fields: std::iter::IntoIterator::into_iter([(
                            s("name"),
                            ftd::PropertyValue::Reference {
                                name: s("foo/bar#bar"),
                                kind: ftd::ftd2021::p2::Kind::String {
                                    caption: true,
                                    body: false,
                                    default: None,
                                    is_reference: false,
                                },
                            },
                        )])
                        .collect(),
                    },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            s("foo/bar#person"),
            ftd::ftd2021::p2::Thing::Record(ftd::ftd2021::p2::Record {
                name: s("foo/bar#person"),
                fields: std::iter::IntoIterator::into_iter([(
                    s("name"),
                    ftd::ftd2021::p2::Kind::caption(),
                )])
                .collect(),
                instances: Default::default(),
                order: vec![s("name")],
            }),
        );

        bag.insert(
            s("foo/bar#bar"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("bar"),
                value: ftd::PropertyValue::Reference {
                    name: s("foo/bar#foo"),
                    kind: ftd::ftd2021::p2::Kind::String {
                        caption: true,
                        body: false,
                        default: None,
                        is_reference: false,
                    },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            s("foo/bar#foo"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("foo"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("Arpita"),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            s("foo/bar#ibar"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("ibar"),
                value: ftd::PropertyValue::Reference {
                    name: s("foo/bar#ifoo"),
                    kind: ftd::ftd2021::p2::Kind::Integer {
                        default: None,
                        is_reference: false,
                    },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            s("foo/bar#ifoo"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("ifoo"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Integer { value: 1 },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            s("foo/bar#lbar"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("foo/bar#lbar"),
                value: ftd::PropertyValue::Reference {
                    name: s("foo/bar#lfoo"),
                    kind: ftd::ftd2021::p2::Kind::List {
                        kind: Box::new(ftd::ftd2021::p2::Kind::String {
                            caption: false,
                            body: false,
                            default: None,
                            is_reference: false,
                        }),
                        default: None,
                        is_reference: false,
                    },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            s("foo/bar#lfoo"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("foo/bar#lfoo"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::List {
                        data: vec![
                            ftd::PropertyValue::Reference {
                                name: s("foo/bar#foo"),
                                kind: ftd::ftd2021::p2::Kind::String {
                                    caption: true,
                                    body: false,
                                    default: None,
                                    is_reference: false,
                                },
                            },
                            ftd::PropertyValue::Reference {
                                name: s("foo/bar#bar"),
                                kind: ftd::ftd2021::p2::Kind::String {
                                    caption: true,
                                    body: false,
                                    default: None,
                                    is_reference: false,
                                },
                            },
                            ftd::PropertyValue::Value {
                                value: ftd::Value::String {
                                    text: s("Ayushi"),
                                    source: ftd::TextSource::Caption,
                                },
                            },
                        ],
                        kind: ftd::ftd2021::p2::Kind::String {
                            caption: false,
                            body: false,
                            default: None,
                            is_reference: false,
                        },
                    },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            s("foo/bar#$loop$@2"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("$loop$"),
                value: ftd::PropertyValue::Reference {
                    name: s("foo/bar#foo"),
                    kind: ftd::ftd2021::p2::Kind::caption(),
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#$loop$@3"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("$loop$"),
                value: ftd::PropertyValue::Reference {
                    name: s("foo/bar#bar"),
                    kind: ftd::ftd2021::p2::Kind::caption(),
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#$loop$@4"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("$loop$"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("Ayushi"),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#$loop$@5"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("$loop$"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("$loop$"),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );

        bag.insert(
            s("foo/bar#$loop$@6"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("$loop$"),
                value: ftd::PropertyValue::Reference {
                    name: s("foo/bar#foo"),
                    kind: ftd::ftd2021::p2::Kind::caption(),
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#$loop$@7"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("$loop$"),
                value: ftd::PropertyValue::Reference {
                    name: s("foo/bar#bar"),
                    kind: ftd::ftd2021::p2::Kind::caption(),
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#$loop$@8"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("$loop$"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("Ayushi"),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#$loop$@9"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("$loop$"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("$loop$"),
                        source: ftd::TextSource::Header,
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );

        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@0", -1, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@1", -1, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@2", -1, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@3", -1, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@4", -1, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@5", -1, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@6", -1, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@7", -1, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@8", -1, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@9", -1, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT-MINUS-ONE@10", -1, &mut bag);

        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT@0", 0, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT@1", 0, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT@2", 0, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT@3", 0, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT@4", 0, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT@5", 0, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT@6", 0, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT@7", 0, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT@8", 0, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT@9", 0, &mut bag);
        insert_update_integer_by_root("foo/bar#CHILDREN-COUNT@10", 0, &mut bag);

        insert_update_integer_by_root("foo/bar#SIBLING-INDEX-0@0", 0, &mut bag);
        insert_update_integer_by_root("foo/bar#SIBLING-INDEX-0@1", 1, &mut bag);
        insert_update_integer_by_root("foo/bar#SIBLING-INDEX-0@10", 10, &mut bag);

        insert_update_integer_by_root("foo/bar#SIBLING-INDEX@0", 1, &mut bag);
        insert_update_integer_by_root("foo/bar#SIBLING-INDEX@1", 2, &mut bag);
        insert_update_integer_by_root("foo/bar#SIBLING-INDEX@10", 11, &mut bag);

        let (g_bag, g_col) = ftd::ftd2021::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- string foo: Arpita

                -- string bar: $foo

                -- integer ifoo: 1

                -- integer ibar: $ifoo

                -- string list lfoo:

                -- lfoo: $foo

                -- lfoo: $bar

                -- lfoo: Ayushi

                -- string list lbar: $lfoo

                -- record person:
                caption name:

                -- person arpita: $bar

                -- ftd.text: $bar

                -- ftd.integer: $ibar

                -- ftd.text: $obj
                $loop$: $lfoo as $obj

                -- ftd.text: $obj
                $loop$: $lbar as $obj

                -- ftd.text: $arpita.name
                "
            ),
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .expect("found error");

        pretty_assertions::assert_eq!(g_col, main);
        pretty_assertions::assert_eq!(g_bag, bag);
    }

    #[test]
    fn locals_as_ref() {
        let mut bag = interpreter::default_bag();
        bag.insert(
            s("foo/bar#active"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("active"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Boolean { value: true },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#active@0"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("active"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Boolean { value: false },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#active@1"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("active"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Boolean { value: false },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#bar"),
            ftd::ftd2021::p2::Thing::Component(ftd::Component {
                root: s("ftd#column"),
                full_name: s("foo/bar#bar"),
                arguments: [
                    vec![
                        (
                            s("active"),
                            ftd::ftd2021::p2::Kind::boolean().set_default(Some(s("false"))),
                        ),
                        (
                            s("bio"),
                            ftd::ftd2021::p2::Kind::string().set_default(Some(s("$subtitle"))),
                        ),
                        (
                            s("subtitle"),
                            ftd::ftd2021::p2::Kind::string().set_default(Some(s("$foo/bar#foo"))),
                        ),
                        (s("title"), ftd::ftd2021::p2::Kind::string()),
                        (s("w"), ftd::ftd2021::p2::Kind::integer()),
                    ],
                    universal_arguments_as_vec(),
                ]
                .concat()
                .into_iter()
                .collect(),
                locals: Default::default(),
                properties: std::iter::IntoIterator::into_iter([
                    (
                        s("border-width"),
                        ftd::ftd2021::component::Property {
                            default: Some(ftd::PropertyValue::Variable {
                                name: s("w"),
                                kind: ftd::ftd2021::p2::Kind::Optional {
                                    kind: Box::new(ftd::ftd2021::p2::Kind::integer()),
                                    is_reference: false,
                                },
                            }),
                            conditions: vec![],
                            nested_properties: Default::default(),
                        },
                    ),
                    (
                        s("color"),
                        ftd::ftd2021::component::Property {
                            default: Some(ftd::PropertyValue::Reference {
                                name: s("foo/bar#green"),
                                kind: ftd::ftd2021::p2::Kind::Optional {
                                    kind: Box::new(ftd::ftd2021::p2::Kind::Record {
                                        name: s("ftd#color"),
                                        default: None,
                                        is_reference: false,
                                    }),
                                    is_reference: false,
                                },
                            }),
                            conditions: vec![],
                            nested_properties: Default::default(),
                        },
                    ),
                ])
                .collect(),
                instructions: vec![
                    ftd::Instruction::ChildComponent {
                        child: ftd::ChildComponent {
                            root: s("ftd#text"),
                            condition: None,
                            properties: std::iter::IntoIterator::into_iter([(
                                s("text"),
                                ftd::ftd2021::component::Property {
                                    default: Some(ftd::PropertyValue::Variable {
                                        name: s("title"),
                                        kind: ftd::ftd2021::p2::Kind::caption_or_body(),
                                    }),
                                    conditions: vec![],
                                    nested_properties: Default::default(),
                                },
                            )])
                            .collect(),
                            ..Default::default()
                        },
                    },
                    ftd::Instruction::ChildComponent {
                        child: ftd::ChildComponent {
                            root: s("ftd#text"),
                            condition: None,
                            properties: std::iter::IntoIterator::into_iter([(
                                s("text"),
                                ftd::ftd2021::component::Property {
                                    default: Some(ftd::PropertyValue::Variable {
                                        name: s("subtitle"),
                                        kind: ftd::ftd2021::p2::Kind::caption_or_body()
                                            .set_default(Some(s("$foo/bar#foo"))),
                                    }),
                                    conditions: vec![],
                                    nested_properties: Default::default(),
                                },
                            )])
                            .collect(),
                            ..Default::default()
                        },
                    },
                    ftd::Instruction::ChildComponent {
                        child: ftd::ChildComponent {
                            root: s("ftd#text"),
                            condition: None,
                            properties: std::iter::IntoIterator::into_iter([(
                                s("text"),
                                ftd::ftd2021::component::Property {
                                    default: Some(ftd::PropertyValue::Variable {
                                        name: s("bio"),
                                        kind: ftd::ftd2021::p2::Kind::caption_or_body()
                                            .set_default(Some(s("$subtitle"))),
                                    }),
                                    conditions: vec![],
                                    nested_properties: Default::default(),
                                },
                            )])
                            .collect(),
                            ..Default::default()
                        },
                    },
                    ftd::Instruction::ChildComponent {
                        child: ftd::ChildComponent {
                            root: s("ftd#boolean"),
                            condition: None,
                            properties: std::iter::IntoIterator::into_iter([(
                                s("value"),
                                ftd::ftd2021::component::Property {
                                    default: Some(ftd::PropertyValue::Variable {
                                        name: s("active"),
                                        kind: ftd::ftd2021::p2::Kind::boolean()
                                            .set_default(Some(s("false"))),
                                    }),
                                    conditions: vec![],
                                    nested_properties: Default::default(),
                                },
                            )])
                            .collect(),
                            ..Default::default()
                        },
                    },
                ],
                events: vec![],
                condition: None,
                kernel: false,
                invocations: vec![],
                line_number: 0,
            }),
        );
        bag.insert(
            s("foo/bar#bar1"),
            ftd::ftd2021::p2::Thing::Component(ftd::Component {
                root: s("ftd#column"),
                full_name: s("foo/bar#bar1"),
                arguments: universal_arguments_as_map(),
                locals: Default::default(),
                properties: Default::default(),
                instructions: vec![],
                events: vec![],
                condition: None,
                kernel: false,
                invocations: vec![],
                line_number: 0,
            }),
        );
        bag.insert(
            s("foo/bar#bio@0"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("bio"),
                value: ftd::PropertyValue::Variable {
                    name: s("foo/bar#subtitle@0"),
                    kind: ftd::ftd2021::p2::Kind::string().set_default(Some(s("$foo/bar#foo"))),
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#bio@1"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("bio"),
                value: ftd::PropertyValue::Variable {
                    name: s("foo/bar#subtitle@1"),
                    kind: ftd::ftd2021::p2::Kind::string().set_default(Some(s("$foo/bar#foo"))),
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#foo"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("foo"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("Foo"),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#foo"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("foo"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("Foo"),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#gg@0"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("gg"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Integer { value: 1 },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#subtitle@0"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("subtitle"),
                value: ftd::PropertyValue::Reference {
                    name: s("foo/bar#foo"),
                    kind: ftd::ftd2021::p2::Kind::string().set_default(Some(s("$foo/bar#foo"))),
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#subtitle@1"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("subtitle"),
                value: ftd::PropertyValue::Reference {
                    name: s("foo/bar#foo"),
                    kind: ftd::ftd2021::p2::Kind::string().set_default(Some(s("$foo/bar#foo"))),
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#title@0"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("title"),
                value: ftd::PropertyValue::Reference {
                    name: s("foo/bar#foo"),
                    kind: ftd::ftd2021::p2::Kind::string(),
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#title@1"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("title"),
                value: ftd::PropertyValue::Reference {
                    name: s("foo/bar#foo"),
                    kind: ftd::ftd2021::p2::Kind::string(),
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#w@0"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("w"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Integer { value: 2 },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#w@1"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("w"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Integer { value: 1 },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );
        bag.insert(
            s("foo/bar#green"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("green"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Record {
                        name: s("ftd#color"),
                        fields: std::iter::IntoIterator::into_iter([
                            (
                                s("dark"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: s("green"),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                            (
                                s("light"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: s("green"),
                                        source: ftd::TextSource::Caption,
                                    },
                                },
                            ),
                        ])
                        .collect(),
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );

        insert_universal_variables_by_count(2, "foo/bar", &mut bag);
        insert_update_string_by_root("foo/bar#id@0", "bar-id", "header", &mut bag);
        insert_update_decimal_by_root("foo/bar#scale@0", 1.2, &mut bag);

        let mut main = p2::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::ftd2021::rendered::markup_line("Foo"),
                            line: true,
                            common: Box::new(ftd::Common {
                                reference: Some(s("foo/bar#title@0")),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::ftd2021::rendered::markup_line("Foo"),
                            line: true,
                            common: Box::new(ftd::Common {
                                reference: Some(s("foo/bar#subtitle@0")),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::ftd2021::rendered::markup_line("Foo"),
                            line: true,
                            common: Box::new(ftd::Common {
                                reference: Some(s("foo/bar#bio@0")),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Boolean(ftd::Text {
                            text: ftd::ftd2021::rendered::markup_line("false"),
                            common: Box::new(ftd::Common {
                                reference: Some(s("foo/bar#active@0")),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Integer(ftd::Text {
                            text: ftd::ftd2021::rendered::markup_line("1"),
                            common: Box::new(ftd::Common {
                                reference: Some(s("foo/bar#gg@0")),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    ],
                    external_children: None,
                    wrap: false,
                    ..Default::default()
                },
                spacing: None,
                common: Box::new(ftd::Common {
                    color: Some(ftd::Color {
                        light: ftd::ColorValue {
                            r: 0,
                            g: 128,
                            b: 0,
                            alpha: 1.0,
                        }),
                        dark: ftd::ColorValue {
                            r: 0,
                            g: 128,
                            b: 0,
                            alpha: 1.0,
                        }),
                        reference: Some(s("foo/bar#green")),
                    }),
                    id: Some(s("bar-id")),
                    data_id: Some(s("bar-id")),
                    border_width: 2,
                    scale: Some(1.2),
                    condition: Some(ftd::Condition {
                        variable: s("foo/bar#active"),
                        value: serde_json::Value::Bool(true),
                    }),
                    ..Default::default()
                },
            }));

        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                container: ftd::Container {
                    children: vec![
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::ftd2021::rendered::markup_line("Foo"),
                            line: true,
                            common: Box::new(ftd::Common {
                                reference: Some(s("foo/bar#title@1")),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::ftd2021::rendered::markup_line("Foo"),
                            line: true,
                            common: Box::new(ftd::Common {
                                reference: Some(s("foo/bar#subtitle@1")),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Markup(ftd::Markups {
                            text: ftd::ftd2021::rendered::markup_line("Foo"),
                            line: true,
                            common: Box::new(ftd::Common {
                                reference: Some(s("foo/bar#bio@1")),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        ftd::Element::Boolean(ftd::Text {
                            text: ftd::ftd2021::rendered::markup_line("false"),
                            common: Box::new(ftd::Common {
                                reference: Some(s("foo/bar#active@1")),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    ],
                    external_children: None,
                    wrap: false,
                    ..Default::default()
                },
                spacing: None,
                common: Box::new(ftd::Common {
                    color: Some(ftd::Color {
                        light: ftd::ColorValue {
                            r: 0,
                            g: 128,
                            b: 0,
                            alpha: 1.0,
                        }),
                        dark: ftd::ColorValue {
                            r: 0,
                            g: 128,
                            b: 0,
                            alpha: 1.0,
                        }),
                        reference: Some(s("foo/bar#green")),
                    }),
                    border_width: 1,
                    ..Default::default()
                },
            }));

        p!(
            "
            -- string foo: Foo

            -- boolean active: true

            -- ftd.color green: green
            dark: green

            -- bar:
            if: $active
            id: bar-id
            scale: 1.2
            title: $foo
            w: 2
            integer gg: 1

            --- ftd.integer: $gg

            -- bar:
            title: $foo
            w: 1


            -- ftd.column bar1:

            -- ftd.column bar:
            string title:
            boolean active: false
            string subtitle: $foo
            string bio: $subtitle
            integer w:
            color: $green
            border-width: $w

            --- ftd.text: $title

            --- ftd.text: $subtitle

            --- ftd.text: $bio

            --- ftd.boolean: $active
            ",
            (bag, main),
        );
    }

    #[test]
    fn optional_string_compare() {
        let mut bag = interpreter::default_bag();
        bag.insert(
            s("foo/bar#bar"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("bar"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Optional {
                        data: Box::new(Some(ftd::Value::String {
                            text: "Something".to_string(),
                            source: ftd::TextSource::Caption,
                        })),
                        kind: ftd::ftd2021::p2::Kind::caption(),
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );

        let mut main = p2::default_column();
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("Something"),
                common: Box::new(ftd::Common {
                    condition: Some(ftd::Condition {
                        variable: s("foo/bar#bar"),
                        value: serde_json::Value::String(s("Something")),
                    }),
                    reference: Some(s("foo/bar#bar")),
                    ..Default::default()
                },
                line: true,
                ..Default::default()
            }));

        p!(
            "
            -- optional string bar:

            -- bar: Something

            -- ftd.text: $bar
            if: $bar == Something
            ",
            (bag, main),
        );
    }

    #[test]
    fn hex_color_code() {
        let mut bag = interpreter::default_bag();

        bag.insert(
            s("foo/bar#hex-color"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("hex-color"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Record {
                        name: "ftd#color".to_string(),
                        fields: std::iter::IntoIterator::into_iter([
                            (
                                "light".to_string(),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: "#2cc9b51a".to_string(),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                            (
                                "dark".to_string(),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: "#2cc9b51a".to_string(),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                        ])
                        .collect(),
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );

        let mut main = p2::default_column();

        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("Hello"),
                line: true,
                common: Box::new(ftd::Common {
                    color: Some(ftd::Color {
                        light: ftd::ColorValue {
                            r: 44,
                            g: 201,
                            b: 181,
                            alpha: 0.1,
                        }),
                        dark: ftd::ColorValue {
                            r: 44,
                            g: 201,
                            b: 181,
                            alpha: 0.1,
                        }),
                        reference: Some(s("foo/bar#hex-color")),
                    }),
                    ..Default::default()
                },
                ..Default::default()
            }));

        p!(
            "
            -- ftd.color hex-color:
            light: #2cc9b51a
            dark: #2cc9b51a

            -- ftd.text: Hello
            color: $hex-color
            ",
            (bag, main),
        );
    }

    #[test]
    fn special_variables() {
        let mut bag = interpreter::default_bag();

        bag.insert(
            "foo/bar#current@0".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("current"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Integer { value: 1 },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );

        bag.insert(
            "foo/bar#presentation".to_string(),
            ftd::ftd2021::p2::Thing::Component(ftd::Component {
                root: s("ftd#column"),
                full_name: s("foo/bar#presentation"),
                arguments: [
                    vec![(
                        "current".to_string(),
                        ftd::ftd2021::p2::Kind::integer().set_default(Some(s("1"))),
                    )],
                    universal_arguments_as_vec(),
                ]
                .concat()
                .into_iter()
                .collect(),
                properties: std::iter::IntoIterator::into_iter([
                    (
                        "append-at".to_string(),
                        ftd::ftd2021::component::Property {
                            default: Some(ftd::PropertyValue::Value {
                                value: ftd::Value::String {
                                    text: s("col-id"),
                                    source: ftd::TextSource::Header,
                                },
                            }),
                            ..Default::default()
                        },
                    ),
                    (
                        "open".to_string(),
                        ftd::ftd2021::component::Property {
                            default: Some(ftd::PropertyValue::Value {
                                value: ftd::Value::Boolean { value: true },
                            }),
                            ..Default::default()
                        },
                    ),
                ])
                .collect(),
                instructions: vec![ftd::Instruction::ChildComponent {
                    child: ftd::ChildComponent {
                        root: s("ftd#column"),
                        properties: std::iter::IntoIterator::into_iter([(
                            "id".to_string(),
                            ftd::ftd2021::component::Property {
                                default: Some(ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: s("col-id"),
                                        source: ftd::TextSource::Header,
                                    },
                                }),
                                ..Default::default()
                            },
                        )])
                        .collect(),
                        ..Default::default()
                    },
                }],
                ..Default::default()
            }),
        );

        bag.insert(
            "foo/bar#slide".to_string(),
            ftd::ftd2021::p2::Thing::Component(ftd::Component {
                root: s("ftd#text"),
                full_name: s("foo/bar#slide"),
                arguments: [
                    vec![(s("title"), ftd::ftd2021::p2::Kind::caption())],
                    universal_arguments_as_vec(),
                ]
                .concat()
                .into_iter()
                .collect(),
                properties: std::iter::IntoIterator::into_iter([(
                    s("text"),
                    ftd::ftd2021::component::Property {
                        default: Some(ftd::PropertyValue::Variable {
                            name: s("title"),
                            kind: ftd::ftd2021::p2::Kind::caption_or_body(),
                        }),
                        ..Default::default()
                    },
                )])
                .collect(),
                events: vec![ftd::ftd2021::p2::Event {
                    name: ftd::ftd2021::p2::EventName::OnClick,
                    action: ftd::ftd2021::p2::Action {
                        action: ftd::ftd2021::p2::ActionKind::Increment,
                        target: ftd::PropertyValue::Variable {
                            name: s("PARENT.current"),
                            kind: ftd::ftd2021::p2::Kind::integer(),
                        },
                        parameters: std::iter::IntoIterator::into_iter([(
                            s("clamp"),
                            vec![
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::Integer { value: 1 },
                                },
                                ftd::PropertyValue::Variable {
                                    name: s("PARENT.CHILDREN-COUNT"),
                                    kind: ftd::ftd2021::p2::Kind::integer()
                                        .set_default(Some(s("0"))),
                                },
                            ],
                        )])
                        .collect(),
                    },
                }],
                condition: Some(ftd::ftd2021::p2::Boolean::Equal {
                    left: ftd::PropertyValue::Variable {
                        name: s("PARENT.current"),
                        kind: ftd::ftd2021::p2::Kind::Element,
                    },
                    right: ftd::PropertyValue::Variable {
                        name: s("SIBLING-INDEX"),
                        kind: ftd::ftd2021::p2::Kind::Element,
                    },
                }),
                ..Default::default()
            }),
        );

        bag.insert(
            "foo/bar#title@0,0".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("title"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("First"),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );

        bag.insert(
            "foo/bar#title@0,1".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: s("title"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("Second"),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        );

        let levels = vec![s("0"), s("0,0"), s("0,1")];
        insert_universal_variables_by_levels(levels, "foo/bar", &mut bag);

        let mut main = p2::default_column();
        main.container
            .children
            .push(ftd::Element::Column(ftd::Column {
                container: ftd::Container {
                    children: vec![ftd::Element::Column(ftd::Column {
                        common: Box::new(ftd::Common {
                            data_id: Some(s("col-id")),
                            ..Default::default()
                        },
                        ..Default::default()
                    })],
                    external_children: Some((
                        s("col-id"),
                        vec![vec![0]],
                        vec![ftd::Element::Column(ftd::Column {
                            container: ftd::Container {
                                children: vec![
                                    ftd::Element::Markup(ftd::Markups {
                                        text: ftd::ftd2021::rendered::markup_line("First"),
                                        common: Box::new(ftd::Common {
                                            condition: Some(ftd::Condition {
                                                variable: s("foo/bar#current@0"),
                                                value: serde_json::Value::from(1),
                                            }),
                                            events: vec![ftd::Event {
                                                name: s("onclick"),
                                                action: ftd::Action {
                                                    action: s("increment"),
                                                    target: s("foo/bar#current@0"),
                                                    parameters: std::iter::IntoIterator::into_iter(
                                                        [(
                                                            "clamp".to_string(),
                                                            vec![
                                                        ftd::ftd2021::event::ParameterData {
                                                            value: serde_json::json!(1),
                                                            reference: None,
                                                        }),
                                                        ftd::ftd2021::event::ParameterData {
                                                            value: serde_json::json!(2),
                                                            reference: None,
                                                        }),
                                                    ],
                                                        )],
                                                    )
                                                    .collect(),
                                                }),
                                            }],
                                            reference: Some(s("foo/bar#title@0,0")),
                                            ..Default::default()
                                        },
                                        line: true,
                                        ..Default::default()
                                    }),
                                    ftd::Element::Markup(ftd::Markups {
                                        text: ftd::ftd2021::rendered::markup_line("Second"),
                                        line: true,
                                        common: Box::new(ftd::Common {
                                            condition: Some(ftd::Condition {
                                                variable: s("foo/bar#current@0"),
                                                value: serde_json::json!(2),
                                            }),
                                            is_not_visible: true,
                                            events: vec![ftd::Event {
                                                name: s("onclick"),
                                                action: ftd::Action {
                                                    action: s("increment"),
                                                    target: s("foo/bar#current@0"),
                                                    parameters: std::iter::IntoIterator::into_iter(
                                                        [(
                                                            "clamp".to_string(),
                                                            vec![
                                                        ftd::ftd2021::event::ParameterData {
                                                            value: serde_json::json!(1),
                                                            reference: None,
                                                        }),
                                                        ftd::ftd2021::event::ParameterData {
                                                            value: serde_json::json!(2),
                                                            reference: None,
                                                        }),
                                                    ],
                                                        )],
                                                    )
                                                    .collect(),
                                                }),
                                            }],
                                            reference: Some(s("foo/bar#title@0,1")),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    }),
                                ],
                                ..Default::default()
                            },
                            common: Box::new(ftd::Common {
                                width: Some(ftd::Length::Fill),
                                height: Some(ftd::Length::Fill),
                                position: Some(ftd::Position::Center),
                                ..Default::default()
                            },
                            ..Default::default()
                        })],
                    )),
                    open: Some(true),
                    append_at: Some(s("col-id")),
                    ..Default::default()
                },
                ..Default::default()
            }));

        p!(
            "
            -- presentation:

            --- slide: First

            --- slide: Second


            -- ftd.column presentation:
            open: true
            append-at: col-id
            integer current: 1

            --- ftd.column:
            id: col-id


            -- ftd.text slide: $title
            caption title:
            if: $PARENT.current == $SIBLING-INDEX
            $on-click$: increment $PARENT.current clamp 1 $PARENT.CHILDREN-COUNT

            ",
            (bag, main),
        );
    }

    /*#[test]
    fn optional_condition_on_record() {
        let (_g_bag, g_col) = crate::p2::interpreter::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- record person-data:
                caption name:
                integer age:

                -- person-data person1: Madhav
                age: 10

                -- optional person-data person:

                -- ftd.text: $person.name
                if: $person is not null
                "
            ),
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .expect("found error");
    }*/

    /*#[test]
    fn loop_with_tree_structure_1() {
        let (g_bag, g_col) = ftd::ftd2021::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- record toc-record:
                title: string
                link: string
                children: list toc-record

                -- component toc-item:
                component: ftd.column
                toc-record $toc:
                padding-left: 10

                --- ftd.text: ref $toc.title
                link: ref $toc.link

                --- toc-item:
                $loop$: $toc.children as $obj
                toc: $obj


                -- toc-record list toc:

                -- toc:
                title: ref ab.title
                link: ref ab.link
                children: ref ab.children

                -- toc-record ab:
                title: ab title
                link: ab link

                -- ab.children first_ab
                title: aa title
                link: aa link

                --- children:
                title:

                -- ab.children:
                title: aaa title
                link: aaa link



                -- toc-item:
                $loop$: toc as $obj
                toc: $obj
                "
            ),
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .expect("found error");
        // pretty_assertions::assert_eq!(g_bag, bag);
        // pretty_assertions::assert_eq!(g_col, main);
        // --- toc-item:
        //                 $loop$: $toc.children as $t
        //                 toc: $t
    }

    #[test]
    fn loop_with_tree_structure_2() {
        let (g_bag, g_col) = ftd::ftd2021::test::interpret(
            "foo/bar",
            indoc::indoc!(
                "
                -- record toc-record:
                title: string
                link: string
                children: list toc-record

                -- component toc-item:
                component: ftd.column
                toc-record $toc:
                padding-left: 10

                --- ftd.text: ref $toc.title
                link: ref $toc.link

                --- toc-item:
                $loop$: $toc.children as $obj
                toc: $obj


                -- toc-record list toc:
                $processor$: ft.toc

                - fifthtry/ftd/p1
                  `ftd::p1`: A JSON/YML Replacement
                - fifthtry/ftd/language
                  FTD Language
                  - fifthtry/ftd/p1-grammar
                    `ftd::p1` grammar




                -- toc-item:
                $loop$: $toc as $obj
                toc: $obj
                "
            ),
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .expect("found error");
        // pretty_assertions::assert_eq!(g_bag, bag);
        // pretty_assertions::assert_eq!(g_col, main);
        // --- toc-item:
        //                 $loop$: $toc.children as $t
        //                 toc: $t
    }*/
}

mod component {
    use ftd::ftd2021::test::*;

    macro_rules! p2 {
        ($s:expr, $doc: expr, $t: expr,) => {
            p2!($s, $doc, $t)
        };
        ($s:expr, $doc: expr, $t: expr) => {
            let p1 = ftd::ftd2021::p1::parse(indoc::indoc!($s), $doc.name).unwrap();
            pretty_assertions::assert_eq!(ftd::Component::from_p1(&p1[0], &$doc).unwrap(), $t)
        };
    }

    fn s(s: &str) -> String {
        s.to_string()
    }

    #[test]
    fn component() {
        let mut bag = ftd::ftd2021::p2::interpreter::default_bag();
        let aliases = ftd::ftd2021::p2::interpreter::default_aliases();
        let d = ftd::ftd2021::p2::TDoc {
            name: "foo",
            bag: &mut bag,
            aliases: &aliases,
            local_variables: &mut Default::default(),
            referenced_local_variables: &mut Default::default(),
        };
        p2!(
            "-- ftd.text foo:
            string foo:
            optional integer bar:
            text: hello
            ",
            d,
            ftd::Component {
                full_name: s("foo#foo"),
                root: "ftd#text".to_string(),
                arguments: [
                    vec![
                        (s("foo"), ftd::ftd2021::p2::Kind::string()),
                        (
                            s("bar"),
                            ftd::ftd2021::p2::Kind::optional(ftd::ftd2021::p2::Kind::integer())
                        ),
                    ],
                    universal_arguments_as_vec(),
                ]
                .concat()
                .into_iter()
                .collect(),
                properties: std::iter::IntoIterator::into_iter([(
                    s("text"),
                    ftd::ftd2021::component::Property {
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
                line_number: 1,
                ..Default::default()
            }
        );
    }

    #[test]
    fn properties() {
        let mut bag = ftd::ftd2021::p2::interpreter::default_bag();
        let aliases = ftd::ftd2021::p2::interpreter::default_aliases();
        let d = ftd::ftd2021::p2::TDoc {
            name: "foo",
            bag: &mut bag,
            aliases: &aliases,
            local_variables: &mut Default::default(),
            referenced_local_variables: &mut Default::default(),
        };
        p2!(
            "-- ftd.text foo:
            text: hello
            ",
            d,
            ftd::Component {
                root: "ftd#text".to_string(),
                full_name: s("foo#foo"),
                arguments: universal_arguments_as_map(),
                properties: std::iter::IntoIterator::into_iter([(
                    s("text"),
                    ftd::ftd2021::component::Property {
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
                line_number: 1,
                ..Default::default()
            }
        );
    }

    #[test]
    fn caption_body_conflicts() {
        // Caption and Header Value conflict
        intf!(
            "-- ftd.row A: 
            caption message: Default message
            
            -- A: Im the message here 
            message: No, I'm the chosen one
            ",
            "forbidden usage: pass either caption or header_value for header 'message', line_number: 4, doc: foo"
        );

        // Caption and Body conflict
        intf!(
            "-- ftd.row A: 
            caption or body msg: 
            
            -- A: Caption will say hello

            No, body will say hello

            ",
            "forbidden usage: pass either body or caption or header_value, ambiguity in 'msg', line_number: 4, doc: foo"
        );

        // Body and Header value conflict
        intf!(
            "-- ftd.row A: 
            body msg: 
            
            -- A: 
            msg: Finally I can occupy msg 

            Heh, like you can, Im still here 
            ",
            "forbidden usage: pass either body or header_value for header 'msg', line_number: 4, doc: foo"
        );

        // Caption, Body and Header value conflict
        intf!(
            "-- ftd.text: Im going to catch text first !!
            text: Who knows I might catch it first. 

            Are you sure about that ?
            ",
            "forbidden usage: pass either body or caption or header_value, ambiguity in 'text', line_number: 1, doc: foo"
        );

        // No body accepting header
        intf!(
            "-- ftd.row A:
            caption name:

            -- A: 
            name: Anonymous

            Did I forgot to add some header ?
            Hopefully not             

            ",
            "unknown data: body passed with no header accepting it !!, line_number: 4, doc: foo"
        );

        // No caption accepting header
        intf!(
            "-- ftd.row A:
            body content:

            -- A: There is no victory without sacrifice

            Caption is right but is there any header who will accept it ?

            ",
            "unknown data: caption passed with no header accepting it !!, line_number: 4, doc: foo"
        );

        // No data passed for body
        intf!(
            "-- ftd.row A:
            body content:

            ;; Body not passed here maybe someone mistakenly missed it 
            ;; Or maybe someone out there is testing the fate of this code
            -- A:

            ",
            "missing data: body or header_value, none of them are passed for 'content', line_number: 6, doc: foo"
        );

        // No data passed for caption
        intf!(
            "-- ftd.row A:
            caption title:

            ;; Caption not passed here 
            ;; Maybe someone keeps on forgetting to write necessary stuff
            -- A:

            ",
            "missing data: caption or header_value, none of them are passed for 'title', line_number: 6, doc: foo"
        );
    }

    #[test]
    fn duplicate_headers() {
        // Repeated header definition with the same name (forbidden)
        intf!(
            "-- ftd.row foo:
            caption name:
            string name:
            ",
            "forbidden usage: 'name' is already used as header name/identifier !!, line_number: 3, doc: foo"
        );

        // Value assignment on the same header twice (not allowed)
        intf!(
            "-- ftd.text: Hello friends
            align: center
            align: left
            ",
            "forbidden usage: repeated usage of 'align' not allowed !!, line_number: 3, doc: foo"
        );
    }

    #[test]
    fn referring_variables() {
        let mut bag = default_bag();
        bag.insert(
            "foo/bar#name".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "name".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: s("Amit"),
                        source: ftd::TextSource::Caption,
                    },
                },
                conditions: vec![],
            }),
        );
        let mut main = default_column();
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("Amit"),
                line: true,
                common: Box::new(ftd::Common {
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
            ftd::ftd2021::p2::Thing::Record(ftd::ftd2021::p2::Record {
                name: "foo/bar#person".to_string(),
                fields: person_fields(),
                instances: Default::default(),
                order: vec![s("name"), s("address"), s("bio"), s("age")],
            }),
        );
        bag.insert(
            "foo/bar#x".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "x".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Integer { value: 20 },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#abrar".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "abrar".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Record {
                        name: "foo/bar#person".to_string(),
                        fields: abrar(),
                    },
                },
                conditions: vec![],
            }),
        );

        let mut main = default_column();
        main.container
            .children
            .push(ftd::Element::Markup(ftd::Markups {
                text: ftd::ftd2021::rendered::markup_line("Abrar Khan"),
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

mod record {
    use ftd::ftd2021::test::*;

    #[test]
    fn record() {
        let sourabh: ftd::ftd2021::p2::record::Invocation = std::iter::IntoIterator::into_iter([
            (
                s("name"),
                ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: "Sourabh Garg".to_string(),
                        source: ftd::TextSource::Caption,
                    },
                },
            ),
            (
                s("address"),
                ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: "Ranchi".to_string(),
                        source: ftd::TextSource::Header,
                    },
                },
            ),
            (
                s("bio"),
                ftd::PropertyValue::Value {
                    value: ftd::Value::String {
                        text: "Frontend developer at fifthtry.".to_string(),
                        source: ftd::TextSource::Body,
                    },
                },
            ),
            (
                s("age"),
                ftd::PropertyValue::Value {
                    value: ftd::Value::Integer { value: 28 },
                },
            ),
        ])
        .collect();

        let mut bag = ftd::ftd2021::p2::interpreter::default_bag();
        bag.insert(
            "foo/bar#abrar".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "abrar".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Record {
                        name: "foo/bar#person".to_string(),
                        fields: abrar(),
                    },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#person".to_string(),
            ftd::ftd2021::p2::Thing::Record(ftd::ftd2021::p2::Record {
                name: "foo/bar#person".to_string(),
                fields: person_fields(),
                instances: std::iter::IntoIterator::into_iter([(
                    s("foo/bar"),
                    vec![abrar(), sourabh.clone()],
                )])
                .collect(),
                order: vec![s("name"), s("address"), s("bio"), s("age")],
            }),
        );
        bag.insert(
            "foo/bar#x".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "x".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Integer { value: 20 },
                },
                conditions: vec![],
            }),
        );
        bag.insert(
            "foo/bar#employee".to_string(),
            ftd::ftd2021::p2::Thing::Record(ftd::ftd2021::p2::Record {
                name: "foo/bar#employee".to_string(),
                fields: std::iter::IntoIterator::into_iter([
                    (s("eid"), ftd::ftd2021::p2::Kind::string()),
                    (
                        s("who"),
                        ftd::ftd2021::p2::Kind::Record {
                            name: s("foo/bar#person"),
                            default: None,
                            is_reference: false,
                        },
                    ),
                ])
                .collect(),
                instances: Default::default(),
                order: vec![s("eid"), s("who")],
            }),
        );
        bag.insert(
            "foo/bar#abrar_e".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "abrar_e".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Record {
                        name: "foo/bar#employee".to_string(),
                        fields: std::iter::IntoIterator::into_iter([
                            (
                                s("eid"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: "E04".to_string(),
                                        source: ftd::TextSource::Header,
                                    },
                                },
                            ),
                            (
                                s("who"),
                                ftd::PropertyValue::Reference {
                                    name: s("foo/bar#abrar"),
                                    kind: ftd::ftd2021::p2::Kind::Record {
                                        name: s("foo/bar#person"),
                                        default: None,
                                        is_reference: false,
                                    },
                                },
                            ),
                        ])
                        .collect(),
                    },
                },
                conditions: vec![],
            }),
        );

        bag.insert(
            "foo/bar#sourabh".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "sourabh".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Record {
                        name: "foo/bar#employee".to_string(),
                        fields: std::iter::IntoIterator::into_iter([
                            (
                                s("eid"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: "E05".to_string(),
                                        source: ftd::TextSource::Body,
                                    },
                                },
                            ),
                            (
                                s("who"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::Record {
                                        name: "foo/bar#person".to_string(),
                                        fields: sourabh,
                                    },
                                },
                            ),
                        ])
                        .collect(),
                    },
                },
                conditions: vec![],
            }),
        );

        p!(
            "
            -- record person:
            caption name:
            string address:
            body bio:
            integer age:

            -- integer x: 10

            -- person: Abrar Khan2
            address: Bihar2
            age: $x

            Software developer working at fifthtry2.

            -- person: Sourabh Garg
            address: Ranchi
            age: 28

            Frontend developer at fifthtry.

            -- person abrar: Abrar Khan
            address: Bihar
            age: $x

            Software developer working at fifthtry.

            -- record employee:
            string eid:
            person who:

            -- employee abrar_e:
            eid: E04
            who: $abrar

            -- employee sourabh:

            --- eid:

            E05

            --- who: Sourabh Garg
            address: Ranchi
            age: 28

            Frontend developer at fifthtry.

            -- x: 20

            -- abrar: Abrar Khan2
            address: Bihar2
            age: $x

            Software developer working at fifthtry2.
            ",
            (bag, ftd::ftd2021::p2::interpreter::default_column()),
        );
    }

    #[test]
    fn list() {
        let b = |source: ftd::TextSource| {
            let mut bag = default_bag();

            bag.insert(
                "foo/bar#person".to_string(),
                ftd::ftd2021::p2::Thing::Record(ftd::ftd2021::p2::Record {
                    name: "foo/bar#person".to_string(),
                    fields: std::iter::IntoIterator::into_iter([
                        (s("name"), ftd::ftd2021::p2::Kind::caption()),
                        (
                            s("friends"),
                            ftd::ftd2021::p2::Kind::List {
                                kind: Box::new(ftd::ftd2021::p2::Kind::string()),
                                default: None,
                                is_reference: false,
                            },
                        ),
                    ])
                    .collect(),
                    instances: Default::default(),
                    order: vec![s("name"), s("friends")],
                }),
            );

            bag.insert(
                "foo/bar#abrar".to_string(),
                ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                    flags: ftd::VariableFlags::default(),
                    name: "abrar".to_string(),
                    value: ftd::PropertyValue::Value {
                        value: ftd::Value::Record {
                            name: "foo/bar#person".to_string(),
                            fields: std::iter::IntoIterator::into_iter([
                                (
                                    s("name"),
                                    ftd::PropertyValue::Value {
                                        value: ftd::Value::String {
                                            text: "Abrar Khan".to_string(),
                                            source: ftd::TextSource::Caption,
                                        },
                                    },
                                ),
                                (
                                    s("friends"),
                                    ftd::PropertyValue::Value {
                                        value: ftd::Value::List {
                                            kind: ftd::ftd2021::p2::Kind::string(),
                                            data: vec![
                                                ftd::PropertyValue::Value {
                                                    value: ftd::Value::String {
                                                        text: "Deepak Angrula".to_string(),
                                                        source: source.clone(),
                                                    },
                                                },
                                                ftd::PropertyValue::Value {
                                                    value: ftd::Value::String {
                                                        text: "Amit Upadhyay".to_string(),
                                                        source: source.clone(),
                                                    },
                                                },
                                                ftd::PropertyValue::Value {
                                                    value: ftd::Value::String {
                                                        text: "Saurabh Garg".to_string(),
                                                        source,
                                                    },
                                                },
                                            ],
                                        },
                                    },
                                ),
                            ])
                            .collect(),
                        },
                    },
                    conditions: vec![],
                }),
            );
            bag
        };

        p!(
            "
            -- record person:
            caption name:
            string list friends:

            -- person abrar: Abrar Khan
            friends: Deepak Angrula
            friends: Amit Upadhyay
            friends: Saurabh Garg
            ",
            (b(ftd::TextSource::Header), default_column()),
        );

        p!(
            "
            -- record person:
            caption name:
            string list friends:

            -- person abrar: Abrar Khan

            --- friends: Deepak Angrula
            --- friends: Amit Upadhyay
            --- friends: Saurabh Garg
            ",
            (b(ftd::TextSource::Caption), default_column()),
        );
    }

    #[test]
    fn list_of_records() {
        let mut bag = default_bag();

        bag.insert(
            s("foo/bar#point"),
            ftd::ftd2021::p2::Thing::Record(ftd::ftd2021::p2::Record {
                name: s("foo/bar#point"),
                fields: std::iter::IntoIterator::into_iter([
                    (s("x"), ftd::ftd2021::p2::Kind::integer()),
                    (s("y"), ftd::ftd2021::p2::Kind::integer()),
                ])
                .collect(),
                instances: Default::default(),
                order: vec![s("x"), s("y")],
            }),
        );

        bag.insert(
            "foo/bar#person".to_string(),
            ftd::ftd2021::p2::Thing::Record(ftd::ftd2021::p2::Record {
                name: s("foo/bar#person"),
                fields: std::iter::IntoIterator::into_iter([
                    (s("name"), ftd::ftd2021::p2::Kind::caption()),
                    (
                        s("points"),
                        ftd::ftd2021::p2::Kind::List {
                            kind: Box::new(ftd::ftd2021::p2::Kind::Record {
                                name: s("foo/bar#point"),
                                default: None,
                                is_reference: false,
                            }),
                            default: None,
                            is_reference: false,
                        },
                    ),
                ])
                .collect(),
                instances: Default::default(),
                order: vec![s("name"), s("points")],
            }),
        );

        bag.insert(
            "foo/bar#abrar".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: "abrar".to_string(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Record {
                        name: "foo/bar#person".to_string(),
                        fields: std::iter::IntoIterator::into_iter([
                            (
                                s("name"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::String {
                                        text: "Abrar Khan".to_string(),
                                        source: ftd::TextSource::Caption,
                                    },
                                },
                            ),
                            (
                                s("points"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::List {
                                        kind: ftd::ftd2021::p2::Kind::Record {
                                            name: s("foo/bar#point"),
                                            default: None,
                                            is_reference: false,
                                        },
                                        data: vec![
                                            ftd::PropertyValue::Value {
                                                value: ftd::Value::Record {
                                                    name: "foo/bar#point".to_string(),
                                                    fields: std::iter::IntoIterator::into_iter([
                                                        (
                                                            s("x"),
                                                            ftd::PropertyValue::Value {
                                                                value: ftd::Value::Integer {
                                                                    value: 10,
                                                                },
                                                            },
                                                        ),
                                                        (
                                                            s("y"),
                                                            ftd::PropertyValue::Value {
                                                                value: ftd::Value::Integer {
                                                                    value: 20,
                                                                },
                                                            },
                                                        ),
                                                    ])
                                                    .collect(),
                                                },
                                            },
                                            ftd::PropertyValue::Value {
                                                value: ftd::Value::Record {
                                                    name: "foo/bar#point".to_string(),
                                                    fields: std::iter::IntoIterator::into_iter([
                                                        (
                                                            s("x"),
                                                            ftd::PropertyValue::Value {
                                                                value: ftd::Value::Integer {
                                                                    value: 0,
                                                                },
                                                            },
                                                        ),
                                                        (
                                                            s("y"),
                                                            ftd::PropertyValue::Value {
                                                                value: ftd::Value::Integer {
                                                                    value: 0,
                                                                },
                                                            },
                                                        ),
                                                    ])
                                                    .collect(),
                                                },
                                            },
                                            ftd::PropertyValue::Value {
                                                value: ftd::Value::Record {
                                                    name: "foo/bar#point".to_string(),
                                                    fields: std::iter::IntoIterator::into_iter([
                                                        (
                                                            s("x"),
                                                            ftd::PropertyValue::Value {
                                                                value: ftd::Value::Integer {
                                                                    value: 1,
                                                                },
                                                            },
                                                        ),
                                                        (
                                                            s("y"),
                                                            ftd::PropertyValue::Value {
                                                                value: ftd::Value::Integer {
                                                                    value: 22,
                                                                },
                                                            },
                                                        ),
                                                    ])
                                                    .collect(),
                                                },
                                            },
                                        ],
                                    },
                                },
                            ),
                        ])
                        .collect(),
                    },
                },
                conditions: vec![],
            }),
        );

        p!(
            "
            -- record point:
            integer x:
            integer y:

            -- record person:
            caption name:
            point list points:

            -- person abrar: Abrar Khan

            --- points:
            x: 10
            y: 20

            --- points:
            x: 0
            y: 0

            --- points:
            x: 1
            y: 22
            ",
            (bag, default_column()),
        );
    }

    #[test]
    fn list_of_or_types() {
        let mut bag = default_bag();

        bag.insert(s("foo/bar#entity"), entity());
        bag.insert(
            s("foo/bar#sale"),
            ftd::ftd2021::p2::Thing::Record(ftd::ftd2021::p2::Record {
                name: s("foo/bar#sale"),
                fields: std::iter::IntoIterator::into_iter([
                    (
                        s("party"),
                        ftd::ftd2021::p2::Kind::List {
                            kind: Box::new(ftd::ftd2021::p2::Kind::OrType {
                                name: s("foo/bar#entity"),
                                is_reference: false,
                            }),
                            default: None,
                            is_reference: false,
                        },
                    ),
                    (s("value"), ftd::ftd2021::p2::Kind::integer()),
                ])
                .collect(),
                instances: Default::default(),
                order: vec![s("party"), s("value")],
            }),
        );
        bag.insert(
            s("foo/bar#jan"),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                flags: ftd::VariableFlags::default(),
                name: s("jan"),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::Record {
                        name: s("foo/bar#sale"),
                        fields: std::iter::IntoIterator::into_iter([
                            (
                                s("value"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::Integer { value: 2000 },
                                },
                            ),
                            (
                                s("party"),
                                ftd::PropertyValue::Value {
                                    value: ftd::Value::List {
                                        kind: ftd::ftd2021::p2::Kind::OrType {
                                            name: s("foo/bar#entity"),
                                            is_reference: false,
                                        },
                                        data: vec![
                                            ftd::PropertyValue::Value {value: ftd::Value::OrType {
                                                name: s("foo/bar#entity"),
                                                variant: s("person"),
                                                fields: std::iter::IntoIterator::into_iter([
                                                    (
                                                        s("address"),
                                                        ftd::PropertyValue::Value {
                                                            value: ftd::Value::String {
                                                                text: s("123 Lane"),
                                                                source: ftd::TextSource::Header,
                                                            },
                                                        },
                                                    ),
                                                    (
                                                        s("bio"),
                                                        ftd::PropertyValue::Value {
                                                            value: ftd::Value::String {
                                                                text: s("Owner of Jack Russo\'s Bar"),
                                                                source: ftd::TextSource::Body,
                                                            },
                                                        },
                                                    ),
                                                    (
                                                        s("name"),
                                                        ftd::PropertyValue::Value {
                                                            value: ftd::Value::String {
                                                                text: s("Jack Russo"),
                                                                source: ftd::TextSource::Caption,
                                                            },
                                                        },
                                                    ),
                                                    (
                                                        s("age"),
                                                        ftd::PropertyValue::Value {
                                                            value: ftd::Value::Integer { value: 24 },
                                                        },
                                                    ),
                                                ])
                                                    .collect(),
                                            }},
                                            ftd::PropertyValue::Value {value: ftd::Value::OrType {
                                                name: s("foo/bar#entity"),
                                                variant: s("company"),
                                                fields: std::iter::IntoIterator::into_iter([
                                                    (
                                                        s("industry"),
                                                        ftd::PropertyValue::Value {
                                                            value: ftd::Value::String {
                                                                text: s("Widgets"),
                                                                source: ftd::TextSource::Header,
                                                            },
                                                        },
                                                    ),
                                                    (
                                                        s("name"),
                                                        ftd::PropertyValue::Value {
                                                            value: ftd::Value::String {
                                                                text: s("Acme Inc"),
                                                                source: ftd::TextSource::Caption,
                                                            },
                                                        },
                                                    ),
                                                ])
                                                    .collect(),
                                            }},
                                        ],
                                    },
                                },
                            ),
                        ])
                            .collect(),
                    },
                },
                conditions: vec![],
            }),
        );

        p!(
            "
            -- or-type entity:

            --- person:
            caption name:
            string address:
            body bio:
            integer age:

            --- company:
            caption name:
            string industry:

            -- record sale:
            entity list party:
            integer value:

            -- sale jan:
            value: 2000

            --- party.person: Jack Russo
            address: 123 Lane
            age: 24

            Owner of Jack Russo's Bar

            --- party.company: Acme Inc
            industry: Widgets
            ",
            (bag, default_column()),
        );
    }
}

mod variable {
    use ftd::ftd2021::test::*;

    macro_rules! p2 {
        ($s:expr, $n: expr, $v: expr, $c: expr,) => {
            p2!($s, $n, $v, $c)
        };
        ($s:expr, $n: expr, $v: expr, $c: expr) => {
            let p1 = ftd::ftd2021::p1::parse(indoc::indoc!($s), "foo").unwrap();
            let mut bag = ftd::Map::new();
            let aliases = ftd::Map::new();
            let mut d = ftd::ftd2021::p2::TDoc {
                name: "foo",
                bag: &mut bag,
                aliases: &aliases,
                local_variables: &mut Default::default(),
                referenced_local_variables: &mut Default::default(),
            };
            pretty_assertions::assert_eq!(
                ftd::Variable::from_p1(&p1[0], &mut d).unwrap(),
                ftd::Variable {
                    flags: ftd::VariableFlags::default(),
                    name: $n.to_string(),
                    value: $v,
                    conditions: $c
                }
            )
        };
    }

    #[test]
    fn int() {
        use ftd::Value::Integer;
        p2!(
            "-- integer x: 10",
            "x",
            ftd::PropertyValue::Value {
                value: Integer { value: 10 }
            },
            vec![],
        );
    }

    #[test]
    fn float() {
        use ftd::Value::Decimal;
        p2!(
            "-- decimal x: 10",
            "x",
            ftd::PropertyValue::Value {
                value: Decimal { value: 10.0 }
            },
            vec![],
        );
    }

    #[test]
    fn bool() {
        use ftd::Value::Boolean;
        p2!(
            "-- boolean x: true",
            "x",
            ftd::PropertyValue::Value {
                value: Boolean { value: true }
            },
            vec![],
        );
        p2!(
            "-- boolean x: false",
            "x",
            ftd::PropertyValue::Value {
                value: Boolean { value: false }
            },
            vec![],
        );
    }

    #[test]
    fn str() {
        use ftd::Value::String;
        p2!(
            "-- string x: hello",
            "x",
            ftd::PropertyValue::Value {
                value: String {
                    text: "hello".to_string(),
                    source: ftd::TextSource::Caption
                }
            },
            vec![],
        );
        p2!(
            "-- string x:\n\nhello world\nyo!",
            "x",
            ftd::PropertyValue::Value {
                value: String {
                    text: "hello world\nyo!".to_string(),
                    source: ftd::TextSource::Body
                }
            },
            vec![],
        );
        p2!(
            "-- string x: 10",
            "x",
            ftd::PropertyValue::Value {
                value: String {
                    text: "10".to_string(),
                    source: ftd::TextSource::Caption
                }
            },
            vec![],
        );
        p2!(
            "-- string x: true",
            "x",
            ftd::PropertyValue::Value {
                value: String {
                    text: "true".to_string(),
                    source: ftd::TextSource::Caption
                }
            },
            vec![],
        );
    }

    #[test]
    #[ignore]
    fn list_with_component() {
        let mut bag = default_bag();
        bag.insert(
            s("foo/bar#pull-request"),
            ftd::ftd2021::p2::Thing::Record(ftd::ftd2021::p2::Record {
                name: s("foo/bar#pull-request"),
                fields: std::iter::IntoIterator::into_iter([
                    (s("title"), ftd::ftd2021::p2::Kind::caption()),
                    (s("about"), ftd::ftd2021::p2::Kind::body()),
                ])
                .collect(),
                instances: Default::default(),
                order: vec![s("title"), s("about")],
            }),
        );

        bag.insert(
            "foo/bar#pr".to_string(),
            ftd::ftd2021::p2::Thing::Variable(ftd::Variable {
                name: "foo/bar#pr".to_string(),
                flags: ftd::VariableFlags::default(),
                value: ftd::PropertyValue::Value {
                    value: ftd::Value::List {
                        data: vec![ftd::PropertyValue::Value {
                            value: ftd::Value::Record {
                                name: s("foo/bar#pull-request"),
                                fields: std::iter::IntoIterator::into_iter([
                                    (
                                        s("title"),
                                        ftd::PropertyValue::Value {
                                            value: ftd::Value::String {
                                                text: "some pr".to_string(),
                                                source: ftd::TextSource::Caption,
                                            },
                                        },
                                    ),
                                    (
                                        s("about"),
                                        ftd::PropertyValue::Value {
                                            value: ftd::Value::String {
                                                text: "yo yo".to_string(),
                                                source: ftd::TextSource::Body,
                                            },
                                        },
                                    ),
                                ])
                                .collect(),
                            },
                        }],
                        kind: ftd::ftd2021::p2::Kind::Record {
                            name: s("foo/bar#pull-request"),
                            default: None,
                            is_reference: false,
                        },
                    },
                },
                conditions: vec![],
            }),
        );

        p!(
            "
            -- record pull-request:
            caption title:
            body about:

            -- ftd.column pr-view:
            pull-request pr:

            --- ftd.text:
            text: $pr.title

            --- ftd.text:
            text: $pr.about

            -- list pr:
            type: pull-request

            -- pr: some pr

            yo yo
            ",
            (bag, default_column()),
        );
    }
}

mod document {
    use ftd::ftd2021::test::*;

    #[test]
    fn variable_from_other_doc() {
        let bag = ftd::ftd2021::test::interpret_helper(
            "foo/bar",
            indoc::indoc!(
                "
            -- import: fifthtry/ft

            -- ft.toc:

            foo is the toc
            "
            ),
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .unwrap();

        pretty_assertions::assert_eq!(
            bag.get::<String>("fifthtry/ft#toc").unwrap(),
            "foo is the toc"
        );
    }

    #[test]
    fn meta() {
        #[derive(Debug, PartialEq, serde::Deserialize)]
        #[serde(tag = "type")]
        enum Someone {
            Username { username: String },
            Who { who: String },
        }

        #[derive(Debug, PartialEq, serde::Deserialize)]
        struct Meta {
            license: String,
            reader: Vec<Someone>,
        }

        let bag = ftd::ftd2021::test::interpret_helper(
            "foo/bar",
            indoc::indoc!(
                "
                -- or-type someone:

                --- Username:
                caption username:

                --- Who:
                caption who:

                -- record meta_type:
                string license:
                someone list reader:

                -- meta_type list meta:

                -- meta:
                license: AGPL-3

                --- reader.Username: foo
                --- reader.Who: everyone
            "
            ),
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .unwrap();

        pretty_assertions::assert_eq!(
            bag.get::<Vec<Meta>>("meta").unwrap(),
            vec![Meta {
                license: s("AGPL-3"),
                reader: vec![
                    Someone::Username { username: s("foo") },
                    Someone::Who { who: s("everyone") }
                ],
            }]
        )
    }

    #[test]
    fn instances() {
        #[derive(Debug, PartialEq, serde::Deserialize)]
        struct PR {
            number: i64,
            title: String,
        }

        let bag = ftd::ftd2021::test::interpret_helper(
            "foo/bar",
            indoc::indoc!(
                "
                -- record pr:
                integer number:
                caption title:

                -- pr: some pr
                number: 24

                -- pr: some other pr
                number: 224
                "
            ),
            &ftd::ftd2021::p2::TestLibrary {},
        )
        .unwrap();

        pretty_assertions::assert_eq!(
            bag.instances::<PR>("pr").unwrap(),
            vec![
                PR {
                    number: 24,
                    title: s("some pr")
                },
                PR {
                    number: 224,
                    title: s("some other pr")
                }
            ]
        )
    }
}
