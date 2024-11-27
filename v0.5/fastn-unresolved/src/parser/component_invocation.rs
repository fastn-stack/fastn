pub(super) fn component_invocation(
    section: fastn_section::Section,
    document: &mut fastn_unresolved::Document,
) {
    if let Some(ref m) = section.init.function_marker {
        document
            .errors
            .push(m.wrap(fastn_section::Error::ComponentIsNotAFunction));
        // we will go ahead with this component invocation parsing
    }

    document.content.push(
        fastn_unresolved::ComponentInvocation {
            module: document.module.clone(),
            name: fastn_unresolved::IdentifierReference::Local(fastn_unresolved::Identifier {
                name: section.name_span().clone(),
            })
            .into(),
            caption: section.caption.into(),
            properties: vec![],  // todo
            body: vec![].into(), // todo
            children: vec![],    // todo
        }
        .into(),
    )
}

#[cfg(test)]
mod tests {
    fn tester(mut d: fastn_unresolved::Document, expected: serde_json::Value) {
        // assert!(d.imports.is_empty());
        assert!(d.definitions.is_empty());
        assert_eq!(d.content.len(), 1);

        assert_eq!(
            fastn_jdebug::JDebug::debug(d.content.pop().unwrap().unresolved().unwrap()),
            expected
        )
    }

    fastn_unresolved::tt!(super::component_invocation, tester);

    #[test]
    fn component_invocation() {
        t!("-- ftd.text: hello", { "content": "ftd.text", "caption": ["hello"] });
    }
}
