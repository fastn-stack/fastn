# RFC: `careful` - Enhanced Code Review Annotations

- Feature Name: `careful`
- Start Date: 2025-08-21
- RFC PR: [rust-lang/rfcs#0000](https://github.com/rust-lang/rfcs/pull/0000)
- Rust Issue: [rust-lang/rust#0000](https://github.com/rust-lang/rust/issues/0000)

## Summary

This RFC proposes adding a new keyword `careful` to Rust that can be applied to function declarations and as block annotations to indicate that code requires enhanced code review, careful consideration, and potentially breaks conventional assumptions. Unlike `unsafe`, this keyword doesn't relax memory safety guarantees but serves as a compiler-enforced documentation and tooling hint for code that needs special attention.

## Motivation

### Problem Statement

In codebases, certain functions or code sections require extra scrutiny during code review despite being memory-safe. The need for special attention can occur at different granularities:

1. **Function-level concerns**: APIs that are easy to misuse or have unexpected behavior
2. **Implementation-level concerns**: Specific algorithms or operations within otherwise normal functions
3. **Statement-level concerns**: Individual operations that require careful consideration

Examples include:
- **APIs with surprising contracts** where incorrect usage leads to logical errors
- **Complex algorithms** embedded within normal functions
- **Performance-critical sections** with non-obvious trade-offs
- **Cryptographic operations** where implementation details matter
- **Domain-specific logic** where "common sense" doesn't apply

### Use Cases

#### Function-level: API that's easy to misuse
```rust
/// Processes user input with different validation based on trust level
/// 
/// # Arguments
/// * `input` - The user input to process  
/// * `trusted` - Whether input is pre-validated
/// 
/// # Careful Usage
/// The `trusted` parameter is counter-intuitive: `true` means input is 
/// already validated and will NOT be escaped, while `false` means input
/// will be HTML-escaped. Many callers get this backwards, leading to XSS.
/// 
/// Consider using separate functions like `process_trusted_input()` and
/// `process_untrusted_input()` instead.
careful fn process_user_input(input: &str, trusted: bool) -> Result<String, Error> {
    if trusted {
        Ok(input.to_string())  // No escaping!
    } else {
        Ok(html_escape(input))
    }
}
```

#### Block-level: Implementation details that need care
```rust
fn constant_time_compare(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    
    /// This timing-sensitive loop must not be "optimized" with early returns
    /// as it would create timing attack vulnerabilities in crypto operations
    careful {
        let mut result = 0u8;
        for i in 0..a.len() {
            result |= a[i] ^ b[i];
        }
        result == 0
    }
}
```

#### Statement-level: Specific operations requiring care
```rust
fn fast_math_operation(values: &[f64]) -> f64 {
    let mut sum = 0.0;
    
    for value in values {
        sum += value;
    }
    
    /// This bit manipulation relies on IEEE 754 representation
    /// and will break if floating point format changes
    careful {
        let bits = sum.to_bits();
        let magic = 0x5f3759df - (bits >> 1);
        f64::from_bits(magic) * (1.5 - (sum * 0.5 * f64::from_bits(magic).powi(2)))
    }
}
```

#### Nested precision for surgical marking
```rust
fn complex_algorithm(data: &[u8]) -> Vec<u8> {
    let mut result = Vec::new();
    
    // Normal processing
    for chunk in data.chunks(8) {
        result.extend_from_slice(chunk);
    }
    
    /// This entire section uses a complex bit manipulation scheme
    careful {
        let mut state = 0xdeadbeef_u32;
        
        for byte in &mut result {
            // Most of this is straightforward bit operations
            state = state.wrapping_mul(1103515245).wrapping_add(12345);
            *byte ^= (state >> 16) as u8;
            
            /// This specific line has endianness assumptions
            /// that only work on little-endian systems
            careful {
                *byte = byte.to_le().swap_bytes();
            }
        }
    }
    
    result
}
```

## Detailed Design

### Syntax

#### Function-level annotation
```rust
careful fn function_name(params) -> return_type {
    // entire function body needs review
}
```

#### Block-level annotation
```rust
fn normal_function() {
    // normal code
    
    /// Documentation explaining what makes this block require care
    careful {
        // this specific block needs careful review
        dangerous_operations();
    }
    
    // more normal code
}
```

#### Statement-level annotation (single statement blocks)
```rust
fn another_function() {
    let x = normal_computation();
    
    /// Brief explanation of why this operation is tricky
    careful { risky_operation(x); }
    
    let y = more_normal_code();
}
```

### Semantics

1. **Compilation**: The keyword has no effect on compilation or runtime behavior
2. **Documentation**: Marked functions/blocks are highlighted in generated documentation
3. **Tooling Integration**: Linters, IDEs, and code review tools can enforce special handling
4. **Scope**: Block-level marking is more specific than function-level marking
5. **Nesting**: `careful` blocks can be nested for increasingly specific concerns
6. **Block Documentation**: `careful` blocks can be documented with `///` comments placed immediately before them

### Restrictions

1. Function-level: Can only be applied to function declarations
2. Block-level: Creates a new scope (like `unsafe` blocks)
3. Cannot be combined with `unsafe` on the same function (use `unsafe` for memory safety)
4. Within `unsafe` blocks, `careful` can be used for non-memory-safety concerns

### Error Messages and Warnings

The compiler can optionally emit informational messages:

```
note: calling function marked `careful`
  --> src/main.rs:10:5
   |
10 |     process_user_input(data, true);
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: The `trusted` parameter is counter-intuitive: `true` means input is already validated and will NOT be escaped, while `false` means input will be HTML-escaped. Many callers get this backwards, leading to XSS.

note: entering `careful` block
  --> src/crypto.rs:15:5
   |
15 |     careful {
   |     ^^^^^^^
   |
   = note: This timing-sensitive loop must not be "optimized" with early returns as it would create timing attack vulnerabilities in crypto operations
```

## Tooling Integration

### Cargo Integration
```toml
[lints.careful]
# Require explicit acknowledgment when calling careful functions
require-explicit-use = "warn"

# Enforce that careful functions have "# Careful Usage" documentation section
require-documentation = "error"

# Enforce that careful blocks have preceding doc comments
require-block-docs = "warn"

# Show info about careful usage in regular builds
show-info = true
```

### IDE Support
- Syntax highlighting to make `careful` regions visually distinct
- Hover tooltips showing the specific "# Careful Usage" section for functions or block doc comments
- Code completion warnings when calling such functions, displaying the careful usage documentation
- Different highlighting intensity for function vs block vs statement level

### Code Review Tools
- Automatic flagging for enhanced review at appropriate granularity
- Integration with review assignment systems  
- Diff highlighting that shows when `careful` annotations are added/modified/removed

## Rationale

### Granularity Benefits

1. **Function-level**: For APIs that are fundamentally tricky to use correctly
2. **Block-level**: For implementation details that need care within otherwise normal functions
3. **Statement-level**: For surgical marking of individual dangerous operations
4. **Nesting**: Allows expressing "this whole algorithm is complex, but THIS part is especially tricky"

### Why Blocks Are Essential

The original example of `constant_time_compare` illustrates why function-level marking alone is insufficient:
- The function is perfectly safe to call
- Only the specific implementation technique needs review
- Future maintainers need to know which parts to be careful with

### Comparison with `unsafe`

```rust
fn mixed_concerns(data: &mut [u8]) {
    // Normal safe operations
    data.sort();
    
    unsafe {
        // Memory safety concerns
        let ptr = data.as_mut_ptr();
        ptr.add(1).write(42);
    }
    
    /// Complex algorithm requiring careful review
    careful {
        // Logic/algorithm concerns (still memory safe)
        complex_bit_manipulation(data);
    }
}
```

## Drawbacks

1. **Keyword Pollution**: Adds another keyword to the language  
2. **Subjective Usage**: Determining what deserves `careful` may be inconsistent
3. **Documentation Overhead**: Requires writing and maintaining explanatory documentation
4. **Cognitive Load**: Another concept for developers to understand

## Rationale and Alternatives

### Why Not Just Comments?
Comments can be ignored, removed, or missed during reviews. A language-level feature ensures consistency and enables tooling support.

### Why Not Just Function-level Attributes?
Block-level granularity is essential for marking specific implementation concerns without marking entire function APIs as problematic.

### Alternative Syntax Considered

#### Alternative Keywords Considered
- `there-be-dragons` - More dramatic but overly long (15 characters)
- `fragile` - Could imply the code might break easily
- `tricky` - Informal tone may not convey sufficient gravity
- `review` - Too generic, everything should be reviewed
- `careful` - **Selected**: Clear intent, appropriate length, serious but not alarming

#### Using attributes:
```rust
#[careful]
fn function() { }

// Attributes don't work for blocks
```
Attributes don't work well for blocks and are less visually prominent.

## Prior Art

### Other Languages
- **C++**: `[[deprecated]]` attribute, `#pragma` directives for compiler hints
- **Rust**: `unsafe` blocks for memory safety concerns
- **Python**: Naming conventions like `_dangerous_` but no language support

### Design Philosophy
The word "careful" was chosen to convey the need for extra attention without being alarmist. It suggests thoughtful consideration rather than danger, making it appropriate for code that is logically complex rather than unsafe.

## Unresolved Questions

1. **Interaction with macros**: How should `careful` work in macro-generated code?
2. **Standard library usage**: Which standard library functions/blocks would benefit?
3. **Interaction with traits**: Should trait methods be able to require careful implementations?
4. **Documentation standards**: Should there be standardized formats for careful usage explanations?
5. **Tooling standards**: How should different tools consistently handle careful annotations?

## Future Possibilities

1. **Severity levels**: `careful`, `very-careful`, or parameterized `careful(level = "high")`
2. **Categorization**: `careful(category = "crypto")`, `careful(category = "perf")`
3. **Documentation integration**: Automatic generation of "careful usage" sections in docs
4. **Metrics and reporting**: Compiler flags to report careful usage statistics
5. **CI integration**: Required approvals for PRs touching careful code

## Implementation Strategy

### Phase 1: Basic Language Support
- Add `careful` as a reserved keyword
- Implement function-level parsing and AST support
- Basic semantic analysis and error checking

### Phase 2: Block Support and Documentation
- Extend parser for block-level `careful` 
- Implement doc comment support for careful blocks
- Implement scope and nesting rules
- Update error messages to show relevant documentation

### Phase 3: Tooling Integration
- rustdoc integration showing "# Careful Usage" sections prominently
- IDE support with hover tooltips showing careful documentation
- Cargo integration for linting undocumented careful code

### Phase 4: Advanced Features
- Smart compiler diagnostics showing context-specific warnings
- Code review tool integration with careful-specific workflows
- Ecosystem adoption guidelines and best practices

## Conclusion

The `careful` keyword addresses a real need for marking code that requires enhanced scrutiny beyond memory safety concerns. By providing both function-level and block-level granularity with integrated documentation support, it enables precise communication about which parts of code need careful review and why.

## Documentation Integration

### Function Documentation Convention
Functions marked `careful` should include a "# Careful Usage" section in their doc comments explaining specific concerns:

```rust
/// Brief description of what the function does
/// 
/// # Careful Usage
/// Detailed explanation of what makes this function require extra attention,
/// common mistakes, and guidance for correct usage.
careful fn tricky_function() { }
```

### Block Documentation
`careful` blocks should be immediately preceded by `///` doc comments explaining the specific concerns:

```rust
/// Explanation of why this block needs careful review
/// and what constraints must be maintained
careful {
    // implementation requiring care
}
```

### IDE Integration
IDEs can display the relevant documentation when:
- Hovering over `careful` function calls (show "# Careful Usage" section)
- Hovering over `careful` blocks (show preceding doc comment)
- Providing code completion warnings with context-specific guidance

This approach ensures that the reasoning behind `careful` annotations is always available to developers and tooling.

### Relationship to General Block Documentation
While this RFC focuses on documentation for `careful` blocks, the ability for any block to have preceding `///` doc comments could be a valuable general language feature. However, that broader capability is orthogonal to this RFC and should be considered as a separate language enhancement.

The feature is designed to be lightweight, optional, and primarily serve as a communication and tooling aid, making it a low-risk addition that can significantly improve code review practices and code maintainability.