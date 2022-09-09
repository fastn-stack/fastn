#![allow(dead_code)]

pub fn interpret_helper(
    name: &str,
    source: &str,
) -> ftd::interpreter::Result<ftd::interpreter::Document> {
    let mut s = ftd::interpreter::interpret(name, source)?;
    let document;
    loop {
        match s {
            ftd::interpreter::Interpreter::Done { document: doc } => {
                document = doc;
                break;
            }
            ftd::interpreter::Interpreter::StuckOnImport { module, state: st } => {
                s = st.continue_after_import(module.as_str(), "")?;
            }
        }
    }
    Ok(document)
}

pub fn interpret(
    name: &str,
    source: &str,
) -> ftd::interpreter::Result<ftd::Map<ftd::interpreter::Thing>> {
    let doc = interpret_helper(name, source)?;
    Ok(doc.data)
}

#[cfg(test)]
mod test {
    #[track_caller]
    fn p(s: &str, t: &ftd::Map<ftd::interpreter::Thing>) {
        assert_eq!(
            t,
            &super::interpret("foo", s).unwrap_or_else(|e| panic!("{:?}", e))
        )
    }

    #[test]
    fn integer() {
        let bag: ftd::Map<ftd::interpreter::Thing> = std::iter::IntoIterator::into_iter([(
            "foo#age".to_string(),
            ftd::interpreter::Thing::Variable(ftd::interpreter::Variable {
                name: "age".to_string(),
                value: ftd::interpreter::PropertyValue::Value {
                    value: ftd::interpreter::Value::Integer { value: 40 },
                },
                conditions: vec![],
                flags: Default::default(),
            }),
        )])
        .collect();
        p("-- integer age: 40", &bag)
    }
}
