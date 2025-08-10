/// Parses the initialization part of a section.
///
/// This includes the `--` marker, optional type, name, optional function marker `()`,
/// and the colon separator. The colon is now optional to support error recovery.
///
/// # Grammar
/// ```text
/// section_init = "--" spaces [kind] spaces identifier_reference ["(" ws ")"] [":"]
/// ws = (space | tab | newline | comment)*
/// ```
///
/// # Returns
/// Returns `Some(SectionInit)` if a section start is found, even if the colon is missing.
/// The missing colon can be reported as an error by the caller.
///
/// # Error Recovery
/// - If the colon is missing after a valid section name, we still return the `SectionInit`
///   with `colon: None`. This allows parsing to continue and the error to be reported
///   without stopping the entire parse.
/// - For malformed dash markers (single dash, triple dash), we parse what we can and
///   record errors for the caller to handle.
///
/// # Examples
/// - `-- foo:` - Basic section
/// - `-- string name:` - Section with type
/// - `-- foo():` - Function section
/// - `-- foo` - Missing colon (returns with colon: None)
/// - `-- foo(\n  ;; comment\n):` - Function with whitespace/comments in parens
pub fn section_init(
    scanner: &mut fastn_section::Scanner<fastn_section::Document>,
) -> Option<fastn_section::SectionInit> {
    scanner.skip_spaces();
    
    // Check for dash markers - we want to handle -, --, --- etc for error recovery
    let start_pos = scanner.index();
    let mut dash_count = 0;
    while scanner.peek() == Some('-') {
        scanner.pop();
        dash_count += 1;
        if dash_count >= 3 {
            break; // Stop at 3 or more dashes
        }
    }
    
    // If no dashes found, return None
    if dash_count == 0 {
        return None;
    }
    
    let dashdash = scanner.span(start_pos.clone());
    
    // Record error if not exactly 2 dashes
    if dash_count != 2 {
        scanner.add_error(dashdash.clone(), fastn_section::Error::DashCountError);
    }
    
    scanner.skip_spaces();

    // Try to parse kinded_reference - if missing, record error but continue
    let (name, kind) = match fastn_section::parser::kinded_reference(scanner) {
        Some(kr) => (kr.name, kr.kind),
        None => {
            // No name found - record error
            let error_span = dashdash.clone();
            scanner.add_error(error_span, fastn_section::Error::MissingName);
            
            // Check if there's a function marker without name (like "-- ():")
            scanner.skip_spaces();
            let func_marker = if scanner.peek() == Some('(') {
                let fm = scanner.token("(");
                scanner.skip_all_whitespace();
                if !scanner.take(')') {
                    // Unclosed parenthesis
                    let error_span = scanner.span(start_pos.clone());
                    scanner.add_error(error_span, fastn_section::Error::UnclosedParen);
                }
                fm
            } else {
                None
            };
            
            // Check for colon
            scanner.skip_spaces();
            let colon = scanner.token(":");
            
            // If colon is also missing, report that error too
            if colon.is_none() {
                let error_span = dashdash.clone();
                scanner.add_error(error_span, fastn_section::Error::SectionColonMissing);
            }
            
            // Return partial SectionInit for error recovery
            return Some(fastn_section::SectionInit {
                dashdash,
                name: fastn_section::IdentifierReference::Local(scanner.span(scanner.index())),
                kind: None,
                colon,
                function_marker: func_marker,
                doc: None,
                visibility: None,
            });
        }
    };

    scanner.skip_spaces();

    let function_marker = scanner.token("(");

    if function_marker.is_some() {
        // Allow whitespace, newlines and comments between ()
        scanner.skip_all_whitespace();
        
        if !scanner.take(')') {
            // Unclosed parenthesis - record error
            let error_span = scanner.span(start_pos);
            scanner.add_error(error_span, fastn_section::Error::UnclosedParen);
        }
    }

    scanner.skip_spaces();
    let colon = scanner.token(":");
    
    // Report missing colon error if needed
    if colon.is_none() {
        let error_span = name.span();
        scanner.add_error(error_span, fastn_section::Error::SectionColonMissing);
    }

    // Even if colon is missing, we still want to parse the section
    Some(fastn_section::SectionInit {
        dashdash,
        name,
        kind,
        colon,
        function_marker,
        doc: None,
        visibility: None,
    })
}

#[cfg(test)]
mod test {
    fastn_section::tt!(super::section_init);

    #[test]
    fn section_init() {
        // Basic section init
        t!("-- foo:", {"name": "foo"});
        t!("-- foo: ", {"name": "foo"}, " ");
        t!("-- foo: hello", {"name": "foo"}, " hello");
        
        // With type/kind
        t!("-- integer foo: hello", {"name": "foo", "kind": "integer"}, " hello");
        t!("-- string msg:", {"name": "msg", "kind": "string"});
        
        // Unicode identifiers
        t!("-- integer héllo: foo", {"name": "héllo", "kind": "integer"}, " foo");
        t!("-- नाम: value", {"name": "नाम"}, " value");  // Devanagari "naam" (name)
        
        // Function markers
        t!("-- foo():", {"function": "foo"});
        t!("-- integer foo():", {"function": "foo", "kind": "integer"});
        t!("-- foo( ):", {"function": "foo"});  // Space inside parens
        t!("-- foo(  ):", {"function": "foo"}); // Multiple spaces
        t!("-- foo(\n):", {"function": "foo"}); // Newline inside parens
        t!("-- foo(\n  \n):", {"function": "foo"}); // Multiple newlines and spaces
        t!("-- foo(;; comment\n):", {"function": "foo"}); // Comment inside parens
        t!("-- foo(\n  ;; a comment\n  ):", {"function": "foo"}); // Comment with whitespace
        
        // Qualified names
        t!("-- ftd.text:", {"name": "ftd.text"});
        t!("-- module.component:", {"name": "module.component"});
        t!("-- package#name:", {"name": "package#name"});
        
        // Missing colon (now allowed for error recovery)
        t!("-- foo", {"name": "foo"});
        t!("-- integer bar", {"name": "bar", "kind": "integer"});
        t!("-- baz()", {"function": "baz"});
        
        // Extra spacing
        t!("--   foo  :", {"name": "foo"});
        t!("-- \t foo\t:", {"name": "foo"});
        t!("--  integer  foo  :", {"name": "foo", "kind": "integer"});
        
        // Generic types (already supported!)
        t!("-- list<integer> foo:", {"name": "foo", "kind": {"name": "list", "args": ["integer"]}});
        t!("-- map<string, integer> data:", {"name": "data", "kind": {"name": "map", "args": ["string", "integer"]}});
        t!("-- option<string> maybe:", {"name": "maybe", "kind": {"name": "option", "args": ["string"]}});
        
        // Partial parsing - stops at certain points
        t!("-- foo: bar\n", {"name": "foo"}, " bar\n");
        t!("-- foo: {expr}", {"name": "foo"}, " {expr}");
        
        // No section marker at all - returns None
        f!("foo:");       // No dashes at all
        f!("");           // Empty input
    }
    
    #[test] 
    fn section_init_error_recovery() {
        // We need t_err! macro for these cases - parse with errors
        // Single dash - parse what we can, report error
        t_err!("- foo:", {"name": "foo"}, "dash_count_error");
        t_err!("- integer bar:", {"name": "bar", "kind": "integer"}, "dash_count_error");
        
        // Triple dash - parse what we can, report error  
        t_err!("--- foo:", {"name": "foo"}, "dash_count_error");
        
        // Just dashes with no name - parse partial, report both missing name and colon
        t_err!("--", {}, ["missing_name", "section_colon_missing"]);
        t_err!("-- ", {}, ["missing_name", "section_colon_missing"]);
        t_err!("--:", {}, "missing_name");  // Has colon, only missing name
        t_err!("-- :", {}, "missing_name");  // Has colon, only missing name
        
        // Function marker without name - parse partial, report error
        t_err!("-- ():", {}, "missing_name");
        t_err!("-- ( ):", {}, "missing_name");
        
        // Unclosed function marker - still treated as function with error
        t_err!("-- foo(:", {"function": "foo"}, "unclosed_paren");
        t_err!("-- foo( :", {"function": "foo"}, "unclosed_paren");
        t_err!("-- foo(\n:", {"function": "foo"}, "unclosed_paren");
    }
}
