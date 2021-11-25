macro_rules! p {
    ($s:expr, $t: expr,) => {
        p!($s, $t)
    };
    ($s:expr, $t: expr) => {
        let (ebag, ecol): (std::collections::BTreeMap<String, crate::p2::Thing>, _) = $t;
        let (mut bag, col) = crate::p2::interpreter::interpret(
            "foo/bar",
            indoc::indoc!($s),
            &ftd::p2::TestLibrary {},
        )
        .expect("found error");
        for v in bag.values_mut() {
            if let crate::p2::Thing::Component(c) = v {
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

pub use crate::p2::interpreter::{default_bag, default_column};

pub fn person_fields() -> std::collections::BTreeMap<String, crate::p2::Kind> {
    std::array::IntoIter::new([
        (s("address"), crate::p2::Kind::string()),
        (s("bio"), crate::p2::Kind::body()),
        (s("age"), crate::p2::Kind::integer()),
        (s("name"), crate::p2::Kind::caption()),
    ])
    .collect()
}

pub fn abrar() -> std::collections::BTreeMap<String, crate::PropertyValue> {
    std::array::IntoIter::new([
        (
            s("name"),
            crate::PropertyValue::Value {
                value: crate::Value::String {
                    text: "Abrar Khan2".to_string(),
                    source: crate::TextSource::Caption,
                },
            },
        ),
        (
            s("address"),
            crate::PropertyValue::Value {
                value: crate::Value::String {
                    text: "Bihar2".to_string(),
                    source: crate::TextSource::Header,
                },
            },
        ),
        (
            s("bio"),
            crate::PropertyValue::Value {
                value: crate::Value::String {
                    text: "Software developer working at fifthtry2.".to_string(),
                    source: crate::TextSource::Body,
                },
            },
        ),
        (
            s("age"),
            crate::PropertyValue::Reference {
                kind: crate::p2::Kind::integer(),
                name: s("foo/bar#x"),
            },
        ),
    ])
    .collect()
}

pub fn entity() -> crate::p2::Thing {
    crate::p2::Thing::OrType(ftd::OrType {
        name: s("foo/bar#entity"),
        variants: vec![
            crate::p2::Record {
                name: s("foo/bar#entity.person"),
                fields: person_fields(),
                instances: Default::default(),
            },
            crate::p2::Record {
                name: s("foo/bar#entity.company"),
                fields: std::array::IntoIter::new([
                    (s("industry"), crate::p2::Kind::string()),
                    (s("name"), crate::p2::Kind::caption()),
                ])
                .collect(),
                instances: Default::default(),
            },
        ],
    })
}
