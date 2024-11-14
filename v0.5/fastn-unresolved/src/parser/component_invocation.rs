pub(super) fn component_invocation(
    source: &str,
    section: fastn_section::Section,
    document: &mut fastn_unresolved::Document,
) {
    if let Some(ref m) = section.function_marker {
        document
            .errors
            .push(m.wrap(fastn_section::Error::ComponentIsNotAFunction));
        // we will go ahead with this component invocation parsing
    }

    document
        .content
        .push(fastn_unresolved::ComponentInvocation {
            name: fastn_unresolved::Identifier(section.name(source).to_string()).into(),
            caption: section.caption,
            properties: vec![], // todo
            body: vec![],       // todo
            children: vec![],   // todo
        })
}

#[cfg(test)]
mod tests {
    fn tester(mut d: fastn_unresolved::Document, source: &str, expected: serde_json::Value) {
        assert!(d.imports.is_empty());
        assert!(d.definitions.is_empty());
        assert_eq!(d.content.len(), 1);

        assert_eq!(
            fastn_section::JDebug::debug(&d.content.pop().unwrap(), source),
            expected
        )
    }

    fastn_unresolved::tt!(super::component_invocation, tester);

    #[test]
    fn component_invocation() {
        t!("-- ftd.text: hello", { "content": "ftd.text", "caption": "hello" });
    }
}
