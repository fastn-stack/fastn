pub(super) fn function_definition(
    section: fastn_section::Section,
    document: &mut fastn_unresolved::Document,
    _arena: &mut fastn_section::Arena,
    _package: &Option<&fastn_package::Package>,
) {
    // TODO: remove .unwrap() and put errors in `document.errors`

    let name = section.simple_name_span().clone();
    let visibility = section.init.visibility.map(|v| v.value).unwrap_or_default();

    let return_type: Option<fastn_unresolved::UR<fastn_unresolved::Kind, _>> = section
        .init
        .kind
        .and_then(|k| k.try_into().ok())
        .map(fastn_unresolved::UR::UnResolved);

    let arguments: Vec<_> = section
        .headers
        .into_iter()
        .map(|h| {
            let kind = h.kind.clone().unwrap().try_into().ok().unwrap();
            let visibility = h.visibility.map(|v| v.value).unwrap_or_default();

            fastn_unresolved::Argument {
                name: h.name,
                doc: None,
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
            module: document.module,
            symbol: Default::default(),
            doc: Default::default(),
            aliases: document.aliases.unwrap(),
            visibility,
            name: fastn_section::Identifier { name }.into(),
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
    fn tester(
        mut d: fastn_unresolved::Document,
        expected: serde_json::Value,
        arena: &fastn_section::Arena,
    ) {
        assert!(d.content.is_empty());
        assert_eq!(d.definitions.len(), 1);

        assert_eq!(
            fastn_unresolved::JIDebug::idebug(
                d.definitions.pop().unwrap().unresolved().unwrap(),
                arena
            ),
            expected
        )
    }

    fastn_unresolved::tt!(super::function_definition, tester);

    #[test]
    #[ignore]
    fn function_definition() {
        t!("-- foo():\nstring test:\n\ntodo()", {
            "return_type": "void",
            "name": "foo",
            "content": "todo()",
            "args": [],
        });
    }
}
