# fastn-section

The section parser module for fastn 0.5. This module implements the first stage of the fastn parsing pipeline, converting raw `.ftd` source text into a structured representation of sections, headers, and body content.

## Overview

fastn-section parses the basic structural elements of fastn documents:
- Sections with their initialization, headers, and bodies
- Text, expressions, and inline sections (Tes)
- Identifiers and references
- Types (kinds) with generic parameters
- Visibility modifiers
- Documentation comments

## Grammar

See [GRAMMAR.md](GRAMMAR.md) for the complete grammar reference.

## Key Components

### Document
The top-level structure containing module documentation, sections, and diagnostic information.

### Section
The fundamental building block consisting of:
- Section initialization with optional type and visibility
- Optional caption
- Headers (key-value pairs)
- Body content
- Child sections (populated by the wiggin module)

### Tes (Text-Expression-Section)
Mixed content that can contain:
- Plain text
- Expressions in `{...}` or `${...}` syntax
- Inline sections starting with `{--`

### Error Recovery
The parser implements sophisticated error recovery to continue parsing even when encountering malformed input, collecting errors for later reporting.

## Usage

The parser is typically used as part of the larger fastn compilation pipeline:

```rust
let document = fastn_section::Document::parse(&source, module);
```

## Testing

Tests are located alongside each parser module and use custom test macros:
- `t!()` - Test successful parsing with no errors
- `t_err!()` - Test parsing with expected recoverable errors
- `f!()` - Test parse failures