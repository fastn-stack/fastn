# fastn-unresolved Architecture

## Overview

The `fastn-unresolved` crate is responsible for parsing fastn sections into an unresolved document structure. It sits between `fastn-section` (which provides the basic section parsing) and the resolution/compilation phases.

## Core Components

### Document

The `Document` struct is the central data structure:

```rust
pub struct Document {
    pub content: Vec<Content>,           // Raw content sections
    pub definitions: Vec<Definition>,    // Function/component definitions
    pub aliases: Option<AliasesID>,     // Symbol and module aliases
    pub errors: Vec<Error>,             // Parsing errors
}
```

### Aliases and Symbol Management

The crate manages symbol visibility through the `Aliases` type:

```rust
type Aliases = HashMap<String, SoM>

enum SoM {
    Module(Module),  // Reference to imported module
    Symbol(Symbol),  // Reference to specific symbol
}
```

#### How Imports Populate Aliases

1. **Module Import**: Creates `SoM::Module` entry
   - `-- import: foo` → `aliases["foo"] = SoM::Module(foo)`
   - `-- import: foo as f` → `aliases["f"] = SoM::Module(foo)`

2. **Symbol Import/Export**: Creates `SoM::Symbol` entries
   - Behavior depends on package context (main vs other)
   - Symbols can be aliased during import/export

## Parser Organization

### Parser Module Structure

```
src/parser/
├── mod.rs                    # Test infrastructure and utilities
├── import.rs                 # Import statement parser
├── component_invocation.rs   # Component invocation parser
└── function_definition.rs    # Function definition parser
```

### Parser Pattern

Each parser follows a consistent pattern:

1. **Validation**: Check section name, type, required fields
2. **Error Recovery**: Report errors but continue parsing
3. **Construction**: Build appropriate AST nodes
4. **Side Effects**: Update document state (aliases, definitions, etc.)

Example from import parser:

```rust
pub fn import(
    section: fastn_section::Section,
    document: &mut fastn_unresolved::Document,
    arena: &mut fastn_section::Arena,
    package: &Option<&fastn_package::Package>,
    main_package_name: &str,
) {
    // 1. Validation
    if section.init.kind.is_some() {
        document.errors.push(Error::ImportCantHaveType);
    }
    
    // 2. Parse with error recovery
    let import = match parse_import(&section, document, arena) {
        Some(v) => v,
        None => return, // Critical error, can't continue
    };
    
    // 3. Process import
    add_import(document, arena, &import);
    
    // 4. Handle exposing/export
    add_export_and_exposing(document, arena, &import, main_package_name, package);
}
```

## Package Context Behavior

The import parser exhibits different behavior based on package context:

### Main Package
- Processes `exposing` field to add symbol aliases
- Ignores `export` field

### Other Packages  
- Processes `export` field to add symbol aliases
- Ignores `exposing` field

This asymmetry controls symbol visibility across package boundaries.

## Error Handling

### Error Recovery Strategy

The parsers implement robust error recovery:
- Continue parsing after non-critical errors
- Accumulate all errors in `document.errors`
- Return partial results when possible

### Error Types

Common errors handled:
- `ImportMustHaveCaption`
- `ImportCantHaveType`
- `ImportMustBeImport`
- `ImportPackageNotFound`

### Known Issues

- `ExtraArgumentFound` error is too generic and should be replaced with context-specific errors that indicate which header is unexpected and what headers are allowed

## Testing Infrastructure

The crate provides comprehensive test macros:

### Test Macros

- `t!()`: Test successful parsing
- `f!()`: Test parsing failures (errors only)
- `t_err!()`: Test partial results with errors

### Test Organization

Tests are colocated with parsers in submodules:

```rust
#[cfg(test)]
mod tests {
    fastn_unresolved::tt!(super::parser_function, super::tester);
    
    #[test]
    fn test_cases() {
        t!("-- import: foo", {"import": "foo"});
        f!("-- import:", "ImportMustHaveCaption");
        t_err!("-- impart: foo", {"import": "foo"}, "ImportMustBeImport");
    }
}
```

## Integration Points

### Input: fastn-section
- Receives parsed `Section` objects
- Uses `Arena` for string interning
- Works with `Module` and `Symbol` types

### Output: Unresolved Document
- Produces `Document` with unresolved references
- Maintains symbol table via aliases
- Preserves source locations for error reporting

### Next Phase: Resolution
- The unresolved document will be processed by resolver
- Symbols will be looked up and validated
- Cross-references will be resolved

## Design Decisions

### Arena Allocation
Uses `fastn_section::Arena` for efficient string storage and deduplication.

### Error Accumulation
All parsers accumulate errors rather than failing fast, enabling better developer experience with multiple error reports.

### Partial Results
Parsers return partial results even when encountering errors, allowing downstream processing to continue where possible.

### Test-First Development
Heavy emphasis on test macros and test coverage to ensure parser correctness.