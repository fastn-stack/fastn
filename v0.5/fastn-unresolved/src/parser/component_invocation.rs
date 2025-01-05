pub(super) fn component_invocation(
    section: fastn_section::Section,
    document: &mut fastn_unresolved::Document,
    _arena: &mut fastn_section::Arena,
    _package: &Option<&fastn_package::Package>,
) {
    if let Some(ref m) = section.init.function_marker {
        document
            .errors
            .push(m.wrap(fastn_section::Error::ComponentIsNotAFunction));
        // we will go ahead with this component invocation parsing
    }

    document.content.push(
        fastn_unresolved::ComponentInvocation {
            aliases: document.aliases.unwrap(),
            module: document.module,
            name: fastn_unresolved::UR::UnResolved(section.init.name.clone()),
            caption: section.caption.into(),
            properties: vec![],                           // todo
            body: fastn_unresolved::UR::UnResolved(None), // todo
            children: vec![],                             // todo
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
        // assert!(d.imports.is_empty());
        assert!(d.definitions.is_empty());
        assert_eq!(d.content.len(), 1);

        assert_eq!(
            fastn_section::JIDebug::idebug(d.content.pop().unwrap().unresolved().unwrap(), arena),
            expected
        )
    }

    fastn_unresolved::tt!(super::component_invocation, tester);

    #[test]
    fn component_invocation() {
        t!("-- ftd.text: hello", {"content": "ftd.text", "caption": ["hello"]});
    }
}
