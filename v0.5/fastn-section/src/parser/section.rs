pub fn section(
    scanner: &mut fastn_section::Scanner<fastn_section::Document>,
) -> Option<fastn_section::Section> {
    let section_init = fastn_section::parser::section_init(scanner)?;

    scanner.skip_spaces();
    let caption = fastn_section::parser::header_value(scanner);

    // TODO: implement headers, body
    Some(fastn_section::Section {
        init: section_init,
        caption,
        headers: vec![],
        body: None,
        children: vec![],
        function_marker: None,
        is_commented: false,
        has_end: false,
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
