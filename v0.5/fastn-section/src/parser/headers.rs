/// Parses a sequence of headers from the scanner.
///
/// Headers are key-value pairs that appear after a section initialization,
/// each on its own line. They provide metadata and configuration for sections.
///
/// # Grammar
/// ```text
/// headers = (header "\n")*
/// header = spaces [kind] spaces identifier ":" spaces [header_value]
/// ```
///
/// # Parsing Rules
/// - Each header must be on a separate line
/// - Headers can optionally have a type prefix (e.g., `string name: value`)
/// - Headers must have a colon after the name
/// - Headers stop when:
///   - A double newline is encountered (indicating start of body)
///   - No valid kinded name can be parsed
///   - No colon is found after a kinded name
///   - End of input is reached
/// - Empty values are allowed (e.g., `key:` with no value)
/// - Whitespace is allowed around the colon
///
/// # Examples
/// ```text
/// name: John
/// age: 30
/// string city: New York
/// list<string> items: apple, banana
/// empty:
/// ```
///
/// # Returns
/// Returns `Some(Vec<Header>)` if at least one header is successfully parsed,
/// `None` if no headers are found. The scanner position is reset to before
/// the terminating newline (if any) to allow proper body parsing.
pub fn headers(
    scanner: &mut fastn_section::Scanner<fastn_section::Document>,
) -> Option<Vec<fastn_section::Header>> {
    let mut headers = vec![];
    let mut found_new_line_at_header_end = Some(scanner.index());

    loop {
        let index = scanner.index();
        scanner.skip_spaces();
        let kinded_name = match fastn_section::parser::kinded_name(scanner) {
            Some(kn) => kn,
            None => {
                scanner.reset(index);
                break;
            }
        };
        let colon = scanner.token(":");
        if colon.is_none() {
            scanner.reset(index);
            break;
        }

        if found_new_line_at_header_end.is_none() {
            scanner.reset(index);
            break;
        }

        scanner.skip_spaces();
        let value = fastn_section::parser::header_value(scanner).unwrap_or_default();

        // TODO: all the rest
        headers.push(fastn_section::Header {
            name: kinded_name.name,
            kind: kinded_name.kind,
            doc: None,
            visibility: None,
            condition: None,
            value,
            is_commented: false,
        });

        found_new_line_at_header_end = Some(scanner.index());
        let new_line = scanner.token("\n");
        if new_line.is_none() {
            found_new_line_at_header_end = None;
        }
    }

    // Reset the scanner before the new line
    if let Some(index) = found_new_line_at_header_end {
        scanner.reset(index);
    }

    if headers.is_empty() {
        None
    } else {
        Some(headers)
    }
}

mod test {
    fastn_section::tt!(super::headers);

    #[test]
    fn headers() {
        // Basic cases
        t!("greeting: hello", [{"name": "greeting", "value": ["hello"]}]);
        t!("greeting: hello\n", [{"name": "greeting", "value": ["hello"]}], "\n");

        // Multiple headers
        t!(
            "
            greeting: hello
            wishes: Happy New Year
            ",
            [
                {
                    "name": "greeting",
                    "value": ["hello"]
                },
                {
                    "name": "wishes",
                    "value": ["Happy New Year"]
                }
            ],
            "\n"
        );

        // Headers stop at double newline
        t!(
            "
            greeting: hello
            wishes: Happy New Year

            I am not header",
            [
                {
                    "name": "greeting",
                    "value": ["hello"]
                },
                {
                    "name": "wishes",
                    "value": ["Happy New Year"]
                }
            ],
            "\n\nI am not header"
        );

        // Headers stop at double newline even if body looks like header
        t!(
            "
            greeting: hello
            wishes: Happy New Year

            body: This looks like a header
            but: it is not",
            [
                {
                    "name": "greeting",
                    "value": ["hello"]
                },
                {
                    "name": "wishes",
                    "value": ["Happy New Year"]
                }
            ],
            "\n\nbody: This looks like a header\nbut: it is not"
        );

        // Headers with types
        t!(
            "
            string name: John
            integer age: 30",
            [
                {
                    "name": "name",
                    "kind": "string",
                    "value": ["John"]
                },
                {
                    "name": "age",
                    "kind": "integer",
                    "value": ["30"]
                }
            ]
        );

        // Headers with generic types
        t!(
            "
            list<string> items: apple, banana
            map<string, int> scores: math: 95",
            [
                {
                    "name": "items",
                    "kind": {"name": "list", "args": ["string"]},
                    "value": ["apple, banana"]
                },
                {
                    "name": "scores",
                    "kind": {"name": "map", "args": ["string", "int"]},
                    "value": ["math: 95"]
                }
            ]
        );

        // Headers with underscores, hyphens, and numbers
        t!(
            "
            first_name: Alice
            last-name: Smith
            _private: secret
            value123: test
            item_42: answer",
            [
                {
                    "name": "first_name",
                    "value": ["Alice"]
                },
                {
                    "name": "last-name",
                    "value": ["Smith"]
                },
                {
                    "name": "_private",
                    "value": ["secret"]
                },
                {
                    "name": "value123",
                    "value": ["test"]
                },
                {
                    "name": "item_42",
                    "value": ["answer"]
                }
            ]
        );

        // Empty value (no value field in JSON when empty)
        t!(
            "
            empty:
            another: value",
            [
                {
                    "name": "empty"
                },
                {
                    "name": "another",
                    "value": ["value"]
                }
            ]
        );

        // Spaces around colon
        t!(
            "
            spaced : value
            tight:value",
            [
                {
                    "name": "spaced",
                    "value": ["value"]
                },
                {
                    "name": "tight",
                    "value": ["value"]
                }
            ]
        );

        // Type without following name (converts to name)
        t!("string: some value",
            [
                {
                    "name": "string",
                    "value": ["some value"]
                }
            ]
        );

        // Headers must be on separate lines (colon in value doesn't create new header)
        t!("a: 1 b: 2",
            [
                {
                    "name": "a",
                    "value": ["1 b: 2"]
                }
            ]
        );

        // No headers (empty input) - returns None
        f!("");

        // Just whitespace - returns None
        f!("   \n  ");

        // Missing colon - returns None and doesn't consume
        f!("no colon here");
    }
}
