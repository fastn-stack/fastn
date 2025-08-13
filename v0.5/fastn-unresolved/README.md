# fastn-unresolved

The `fastn-unresolved` crate handles the parsing and initial processing of fastn documents, creating an unresolved AST (Abstract Syntax Tree) that will later be resolved and compiled.

## Overview

This crate is responsible for:
- Parsing fastn sections into unresolved document structures
- Managing imports and module dependencies
- Building symbol tables and alias mappings
- Tracking unresolved symbols for later resolution
- Providing error recovery and reporting during parsing

## Key Components

### Document Structure

The core `Document` struct represents an unresolved fastn document containing:
- **content**: Raw content sections
- **definitions**: Function and component definitions
- **aliases**: Symbol and module aliases from imports
- **errors**: Parsing errors with source locations

### Parsers

Currently implemented parsers:
- **import**: Handles `-- import:` statements with exposing/export
- **component_invocation**: Parses component invocations
- **function_definition**: Parses function definitions (in progress)

### Symbol Management

The crate uses `fastn_section::SoM` (Symbol or Module) enum to track:
- `Module(m)`: Imported modules
- `Symbol(s)`: Specific symbols from modules

Aliases are stored in a `HashMap<String, SoM>` for name resolution.

## Usage

```rust
use fastn_unresolved::Document;
use fastn_section::{Document as SectionDoc, Arena};

let mut arena = Arena::default();
let module = fastn_section::Module::main(&mut arena);
let parsed = SectionDoc::parse(&source, module);
let (mut document, sections) = Document::new(module, parsed, &mut arena);

// Process sections...
```

## Testing

The crate provides comprehensive test infrastructure with macros:
- `t!()`: Test successful parsing
- `f!()`: Test parsing failures
- `t_err!()`: Test partial results with errors

See [TESTING.md](TESTING.md) for details.

## Architecture

See [ARCHITECTURE.md](ARCHITECTURE.md) for detailed design information.

## Grammar

See [GRAMMAR.md](GRAMMAR.md) for the complete grammar specification.