# fastn-section Parsing Tutorial

This document provides a comprehensive walkthrough of how the fastn-section parser works, from the initial text input to the final structured AST.

## Table of Contents
1. [Overview](#overview)
2. [The Scanner](#the-scanner)
3. [Parser Architecture](#parser-architecture)
4. [Parsing Flow](#parsing-flow)
5. [Error Recovery](#error-recovery)
6. [Testing Framework](#testing-framework)
7. [Code Walkthrough](#code-walkthrough)

## Overview

The fastn-section parser is a hand-written recursive descent parser that transforms `.ftd` source text into a structured AST. It's designed to be:
- **Resilient**: Continues parsing even when encountering errors
- **Fast**: Single-pass parsing with minimal backtracking
- **Precise**: Tracks exact source locations for all parsed elements

### Key Design Principles

1. **Scanner-based**: Uses a stateful scanner that tracks position and can backtrack
2. **Error Recovery**: Collects errors without stopping the parse
3. **Span Preservation**: Every parsed element knows its exact source location
4. **Module-aware**: Tracks which module each element belongs to

## The Scanner

The `Scanner` is the heart of the parsing system. It provides a cursor over the input text with these key capabilities:

```rust
pub struct Scanner<'input, O> {
    source: &'input arcstr::ArcStr,
    index: Cell<scanner::Index<'input>>,
    fuel: fastn_section::Fuel,
    pub module: Module,
    pub output: O,
}
```

### Scanner Operations

#### Basic Movement
- `peek()` - Look at next character without consuming
- `pop()` - Consume and return next character  
- `take(char)` - Consume if next char matches
- `token(&str)` - Consume if next chars match string

#### Position Management
- `index()` - Save current position
- `reset(&Index)` - Restore to saved position
- `span(Index)` - Create span from saved position to current

#### Advanced Operations
- `skip_spaces()` - Skip whitespace (not newlines)
- `skip_new_lines()` - Skip newline characters
- `take_while(predicate)` - Consume while condition true
- `one_of(&[str])` - Try multiple tokens, return first match

### Example: Parsing an Identifier

```rust
pub fn identifier(scanner: &mut Scanner<Document>) -> Option<Identifier> {
    let start = scanner.index();  // Save start position
    
    // First character must be Unicode letter
    if !scanner.peek()?.is_alphabetic() {
        return None;
    }
    
    // Consume identifier characters
    let span = scanner.take_while(|c| 
        c.is_alphanumeric() || c == '-' || c == '_'
    )?;
    
    Some(Identifier { name: span })
}
```

## Parser Architecture

### Parser Modules

The parser is organized into focused modules, each responsible for parsing specific constructs:

```
parser/
├── mod.rs              # Entry point, test macros
├── document.rs         # Top-level document parser (inline in mod.rs)
├── section.rs          # Section parser
├── section_init.rs     # Section initialization (-- foo:)
├── headers.rs          # Header key-value pairs
├── body.rs            # Body content
├── tes.rs             # Text-Expression-Section
├── identifier.rs       # Simple identifiers
├── identifier_reference.rs  # Qualified references (a.b, pkg#item)
├── kind.rs            # Types with generics
├── kinded_name.rs     # Type + identifier
├── kinded_reference.rs # Type + reference
├── visibility.rs      # public/private modifiers
├── doc_comment.rs     # Documentation comments
└── header_value.rs    # Header/caption values
```

### Parser Signatures

Most parsers follow this pattern:

```rust
pub fn parser_name(
    scanner: &mut Scanner<Document>
) -> Option<ParsedType>
```

- Takes mutable scanner reference
- Returns `Option` - `None` means couldn't parse
- Scanner position is reset on failure (unless error recovery)

## Parsing Flow

### 1. Document Parsing

The entry point is `Document::parse()`:

```rust
impl Document {
    pub fn parse(source: &ArcStr, module: Module) -> Document {
        let mut scanner = Scanner::new(source, ..., Document { ... });
        document(&mut scanner);
        scanner.output  // Return the document with collected sections/errors
    }
}
```

### 2. Document Structure

The `document()` parser loops, trying to parse sections:

```rust
pub fn document(scanner: &mut Scanner<Document>) {
    scanner.skip_spaces();
    loop {
        if let Some(section) = section(scanner) {
            scanner.output.sections.push(section);
            scanner.skip_spaces();
            scanner.skip_new_lines();
        } else if let Some(doc) = doc_comment(scanner) {
            // Orphaned doc comment - report error
            scanner.add_error(doc, Error::UnexpectedDocComment);
        } else {
            break;  // No more content
        }
    }
}
```

### 3. Section Parsing

A section consists of multiple parts parsed in sequence:

```rust
pub fn section(scanner: &mut Scanner<Document>) -> Option<Section> {
    let doc = doc_comment(scanner);         // Optional doc comment
    let is_commented = scanner.take('/');   // Optional comment marker
    let init = section_init(scanner)?;      // Required section init
    let caption = header_value(scanner);    // Optional caption
    
    scanner.token("\n");
    let headers = headers(scanner).unwrap_or_default();
    
    // Body requires double newline separator
    let body = if scanner.token("\n").is_some() {
        body(scanner)
    } else {
        None
    };
    
    Some(Section { init, caption, headers, body, ... })
}
```

### 4. Complex Parser: section_init

The `section_init` parser shows error recovery in action:

```rust
pub fn section_init(scanner: &mut Scanner<Document>) -> Option<SectionInit> {
    let dashdash_index = scanner.index();
    
    // Try different dash counts
    let dashdash = if scanner.token("---").is_some() {
        scanner.add_error(..., Error::DashCountError);
        scanner.span(dashdash_index)
    } else if let Some(dd) = scanner.token("--") {
        dd
    } else if scanner.token("-").is_some() {
        scanner.add_error(..., Error::DashCountError);
        scanner.span(dashdash_index)
    } else {
        return None;  // No section marker
    };
    
    scanner.skip_spaces();
    
    // Parse optional visibility and type
    let visibility = visibility(scanner);
    scanner.skip_spaces();
    
    // Parse name (with recovery for missing name)
    let name = if let Some(kr) = kinded_reference(scanner) {
        kr
    } else {
        // Error recovery: create empty name
        scanner.add_error(..., Error::MissingName);
        KindedReference {
            name: IdentifierReference::Local(scanner.span(scanner.index())),
            kind: None,
        }
    };
    
    // Check for function marker
    let function_marker = parse_function_marker(scanner);
    
    // Parse colon (with error recovery)
    let colon = if let Some(c) = scanner.token(":") {
        Some(c)
    } else {
        scanner.add_error(..., Error::SectionColonMissing);
        None
    };
    
    Some(SectionInit { dashdash, name, visibility, colon, ... })
}
```

### 5. The Tes Parser

The most complex parser handles mixed text/expressions:

```rust
pub fn tes_till(
    scanner: &mut Scanner<Document>,
    terminator: &dyn Fn(&mut Scanner<Document>) -> bool,
) -> Vec<Tes> {
    let mut result = vec![];
    
    loop {
        if terminator(scanner) {
            break;
        }
        
        if scanner.peek() == Some('{') {
            // Parse expression
            result.push(parse_expression(scanner));
        } else {
            // Parse text until next { or terminator
            let text = parse_text_segment(scanner, terminator);
            if !text.is_empty() {
                result.push(Tes::Text(text));
            }
        }
    }
    
    result
}
```

## Error Recovery

The parser uses several strategies for error recovery:

### 1. Continue with Defaults

When a required element is missing, create a default:

```rust
// Missing name in section_init
let name = kinded_reference(scanner).unwrap_or_else(|| {
    scanner.add_error(span, Error::MissingName);
    // Return empty name to continue parsing
    KindedReference {
        name: IdentifierReference::Local(empty_span),
        kind: None,
    }
});
```

### 2. Look for Recovery Points

For unclosed braces, find a reasonable stopping point:

```rust
fn find_recovery_point(scanner: &mut Scanner<Document>) -> Index {
    let mut depth = 1;
    while let Some(ch) = scanner.peek() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return scanner.index();
                }
            }
            '\n' if depth == 1 => {
                // Newline at depth 1 is a good recovery point
                return scanner.index();
            }
            _ => {}
        }
        scanner.pop();
    }
    scanner.index()  // EOF
}
```

### 3. Report and Continue

Report errors but keep parsing:

```rust
if scanner.token("---").is_some() {
    // Wrong dash count, but we can continue
    scanner.add_error(span, Error::DashCountError);
    // Continue parsing with what we have
}
```

## Testing Framework

The crate includes sophisticated test macros for parser testing:

### Test Macros

```rust
// Success case - no errors expected
t!("-- foo: bar", {"name": "foo", "caption": ["bar"]});

// Error recovery case - expects specific errors
t_err!("-- foo", {"name": "foo"}, "section_colon_missing");

// Failure case - parsing should fail
f!("not valid");

// Raw variants (without indoc processing)
t_raw!("literal\ttabs", ["literal\ttabs"]);
```

### Test Structure

Each parser module includes tests:

```rust
mod test {
    fastn_section::tt!(super::parser_function);  // Generate test macros
    
    #[test]
    fn test_name() {
        // Success cases
        t!("input", expected_json_output);
        
        // Error cases
        t_err!("bad input", partial_output, "error_name");
        
        // Failure cases  
        f!("completely invalid");
    }
}
```

## Code Walkthrough

Let's trace through parsing a complete example:

### Input
```ftd
;;; Component documentation
-- public component button: Click Me
string type: primary
enabled: true

Renders a button element
```

### Step-by-Step Parsing

1. **Document parser starts**
   - Calls `section()` in a loop

2. **Section parser**
   - `doc_comment()` finds and captures `;;; Component documentation\n`
   - No `/` comment marker
   - `section_init()` is called

3. **Section init parser**
   - Finds `--` token
   - `visibility()` parses `public`
   - `kinded_reference()` is called
     - `kind()` parses `component`
     - `identifier_reference()` parses `button`
   - No function marker `()`
   - Finds `:` token
   - Returns `SectionInit`

4. **Back in section parser**
   - `header_value()` parses caption `Click Me`
   - Finds `\n`
   - `headers()` is called

5. **Headers parser**
   - First header:
     - `kinded_name()` parses `string type`
     - Finds `:`
     - `header_value()` parses `primary`
   - Second header:
     - `kinded_name()` parses `enabled` (no kind)
     - Finds `:`
     - `header_value()` parses `true`
   - Stops at `\n\n` (double newline)

6. **Body parser**
   - `body()` is called
   - Uses `tes_till()` to parse mixed content
   - Returns text `Renders a button element`

7. **Section complete**
   - Returns fully parsed `Section` structure
   - Document adds it to `sections` vector

### Result Structure

```rust
Document {
    sections: vec![Section {
        init: SectionInit {
            name: IdentifierReference::Local("button"),
            kind: Some(Kind { name: "component", args: None }),
            visibility: Some(Visibility::Public),
            doc: Some(";;; Component documentation\n"),
            ...
        },
        caption: Some(HeaderValue(vec![Tes::Text("Click Me")])),
        headers: vec![
            Header {
                name: "type",
                kind: Some(Kind { name: "string", ... }),
                value: HeaderValue(vec![Tes::Text("primary")]),
                ...
            },
            Header {
                name: "enabled",
                value: HeaderValue(vec![Tes::Text("true")]),
                ...
            },
        ],
        body: Some(HeaderValue(vec![
            Tes::Text("Renders a button element")
        ])),
        ...
    }],
    errors: vec![],  // No errors in this example
    ...
}
```

## Advanced Topics

### Backtracking

Some parsers need to backtrack when ambiguity is discovered:

```rust
// In kinded_reference parser
let start = scanner.index();
let kind = kind(scanner);
let name = identifier_reference(scanner);

match (kind, name) {
    (Some(k), Some(n)) => {
        // Both found: "string foo"
        Some(KindedReference { kind: Some(k), name: n })
    }
    (Some(k), None) if k.args.is_none() => {
        // Just kind found, might be the name: "foo"
        scanner.reset(&start);  // Backtrack
        let name = identifier_reference(scanner)?;
        Some(KindedReference { kind: None, name })
    }
    _ => None
}
```

### Recursive Parsing

The Tes parser handles arbitrary nesting:

```rust
fn parse_expression(scanner: &mut Scanner<Document>) -> Tes {
    scanner.pop();  // Consume '{'
    let content = tes_till(scanner, &|s| s.peek() == Some('}'));
    
    if scanner.take('}') {
        Tes::Expression { content: HeaderValue(content), ... }
    } else {
        scanner.add_error(..., Error::UnclosedBrace);
        // Error recovery...
    }
}
```

### Performance Considerations

1. **Minimal Allocations**: Uses `Span` (substring references) instead of cloning strings
2. **Single Pass**: Most parsing happens in one forward pass
3. **Lazy Evaluation**: Only parses what's needed
4. **Arena Allocation**: Uses arena for symbol interning

## Conclusion

The fastn-section parser demonstrates a robust approach to parsing with:
- Clear separation of concerns
- Comprehensive error recovery
- Precise source location tracking
- Extensive testing

This architecture allows the parser to handle malformed input gracefully while providing detailed error information for debugging and IDE support.