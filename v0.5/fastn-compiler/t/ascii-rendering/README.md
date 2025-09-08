# ASCII Rendering Test Cases

This directory contains test cases for verifying ASCII rendering output of FTD components.

## Test Structure

Each test case consists of:
- `*.ftd` - Input FTD code
- `*.ftd-rendered` - Expected ASCII output

## Running Tests

```bash
cargo test ascii_rendering
```

## Test Cases

### Basic Components
- `text-basic.ftd` / `text-basic.ftd-rendered` - Simple text rendering
- `text-border.ftd` / `text-border.ftd-rendered` - Text with border and padding
- `column-simple.ftd` / `column-simple.ftd-rendered` - Basic column layout
- `column-spacing.ftd` / `column-spacing.ftd-rendered` - Column with spacing
- `row-simple.ftd` / `row-simple.ftd-rendered` - Basic row layout

### Layout Attributes
- `spacing-fixed.ftd` / `spacing-fixed.ftd-rendered` - Fixed spacing effects
- `spacing-between.ftd` / `spacing-between.ftd-rendered` - Space-between behavior
- `padding-effects.ftd` / `padding-effects.ftd-rendered` - Padding visualization
- `nested-layout.ftd` / `nested-layout.ftd-rendered` - Complex nested structures

### Interactive Elements
- `checkbox-states.ftd` / `checkbox-states.ftd-rendered` - Checkbox checked/unchecked
- `text-input.ftd` / `text-input.ftd-rendered` - Input field rendering
- `button-states.ftd` / `button-states.ftd-rendered` - Button appearances

## Purpose

These test cases serve multiple purposes:

1. **Specification** - Define exactly how components should render
2. **Testing** - Automated verification of rendering pipeline
3. **Documentation** - Visual examples of component behavior  
4. **Regression Testing** - Catch rendering changes

## Implementation

The test runner:
1. Parses each `.ftd` file through the fastn compiler
2. Renders to ASCII string output (no terminal dependency)
3. Compares output with corresponding `.ftd-rendered` file
4. Reports any differences as test failures

This ensures the ASCII rendering pipeline produces consistent, predictable output.