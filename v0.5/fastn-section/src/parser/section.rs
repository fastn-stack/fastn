pub fn section(
    scanner: &mut fastn_section::Scanner<fastn_section::Document>,
) -> Option<fastn_section::Section> {
    let section_init = fastn_section::parser::section_init(scanner)?;

    scanner.skip_spaces();
    let caption = fastn_section::parser::header_value(scanner);

    // Get headers
    let mut new_line = scanner.token("\n");
    let mut headers = vec![];
    if new_line.is_some() {
        headers = fastn_section::parser::headers(scanner);
        if !headers.is_empty() {
            new_line = scanner.token("\n");
        }
    }

    // Get body
    let mut body = None;
    // TODO: Throw error if there are no two new lines before body is found
    if new_line.is_some() && scanner.token("\n").is_some() {
        body = fastn_section::parser::body(scanner);
    }

    Some(fastn_section::Section {
        init: section_init,
        module: scanner.module,
        caption,
        headers,
        body,
        children: vec![],    // children is populated by the wiggin::ender.
        is_commented: false, // TODO
        has_end: false,      // has_end is populated by the wiggin::ender.
    })
}

#[cfg(test)]
mod test {
    fastn_section::tt!(super::section);

    #[test]
    fn section() {
        t!("-- foo: Hello World", {"init": {"name": "foo"}, "caption": ["Hello World"]});
        t!(
            "-- foo: Hello World\ngreeting: hello",
            {
                "init": {"name": "foo"},
                "caption": ["Hello World"],
                "headers": [{
                    "name": "greeting",
                    "value": ["hello"]
                }]
            }
        );
        t!(
            "-- foo: Hello World\ngreeting: hello\nwishes: Be happy",
            {
                "init": {"name": "foo"},
                "caption": ["Hello World"],
                "headers": [
                    {
                        "name": "greeting",
                        "value": ["hello"]
                    },
                    {
                        "name": "wishes",
                        "value": ["Be happy"]
                    }
                ]
            }
        );

        t!(
            "-- foo: Hello World\n\nMy greetings to world!",
            {
                "init": {"name": "foo"},
                "caption": ["Hello World"],
                "body": ["My greetings to world!"]
            }
        );
        t!(
            "-- foo: Hello World\ngreeting: hello\n\nMy greetings to world!",
            {
                "init": {"name": "foo"},
                "caption": ["Hello World"],
                "headers": [{
                    "name": "greeting",
                    "value": ["hello"]
                }],
                "body": ["My greetings to world!"]
            }
        );
    }
}
