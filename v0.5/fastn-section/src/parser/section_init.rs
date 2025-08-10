/// Parses the initialization part of a section.
///
/// This includes the `--` marker, optional type, name, optional function marker `()`,
/// and the colon separator. The colon is now optional to support error recovery.
///
/// # Grammar
/// ```text
/// section_init = "--" spaces [kind] spaces identifier_reference ["()"] [":"]
/// ```
///
/// # Returns
/// Returns `Some(SectionInit)` if a section start is found, even if the colon is missing.
/// The missing colon can be reported as an error by the caller.
///
/// # Error Recovery
/// If the colon is missing after a valid section name, we still return the `SectionInit`
/// with `colon: None`. This allows parsing to continue and the error to be reported
/// without stopping the entire parse.
///
/// # Examples
/// - `-- foo:` - Basic section
/// - `-- string name:` - Section with type
/// - `-- foo():` - Function section
/// - `-- foo` - Missing colon (returns with colon: None)
pub fn section_init(
    scanner: &mut fastn_section::Scanner<fastn_section::Document>,
) -> Option<fastn_section::SectionInit> {
    scanner.skip_spaces();
    let dashdash = scanner.token("--")?;
    scanner.skip_spaces();

    let kinded_ref = fastn_section::parser::kinded_reference(scanner)?;

    scanner.skip_spaces();

    let function_marker = scanner.token("(");

    if function_marker.is_some() {
        scanner.skip_spaces();
        scanner.token(")")?;
    }

    scanner.skip_spaces();
    let colon = scanner.token(":");

    // Even if colon is missing, we still want to parse the section
    // The missing colon can be reported as an error elsewhere
    Some(fastn_section::SectionInit {
        dashdash,
        name: kinded_ref.name,
        kind: kinded_ref.kind,
        colon,
        function_marker,
        doc: None,
        visibility: None,
    })
}

#[cfg(test)]
mod test {
    fastn_section::tt!(super::section_init);

    #[test]
    fn section_init() {
        t!("-- foo:", {"name": "foo"});
        t!("-- foo: ", {"name": "foo"}, " ");
        t!("-- foo: hello", {"name": "foo"}, " hello");
        t!("-- integer foo: hello", {"name": "foo", "kind": "integer"}, " hello");
        t!("-- integer héllo: foo", {"name": "héllo", "kind": "integer"}, " foo");
        t!("-- integer foo():", {"function": "foo", "kind": "integer"});
        // t!("-- list<integer> foo:", {"name": {"name": "foo", "kind": "integer"}}, "");
    }
}
