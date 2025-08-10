/// Represents the initial parts of a header that were successfully parsed
struct ParsedHeaderPrefix {
    kinded_name: fastn_section::KindedName,
    visibility: Option<fastn_section::Spanned<fastn_section::Visibility>>,
    is_commented: bool,
}

/// Result of attempting to parse a single header
enum HeaderParseResult {
    /// Successfully parsed a header
    Success(Box<fastn_section::Header>),
    /// Failed to parse, but consumed input and added errors (e.g., orphaned doc comment)
    FailedWithProgress,
    /// Failed to parse, no input consumed, no errors added
    FailedNoProgress,
}

/// Attempts to parse a single header from the scanner
///
/// Returns a result indicating whether parsing succeeded, failed with progress,
/// or failed without progress. This distinction is important for maintaining
/// parser invariants.
fn parse_single_header(
    scanner: &mut fastn_section::Scanner<fastn_section::Document>,
) -> HeaderParseResult {
    let start_index = scanner.index();

    // Check for doc comments before the header
    let doc = fastn_section::parser::doc_comment(scanner);

    // Save position after doc comment but before skipping spaces
    let after_doc = scanner.index();

    scanner.skip_spaces();

    // Save position after doc comment and spaces
    let header_start = scanner.index();

    // Parse the header prefix (comment marker, visibility, name)
    let prefix = match parse_header_prefix(scanner, &header_start) {
        Some(p) => p,
        None => {
            // If we found a doc comment but no header follows, that's an error
            if let Some(doc_span) = doc {
                scanner.add_error(doc_span, fastn_section::Error::UnexpectedDocComment);
                // Reset to after doc comment but before spaces - don't consume trailing spaces
                scanner.reset(&after_doc);
                // The doc comment has been consumed, so we've made progress
                return HeaderParseResult::FailedWithProgress;
            } else {
                // No doc comment and no header, reset to original position
                scanner.reset(&start_index);
                return HeaderParseResult::FailedNoProgress;
            }
        }
    };

    // Expect a colon after the name
    if scanner.token(":").is_none() {
        // If we found a doc comment but the header is malformed, that's an error
        if let Some(doc_span) = doc {
            scanner.add_error(doc_span, fastn_section::Error::UnexpectedDocComment);
            // Reset to after doc comment but before we started parsing the header
            scanner.reset(&after_doc);
            return HeaderParseResult::FailedWithProgress;
        } else {
            scanner.reset(&start_index);
            return HeaderParseResult::FailedNoProgress;
        }
    }

    // Parse the header value
    scanner.skip_spaces();
    let value = fastn_section::parser::header_value(scanner).unwrap_or_default();

    HeaderParseResult::Success(Box::new(fastn_section::Header {
        name: prefix.kinded_name.name,
        kind: prefix.kinded_name.kind,
        doc,
        visibility: prefix.visibility,
        condition: None, // TODO: implement conditions
        value,
        is_commented: prefix.is_commented,
    }))
}

/// Parses the prefix of a header: comment marker, visibility, and kinded name
///
/// This handles the complex interaction between:
/// - Comment markers (/)
/// - Visibility modifiers (public, private, public<scope>)
/// - The fact that visibility keywords can also be valid identifiers
fn parse_header_prefix<'input>(
    scanner: &mut fastn_section::Scanner<'input, fastn_section::Document>,
    start_index: &fastn_section::scanner::Index<'input>,
) -> Option<ParsedHeaderPrefix> {
    // Check for comment marker
    let is_commented = scanner.take('/');
    if is_commented {
        scanner.skip_spaces();
    }

    // Try to parse visibility modifier
    let visibility_start = scanner.index();
    let visibility = fastn_section::parser::visibility(scanner).map(|v| {
        let visibility_end = scanner.index();
        fastn_section::Spanned {
            span: scanner.span_range(visibility_start, visibility_end),
            value: v,
        }
    });

    scanner.skip_spaces();

    // Try to parse the kinded name
    match fastn_section::parser::kinded_name(scanner) {
        Some(kn) => Some(ParsedHeaderPrefix {
            kinded_name: kn,
            visibility,
            is_commented,
        }),
        None if visibility.is_some() => {
            // If we parsed visibility but can't find a kinded_name,
            // the visibility keyword might actually be the identifier name.
            // Reset and try parsing without visibility.
            parse_header_prefix_without_visibility(scanner, start_index)
        }
        None => None,
    }
}

/// Fallback parser when a visibility keyword might actually be an identifier
///
/// This handles cases like "public: value" where "public" is the header name,
/// not a visibility modifier.
fn parse_header_prefix_without_visibility<'input>(
    scanner: &mut fastn_section::Scanner<'input, fastn_section::Document>,
    start_index: &fastn_section::scanner::Index<'input>,
) -> Option<ParsedHeaderPrefix> {
    scanner.reset(start_index);
    scanner.skip_spaces();

    // Re-parse comment marker if present
    let is_commented = scanner.take('/');
    if is_commented {
        scanner.skip_spaces();
    }

    // Try parsing as a regular kinded_name (no visibility)
    fastn_section::parser::kinded_name(scanner).map(|kn| ParsedHeaderPrefix {
        kinded_name: kn,
        visibility: None,
        is_commented,
    })
}

/// Parses a sequence of headers from the scanner.
///
/// Headers are key-value pairs that appear after a section initialization,
/// each on its own line. They provide metadata and configuration for sections.
///
/// # Grammar
/// ```text
/// headers = (header "\n")*
/// header = spaces ["/"] spaces [visibility] spaces [kind] spaces identifier ":" spaces [header_value]
/// ```
///
/// # Parsing Rules
/// - Each header must be on a separate line
/// - Headers can optionally be commented out with a "/" prefix
/// - Headers can optionally have a visibility modifier (public, private, etc.)
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
/// /disabled: true
/// public api_key: secret
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
    let mut made_progress = false;

    loop {
        let index = scanner.index();

        // Check if we can continue parsing headers
        if found_new_line_at_header_end.is_none() {
            scanner.reset(&index);
            break;
        }

        // Try to parse a single header
        match parse_single_header(scanner) {
            HeaderParseResult::Success(header) => headers.push(*header),
            HeaderParseResult::FailedWithProgress => {
                // We consumed input and added errors, don't reset
                made_progress = true;
                break;
            }
            HeaderParseResult::FailedNoProgress => {
                // No progress made, reset if this was the first attempt
                if headers.is_empty() && !made_progress {
                    scanner.reset(&index);
                }
                break;
            }
        }

        // Track newline position for proper termination
        found_new_line_at_header_end = Some(scanner.index());
        if scanner.token("\n").is_none() {
            found_new_line_at_header_end = None;
        }
    }

    // Reset the scanner before the new line (but only if we didn't fail with progress)
    if !made_progress {
        if let Some(index) = found_new_line_at_header_end {
            scanner.reset(&index);
        }
    }

    if headers.is_empty() && !made_progress {
        None
    } else if headers.is_empty() {
        // Made progress (consumed input with errors) but found no valid headers
        // Return empty vector to indicate we processed something
        Some(vec![])
    } else {
        Some(headers)
    }
}

#[cfg(test)]
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

        // Simple visibility test - single header
        t!("public name: John", [{
            "name": "name",
            "visibility": "Public",
            "value": ["John"]
        }]);

        // Headers with visibility modifiers
        t!(
            "
            public name: John
            private age: 30",
            [
                {
                    "name": "name",
                    "visibility": "Public",
                    "value": ["John"]
                },
                {
                    "name": "age",
                    "visibility": "Private",
                    "value": ["30"]
                }
            ]
        );

        // Headers with package/module visibility
        t!(
            "
            public<package> api_key: secret123
            public<module> internal_id: 42",
            [
                {
                    "name": "api_key",
                    "visibility": "Package",
                    "value": ["secret123"]
                },
                {
                    "name": "internal_id",
                    "visibility": "Module",
                    "value": ["42"]
                }
            ]
        );

        // Visibility with types
        t!(
            "
            public string name: Alice
            public<module> integer count: 100",
            [
                {
                    "name": "name",
                    "kind": "string",
                    "visibility": "Public",
                    "value": ["Alice"]
                },
                {
                    "name": "count",
                    "kind": "integer",
                    "visibility": "Module",
                    "value": ["100"]
                }
            ]
        );

        // Visibility with generic types
        t!(
            "
            public list<string> tags: web, api
            private map<string, int> scores: math: 95",
            [
                {
                    "name": "tags",
                    "kind": {"name": "list", "args": ["string"]},
                    "visibility": "Public",
                    "value": ["web, api"]
                },
                {
                    "name": "scores",
                    "kind": {"name": "map", "args": ["string", "int"]},
                    "visibility": "Private",
                    "value": ["math: 95"]
                }
            ]
        );

        // Visibility with spaces
        t!(
            "
            public   name: value
            public <module>  config: setting",
            [
                {
                    "name": "name",
                    "visibility": "Public",
                    "value": ["value"]
                },
                {
                    "name": "config",
                    "visibility": "Module",
                    "value": ["setting"]
                }
            ]
        );

        // Visibility with newlines inside angle brackets
        t!(
            "
            public<
              package
            > distributed: true
            public<
              ;; Module-only setting
              module
            > debug: false",
            [
                {
                    "name": "distributed",
                    "visibility": "Package",
                    "value": ["true"]
                },
                {
                    "name": "debug",
                    "visibility": "Module",
                    "value": ["false"]
                }
            ]
        );

        // Mixed headers with and without visibility
        t!(
            "
            name: Default
            public visible: yes
            private hidden: secret
            config: value",
            [
                {
                    "name": "name",
                    "value": ["Default"]
                },
                {
                    "name": "visible",
                    "visibility": "Public",
                    "value": ["yes"]
                },
                {
                    "name": "hidden",
                    "visibility": "Private",
                    "value": ["secret"]
                },
                {
                    "name": "config",
                    "value": ["value"]
                }
            ]
        );

        // Test commented headers
        t!("/name: John", [{
            "name": "name",
            "is_commented": true,
            "value": ["John"]
        }]);

        // Commented header with type
        t!("/string name: Alice", [{
            "name": "name",
            "kind": "string",
            "is_commented": true,
            "value": ["Alice"]
        }]);

        // Multiple headers with some commented
        t!(
            "
            name: Bob
            /age: 30
            city: New York",
            [
                {
                    "name": "name",
                    "value": ["Bob"]
                },
                {
                    "name": "age",
                    "is_commented": true,
                    "value": ["30"]
                },
                {
                    "name": "city",
                    "value": ["New York"]
                }
            ]
        );

        // Commented header with visibility
        t!("/public name: Test", [{
            "name": "name",
            "visibility": "Public",
            "is_commented": true,
            "value": ["Test"]
        }]);

        // Commented header with visibility and type
        t!("/public string api_key: secret123", [{
            "name": "api_key",
            "kind": "string",
            "visibility": "Public",
            "is_commented": true,
            "value": ["secret123"]
        }]);

        // Commented header with package visibility
        t!("/public<package> internal: data", [{
            "name": "internal",
            "visibility": "Package",
            "is_commented": true,
            "value": ["data"]
        }]);

        // Mixed commented and uncommented with visibility
        t!(
            "
            public name: Active
            /private secret: hidden
            public<module> config: enabled
            /config2: disabled",
            [
                {
                    "name": "name",
                    "visibility": "Public",
                    "value": ["Active"]
                },
                {
                    "name": "secret",
                    "visibility": "Private",
                    "is_commented": true,
                    "value": ["hidden"]
                },
                {
                    "name": "config",
                    "visibility": "Module",
                    "value": ["enabled"]
                },
                {
                    "name": "config2",
                    "is_commented": true,
                    "value": ["disabled"]
                }
            ]
        );

        // Commented header with generic type
        t!("/list<string> items: apple, banana", [{
            "name": "items",
            "kind": {"name": "list", "args": ["string"]},
            "is_commented": true,
            "value": ["apple, banana"]
        }]);

        // Commented empty header
        t!("/empty:", [{
            "name": "empty",
            "is_commented": true
        }]);

        // Edge case: visibility keyword as name when commented
        t!("/public: this is a value", [{
            "name": "public",
            "is_commented": true,
            "value": ["this is a value"]
        }]);

        // Header with doc comment
        t!(
            "
            ;;; This is documentation for name
            name: John",
            [{
                "name": "name",
                "doc": ";;; This is documentation for name\n",
                "value": ["John"]
            }]
        );

        // Multiple headers with doc comments
        t!(
            "
            ;;; User's full name
            name: Alice
            ;;; User's age in years
            age: 25",
            [
                {
                    "name": "name",
                    "doc": ";;; User's full name\n",
                    "value": ["Alice"]
                },
                {
                    "name": "age",
                    "doc": ";;; User's age in years\n",
                    "value": ["25"]
                }
            ]
        );

        // Header with multi-line doc comment
        t!(
            "
            ;;; API key for authentication
            ;;; Should be kept secret
            ;;; Rotate every 90 days
            api_key: sk_123456",
            [{
                "name": "api_key",
                "doc": ";;; API key for authentication\n;;; Should be kept secret\n;;; Rotate every 90 days\n",
                "value": ["sk_123456"]
            }]
        );

        // Header with doc comment and type
        t!(
            "
            ;;; Configuration value
            string config: production",
            [{
                "name": "config",
                "kind": "string",
                "doc": ";;; Configuration value\n",
                "value": ["production"]
            }]
        );

        // Header with doc comment and visibility
        t!(
            "
            ;;; Public API endpoint
            public endpoint: /api/v1",
            [{
                "name": "endpoint",
                "visibility": "Public",
                "doc": ";;; Public API endpoint\n",
                "value": ["/api/v1"]
            }]
        );

        // Header with doc comment, visibility, and type
        t!(
            "
            ;;; List of supported features
            public list<string> features: auth, logging",
            [{
                "name": "features",
                "kind": {"name": "list", "args": ["string"]},
                "visibility": "Public",
                "doc": ";;; List of supported features\n",
                "value": ["auth, logging"]
            }]
        );

        // Commented header with doc comment
        t!(
            "
            ;;; Deprecated setting
            /old_config: value",
            [{
                "name": "old_config",
                "is_commented": true,
                "doc": ";;; Deprecated setting\n",
                "value": ["value"]
            }]
        );

        // Mixed headers with and without doc comments
        t!(
            "
            ;;; Main configuration
            config: value1
            simple: value2
            ;;; Another documented header
            documented: value3",
            [
                {
                    "name": "config",
                    "doc": ";;; Main configuration\n",
                    "value": ["value1"]
                },
                {
                    "name": "simple",
                    "value": ["value2"]
                },
                {
                    "name": "documented",
                    "doc": ";;; Another documented header\n",
                    "value": ["value3"]
                }
            ]
        );

        // Header with empty value and doc comment
        t!(
            "
            ;;; Optional field
            optional:",
            [{
                "name": "optional",
                "doc": ";;; Optional field\n"
            }]
        );

        // Doc comment with special characters in documentation
        t!(
            "
            ;;; Price in USD ($)
            ;;; Use format: $XX.XX
            price: $19.99",
            [{
                "name": "price",
                "doc": ";;; Price in USD ($)\n;;; Use format: $XX.XX\n",
                "value": ["$19.99"]
            }]
        );

        // Orphaned doc comment (no header after doc comment) - returns empty vector with error
        // The parser must advance when adding errors to satisfy invariants
        // Test with raw strings to avoid indoc issues
        t_err_raw!(
            ";;; This doc comment has no header\n",
            [],
            "unexpected_doc_comment",
            ""
        );
        t_err_raw!(
            "    ;;; This doc comment has no header\n    ",
            [],
            "unexpected_doc_comment",
            "    "
        );

        // Orphaned doc comment followed by invalid header (missing colon) - also reports error
        // The doc comment is consumed, but the invalid header text remains
        t_err_raw!(
            ";;; Documentation\nno_colon",
            [],
            "unexpected_doc_comment",
            "no_colon"
        );
        t_err_raw!(
            "    ;;; Documentation\n    no_colon",
            [],
            "unexpected_doc_comment",
            "    no_colon"
        );
    }
}
