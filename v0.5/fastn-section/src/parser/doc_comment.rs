/// Parses documentation comments (;;; lines) that appear before sections.
///
/// Doc comments in fastn use triple semicolons (;;;) and must appear
/// immediately before the section they document, with no blank lines in between.
///
/// # Grammar
/// ```text
/// doc_comments = (";;;" <text until newline> "\n")+
/// ```
///
/// # Returns
/// Returns `Some(Span)` containing all consecutive doc comment lines combined,
/// or `None` if no doc comments are found at the current position.
///
/// # Example
/// ```text
/// ;;; This is a doc comment
/// ;;; It can span multiple lines
/// -- section-name:
/// ```
pub fn doc_comment(
    scanner: &mut fastn_section::Scanner<fastn_section::Document>,
) -> Option<fastn_section::Span> {
    let original_position = scanner.index();
    let mut found_doc_comment = false;
    let mut last_position = scanner.index();

    loop {
        // Skip any leading spaces/tabs on the line
        scanner.skip_spaces();

        // Look for ;;;
        if scanner.peek() != Some(';') {
            if found_doc_comment {
                scanner.reset(&last_position);
            } else {
                scanner.reset(&original_position);
            }
            break;
        }
        scanner.pop();

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

        // Check if the next line is also a doc comment
        // If not, we're done collecting doc comments
        let next_line_start = scanner.index();

        // Skip leading spaces on next line
        scanner.skip_spaces();

        // Check for ;;;
        if scanner.peek() == Some(';') {
            scanner.pop();
            if scanner.peek() == Some(';') {
                scanner.pop();
                if scanner.peek() == Some(';') {
                    // Next line is also a doc comment, continue
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

#[cfg(test)]
mod test {
    fastn_section::tt!(super::doc_comment);

    #[test]
    fn doc_comment() {
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
}
