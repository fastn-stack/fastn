# ASCII Rendering Pipeline Design

## Overview

Design for implementing a comprehensive ASCII rendering pipeline that can render FTD components to terminal-friendly text output without dependencies on terminal/curses libraries.

## Goals

1. **Pure String Output** - Generate ASCII art as strings, no terminal dependencies
2. **Component Faithful** - Each FTD component renders with clear visual representation
3. **Layout Accurate** - Spacing, borders, padding render correctly in ASCII
4. **Test Driven** - Output can be verified against expected ASCII files
5. **Debuggable** - Clear mapping between FTD code and ASCII output

## Architecture Design

### 1. Rendering Pipeline Stages

```
FTD Source → Parser → AST → Layout Engine → ASCII Renderer → String Output
```

**Components:**
- **Parser** - Existing fastn-section/fastn-compiler pipeline
- **Layout Engine** - NEW: Calculate dimensions, positions in character space
- **ASCII Renderer** - NEW: Convert layout to ASCII art with box drawing
- **String Output** - Final ASCII text representation

### 2. Layout Engine Design

The layout engine needs to:

#### 2.1 Character-based Coordinate System
- Use character positions instead of pixels
- Standard mapping: `16px ≈ 2 chars` for spacing
- Fixed-width font assumptions for predictable layout

#### 2.2 Layout Tree Construction
```rust
struct AsciiLayout {
    width: usize,        // characters
    height: usize,       // lines  
    x: usize,           // horizontal position
    y: usize,           // vertical position
    border: BorderStyle,
    padding: Padding,
    children: Vec<AsciiLayout>,
}

struct BorderStyle {
    width: usize,       // 0, 1, or 2 for none/single/double
    style: LineStyle,   // single, double, dashed
}
```

#### 2.3 Layout Algorithms
- **Column Layout**: Stack children vertically with spacing
- **Row Layout**: Place children horizontally with spacing  
- **Flexbox-like**: Handle space-between, space-around, space-evenly
- **Constraint Resolution**: Handle width/height constraints, min/max

### 3. ASCII Renderer Design

#### 3.1 Box Drawing Characters
```rust
// Unicode box drawing characters
const SINGLE_LINE: &str = "┌─┐│└┘";
const DOUBLE_LINE: &str = "╔═╗║╚╝";  
const CORNERS: &str = "┌┬┐├┼┤└┴┘";
```

#### 3.2 Canvas Approach
```rust
struct Canvas {
    grid: Vec<Vec<char>>,
    width: usize,
    height: usize,
}

impl Canvas {
    fn draw_border(&mut self, x: usize, y: usize, width: usize, height: usize);
    fn draw_text(&mut self, x: usize, y: usize, text: &str);
    fn to_string(&self) -> String;
}
```

#### 3.3 Rendering Strategy
1. Calculate total layout dimensions
2. Create canvas of required size
3. Render background to foreground:
   - Borders first
   - Background fills  
   - Text content last
4. Handle overlapping/clipping

### 4. Component-Specific Rendering

#### 4.1 Text Component
```
Input: ftd.text: "Hello World", border-width: 1, padding: 4
Output:
┌─────────────────┐
│                 │  
│  Hello World    │
│                 │
└─────────────────┘
```

#### 4.2 Column Component  
```
Input: ftd.column with spacing.fixed.px: 16
Output: Children stacked vertically with 2-line gaps
```

#### 4.3 Row Component
```
Input: ftd.row with spacing.fixed.px: 20  
Output: Children placed horizontally with 4-char gaps
```

### 5. Implementation Phases

#### Phase 1: Foundation (Week 1)
1. **Layout Engine Core** - Basic layout tree and positioning
2. **Canvas Implementation** - ASCII drawing primitives  
3. **Basic Text Rendering** - Simple text output without styling

#### Phase 2: Layout Components (Week 2)
1. **Column Layout** - Vertical stacking with spacing
2. **Row Layout** - Horizontal arrangement
3. **Border Rendering** - Box drawing characters
4. **Padding/Margin** - Space handling

#### Phase 3: Advanced Features (Week 3)
1. **Flexbox Spacing** - space-between, space-around, space-evenly
2. **Nested Layouts** - Complex component trees
3. **Constraint Resolution** - Width/height limits
4. **Text Wrapping** - Long text in constrained width

#### Phase 4: Polish & Testing (Week 4)
1. **Test Framework** - Automated .ftd vs .ftd-rendered verification
2. **Edge Cases** - Overflow, empty components, complex nesting
3. **Performance** - Efficient rendering for large layouts
4. **Error Handling** - Graceful degradation

## Integration Points

### With Existing Codebase
- **Input**: Use existing fastn-compiler AST output
- **Output**: New ASCII renderer parallel to existing renderers
- **Testing**: Integrate with existing cargo test infrastructure

### API Design
```rust
// Main API
pub fn render_ascii(ast: &CompiledDocument) -> String;

// For testing
pub fn render_ftd_file(path: &Path) -> Result<String, RenderError>;
pub fn verify_rendering(ftd_file: &Path, expected_file: &Path) -> Result<(), TestError>;
```

## Success Criteria

1. **Complete Component Coverage** - All kernel components render correctly
2. **Layout Accuracy** - Spacing, borders, padding match expectations  
3. **Test Completeness** - Comprehensive test suite with .ftd/.ftd-rendered pairs
4. **Performance** - Renders complex layouts quickly
5. **Maintainability** - Clear separation of layout logic and rendering logic

## Risks & Mitigations

**Risk**: Complex layout algorithms 
**Mitigation**: Start with simple cases, iterate incrementally

**Risk**: ASCII art limitations for complex designs
**Mitigation**: Focus on structural clarity over visual perfection

**Risk**: Large implementation effort
**Mitigation**: Phase approach with early wins

This design provides a foundation for implementing ASCII rendering that serves both specification documentation and automated testing purposes.