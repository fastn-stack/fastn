/// Parses documentation comments that appear before sections or at module level.
///
/// Doc comments in fastn use different patterns:
/// - Module docs: `;-;` (semicolon-dash-semicolon)
/// - Regular docs: `;;;` (triple semicolons)
///
/// # Grammar
/// ```text
/// module_doc = (";-;" <text until newline> "\n")+
/// doc_comments = (";;;" <text until newline> "\n")+
/// ```
///
/// # Parameters
/// - `scanner`: The scanner to parse from
/// - `is_module_doc`: If true, parse `;-;` module docs, otherwise parse `;;;` regular docs
///
/// # Returns
/// Returns `Some(Span)` containing all consecutive doc comment lines combined,
/// or `None` if no doc comments are found at the current position.
///
/// # Examples
/// ```text
/// ;-; This is a module doc comment
/// ;-; It documents the entire file
///
/// ;;; This is a regular doc comment
/// ;;; It documents the next section
/// -- section-name:
/// ```
fn doc_comment(
    scanner: &mut fastn_section::Scanner<fastn_section::Document>,
    is_module_doc: bool,
) -> Option<fastn_section::Span> {
    let original_position = scanner.index();
    let mut found_doc_comment = false;
    let mut last_position = scanner.index();

    loop {
        // Skip any leading spaces/tabs on the line
        scanner.skip_spaces();

        // Look for the doc comment pattern
        if scanner.peek() != Some(';') {
            if found_doc_comment {
                scanner.reset(&last_position);
            } else {
                scanner.reset(&original_position);
            }
            break;
        }
        scanner.pop();

        // Check for the second character (either '-' for module doc or ';' for regular doc)
        let expected_second = if is_module_doc { '-' } else { ';' };
        if scanner.peek() != Some(expected_second) {
            // Not the expected doc comment type, reset
            if found_doc_comment {
                scanner.reset(&last_position);
            } else {
                scanner.reset(&original_position);
            }
            break;
        }
        scanner.pop();

        // Check for the third character (';' in both cases)
        if scanner.peek() != Some(';') {
            // Not a doc comment, reset to before we consumed anything
            if found_doc_comment {
                scanner.reset(&last_position);
            } else {
                scanner.reset(&original_position);
            }
            break;
        }
        scanner.pop();

        found_doc_comment = true;

        // Skip the rest of the line (the actual doc comment text)
        while let Some(c) = scanner.peek() {
            if c == '\n' {
                break;
            }
            scanner.pop();
        }

        // Consume the newline
        if !scanner.take('\n') {
            // End of file after doc comment
            break;
        }

        last_position = scanner.index();

        // Check if the next line is also a doc comment of the same type
        // If not, we're done collecting doc comments
        let next_line_start = scanner.index();

        // Skip leading spaces on next line
        scanner.skip_spaces();

        // Check for the same doc comment pattern
        if scanner.peek() == Some(';') {
            scanner.pop();
            let expected_second = if is_module_doc { '-' } else { ';' };
            if scanner.peek() == Some(expected_second) {
                scanner.pop();
                if scanner.peek() == Some(';') {
                    // Next line is also a doc comment of the same type, continue
                    scanner.reset(&next_line_start);
                    continue;
                }
            }
        }
        scanner.reset(&next_line_start);
        break;
    }

    if found_doc_comment {
        Some(scanner.span(original_position))
    } else {
        None
    }
}

/// Parses regular documentation comments (;;; lines) that appear before sections and headers.
pub fn regular_doc_comment(
    scanner: &mut fastn_section::Scanner<fastn_section::Document>,
) -> Option<fastn_section::Span> {
    doc_comment(scanner, false)
}

/// Parses module documentation comments (;-; lines).
pub fn module_doc_comment(
    scanner: &mut fastn_section::Scanner<fastn_section::Document>,
) -> Option<fastn_section::Span> {
    doc_comment(scanner, true)
}

#[cfg(test)]
mod test {
    #[test]
    fn regular_doc_comment() {
        fastn_section::tt!(super::regular_doc_comment);
        // Single line doc comment (no indentation)
        t!(
            ";;; This is a doc comment
",
            ";;; This is a doc comment\n"
        );

        // Single line doc comment (with indentation)
        t!(
            "    ;;; Indented doc comment
",
            "    ;;; Indented doc comment\n"
        );

        // Multiple line doc comments (no indentation)
        t!(
            ";;; First line
;;; Second line
",
            ";;; First line\n;;; Second line\n"
        );

        // Multiple line doc comments (indoc removes common leading whitespace)
        t!(
            "
            ;;; First line
            ;;; Second line
            ",
            ";;; First line\n;;; Second line\n"
        );

        // Test with raw strings to prove our parser preserves leading whitespace
        t_raw!(
            "    ;;; First line\n    ;;; Second line\n",
            "    ;;; First line\n    ;;; Second line\n"
        );

        // Another raw test with mixed indentation
        t_raw!(
            "  ;;; Less indent\n        ;;; More indent\n",
            "  ;;; Less indent\n        ;;; More indent\n"
        );

        // Doc comment with content after
        t!(
            "
            ;;; Doc comment
            -- section:",
            ";;; Doc comment\n",
            "-- section:"
        );

        // Not a doc comment (only two semicolons) - no indentation
        f!(";; Regular comment");

        // Not a doc comment (only two semicolons) - with indentation
        f_raw!("    ;; Regular comment\n");

        // Not a doc comment (no semicolons)
        f!("Not a comment");

        // Doc comment without newline at end (EOF)
        t!(";;; Doc at EOF", ";;; Doc at EOF");

        // Multiple doc comments with blank line should stop at blank
        t!(
            "
            ;;; First block

            ;;; Second block
            ",
            ";;; First block\n",
            "\n;;; Second block\n"
        );

        // Doc comment with spaces after ;;; (no trailing line)
        t!(
            ";;;   Indented doc
",
            ";;;   Indented doc\n"
        );

        // Doc comment with spaces after ;;; (with trailing blank line)
        t!(
            ";;;   Indented doc

",
            ";;;   Indented doc\n",
            "\n"
        );

        // Empty doc comment (no trailing)
        t!(
            ";;;
", ";;;\n"
        );

        // Empty doc comment (with trailing blank)
        t!(
            ";;;

", ";;;\n", "\n"
        );

        // Mixed with regular comments should stop
        t!(
            "
            ;;; Doc comment
            ;; Regular comment
            ;;; Another doc",
            ";;; Doc comment\n",
            ";; Regular comment\n;;; Another doc"
        );

        // Doc comments with complex content
        t!(
            "
            ;;; This function calculates the sum of two numbers
            ;;; Parameters:
            ;;;   - a: first number
            ;;;   - b: second number
            ",
            ";;; This function calculates the sum of two numbers\n;;; Parameters:\n;;;   - a: first number\n;;;   - b: second number\n"
        );

        // Raw test: Doc comment cannot start with a newline (would not be a doc comment)
        f_raw!("\n;;; After newline");

        // Raw test: Tab indentation is preserved
        t_raw!("\t;;; Tab indented\n", "\t;;; Tab indented\n");

        // Raw test: Mixed spaces and tabs are preserved
        t_raw!(
            "  \t  ;;; Mixed indentation\n",
            "  \t  ;;; Mixed indentation\n"
        );
    }

    #[test]
    fn module_doc_comment() {
        fastn_section::tt!(super::module_doc_comment);

        // Single line module doc comment
        t!(
            ";-; This is a module doc comment
",
            ";-; This is a module doc comment\n"
        );

        // Multiple line module doc comments
        t!(
            ";-; Module documentation
;-; Second line of module docs
",
            ";-; Module documentation\n;-; Second line of module docs\n"
        );

        // Module doc with content after
        t!(
            "
            ;-; Module doc
            -- section:",
            ";-; Module doc\n",
            "-- section:"
        );

        // Not a module doc comment (;;; is regular doc)
        f!(";;; Regular comment");

        // Not a module doc comment (;; is regular comment)
        f!(";; Regular comment");

        // Module doc with spaces after ;-;
        t!(
            ";-;   Module with spaces
",
            ";-;   Module with spaces\n"
        );

        // Multiple module docs with blank line should stop at blank
        t!(
            "
            ;-; First block

            ;-; Second block
            ",
            ";-; First block\n",
            "\n;-; Second block\n"
        );

        // Empty module doc comment
        t!(
            ";-;
", ";-;\n"
        );

        // Raw test: Module doc with tab indentation
        t_raw!("\t;-; Tab indented\n", "\t;-; Tab indented\n");
    }
}
