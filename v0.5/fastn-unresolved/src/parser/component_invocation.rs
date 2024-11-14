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
            name: fastn_unresolved::Identifier(section.name(source).to_string()),
            caption: section.caption,
            arguments: vec![], // todo
            body: vec![],      // todo
            children: vec![],  // todo
        })
}
