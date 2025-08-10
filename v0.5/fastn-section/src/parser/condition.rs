/// Parses a condition expression in the form: `if { expression }`
///
/// # Grammar
/// ```text
/// condition ::= "if" spaces "{" condition_tes_list "}"
/// ```
///
/// The content inside the braces is parsed as a restricted TES expression that
/// CANNOT contain inline sections (-- syntax). Only text and expressions are allowed.
/// Comments and newlines are allowed inside the braces.
///
/// # Examples
/// ```text
/// if { dark-mode }
/// if { mobile && logged-in }
/// if { $count > 5 }  // $ here is just text, not a dollar expression
/// if { ${count} > 5 }  // This is a dollar expression
/// if { user has {premium access} }
/// if {
///   ;; Check multiple conditions
///   dark-mode &&
///   high-contrast
/// }
/// ```
///
/// # Returns
/// Returns `Some(HeaderValue)` containing the parsed expression,
/// or `None` if no condition is found.
pub fn condition(
    scanner: &mut fastn_section::Scanner<fastn_section::Document>,
) -> Option<fastn_section::HeaderValue> {
    let start = scanner.index();
    
    // Check for "if" keyword
    if !scanner.token("if").is_some() {
        scanner.reset(&start);
        return None;
    }
    
    scanner.skip_spaces();
    
    // Check for opening brace
    if scanner.peek() != Some('{') {
        scanner.reset(&start);
        return None;
    }
    
    // Parse the condition expression using a restricted TES parser
    // that doesn't allow inline sections
    let error_count_before = scanner.output.errors.len();
    match parse_condition_expression(scanner) {
        Some(content) => Some(content),
        None => {
            // Only reset if no errors were added (if errors were added, we must advance)
            if scanner.output.errors.len() == error_count_before {
                scanner.reset(&start);
            }
            None
        }
    }
}

/// Parses the expression inside condition braces
/// This is like TES but without inline sections
fn parse_condition_expression(
    scanner: &mut fastn_section::Scanner<fastn_section::Document>,
) -> Option<fastn_section::HeaderValue> {
    if !scanner.take('{') {
        return None;
    }
    
    let mut result = Vec::new();
    let mut text_start = scanner.index();
    
    while let Some(ch) = scanner.peek() {
        match ch {
            '}' => {
                // Capture any trailing text before the closing brace
                let text_end = scanner.index();
                if text_start.clone() != text_end {
                    let span = scanner.span_range(text_start.clone(), text_end);
                    if !span.str().is_empty() {
                        result.push(fastn_section::Tes::Text(span));
                    }
                }
                scanner.take('}');
                return Some(fastn_section::HeaderValue(result));
            }
            '{' => {
                // Capture text before the nested expression
                let text_end = scanner.index();
                if text_start.clone() != text_end {
                    let span = scanner.span_range(text_start.clone(), text_end);
                    if !span.str().is_empty() {
                        result.push(fastn_section::Tes::Text(span));
                    }
                }
                
                // Parse nested expression
                let expr_start = scanner.index();
                if let Some(nested) = parse_condition_expression(scanner) {
                    let expr_end = scanner.index();
                    // Create a span for the expression
                    let expr_span = scanner.span_range(expr_start, expr_end);
                    result.push(fastn_section::Tes::Expression {
                        start: expr_span.start(),
                        end: expr_span.end(),
                        content: nested,
                        is_dollar: false,
                    });
                    text_start = scanner.index();
                } else {
                    // If nested parsing fails, treat { as regular text
                    scanner.pop();
                }
            }
            '$' => {
                // Check for dollar expression - only ${} is a dollar expression
                let dollar_pos = scanner.index();
                scanner.pop(); // consume $
                if scanner.peek() == Some('{') {
                    // This is a dollar expression
                    // Capture text before the dollar expression
                    let text_end = dollar_pos.clone();
                    if text_start.clone() != text_end {
                        let span = scanner.span_range(text_start.clone(), text_end);
                        if !span.str().is_empty() {
                            result.push(fastn_section::Tes::Text(span));
                        }
                    }
                    
                    // Parse dollar expression - remember the $ position
                    let dollar_start = dollar_pos.clone();
                    let expr_start_idx = scanner.index();
                    if let Some(nested) = parse_condition_expression(scanner) {
                        let expr_end_idx = scanner.index();
                        // Calculate positions for the Tes::Expression
                        let expr_span_start = scanner.span_range(dollar_start.clone(), expr_start_idx);
                        let expr_span_end = scanner.span_range(dollar_start, expr_end_idx);
                        result.push(fastn_section::Tes::Expression {
                            start: expr_span_start.start(),
                            end: expr_span_end.end(),
                            content: nested,
                            is_dollar: true,
                        });
                        text_start = scanner.index();
                    } else {
                        // If nested parsing fails, continue ($ and { will be part of text)
                    }
                } else {
                    // $ without { is just regular text, continue scanning
                }
            }
            ';' => {
                // Check for comments (;; for line comments)
                let semi_pos = scanner.index();
                scanner.pop(); // consume first ;
                if scanner.peek() == Some(';') {
                    // This is a comment - capture any text before it
                    let text_end = semi_pos.clone();
                    if text_start.clone() != text_end {
                        let span = scanner.span_range(text_start.clone(), text_end);
                        if !span.str().is_empty() {
                            result.push(fastn_section::Tes::Text(span));
                        }
                    }
                    
                    // Skip the comment
                    scanner.pop(); // consume second ;
                    while let Some(ch) = scanner.peek() {
                        if ch == '\n' {
                            // Don't consume the newline, leave it for the next text segment
                            break;
                        }
                        scanner.pop();
                    }
                    // Reset text_start after the comment
                    text_start = scanner.index();
                } else {
                    // Single ; is just regular text, continue scanning
                }
            }
            '-' => {
                // Check if this might be an inline section (which we must reject)
                let dash_pos = scanner.index();
                scanner.pop(); // consume first -
                if scanner.peek() == Some('-') {
                    scanner.pop(); // consume second -
                    scanner.skip_spaces();
                    // If we see an identifier after --, this is an inline section attempt
                    if scanner.peek().map_or(false, |c| c.is_alphabetic() || c == '_') {
                        // This is an inline section - not allowed in conditions
                        // Add error and fail the condition parsing
                        let error_start = dash_pos;
                        // Consume the section name for error reporting
                        while scanner.peek().map_or(false, |c| c.is_alphanumeric() || c == '_' || c == '-') {
                            scanner.pop();
                        }
                        let error_end = scanner.index();
                        let error_span = scanner.span_range(error_start, error_end);
                        scanner.add_error(error_span, fastn_section::Error::SectionNotAllowedInCondition);
                        // Continue scanning to find the closing brace to satisfy invariant
                        // (parser must advance if it adds an error)
                        let mut brace_depth = 1;
                        while let Some(ch) = scanner.peek() {
                            scanner.pop();
                            if ch == '{' {
                                brace_depth += 1;
                            } else if ch == '}' {
                                brace_depth -= 1;
                                if brace_depth == 0 {
                                    break;
                                }
                            }
                        }
                        return None;
                    }
                    // Not an inline section, continue as text
                }
                // Continue scanning as regular text
            }
            _ => {
                scanner.pop();
            }
        }
    }
    
    // Unclosed brace
    None
}

#[cfg(test)]
mod test {
    fastn_section::tt!(super::condition);
    
    #[test]
    fn condition() {
        // Basic conditions
        t!("if { dark-mode }", [" dark-mode "]);
        t!("if { mobile }", [" mobile "]);
        t!("if {desktop}", ["desktop"]);
        
        // Conditions with operators (as plain text, $ is just text)
        t!("if { $count > 5 }", [" $count > 5 "]);  // $ is just text
        t!("if { dark-mode && high-contrast }", [" dark-mode && high-contrast "]);
        t!("if { hover || focus }", [" hover || focus "]);
        
        // Conditions with spaces
        t!("if   {   spaced   }", ["   spaced   "]);
        t!("if{tight}", ["tight"]);
        
        // Complex conditions ($ is just text)
        t!("if { user.role == 'admin' }", [" user.role == 'admin' "]);
        t!("if { (a && b) || c }", [" (a && b) || c "]);
        t!("if { $var.field }", [" $var.field "]);  // $ is just text
        
        // Actual dollar expressions use ${}
        t!("if { ${count} > 5 }", [" ", {"$expression": ["count"]}, " > 5 "]);
        t!("if { dark-mode && ${user.premium} }", [" dark-mode && ", {"$expression": ["user.premium"]}, " "]);
        t!("if { prefix${value}suffix }", [" prefix", {"$expression": ["value"]}, "suffix "]);
        
        // Nested expressions in condition
        t!("if { check {nested} }", [" check ", {"expression": ["nested"]}, " "]);
        t!("if { a || {b && c} }", [" a || ", {"expression": ["b && c"]}, " "]);
        
        // Mixed nested and dollar expressions
        t!("if { ${outer {inner}} }", [" ", {"$expression": ["outer ", {"expression": ["inner"]}]}, " "]);
        
        // Conditions with newlines using indoc
        t!(
            "if {
              dark-mode
            }",
            ["\n  dark-mode\n"]
        );
        
        t!(
            "if {
              mobile &&
              logged-in
            }",
            ["\n  mobile &&\n  logged-in\n"]
        );
        
        t!(
            "if {
            
              multi-line
            
            }",
            ["\n\n  multi-line\n\n"]
        );
        
        // Conditions with comments (comments are skipped, not included in output)
        // Using t_raw to preserve exact spacing
        t_raw!(
            "if { ;; this is a comment\n              value }",
            [" ", "\n              value "]
        );
        
        t_raw!(
            "if { before ;; inline comment\n              after }",
            [" before ", "\n              after "]
        );
        
        t!(
            "if {
              ;; Comment at start
              dark-mode &&
              ;; Comment in middle
              high-contrast
              ;; Comment at end
            }",
            ["\n  ", "\n  dark-mode &&\n  ", "\n  high-contrast\n  ", "\n"]
        );
        
        // Multi-line with mixed content
        t!(
            "if {
              ${value} &&
              {nested
                content}
            }",
            ["\n  ", {"$expression": ["value"]}, " &&\n  ", {"expression": ["nested\n    content"]}, "\n"]
        );
        
        // Comments don't affect parsing
        t_raw!(
            "if { a ;; comment here\n             && b }",
            [" a ", "\n             && b "]
        );
        
        t_raw!(
            "if { ;; start comment\n             x ;; middle\n             ;; end comment\n             }",
            [" ", "\n             x ", "\n             ", "\n             "]
        );
        
        // No condition
        f!("no condition here");
        f!("if without braces");
        f!("if {");  // Unclosed brace
        f!("if }");  // No opening brace
        
        // Not quite conditions
        f!("iff { not if }");
        f!("if");
        f!("{ just braces }");
        
        // Inline sections are NOT allowed in conditions (even with newlines)
        t_err!("if { -- section: not allowed }", null, "section_not_allowed_in_condition");
        t_err!("if { text -- foo: bar }", null, "section_not_allowed_in_condition");
        t_err!("if { before -- component: test }", null, "section_not_allowed_in_condition");
        
        t_err!(
            "if {
              -- section: nope
            }",
            null,
            "section_not_allowed_in_condition"
        );
        
        t_err!(
            "if { ;; comment
              -- foo: bar
            }",
            null,
            "section_not_allowed_in_condition"
        );
    }
}