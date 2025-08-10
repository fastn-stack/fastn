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

        // Try to get text up to '{'
        let text_start = scanner.index();
        let mut found_brace = false;

        while let Some(ch) = scanner.peek() {
            if terminator(scanner) {
                break;
            }
            if ch == '{' {
                found_brace = true;
                break;
            }
            scanner.pop();
        }

        // Add any text we found before '{'
        let text_end = scanner.index();
        if text_start != text_end {
            let text_span = scanner.span(text_start);
            result.push(fastn_section::Tes::Text(text_span));
        }

        // If we found a brace, try to parse expression
        if found_brace {
            let brace_pos = scanner.index(); // Save position at '{'

            if let Some(expr) = parse_expression(scanner) {
                result.push(expr);
            } else {
                // Failed to parse expression (unclosed brace)
                // Reset scanner to the '{' position so it's not lost
                scanner.reset(&brace_pos);
                // Stop parsing here
                break;
            }
        }
    }

    result
}

/// Parse a single expression starting at '{'
fn parse_expression(
    scanner: &mut fastn_section::Scanner<fastn_section::Document>,
) -> Option<fastn_section::Tes> {
    let start_index = scanner.index(); // Save scanner position before '{'

    // Consume the '{'
    if !scanner.take('{') {
        return None;
    }

    // Remember the byte position where '{' started (need to get it from the original index)
    // We'll calculate start and end positions at the end

    // Recursively parse the content inside braces
    let content_tes = parse_expression_content(scanner);

    // Check if we found the closing '}'
    if !scanner.take('}') {
        // No closing brace found - this is an error case
        // Reset to before the '{' so it's not consumed
        scanner.reset(&start_index);

        // Create a span for the '{' character to report error
        let brace_index = scanner.index();
        scanner.take('{'); // consume it temporarily to create span
        let brace_span = scanner.span(brace_index);
        scanner.reset(&start_index); // reset back

        scanner.add_error(brace_span, fastn_section::Error::UnclosedBrace);
        return None;
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
    })
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

        // Unclosed brace - returns text before it, leaves rest unconsumed
        t!("hello {unclosed", ["hello "], "{unclosed");

        // Empty expression
        t!("empty {}", ["empty ", {"expression": []}]);

        // Complex nesting
        t!(
            "{a {b {c} d} e}",
            [{"expression": ["a ", {"expression": ["b ", {"expression": ["c"]}, " d"]}, " e"]}]
        );

        // Text after expressions
        t!("start {middle} end", ["start ", {"expression": ["middle"]}, " end"]);
    }
}
