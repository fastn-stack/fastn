pub(super) fn function_definition(
    section: fastn_section::Section,
    document: &mut fastn_unresolved::Document,
) {
    // TODO: get rid of all the Default::default below
    document.definitions.push(
        fastn_unresolved::Definition {
            symbol: Default::default(),
            module: Default::default(),
            package: Default::default(),
            doc: Default::default(),
            visibility: Default::default(),
            name: fastn_unresolved::Identifier {
                name: section.name_span().clone(),
            }
            .into(),
            inner: fastn_unresolved::InnerDefinition::Function {
                arguments: Default::default(),
                return_type: Default::default(),
                body: Default::default(),
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
    fn component_invocation() {
        t!("-- void foo():\n\ntodo()", { "content": "ftd.text", "caption": "hello" });
    }
}
