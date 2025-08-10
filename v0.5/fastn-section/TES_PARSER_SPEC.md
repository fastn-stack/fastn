# Tes Parser Specification

This document describes the Text-Expression-Section (Tes) parser implementation in fastn-section, documenting the actual behavior as of the latest implementation.

## Overview

The Tes parser handles mixed content that can contain:
1. **Text**: Plain text content
2. **Expressions**: Content within braces `{...}` or dollar expressions `${...}`  
3. **Sections**: Inline sections starting with `{--`

## Grammar

```bnf
tes         ::= (text | expression | section)*
text        ::= <any content except '{' or '}'>
expression  ::= '{' tes* '}' | '${' tes* '}'
section     ::= '{--' section_content '}'

section_content ::= section_init [caption] [headers] [body]
section_init    ::= spaces identifier [':']
caption         ::= tes_till_newline
headers         ::= (header '\n')*
body            ::= '\n' tes*
```

## Key Behaviors

### 1. Unmatched Braces

#### Unmatched Closing Brace `}`
- **Behavior**: Stops parsing immediately
- **Treatment**: NOT treated as text
- **Error**: No error recorded, parsing simply stops
- **Example**: `"hello } world"` parses as `["hello "]` with `"} world"` remaining

#### Unmatched Opening Brace `{`
- **Behavior**: Triggers error recovery
- **Treatment**: Records `UnclosedBrace` error and recovers
- **Recovery Strategy**: Hybrid approach that:
  - Tracks nesting depth
  - Stops at structural boundaries (sections, doc comments)
  - Has maximum lookahead limits (1000 chars or 100 lines)
- **Example**: `"hello {world"` parses as `["hello ", {expression: ["world"]}]` with error

### 2. Expression Types

#### Regular Expression `{...}`
- Can contain nested expressions
- Can span multiple lines (newlines allowed inside)
- Recursive parsing of content

#### Dollar Expression `${...}`
- Distinguished by `is_dollar: true` flag in output
- Otherwise behaves identically to regular expressions
- `$` without `{` is treated as plain text

### 3. Inline Sections `{-- ...}`

#### Recognition
- `{--` is ALWAYS treated as inline section attempt
- Even `{--` without following space is recognized
- User intent: `{--` means section, not expression with text "--"

#### Missing Syntax Handling
- Missing colon after section name: Records `ColonNotFound` error but continues
- Allows recovery from incomplete syntax
- Example: `{-- foo` parses as section with error, not as expression

#### Structure
- Can contain: caption, headers, and body
- All content must be within the braces
- Unclosed inline section triggers `UnclosedBrace` error

### 4. Error Recovery

The parser implements a sophisticated error recovery strategy for unclosed braces:

1. **Continues parsing** to avoid cascading errors
2. **Tracks nesting depth** to handle nested expressions correctly
3. **Respects boundaries**: Stops at section markers (`--`), doc comments (`;;;`)
4. **Has limits**: Maximum 1000 characters or 100 lines lookahead
5. **Records errors**: Adds `UnclosedBrace` error to document's error list

### 5. Context-Specific Parsing

#### `tes_till(scanner, terminator)`
- General purpose Tes parsing
- Continues until terminator function returns true
- Used internally by other parsers

#### `tes_till_newline(scanner)`
- Specialized for single-line content (headers, captions)
- Stops at newline without consuming it
- Returns `None` if starts with `}` (even with leading spaces)
- Expressions can still contain newlines internally

## Implementation Details

### Scanner Consumption Model
- Parsers consume input as they parse
- On failure, parsers backtrack to original position
- Unmatched `}` is left unconsumed for caller to handle

### Error Reporting
- Errors are added to the document's error list
- Parsing continues after errors when possible
- Multiple errors can be recorded in a single parse

### SectionInit Structure
- `colon` field is `Option<Span>` to support error recovery
- Missing colon doesn't prevent section parsing
- Error is recorded but parsing continues

## Test Coverage

The implementation includes comprehensive tests for:
- Basic text and expressions
- Nested expressions (arbitrary depth)
- Dollar expressions
- Inline sections with various content types
- Edge cases with whitespace
- Unmatched braces (both opening and closing)
- Error recovery scenarios
- Unicode content
- Special characters

## Examples

### Valid Input
```
"hello {world}"          → ["hello ", {expression: ["world"]}]
"${price}"               → [{$expression: ["price"]}]
"{-- foo: bar}"          → [{section: [{name: "foo", caption: ["bar"]}]}]
"{a {b {c}}}"            → [{expression: ["a ", {expression: ["b ", {expression: ["c"]}]}]}]
```

### Error Cases
```
"{"                      → [{expression: []}] + UnclosedBrace error
"}"                      → Parse failure (returns None/stops)
"{-- foo"                → [{section: [{name: "foo"}]}] + ColonNotFound + UnclosedBrace errors
"hello {world"           → ["hello ", {expression: ["world"]}] + UnclosedBrace error
```

## Migration Notes

From previous versions:
- Block section headers (`-- foo.bar:`) deprecated in favor of brace syntax
- The parser now strictly distinguishes between `}` as parse terminator vs error
- `{--` always triggers section parsing, even with malformed syntax