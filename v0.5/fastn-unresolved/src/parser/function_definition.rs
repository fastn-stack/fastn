pub(super) fn function_definition(
    section: fastn_section::Section,
    document: &mut fastn_unresolved::Document,
) {
    // TODO: remove .unwrap() and put errors in `document.errors`

    let name = section.name_span().clone();
    let visibility = section
        .init
        .name
        .kind
        .as_ref()
        .and_then(|x| x.visibility.clone())
        .unwrap_or_default()
        .value;

    let return_type: Option<fastn_unresolved::UR<fastn_unresolved::Kind, _>> = section
        .init
        .name
        .kind
        .and_then(|k| k.try_into().ok())
        .map(|k| fastn_unresolved::UR::UnResolved(k));

    let arguments: Vec<_> = section
        .headers
        .into_iter()
        .map(|h| {
            let kind = h.name.kind.clone().unwrap().try_into().ok().unwrap();
            let visibility = h
                .name
                .kind
                .and_then(|x| x.visibility)
                .unwrap_or_default()
                .value;

            fastn_unresolved::Argument {
                name: h.name.name,
                kind,
                visibility,
                default: Default::default(), // TODO: parse TES
            }
            .into()
        })
        .collect();

    let body = section
        .body
        .unwrap()
        .0
        .into_iter()
        .map(|b| b.into())
        .collect();

    // TODO: get rid of all the Default::default below
    document.definitions.push(
        fastn_unresolved::Definition {
            symbol: Default::default(),
            module: Default::default(),
            package: Default::default(),
            doc: Default::default(),
            visibility,
            name: fastn_unresolved::Identifier { name }.into(),
            inner: fastn_unresolved::InnerDefinition::Function {
                arguments,
                return_type,
                body,
            },
        }
        .into(),
    )
}

#[cfg(test)]
mod tests {
    fn tester(mut d: fastn_unresolved::Document, expected: serde_json::Value) {
        assert!(d.imports.is_empty());
        assert!(d.content.is_empty());
        assert_eq!(d.definitions.len(), 1);

        assert_eq!(
            fastn_jdebug::JDebug::debug(d.definitions.pop().unwrap().unresolved().unwrap()),
            expected
        )
    }

    fastn_unresolved::tt!(super::function_definition, tester);

    #[test]
    fn function_definition() {
        t!("-- foo():\nstring test:\n\ntodo()", {
            "return_type": "void",
            "name": "foo",
            "content": "todo()",
            "args": [],
        });
    }
}
