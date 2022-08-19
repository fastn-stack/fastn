#[test]
fn get_name() {
    assert_eq!(ftd::p2::utils::get_name("fn", "fn foo", "test").unwrap(), "foo")
}

pub fn interpret_helper(
    name: &str,
    source: &str,
    lib: &ftd::p2::TestLibrary,
) -> ftd::p1::Result<ftd::p2::Document> {
    let mut s = ftd::p2::interpreter::interpret(name, source)?;
    let document;
    loop {
        match s {
            ftd::p2::interpreter::Interpreter::Done { document: doc } => {
                document = doc;
                break;
            }
            ftd::p2::interpreter::Interpreter::StuckOnProcessor { state, section } => {
                let value = lib.process(
                    &section,
                    &state.tdoc(&mut Default::default(), &mut Default::default()),
                )?;
                s = state.continue_after_processor(&section, value)?;
            }
            ftd::p2::interpreter::Interpreter::StuckOnImport { module, state: st } => {
                let source = lib.get_with_result(
                    module.as_str(),
                    &st.tdoc(&mut Default::default(), &mut Default::default()),
                )?;
                s = st.continue_after_import(module.as_str(), source.as_str())?;
            }
            ftd::p2::interpreter::Interpreter::StuckOnForeignVariable { state, .. } => {
                s = state.continue_after_variable(
                    "foo",
                    ftd::Value::String {
                        text: "This is a test".to_string(),
                        source: ftd::TextSource::Header,
                    },
                )?;
            }
        }
    }
    Ok(document)
}

pub fn interpret(
    name: &str,
    source: &str,
    lib: &ftd::p2::TestLibrary,
) -> ftd::p1::Result<(
    std::collections::BTreeMap<String, ftd::p2::Thing>,
    ftd::Column,
)> {
    let doc = ftd::test::interpret_helper(name, source, lib)?;
    Ok((doc.data, doc.main))
}

macro_rules! p {
    ($s:expr, $t: expr,) => {
        p!($s, $t)
    };
    ($s:expr, $t: expr) => {
        let (ebag, ecol): (std::collections::BTreeMap<String, ftd::p2::Thing>, _) = $t;
        let (mut bag, col) =
            ftd::test::interpret("foo/bar", indoc::indoc!($s), &ftd::p2::TestLibrary {})
                .expect("found error");
        for v in bag.values_mut() {
            if let ftd::p2::Thing::Component(c) = v {
                c.invocations.clear();
                c.line_number = 0;
                for instruction in &mut c.instructions {
                    instruction.without_line_number()
                }
            }
        }
        bag = bag
            .into_iter()
            .filter(|(k, _)| {
                !["SIBLING-INDEX", "CHILDREN-COUNT"]
                    .iter()
                    .any(|v| k.contains(v))
            })
            .collect();
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
        match ftd::test::interpret("foo", indoc::indoc!($s), &ftd::p2::TestLibrary {}) {
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

pub use ftd::p2::interpreter::{default_bag, default_column};

pub fn person_fields() -> std::collections::BTreeMap<String, ftd::p2::Kind> {
    std::iter::IntoIterator::into_iter([
        (s("address"), ftd::p2::Kind::string()),
        (s("bio"), ftd::p2::Kind::body()),
        (s("age"), ftd::p2::Kind::integer()),
        (s("name"), ftd::p2::Kind::caption()),
    ])
    .collect()
}

pub fn abrar() -> std::collections::BTreeMap<String, ftd::PropertyValue> {
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
                kind: ftd::p2::Kind::integer(),
                name: s("foo/bar#x"),
            },
        ),
    ])
    .collect()
}

pub fn entity() -> ftd::p2::Thing {
    ftd::p2::Thing::OrType(ftd::OrType {
        name: s("foo/bar#entity"),
        variants: vec![
            ftd::p2::Record {
                name: s("foo/bar#entity.person"),
                fields: person_fields(),
                instances: Default::default(),
                order: vec![s("name"), s("address"), s("bio"), s("age")],
            },
            ftd::p2::Record {
                name: s("foo/bar#entity.company"),
                fields: std::iter::IntoIterator::into_iter([
                    (s("industry"), ftd::p2::Kind::string()),
                    (s("name"), ftd::p2::Kind::caption()),
                ])
                .collect(),
                instances: Default::default(),
                order: vec![s("name"), s("industry")],
            },
        ],
    })
}
