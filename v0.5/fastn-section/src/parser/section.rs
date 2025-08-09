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
        if let Some(h) = fastn_section::parser::headers(scanner) {
            headers = h;
            new_line = scanner.token("\n");
        }
    }

    // Get body
    let body = parse_body_with_separator_check(scanner, new_line);

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

/// Parses body content, ensuring proper double newline separator when headers are present.
///
/// If there's content after headers without a double newline separator, reports an error
/// but still parses the body to allow parsing to continue.
fn parse_body_with_separator_check(
    scanner: &mut fastn_section::Scanner<fastn_section::Document>,
    first_newline: Option<fastn_section::Span>,
) -> Option<fastn_section::HeaderValue> {
    first_newline.as_ref()?;

    let second_newline = scanner.token("\n");
    if second_newline.is_some() {
        // Found double newline, parse body normally
        return fastn_section::parser::body(scanner);
    }

    // Single newline but no second newline - check if there's content
    let index = scanner.index();
    scanner.skip_spaces();

    // Check if the next content is a new section (starts with --)
    if scanner.token("--").is_some() {
        // This is a new section, not body content
        scanner.reset(index);
        return None;
    }

    if scanner.peek().is_some() {
        // There's content after single newline that's not a new section - this is an error
        let error_pos = scanner.index();

        // Get a sample of the problematic content for the error message
        // Save position before sampling
        let sample_start = scanner.index();
        let error_sample = scanner
            .take_while(|c| c != '\n')
            .unwrap_or_else(|| scanner.span(error_pos));

        scanner.add_error(error_sample, fastn_section::Error::BodyWithoutDoubleNewline);

        // Reset to before we sampled the error
        scanner.reset(sample_start);

        // Parse the body anyway to consume it and allow parsing to continue
        return fastn_section::parser::body(scanner);
    }

    // No content after single newline
    scanner.reset(index);

    // No content after single newline - no body to parse
    None
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
            "-- foo:\n\nMy greetings to world!",
            {
                "init": {"name": "foo"},
                "body": ["My greetings to world!"]
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

        // Test: Headers followed by body without double newline
        // This should parse the section WITH body but also report an error
        // The body is parsed to allow subsequent sections to be parsed
        // TODO: Verify that BodyWithoutDoubleNewline error is reported at the correct position
        t!(
            "
            -- foo: Hello
            bar: baz
            This is invalid body",
            {
                "init": {"name": "foo"},
                "caption": ["Hello"],
                "headers": [{"name": "bar", "value": ["baz"]}],
                "body": ["This is invalid body"]
            }
        );
    }
}
