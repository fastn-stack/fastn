pub fn section_init(
    scanner: &mut fastn_parser::section::Scanner,
) -> Option<fastn_parser::SectionInit> {
    scanner.skip_spaces();
    let dashdash = scanner.token("--")?;
    scanner.skip_spaces();
    let name = fastn_parser::section::kinded_name(scanner)?;
    scanner.skip_spaces();
    let colon = scanner.token(":")?;
    Some(fastn_parser::SectionInit {
        dashdash,
        name,
        colon,
    })
}

#[cfg(test)]
mod test {
    fastn_parser::tt!(super::section_init);

    #[test]
    fn section_init() {
        t!("-- foo:", {"name": {"name": "foo"}});
        t!("-- foo: ", {"name": {"name": "foo"}}, " ");
        t!("-- foo: hello", {"name": {"name": "foo"}}, " hello");
        t!("-- integer foo: hello", {"name": {"name": "foo", "kind": "integer"}}, " hello");
        t!("-- integer héllo: foo", {"name": {"name": "héllo", "kind": "integer"}}, " foo");
        // t!("-- list<integer> foo:", {"name": {"name": "foo", "kind": "integer"}}, "");
    }
}
