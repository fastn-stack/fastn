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

    // Remember the byte position where '{' started (need to get it from the original index)
    // We'll calculate start and end positions at the end

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

/// Find a reasonable recovery point for an unclosed brace error
///
/// Implements the hybrid recovery strategy:
/// - Tracks nesting depth of braces
/// - Stops at structural boundaries
/// - Has maximum lookahead limits
/// 
/// This function consumes content up to the recovery point
fn find_recovery_point(
    scanner: &mut fastn_section::Scanner<fastn_section::Document>,
) {
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
    }
}
