# fastn-unresolved Testing Guide

## Test Infrastructure

The `fastn-unresolved` crate provides a comprehensive testing framework with specialized macros for different testing scenarios.

## Test Macros

### Core Test Functions

The test infrastructure is built on three core functions, each enforcing specific invariants:

#### `t1()` - Test successful parsing
```rust
fn t1<PARSER, TESTER>(
    source: &str, 
    expected: serde_json::Value, 
    parser: PARSER, 
    tester: TESTER
)
```
**Invariants:**
- ✅ Parsing must succeed without any errors
- ✅ Expected output must match
- ❌ Fails if any errors are produced

#### `f1()` - Test parsing failures
```rust
fn f1<PARSER>(
    source: &str, 
    expected_errors: serde_json::Value, 
    parser: PARSER
)
```
**Invariants:**
- ✅ Must produce at least one error
- ✅ Must produce the exact expected errors
- ❌ Must NOT produce any partial results:
  - No definitions added
  - No content added
  - No aliases added (beyond defaults)
- ❌ Fails if parser produces any output

#### `t_err1()` - Test partial results with errors
```rust
fn t_err1<PARSER, TESTER>(
    source: &str,
    expected: serde_json::Value,
    expected_errors: serde_json::Value,
    parser: PARSER,
    tester: TESTER
)
```
**Invariants:**
- ✅ Must produce at least one error
- ✅ Must produce some partial results (otherwise use `f!()`)
- ✅ Both output and errors must match expected values
- ❌ Fails if no errors produced
- ❌ Fails if no partial results produced

### Choosing the Right Macro

| Scenario | Use | Don't Use |
|----------|-----|-----------|
| Parser succeeds completely | `t!()` | `t_err!()` |
| Parser fails with no output | `f!()` | `t_err!()` |
| Parser produces partial results with errors | `t_err!()` | `f!()` or `t!()` |
| Testing error recovery | `t_err!()` | `f!()` |

### Macro Wrappers

The `tt!` macro generates test-specific macros for a parser:

```rust
fastn_unresolved::tt!(parser_function, tester_function);
```

This generates:
- `t!()` - Wrapper for `t1()` with indoc support
- `t_raw!()` - Wrapper for `t1()` without indoc
- `f!()` - Wrapper for `f1()` with indoc support
- `f_raw!()` - Wrapper for `f1()` without indoc
- `t_err!()` - Wrapper for `t_err1()` with indoc support
- `t_err_raw!()` - Wrapper for `t_err1()` without indoc

The `tt_error!` macro generates only error-testing macros:

```rust
fastn_unresolved::tt_error!(parser_function);
```

This generates:
- `f!()` and `f_raw!()` macros only

## Writing Tests

### Basic Test Structure

```rust
#[cfg(test)]
mod tests {
    // Generate test macros for your parser
    fastn_unresolved::tt!(super::my_parser, super::my_tester);
    
    #[test]
    fn test_success_cases() {
        // Test successful parsing
        t!("-- import: foo", {"import": "foo"});
        
        // Test with multiline input (indoc strips indentation)
        t!(
            "-- import: foo
            exposing: bar",
            {"import": "foo", "symbols": ["foo#bar"]}
        );
    }
    
    #[test]
    fn test_error_cases() {
        // Test single error - no partial results
        f!("-- import:", "ImportMustHaveCaption");
        
        // Test multiple errors - no partial results
        f!(
            "-- invalid section",
            ["SectionNameError", "MissingColon"]
        );
    }
    
    #[test]
    fn test_partial_results() {
        // Test cases with both results and errors
        t_err!(
            "-- impart: foo",
            {"import": "foo"},
            "ImportMustBeImport"
        );
        
        // Multiple errors with partial result
        t_err!(
            "-- string import: foo",
            {"import": "foo"},
            ["ImportCantHaveType", "ImportPackageNotFound"]
        );
    }
}
```

### Parser Function Pattern

Parser functions must follow this signature:

```rust
fn my_parser(
    section: fastn_section::Section,
    document: &mut fastn_unresolved::Document,
    arena: &mut fastn_section::Arena,
    package: &Option<&fastn_package::Package>,
)
```

### Tester Function Pattern

Tester functions validate the parsed output:

```rust
fn my_tester(
    document: fastn_unresolved::Document,
    expected: serde_json::Value,
    arena: &fastn_section::Arena,
) {
    // Validate document state
    assert!(document.content.is_empty());
    assert!(document.definitions.is_empty());
    
    // Compare with expected JSON
    assert_eq!(
        to_json(&document, arena),
        expected
    );
}
```

## Testing Patterns

### Error Recovery Testing

Use `t_err!()` to test that parsers can recover from errors and produce partial results:

```rust
t_err!(
    "-- impart: foo",  // Typo in "import"
    {"import": "foo"}, // Still produces result
    "ImportMustBeImport"
);
```

### Complete Failure Testing

Use `f!()` to test that parsers fail completely without producing results:

```rust
f!(
    "-- import:",      // Missing required caption
    "ImportMustHaveCaption"  // No partial result possible
);
```

### Multiple Error Testing

Test that all applicable errors are reported:

```rust
f!(
    "-- string import:",
    ["ImportCantHaveType", "ImportMustHaveCaption"]
);
```

### Indoc for Readability

Use indoc-style strings for complex test inputs:

```rust
t!(
    "-- import: foo
    exposing: bar, baz
    export: qux",
    {"import": "foo", "symbols": ["foo#bar", "foo#baz"]}
);
```

### Raw Strings When Needed

Use `t_raw!()` when you need precise control over whitespace:

```rust
t_raw!(
    "-- import: foo\n  exposing: bar",
    {"import": "foo", "symbols": ["foo#bar"]}
);
```

## Invariant Violations

The test framework will panic with descriptive messages when invariants are violated:

```
// Using f!() when parser produces partial results:
panic: f!() should not produce definitions. Found: [...]

// Using t!() when parser produces errors:
panic: t!() should not be used when errors are expected. Use t_err!() instead. Errors: [...]

// Using t_err!() when no partial results produced:
panic: t_err!() should produce partial results. Use f!() for error-only cases
```

## Best Practices

1. **Choose the Right Macro**: Use `t!()` for success, `f!()` for complete failure, `t_err!()` for partial results with errors

2. **Test Invariants**: The framework enforces invariants - don't try to work around them

3. **Test Error Recovery**: Use `t_err!()` to ensure parsers can produce partial results

4. **Test Complete Failures**: Use `f!()` to ensure parsers fail cleanly when they can't recover

5. **Test Edge Cases**: Empty inputs, missing required fields, extra fields

6. **Use Descriptive Test Names**: Make it clear what scenario is being tested

7. **Group Related Tests**: Organize tests by feature or error type

8. **Document Complex Tests**: Add comments explaining non-obvious test cases

## Test Output Format

Tests use JSON for expected output, making it easy to specify complex structures:

```rust
t!(
    "-- import: foo as f
    exposing: bar as b, baz",
    {
        "import": "foo=>f",
        "symbols": ["foo#bar=>b", "foo#baz"]
    }
);
```

The `=>` notation indicates aliasing in the debug output.

## Running Tests

```bash
# Run all tests
cargo test

# Run specific test module
cargo test import

# Run with output for debugging
cargo test -- --nocapture

# Run a specific test
cargo test test_import_errors
```

## Debugging Failed Tests

When tests fail, the output shows:
- The source input being tested
- Expected vs actual output/errors
- Line numbers and error details
- Invariant violations with clear messages

Use `--nocapture` to see the `println!` debug output from test functions.