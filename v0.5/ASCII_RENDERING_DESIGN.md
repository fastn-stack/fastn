# ASCII Rendering Pipeline Design for fastn v0.5

## Executive Summary

Design for implementing ASCII rendering as a first-class citizen in fastn v0.5, enabling test-driven UI specification and terminal-friendly output from day one.

## Strategic Decision: v0.5 Implementation

**Why v0.5 instead of v0.4:**
- v0.5 is early stage - can integrate ASCII rendering from the beginning
- Avoid technical debt from retrofitting existing codebase
- Test-driven development approach ensures quality from start
- ASCII rendering becomes core feature, not afterthought

## Architecture Overview

### Current v0.5 Pipeline
```
FTD Source → fastn_compiler::compile → fastn_runtime::HtmlData → HTML
```

### New ASCII Pipeline (Parallel)
```
FTD Source → fastn_compiler::compile → fastn_ascii_renderer::AsciiData → ASCII Art
```

### Integration Points
- **Input**: Same `CompiledDocument` from fastn-compiler
- **Output**: ASCII string (parallel to HTML string)
- **CLI**: `fastn render --format=ascii input.ftd`
- **Testing**: Automated `.ftd`/`.ftd-rendered` verification

## Technical Design

### 1. Crate Structure: `fastn-ascii-renderer`

```
fastn-ascii-renderer/
├── src/
│   ├── lib.rs                 // Public API: render_ascii()
│   ├── canvas.rs              // Unicode box drawing, text positioning
│   ├── layout.rs              // Character-based layout engine
│   ├── renderer.rs            // AsciiData (parallel to HtmlData)
│   └── components/            // Component-specific renderers
│       ├── mod.rs
│       └── text.rs            // TextRenderer implementation
└── tests/
    └── basic_rendering.rs     // Foundation tests
```

### 2. Core Abstractions

#### Canvas System
```rust
struct Canvas {
    grid: Vec<Vec<char>>,    // Character grid
    width: usize,            // Characters wide
    height: usize,           // Lines tall
}

// Methods: draw_border(), draw_text(), to_string()
```

#### Component Renderer Trait
```rust
trait ComponentRenderer {
    fn calculate_layout(&self, constraints: &LayoutConstraints) -> ComponentLayout;
    fn render(&self, canvas: &mut Canvas, layout: &ComponentLayout);
}
```

#### Layout Engine
```rust
struct LayoutConstraints {
    max_width: Option<usize>,   // Character limits
    max_height: Option<usize>,
}

struct ComponentLayout {
    width: usize,               // Calculated dimensions
    height: usize,
    content_width: usize,       // Inner content area
    content_height: usize,
}
```

### 3. Coordinate System

**Character-based Layout:**
- 1 character = 1 unit width
- 1 line = 1 unit height  
- Pixel conversions: `16px ≈ 2 chars` for spacing
- Box drawing uses Unicode: `┌─┐│└┘` for borders

### 4. Test-Driven Specification

**Test Case Pairs:**
```
text-basic.ftd              → text-basic.ftd-rendered
text-with-border.ftd        → text-with-border.ftd-rendered
column-spacing.ftd          → column-spacing.ftd-rendered
```

**Verification:**
```rust
#[test]
fn verify_text_basic() {
    verify_rendering("text-basic.ftd", "text-basic.ftd-rendered").unwrap();
}
```

## Development Approach: Spec-Implement-Test Cycle

### Traditional Approach (Problematic)
```
Implement Component → Add Tests Later → Document Eventually
```

### New Integrated Approach
```
Design ASCII Spec → Implement Component + ASCII → Write Tests → Verify → Next Component
```

### Benefits
1. **Quality from Day 1** - ASCII rendering integrated, not retrofitted
2. **Specification IS Testing** - `.ftd-rendered` files ARE the spec
3. **No Rework** - Built correctly the first time
4. **Comprehensive Coverage** - Every attribute tested and specified

## Implementation Phases

### Phase 1: Foundation (Week 1)
- ✅ **Crate Structure** - Basic fastn-ascii-renderer crate
- ✅ **Canvas System** - Unicode box drawing primitives
- 🚧 **Fix Compilation** - Resolve import/dependency issues
- 🚧 **Basic Text** - Simple text rendering without styling

### Phase 2: Core Components (Week 2)  
- **TextRenderer** - Borders, padding, wrapping, typography roles
- **ColumnRenderer** - Vertical layout with spacing
- **RowRenderer** - Horizontal layout with spacing
- **Test Cases** - Comprehensive .ftd/.ftd-rendered pairs

### Phase 3: Advanced Layout (Week 3)
- **Flexbox Spacing** - space-between, space-around, space-evenly
- **Nested Layouts** - Complex component trees
- **Constraint Resolution** - Width/height limits and wrapping
- **Container Component** - Generic positioning

### Phase 4: Form & Media (Week 4)
- **Form Components** - checkbox, text-input ASCII representation
- **Media Components** - image, audio, video placeholders
- **Interactive States** - Hover, focus, disabled states in terminal
- **Performance & Polish** - Optimization and edge case handling

## Success Metrics

1. **Component Parity** - All v0.5 components have ASCII rendering
2. **Test Coverage** - Every component attribute tested via .ftd-rendered files  
3. **CLI Integration** - `fastn render --format=ascii` works seamlessly
4. **Specification Quality** - ASCII output clearly shows component behavior
5. **Developer Experience** - Easy to add new components with ASCII support

## Long-term Vision

This foundation enables:
- **Terminal-based fastn apps** - Real applications that run in terminals
- **Debug visualization** - Visual debugging of layout issues
- **Documentation generation** - Automatic ASCII examples in docs
- **Test-driven UI development** - Specification through executable tests
- **Cross-platform consistency** - Same layout logic for web and terminal

The ASCII rendering pipeline becomes a cornerstone of fastn v0.5, ensuring visual consistency and providing comprehensive testing infrastructure for all UI components.