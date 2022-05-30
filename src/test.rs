#[test]
fn get_name() {
    assert_eq!(ftd::get_name("fn", "fn foo", "test").unwrap(), "foo")
}

macro_rules! p {
    ($s:expr, $t: expr,) => {
        p!($s, $t)
    };
    ($s:expr, $t: expr) => {
        let (ebag, ecol): (std::collections::BTreeMap<String, ftd::p2::Thing>, _) = $t;
        let (mut bag, col) =
            ftd::p2::interpreter::interpret("foo/bar", indoc::indoc!($s), &ftd::p2::TestLibrary {})
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
        if !ebag.is_empty() {
            pretty_assertions::assert_eq!(bag, ebag);
        }
        pretty_assertions::assert_eq!(col, ecol);
    };
}

macro_rules! p2 {
    ($s:expr, $t: expr,) => {
        p2!($s, $t)
    };
    ($s:expr, $t: expr) => {
        let (ebag, ecol): (std::collections::BTreeMap<String, ftd::p2::Thing>, _) = $t;
        let (mut bag, col) = ftd::p2::interpreter2::interpret_helper(
            "foo/bar",
            indoc::indoc!($s),
            &ftd::p2::TestLibrary {},
        )
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
        if !ebag.is_empty() {
            pretty_assertions::assert_eq!(bag, ebag);
        }
        pretty_assertions::assert_eq!(col, ecol);
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
    std::array::IntoIter::new([
        (s("address"), ftd::p2::Kind::string()),
        (s("bio"), ftd::p2::Kind::body()),
        (s("age"), ftd::p2::Kind::integer()),
        (s("name"), ftd::p2::Kind::caption()),
    ])
    .collect()
}

pub fn abrar() -> std::collections::BTreeMap<String, ftd::PropertyValue> {
    std::array::IntoIter::new([
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
                fields: std::array::IntoIter::new([
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
