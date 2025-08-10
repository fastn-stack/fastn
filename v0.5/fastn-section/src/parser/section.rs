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
/// section = ["/"] section_init [caption] "\n" [headers] ["\n\n" body]
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
///
/// /-- commented: This section is commented out
/// /header: also commented
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
    // Save position before checking for doc comments
    let start_pos = scanner.index();

    // Check for doc comments before section
    let doc = fastn_section::parser::doc_comment(scanner);

    // Check for comment marker before section
    scanner.skip_spaces();
    let is_commented = scanner.take('/');

    let section_init = match fastn_section::parser::section_init(scanner) {
        Some(mut init) => {
            // section_init parser already handles error reporting for missing colon
            init.doc = doc;
            init
        }
        None => {
            // No section found
            if doc.is_some() || is_commented {
                // We consumed a doc comment or comment marker but found no section
                // Reset to start so the document parser can handle it
                scanner.reset(&start_pos);
            }
            return None;
        }
    };

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
        children: vec![], // children is populated by the wiggin::ender.
        is_commented,
        has_end: false, // has_end is populated by the wiggin::ender.
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

    // Check if the next content is a new section (starts with -- or /--)
    if scanner.token("--").is_some() || scanner.token("/--").is_some() {
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
        t_err!(
            "
            -- foo: Hello
            bar: baz
            This is invalid body",
            {
                "init": {"name": "foo"},
                "caption": ["Hello"],
                "headers": [{"name": "bar", "value": ["baz"]}],
                "body": ["This is invalid body"]
            },
            "body_without_double_newline"
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

        // Test commented sections
        t!("/-- foo: Commented", {
            "init": {"name": "foo"},
            "caption": ["Commented"],
            "is_commented": true
        });

        // Commented section with headers
        t!(
            "
            /-- foo: Caption
            header1: value1
            header2: value2",
            {
                "init": {"name": "foo"},
                "caption": ["Caption"],
                "is_commented": true,
                "headers": [
                    {"name": "header1", "value": ["value1"]},
                    {"name": "header2", "value": ["value2"]}
                ]
            }
        );

        // Commented section with body
        t!(
            "
            /-- foo: Test

            This is the body",
            {
                "init": {"name": "foo"},
                "caption": ["Test"],
                "is_commented": true,
                "body": ["This is the body"]
            }
        );

        // Commented section with everything
        t!(
            "
            /-- foo: Complete
            prop: value

            Body content here",
            {
                "init": {"name": "foo"},
                "caption": ["Complete"],
                "is_commented": true,
                "headers": [{"name": "prop", "value": ["value"]}],
                "body": ["Body content here"]
            }
        );

        // Commented section with type
        t!("/-- string msg: Hello", {
            "init": {"name": "msg", "kind": "string"},
            "caption": ["Hello"],
            "is_commented": true
        });

        // Commented section with function marker
        t!("/-- foo(): Func", {
            "init": {"function": "foo"},
            "caption": ["Func"],
            "is_commented": true
        });

        // Commented section with qualified name
        t!("/-- ftd.text: Commented text", {
            "init": {"name": "ftd.text"},
            "caption": ["Commented text"],
            "is_commented": true
        });

        // Mix of commented and uncommented sections
        t!(
            "-- active: Yes
            /-- inactive: No",
            {
                "init": {"name": "active"},
                "caption": ["Yes"]
            },
            "/-- inactive: No"
        );

        // Commented section with commented headers
        t!(
            "
            /-- foo: Section
            /header1: also commented
            header2: normal",
            {
                "init": {"name": "foo"},
                "caption": ["Section"],
                "is_commented": true,
                "headers": [
                    {"name": "header1", "value": ["also commented"], "is_commented": true},
                    {"name": "header2", "value": ["normal"]}
                ]
            }
        );

        // Commented section with spaces
        t!("  /-- foo: Spaced", {
            "init": {"name": "foo"},
            "caption": ["Spaced"],
            "is_commented": true
        });

        // Section with doc comment
        t!(
            "
            ;;; This is documentation
            -- foo: Hello",
            {
                "init": {
                    "name": "foo",
                    "doc": ";;; This is documentation\n"
                },
                "caption": ["Hello"]
            }
        );

        // Section with multi-line doc comment
        t!(
            "
            ;;; This is line 1
            ;;; This is line 2
            ;;; This is line 3
            -- foo: Test",
            {
                "init": {
                    "name": "foo",
                    "doc": ";;; This is line 1\n;;; This is line 2\n;;; This is line 3\n"
                },
                "caption": ["Test"]
            }
        );

        // Section with doc comment and headers
        t!(
            "
            ;;; Documentation for section
            -- foo: Caption
            header1: value1
            header2: value2",
            {
                "init": {
                    "name": "foo",
                    "doc": ";;; Documentation for section\n"
                },
                "caption": ["Caption"],
                "headers": [
                    {"name": "header1", "value": ["value1"]},
                    {"name": "header2", "value": ["value2"]}
                ]
            }
        );

        // Section with doc comment and body
        t!(
            "
            ;;; Section docs
            -- foo: Test

            Body content here",
            {
                "init": {
                    "name": "foo",
                    "doc": ";;; Section docs\n"
                },
                "caption": ["Test"],
                "body": ["Body content here"]
            }
        );

        // Commented section with doc comment
        t!(
            "
            ;;; This section is commented
            /-- foo: Commented",
            {
                "init": {
                    "name": "foo",
                    "doc": ";;; This section is commented\n"
                },
                "caption": ["Commented"],
                "is_commented": true
            }
        );

        // Section with indented doc comment
        t!(
            "    ;;; Indented doc
            -- foo: Test",
            {
                "init": {
                    "name": "foo",
                    "doc": "    ;;; Indented doc\n"
                },
                "caption": ["Test"]
            }
        );

        // Section with doc comment containing special characters
        t!(
            "
            ;;; This uses special chars: @#$%^&*()
            ;;; And unicode: नमस्ते 你好
            -- foo: International",
            {
                "init": {
                    "name": "foo",
                    "doc": ";;; This uses special chars: @#$%^&*()\n;;; And unicode: नमस्ते 你好\n"
                },
                "caption": ["International"]
            }
        );

        // Section with typed name and doc comment
        t!(
            "
            ;;; String type section
            -- string msg: Hello",
            {
                "init": {
                    "name": "msg",
                    "kind": "string",
                    "doc": ";;; String type section\n"
                },
                "caption": ["Hello"]
            }
        );

        // Section with function marker and doc comment
        t!(
            "
            ;;; Function documentation
            -- foo(): Calculate",
            {
                "init": {
                    "function": "foo",
                    "doc": ";;; Function documentation\n"
                },
                "caption": ["Calculate"]
            }
        );

        // Section with headers that have doc comments
        t!(
            "
            -- foo: Section
            ;;; Documentation for header1
            header1: value1
            ;;; Documentation for header2
            header2: value2",
            {
                "init": {"name": "foo"},
                "caption": ["Section"],
                "headers": [
                    {
                        "name": "header1",
                        "doc": ";;; Documentation for header1\n",
                        "value": ["value1"]
                    },
                    {
                        "name": "header2",
                        "doc": ";;; Documentation for header2\n",
                        "value": ["value2"]
                    }
                ]
            }
        );

        // Section with doc comment and headers with doc comments
        t!(
            "
            ;;; Section documentation
            -- foo: Test
            ;;; Header doc
            param: value",
            {
                "init": {
                    "name": "foo",
                    "doc": ";;; Section documentation\n"
                },
                "caption": ["Test"],
                "headers": [{
                    "name": "param",
                    "doc": ";;; Header doc\n",
                    "value": ["value"]
                }]
            }
        );

        // Section with multi-line doc comments on headers
        t!(
            "
            -- config: Settings
            ;;; API endpoint URL
            ;;; Should use HTTPS
            endpoint: https://api.example.com
            ;;; Timeout in seconds
            ;;; Default is 30
            timeout: 60",
            {
                "init": {"name": "config"},
                "caption": ["Settings"],
                "headers": [
                    {
                        "name": "endpoint",
                        "doc": ";;; API endpoint URL\n;;; Should use HTTPS\n",
                        "value": ["https://api.example.com"]
                    },
                    {
                        "name": "timeout",
                        "doc": ";;; Timeout in seconds\n;;; Default is 30\n",
                        "value": ["60"]
                    }
                ]
            }
        );

        // Section with commented headers that have doc comments
        t!(
            "
            -- foo: Test
            ;;; Active setting
            active: true
            ;;; Deprecated setting
            /old: false",
            {
                "init": {"name": "foo"},
                "caption": ["Test"],
                "headers": [
                    {
                        "name": "active",
                        "doc": ";;; Active setting\n",
                        "value": ["true"]
                    },
                    {
                        "name": "old",
                        "doc": ";;; Deprecated setting\n",
                        "is_commented": true,
                        "value": ["false"]
                    }
                ]
            }
        );

        // Section with headers having doc comments, types, and visibility
        t!(
            "
            -- foo: Complex
            ;;; Public configuration
            public string config: production
            ;;; Private key
            private api_key: secret",
            {
                "init": {"name": "foo"},
                "caption": ["Complex"],
                "headers": [
                    {
                        "name": "config",
                        "kind": "string",
                        "visibility": "Public",
                        "doc": ";;; Public configuration\n",
                        "value": ["production"]
                    },
                    {
                        "name": "api_key",
                        "visibility": "Private",
                        "doc": ";;; Private key\n",
                        "value": ["secret"]
                    }
                ]
            }
        );
    }
}
