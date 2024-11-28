#[allow(dead_code)]
pub fn section_init(
    scanner: &mut fastn_section::Scanner<fastn_section::Document>,
) -> Option<fastn_section::SectionInit> {
    scanner.skip_spaces();
    let dashdash = scanner.token("--")?;
    scanner.skip_spaces();
    let kinded_name = fastn_section::parser::kinded_name(scanner)?;
    scanner.skip_spaces();

    let function_marker = scanner.token("(");

    if function_marker.is_some() {
        scanner.skip_spaces();
        scanner.token(")")?;
    }

    scanner.skip_spaces();
    let colon = scanner.token(":")?;

    Some(fastn_section::SectionInit {
        dashdash,
        name: kinded_name.name,
        kind: kinded_name.kind,
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
