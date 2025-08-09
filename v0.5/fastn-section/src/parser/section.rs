/// Parses a section from the scanner.
///
/// A section is the fundamental building block of fastn documents, consisting of:
/// - Section initialization (`-- name:` with optional type and function marker)
/// - Optional caption (inline content after the colon)
/// - Optional headers (key-value pairs on subsequent lines)
/// - Optional body (free-form content after double newline)
/// - Optional children sections (populated later by wiggin::ender)
///
/// # Grammar
/// ```text
/// section = section_init [caption] "\n" [headers] ["\n\n" body]
/// section_init = "--" spaces [kind] spaces identifier_reference ["()"] ":"
/// headers = (header "\n")*
/// body = <any content until next section or end>
/// ```
///
/// # Examples
/// ```text
/// -- foo: Caption text
/// header1: value1
/// header2: value2
///
/// This is the body content
/// ```
///
/// # Parsing Rules
/// - Headers must be on consecutive lines after the section init
/// - Body must be separated from headers by a double newline
/// - If body content appears without double newline, an error is reported
/// - Children sections are handled separately by the wiggin::ender
///
/// # Error Recovery
/// If body content is detected without proper double newline separator,
/// the parser reports a `BodyWithoutDoubleNewline` error but continues
/// parsing to avoid cascading errors.
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
        scanner.reset(&index);
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
        scanner.reset(&sample_start);

        // Parse the body anyway to consume it and allow parsing to continue
        return fastn_section::parser::body(scanner);
    }

    // No content after single newline
    scanner.reset(&index);

    // No content after single newline - no body to parse
    None
}

#[cfg(test)]
mod test {
    fastn_section::tt!(super::section);

    #[test]
    fn section() {
        // Basic section with just caption
        t!("-- foo: Hello World", {"init": {"name": "foo"}, "caption": ["Hello World"]});

        // Section with one header
        t!(
            "
            -- foo: Hello World
            greeting: hello",
            {
                "init": {"name": "foo"},
                "caption": ["Hello World"],
                "headers": [{
                    "name": "greeting",
                    "value": ["hello"]
                }]
            }
        );

        // Section with multiple headers
        t!(
            "
            -- foo: Hello World
            greeting: hello
            wishes: Be happy",
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

        // Section with body only (no caption, no headers)
        t!(
            "
            -- foo:

            My greetings to world!",
            {
                "init": {"name": "foo"},
                "body": ["My greetings to world!"]
            }
        );

        // Section with caption and body
        t!(
            "
            -- foo: Hello World

            My greetings to world!",
            {
                "init": {"name": "foo"},
                "caption": ["Hello World"],
                "body": ["My greetings to world!"]
            }
        );

        // Section with caption, headers, and body
        t!(
            "
            -- foo: Hello World
            greeting: hello

            My greetings to world!",
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

        // Section with typed name
        t!(
            "
            -- string message: Important",
            {
                "init": {"name": "message", "kind": "string"},
                "caption": ["Important"]
            }
        );

        // Section with generic typed name
        t!(
            "
            -- list<string> items: Shopping List
            count: 5",
            {
                "init": {
                    "name": "items",
                    "kind": {"name": "list", "args": ["string"]}
                },
                "caption": ["Shopping List"],
                "headers": [{
                    "name": "count",
                    "value": ["5"]
                }]
            }
        );

        // Section with qualified name (component invocation)
        t!(
            "
            -- ftd.text: Hello World
            color: red",
            {
                "init": {"name": "ftd.text"},
                "caption": ["Hello World"],
                "headers": [{
                    "name": "color",
                    "value": ["red"]
                }]
            }
        );

        // Section with function marker
        t!(
            "
            -- foo():
            param: value",
            {
                "init": {"function": "foo"},
                "headers": [{
                    "name": "param",
                    "value": ["value"]
                }]
            }
        );

        // Section with empty header value
        t!(
            "
            -- foo: Caption
            empty:
            filled: value",
            {
                "init": {"name": "foo"},
                "caption": ["Caption"],
                "headers": [
                    {"name": "empty"},
                    {"name": "filled", "value": ["value"]}
                ]
            }
        );

        // Headers with types
        t!(
            "
            -- foo:
            string name: John
            integer age: 30",
            {
                "init": {"name": "foo"},
                "headers": [
                    {"name": "name", "kind": "string", "value": ["John"]},
                    {"name": "age", "kind": "integer", "value": ["30"]}
                ]
            }
        );

        // Body with multiple lines
        t!(
            "
            -- foo:

            Line 1
            Line 2
            Line 3",
            {
                "init": {"name": "foo"},
                "body": ["Line 1\nLine 2\nLine 3"]
            }
        );

        // Consecutive sections (no body in first)
        t!(
            "-- first: One
            -- second: Two",
            {
                "init": {"name": "first"},
                "caption": ["One"]
            },
            "-- second: Two"
        );

        // Section stops at next section marker
        t!(
            "
            -- foo: First

            Body content
            -- bar: Second",
            {
                "init": {"name": "foo"},
                "caption": ["First"],
                "body": ["Body content\n"]
            },
            "-- bar: Second"
        );

        // Error case: Headers followed by body without double newline
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

        // Section with no content after colon
        t!(
            "-- foo:",
            {
                "init": {"name": "foo"}
            }
        );

        // Section with whitespace-only caption (whitespace is consumed)
        t!(
            "-- foo:   ",
            {
                "init": {"name": "foo"}
            }
        );

        // Headers with unicode names and values
        t!(
            "
            -- foo:
            नाम: राम
            名前: 太郎",
            {
                "init": {"name": "foo"},
                "headers": [
                    {"name": "नाम", "value": ["राम"]},
                    {"name": "名前", "value": ["太郎"]}
                ]
            }
        );
    }
}
