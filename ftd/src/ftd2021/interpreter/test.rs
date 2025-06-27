#![allow(dead_code)]

pub fn interpret_helper(
    name: &str,
    source: &str,
) -> ftd::ftd2021::interpreter::Result<ftd::ftd2021::interpreter::Document> {
    let mut s = ftd::ftd2021::interpreter::interpret(name, source)?;
    let document;
    loop {
        match s {
            ftd::ftd2021::interpreter::Interpreter::Done { document: doc } => {
                document = doc;
                break;
            }
            ftd::ftd2021::interpreter::Interpreter::StuckOnImport { module, state: st } => {
                s = st.continue_after_import(module.as_str(), "")?;
            }
        }
    }
    Ok(document)
}

pub fn interpret(
    name: &str,
    source: &str,
) -> ftd::ftd2021::interpreter::Result<ftd::Map<ftd::ftd2021::interpreter::Thing>> {
    let doc = interpret_helper(name, source)?;
    Ok(doc.data)
}

#[cfg(test)]
#[track_caller]
fn p(s: &str, t: &ftd::Map<ftd::ftd2021::interpreter::Thing>) {
    assert_eq!(t, &interpret("foo", s).unwrap_or_else(|e| panic!("{e:?}")))
}

#[test]
fn integer() {
    let bag: ftd::Map<ftd::ftd2021::interpreter::Thing> = std::iter::IntoIterator::into_iter([(
        "foo#age".to_string(),
        ftd::ftd2021::interpreter::Thing::Variable(ftd::ftd2021::interpreter::Variable {
            name: "age".to_string(),
            value: ftd::ftd2021::interpreter::PropertyValue::Value {
                value: ftd::ftd2021::interpreter::Value::Integer { value: 40 },
            },
            conditions: vec![],
            flags: Default::default(),
        }),
    )])
    .collect();
    p("-- integer age: 40", &bag)
}
