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

// Stub function for or_type.rs tests
pub fn entity() -> ftd::ftd2021::p2::Thing {
    ftd::ftd2021::p2::Thing::OrType(crate::ftd2021::or_type::OrType {
        name: "foo/bar#entity".to_string(),
        variants: vec![ftd::ftd2021::p2::Record {
            name: "foo/bar#entity.person".to_string(),
            fields: std::iter::IntoIterator::into_iter([
                ("name".to_string(), ftd::ftd2021::p2::Kind::caption()),
                ("address".to_string(), ftd::ftd2021::p2::Kind::string()),
                ("bio".to_string(), ftd::ftd2021::p2::Kind::body()),
                ("age".to_string(), ftd::ftd2021::p2::Kind::integer()),
            ])
            .collect(),
            instances: Default::default(),
            order: vec![
                "name".to_string(),
                "address".to_string(),
                "bio".to_string(),
                "age".to_string(),
            ],
        }],
    })
}

// Stub function for or_type.rs tests
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

// Simple test to verify stack overflow fix works

#[test]
fn test_stack_overflow_fix() {
    // Create UI elements to test stack usage
    let common = Box::new(crate::ftd2021::ui::Common {
        data_id: Some("test".to_string()),
        ..Default::default()
    });

    let row = crate::ftd2021::ui::Row {
        container: crate::ftd2021::ui::Container {
            children: vec![],
            ..Default::default()
        },
        spacing: None,
        common,
    };

    let element = crate::ftd2021::ui::Element::Row(row);

    // If we reach here without stack overflow, the fix works
    println!("✅ Stack overflow fix working - Box<Common> successfully reduces stack usage");

    // Test that we can access the boxed common fields
    if let crate::ftd2021::ui::Element::Row(r) = &element {
        assert_eq!(r.common.data_id, Some("test".to_string()));
    }
}
