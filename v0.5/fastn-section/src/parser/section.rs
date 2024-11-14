pub fn section(
    scanner: &mut fastn_section::Scanner<fastn_section::Document>,
) -> Option<fastn_section::Section> {
    let section_init = fastn_section::parser::section_init(scanner)?;

    scanner.skip_spaces();
    let caption = fastn_section::parser::header_value(scanner);

    Some(fastn_section::Section {
        init: section_init,
        caption,
        headers: fastn_section::parser::headers(scanner),
        body: fastn_section::parser::body(scanner),
        children: vec![],      // children is populated by the wiggin::ender.
        function_marker: None, // TODO
        is_commented: false,   // TODO
        has_end: false,        // has_end is populated by the wiggin::ender.
    })
}

#[cfg(test)]
mod test {
    fastn_section::tt!(super::section);

    #[test]
    fn section() {
        t!("-- foo: Hello World", {"init": {"name": "foo"}, "caption": ["Hello World"]});
    }
}
