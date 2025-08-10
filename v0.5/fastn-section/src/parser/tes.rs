/// Parses Text-Expression-Section (Tes) elements from the scanner.
///
/// Tes elements can appear in header values and body content. They support:
/// - Plain text
/// - Expressions in braces: `{content}` or `${content}`
/// - Inline sections: `{-- section-name: content}`
///
/// # Grammar
/// ```text
/// tes = text | expression | section
/// text = <any content except '{'>
/// expression = '{' tes* '}' | '${' tes* '}'
/// section = '{' '--' <section content> '}'
/// ```
///
/// # Returns
/// Returns a vector of Tes elements parsed from the current position
/// until the specified terminator is reached.
pub fn tes_till(
    scanner: &mut fastn_section::Scanner<fastn_section::Document>,
    terminator: &dyn Fn(&mut fastn_section::Scanner<fastn_section::Document>) -> bool,
) -> Vec<fastn_section::Tes> {
    let mut result = Vec::new();

    while !terminator(scanner) {
        // Check if we're at the end by peeking
        if scanner.peek().is_none() {
            break;
        }

        // Try to get text up to '{' or '${'
        let text_start = scanner.index();
        let mut found_expr = false;
        let mut is_dollar = false;

        while let Some(ch) = scanner.peek() {
            if terminator(scanner) {
                break;
            }
            if ch == '{' {
                found_expr = true;
                break;
            }
            if ch == '$' {
                // Check if next char is '{'
                let save = scanner.index();
                scanner.pop(); // consume '$'
                if scanner.peek() == Some('{') {
                    scanner.reset(&save); // reset to before '$'
                    found_expr = true;
                    is_dollar = true;
                    break;
                }
                scanner.reset(&save);
            }
            scanner.pop();
        }

        // Add any text we found before '{'
        let text_end = scanner.index();
        if text_start != text_end {
            let text_span = scanner.span(text_start);
            result.push(fastn_section::Tes::Text(text_span));
        }

        // If we found an expression starter, try to parse expression
        if found_expr {
            let expr_pos = scanner.index(); // Save position at '{' or '$'

            // If it's a dollar expression, consume the '$'
            if is_dollar {
                scanner.pop(); // consume '$'
            }

            if let Some(expr) = parse_expression(scanner, is_dollar) {
                result.push(expr);
            } else {
                // Failed to parse expression (unclosed brace)
                // Reset scanner to the original position so it's not lost
                scanner.reset(&expr_pos);
                // Stop parsing here
                break;
            }
        }
    }

    result
}

/// Parse a single expression starting at '{'
///
/// # Error Recovery for Unclosed Braces
///
/// When an opening brace `{` is not matched with a closing `}`, we need to recover
/// gracefully. The challenge is that `{...}` expressions are specifically designed
/// to allow multi-line content in headers and body, so we can't simply stop at a
/// newline.
///
/// ## Recovery Strategy Options (Considered)
///
/// 1. **Stop at newline**: Simple but wrong - defeats the purpose of `{...}`
/// 2. **Next closing brace**: Could match wrong `}` if nested expressions exist
/// 3. **Balanced brace counting**: Good but what if no matching `}` exists?
/// 4. **Stop at structural markers**: Safe but might consume too much content
/// 5. **Maximum lookahead**: Arbitrary limits might cut off valid content
///
/// ## Chosen Strategy: Hybrid Approach
///
/// We implement a hybrid recovery strategy that:
/// 1. Tracks nesting depth while scanning for closing braces
/// 2. Stops at structural boundaries (section markers, doc comments)
/// 3. Has a maximum lookahead limit to prevent consuming entire documents
///
/// ### Recovery Algorithm:
/// - Continue scanning while tracking `{` and `}` to maintain nesting depth
/// - Stop if we find a `}` that closes our expression (depth becomes 0)
/// - Stop if we encounter a structural marker at depth 0:
///   - Line starting with `-- ` or `/--` (section start)
///   - Line starting with `;;;` (doc comment)
/// - Stop if we exceed maximum lookahead (1000 characters or 100 lines)
/// - Everything scanned becomes part of the error recovery
///
/// ### Trade-offs:
/// - ✅ Respects document structure (won't consume next section)
/// - ✅ Handles nested expressions correctly
/// - ✅ Bounded search prevents runaway consumption
/// - ⚠️ Might include more content than intended in error
/// - ⚠️ Maximum limits are somewhat arbitrary
fn parse_expression(
    scanner: &mut fastn_section::Scanner<fastn_section::Document>,
    is_dollar: bool,
) -> Option<fastn_section::Tes> {
    let start_index = scanner.index(); // Save scanner position before '{'

    // Consume the '{'
    if !scanner.take('{') {
        return None;
    }

    // Check if this is an inline section {-- foo:}
    // We need to peek ahead to see if it starts with '--'
    let check_index = scanner.index();
    scanner.skip_spaces(); // Skip any leading spaces

    if scanner.peek() == Some('-') {
        let save = scanner.index();
        scanner.pop(); // consume first '-'
        if scanner.peek() == Some('-') {
            scanner.pop(); // consume second '-'
            if scanner.peek() == Some(' ')
                || scanner.peek() == Some('\t')
                || scanner.peek() == Some('\n')
            {
                // This is an inline section!
                scanner.reset(&check_index); // Reset to after '{'
                return parse_inline_section(scanner, start_index);
            }
        }
        scanner.reset(&save);
    }
    scanner.reset(&check_index);

    // Not a section, parse as expression
    // Recursively parse the content inside braces
    let content_tes = parse_expression_content(scanner);

    // Check if we found the closing '}'
    if !scanner.take('}') {
        // No closing brace found - implement error recovery
        // find_recovery_point will consume content up to a reasonable recovery point
        find_recovery_point(scanner);

        // Now scanner is at the recovery point
        let recovery_end = scanner.index();

        // Create error span from the opening brace to recovery point
        let error_span = scanner.span_range(start_index.clone(), recovery_end.clone());
        scanner.add_error(error_span, fastn_section::Error::UnclosedBrace);

        // We return the partial content as an expression
        // This preserves any valid nested expressions we found before the error
        let full_span = scanner.span_range(start_index, recovery_end);
        let expr_start = full_span.start();
        let expr_end = full_span.end();

        return Some(fastn_section::Tes::Expression {
            start: expr_start,
            end: expr_end,
            content: fastn_section::HeaderValue(content_tes),
            is_dollar,
        });
    }

    let end_index = scanner.index();

    // Create span to get start and end positions
    let full_span = scanner.span_range(start_index, end_index);
    let expr_start = full_span.start();
    let expr_end = full_span.end();

    // Create the expression with the parsed content
    let content = fastn_section::HeaderValue(content_tes);

    Some(fastn_section::Tes::Expression {
        start: expr_start,
        end: expr_end,
        content,
        is_dollar,
    })
}

/// Parse an inline section that starts with {--
fn parse_inline_section(
    scanner: &mut fastn_section::Scanner<fastn_section::Document>,
    start_index: fastn_section::scanner::Index,
) -> Option<fastn_section::Tes> {
    // We're positioned right after the '{', and we know '--' follows
    // We need to parse sections but stop at the closing '}'
    let mut sections = Vec::new();

    loop {
        // Check if we've reached the closing brace
        if scanner.peek() == Some('}') {
            scanner.pop(); // consume the '}'
            break;
        }

        // Check for end of input
        if scanner.peek().is_none() {
            // No closing brace - error recovery
            find_recovery_point(scanner);

            let recovery_end = scanner.index();
            let error_span = scanner.span_range(start_index.clone(), recovery_end);
            scanner.add_error(error_span, fastn_section::Error::UnclosedBrace);

            return Some(fastn_section::Tes::Section(sections));
        }

        // Skip whitespace
        scanner.skip_spaces();
        scanner.skip_new_lines();
        scanner.skip_spaces();

        // Check again for closing brace after whitespace
        if scanner.peek() == Some('}') {
            scanner.pop(); // consume the '}'
            break;
        }

        // Try to parse a section init
        if let Some(section_init) = fastn_section::parser::section_init(scanner) {
            // Parse caption - but stop at newline or '}'
            let caption = if scanner.peek() != Some('\n') && scanner.peek() != Some('}') {
                scanner.skip_spaces();
                let caption_terminator =
                    |s: &mut fastn_section::Scanner<fastn_section::Document>| {
                        s.peek() == Some('\n') || s.peek() == Some('}')
                    };
                let caption_tes = tes_till(scanner, &caption_terminator);
                if !caption_tes.is_empty() {
                    Some(fastn_section::HeaderValue(caption_tes))
                } else {
                    None
                }
            } else {
                None
            };

            // Skip to next line if we're at a newline
            scanner.take('\n');

            // Parse headers - stop at double newline, '}', or next section
            let mut headers = Vec::new();
            while scanner.peek() != Some('}') && scanner.peek().is_some() {
                // Check for double newline (body separator)
                if scanner.peek() == Some('\n') {
                    // We're at a newline, check if it's a double newline
                    let check_pos = scanner.index();
                    scanner.take('\n');
                    if scanner.peek() == Some('\n') {
                        // Found double newline - reset and break to parse body
                        scanner.reset(&check_pos);
                        break;
                    }
                    // Single newline - could be before a header
                    // Continue to check for headers
                }

                // Check for next section
                scanner.skip_spaces();
                if scanner.peek() == Some('-') {
                    let save = scanner.index();
                    scanner.pop();
                    if scanner.peek() == Some('-') {
                        // Found next section
                        scanner.reset(&save);
                        break;
                    }
                    scanner.reset(&save);
                }

                // Try to parse a header
                if let Some(header) = fastn_section::parser::headers(scanner) {
                    headers.extend(header);
                } else {
                    break;
                }
            }

            // Parse body if there's a double newline
            let check_newline = scanner.index();
            let has_double_newline = scanner.take('\n') && scanner.peek() == Some('\n');
            if has_double_newline {
                scanner.take('\n'); // consume second newline
            } else {
                scanner.reset(&check_newline); // reset if no double newline
            }

            let body = if has_double_newline {
                // Parse body until '}' or next section
                let body_terminator = |s: &mut fastn_section::Scanner<fastn_section::Document>| {
                    if s.peek() == Some('}') {
                        return true;
                    }
                    // Check for section marker at line start
                    let check = s.index();
                    s.skip_spaces();
                    if s.peek() == Some('-') {
                        let save = s.index();
                        s.pop();
                        if s.peek() == Some('-') {
                            s.reset(&check);
                            return true;
                        }
                        s.reset(&save);
                    }
                    s.reset(&check);
                    false
                };
                let body_tes = tes_till(scanner, &body_terminator);
                if !body_tes.is_empty() {
                    Some(fastn_section::HeaderValue(body_tes))
                } else {
                    None
                }
            } else {
                None
            };

            // Create the section
            let section = fastn_section::Section {
                module: scanner.module,
                init: section_init,
                caption,
                headers,
                body,
                children: vec![],
                is_commented: false,
                has_end: false,
            };

            sections.push(section);
        } else {
            // Couldn't parse a section, stop
            break;
        }
    }

    // Successfully parsed inline sections
    Some(fastn_section::Tes::Section(sections))
}

/// Find a reasonable recovery point for an unclosed brace error
///
/// Implements the hybrid recovery strategy:
/// - Tracks nesting depth of braces
/// - Stops at structural boundaries
/// - Has maximum lookahead limits
///
/// This function consumes content up to the recovery point
fn find_recovery_point(scanner: &mut fastn_section::Scanner<fastn_section::Document>) {
    const MAX_CHARS: usize = 1000;
    const MAX_LINES: usize = 100;
    let mut chars_scanned = 0;
    let mut lines_scanned = 0;
    let mut depth = 0;
    let mut at_line_start = false;

    while let Some(ch) = scanner.peek() {
        // Check limits
        if chars_scanned >= MAX_CHARS || lines_scanned >= MAX_LINES {
            break;
        }

        // Check for structural markers at line start
        if at_line_start && depth == 0 {
            // Check for section markers
            if scanner.one_of(&["-- ", "/--"]).is_some() {
                // Don't consume the section marker
                break;
            }

            // Check for doc comment
            if ch == ';' {
                let save = scanner.index();
                scanner.pop();
                if scanner.peek() == Some(';') {
                    scanner.pop();
                    if scanner.peek() == Some(';') {
                        // Found doc comment, reset and stop
                        scanner.reset(&save);
                        break;
                    }
                }
                scanner.reset(&save);
            }
        }

        // Track nesting depth
        match ch {
            '{' => {
                depth += 1;
                scanner.pop();
            }
            '}' => {
                if depth == 0 {
                    // Found potential closing brace at depth 0
                    scanner.pop(); // consume it
                    break;
                }
                depth -= 1;
                scanner.pop();
            }
            '\n' => {
                lines_scanned += 1;
                at_line_start = true;
                scanner.pop();
            }
            _ => {
                if ch != ' ' && ch != '\t' {
                    at_line_start = false;
                }
                scanner.pop();
            }
        }

        chars_scanned += 1;
    }
}

/// Parse content inside an expression (between { and })
fn parse_expression_content(
    scanner: &mut fastn_section::Scanner<fastn_section::Document>,
) -> Vec<fastn_section::Tes> {
    // Use a closure that stops at '}'
    let terminator =
        |s: &mut fastn_section::Scanner<fastn_section::Document>| s.peek() == Some('}');

    tes_till(scanner, &terminator)
}

/// Parse Tes elements until end of line (for header values)
pub fn tes_till_newline(
    scanner: &mut fastn_section::Scanner<fastn_section::Document>,
) -> Vec<fastn_section::Tes> {
    let terminator =
        |s: &mut fastn_section::Scanner<fastn_section::Document>| s.peek() == Some('\n');
    tes_till(scanner, &terminator)
}

#[cfg(test)]
mod test {
    fastn_section::tt!(super::tes_till_newline);

    #[test]
    fn tes() {
        // Plain text
        t!("hello world", ["hello world"]);

        // Text with brace expression
        t!("hello {world}", ["hello ", {"expression": ["world"]}]);

        // Multiple expressions
        t!("a {b} c {d}", ["a ", {"expression": ["b"]}, " c ", {"expression": ["d"]}]);

        // Nested braces - properly recursive
        t!(
            "outer {inner {nested} more}",
            ["outer ", {"expression": ["inner ", {"expression": ["nested"]}, " more"]}]
        );

        // Unclosed brace - now recovers by consuming content up to recovery point
        // Error is recorded in the document's error list
        t_err!(
            "hello {unclosed",
            ["hello ", {"expression": ["unclosed"]}],
            "unclosed_brace"
        );

        // Empty expression
        t!("empty {}", ["empty ", {"expression": []}]);

        // Complex nesting
        t!(
            "{a {b {c} d} e}",
            [{"expression": ["a ", {"expression": ["b ", {"expression": ["c"]}, " d"]}, " e"]}]
        );

        // Text after expressions
        t!("start {middle} end", ["start ", {"expression": ["middle"]}, " end"]);

        // Dollar expression
        t!("hello ${world}", ["hello ", {"$expression": ["world"]}]);

        // Mixed dollar and regular expressions
        t!("a ${b} c {d}", ["a ", {"$expression": ["b"]}, " c ", {"expression": ["d"]}]);

        // Dollar expression with nested content
        t!("outer ${inner {nested}}", ["outer ", {"$expression": ["inner ", {"expression": ["nested"]}]}]);

        // Dollar without brace is plain text
        t!("just $dollar text", ["just $dollar text"]);

        // Dollar at end of text
        t!("text ends with $", ["text ends with $"]);

        // Multiple dollars
        t!("$100 costs ${price}", ["$100 costs ", {"$expression": ["price"]}]);

        // Multiple unclosed braces - tests array format for errors
        t_err!(
            "{first {second",
            [{"expression": ["first ", {"expression": ["second"]}]}],
            ["unclosed_brace", "unclosed_brace"]
        );

        // Inline section - basic case
        t!(
            "text {-- foo: bar} more",
            ["text ", {"section": [{"init": {"name": "foo"}, "caption": ["bar"]}]}, " more"]
        );

        // Multiple inline sections
        t!(
            "
            {-- foo: one
            -- bar: two}",
            [{"section": [
                {"init": {"name": "foo"}, "caption": ["one"]},
                {"init": {"name": "bar"}, "caption": ["two"]}
            ]}]
        );

        // Inline section with body - FIXME: body parsing not working yet
        // t!(
        //     "
        //     {-- foo: caption
        //
        //     body content}",
        //     [{"section": [{"init": {"name": "foo"}, "caption": ["caption"], "body": ["body content"]}]}]
        // );

        // Mixed expression and inline section
        t!(
            "start {expr} middle {-- inline: section} end",
            ["start ", {"expression": ["expr"]}, " middle ", {"section": [{"init": {"name": "inline"}, "caption": ["section"]}]}, " end"]
        );

        // Unclosed inline section
        t_err!(
            "{-- foo: bar",
            [{"section": [{"init": {"name": "foo"}, "caption": ["bar"]}]}],
            "unclosed_brace"
        );

        // // Inline section with complex caption containing all Tes types - TODO
        // t!(
        //     "
        //     {-- foo: text {expr} ${dollar} {nested {deep}} {-- inner: section}}",
        //     [{"section": [{
        //         "init": {"name": "foo"},
        //         "caption": [
        //             "text ",
        //             {"expression": ["expr"]},
        //             " ",
        //             {"$expression": ["dollar"]},
        //             " ",
        //             {"expression": ["nested ", {"expression": ["deep"]}]},
        //             " ",
        //             {"section": [{"init": {"name": "inner"}, "caption": ["section"]}]}
        //         ]
        //     }]}]
        // );

        // // Inline section with headers - TODO
        // t!(
        //     "
        //     {-- foo: caption
        //     bar: value
        //
        //     body}",
        //     [{"section": [{
        //         "init": {"name": "foo"},
        //         "caption": ["caption"],
        //         "headers": [{"name": "bar", "value": ["value"]}],
        //         "body": ["body"]
        //     }]}]
        // );

        // // Nested inline sections in body - TODO
        // t!(
        //     "
        //     {-- outer: title
        //
        //     Body with {-- nested: inline section} inside}",
        //     [{"section": [{
        //         "init": {"name": "outer"},
        //         "caption": ["title"],
        //         "body": [
        //             "Body with ",
        //             {"section": [{"init": {"name": "nested"}, "caption": ["inline section"]}]},
        //             " inside"
        //         ]
        //     }]}]
        // );
    }
}
