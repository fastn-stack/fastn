# fastn-ascii-renderer

ASCII rendering engine for FTD components - converts compiled FTD documents to ASCII art.

## Architecture

### Integration with v0.5 Pipeline

```
fastn_compiler::compile() → CompiledDocument → AsciiRenderer → String
```

Parallel to existing HTML renderer:
```rust
// Current
let html_data = fastn_runtime::HtmlData::from_cd(compiled_doc);
let html = html_data.to_test_html();

// New
let ascii_data = fastn_ascii_renderer::AsciiData::from_cd(compiled_doc);  
let ascii = ascii_data.to_ascii();
```

### Module Structure

```
fastn-ascii-renderer/
├── src/
│   ├── lib.rs                 // Public API
│   ├── renderer.rs            // Main AsciiRenderer implementation
│   ├── layout/
│   │   ├── mod.rs            // Layout engine
│   │   ├── column.rs         // Column layout logic
│   │   ├── row.rs            // Row layout logic
│   │   ├── text.rs           // Text layout and wrapping
│   │   └── container.rs      // Generic container layout
│   ├── canvas/
│   │   ├── mod.rs            // Canvas drawing primitives
│   │   ├── box_drawing.rs    // Unicode box characters
│   │   └── text_rendering.rs // Text positioning and rendering
│   ├── components/
│   │   ├── mod.rs            // Component-specific renderers
│   │   ├── text.rs           // ftd.text rendering
│   │   ├── column.rs         // ftd.column rendering
│   │   ├── row.rs            // ftd.row rendering
│   │   └── forms.rs          // checkbox, text-input rendering
│   └── test_utils.rs         // Test utilities for .ftd-rendered verification
├── tests/
│   ├── integration.rs        // End-to-end rendering tests
│   └── test_cases/           // .ftd/.ftd-rendered test pairs
│       ├── text-basic.ftd
│       ├── text-basic.ftd-rendered
│       ├── column-spacing.ftd
│       └── column-spacing.ftd-rendered
└── Cargo.toml
```

### Core API Design

```rust
// Main public API
pub struct AsciiRenderer;

impl AsciiRenderer {
    pub fn render(compiled_doc: &CompiledDocument) -> String;
    pub fn render_component(component: &Component) -> AsciiLayout;
}

// Data structure parallel to HtmlData
pub struct AsciiData {
    layout: AsciiLayout,
    components: Vec<RenderedComponent>,
}

impl AsciiData {
    pub fn from_cd(compiled_doc: CompiledDocument) -> Self;
    pub fn to_ascii(&self) -> String;
}

// Layout representation
pub struct AsciiLayout {
    width: usize,
    height: usize,
    components: Vec<ComponentLayout>,
}

// Canvas for drawing
pub struct Canvas {
    grid: Vec<Vec<char>>,
    width: usize,
    height: usize,
}

impl Canvas {
    pub fn new(width: usize, height: usize) -> Self;
    pub fn draw_border(&mut self, rect: Rect, style: BorderStyle);
    pub fn draw_text(&mut self, pos: Position, text: &str, wrap_width: Option<usize>);
    pub fn to_string(&self) -> String;
}
```

### Component Renderer Traits

```rust
pub trait ComponentRenderer {
    fn calculate_layout(&self, constraints: LayoutConstraints) -> ComponentLayout;
    fn render(&self, canvas: &mut Canvas, layout: &ComponentLayout);
}

// Example implementations
impl ComponentRenderer for TextComponent {
    fn calculate_layout(&self, constraints: LayoutConstraints) -> ComponentLayout {
        // Calculate text dimensions, wrapping, etc.
    }
    
    fn render(&self, canvas: &mut Canvas, layout: &ComponentLayout) {
        // Draw borders, padding, then text
    }
}

impl ComponentRenderer for ColumnComponent {
    fn calculate_layout(&self, constraints: LayoutConstraints) -> ComponentLayout {
        // Stack children vertically with spacing
    }
    
    fn render(&self, canvas: &mut Canvas, layout: &ComponentLayout) {
        // Render container, then each child
    }
}
```

### Test Framework Integration

```rust
// Test utilities
pub fn verify_ftd_rendering(ftd_path: &Path, expected_path: &Path) -> Result<(), TestError>;
pub fn render_ftd_file(ftd_path: &Path) -> Result<String, RenderError>;

// Integration with cargo test
#[test]
fn text_basic_rendering() {
    verify_ftd_rendering(
        Path::new("test_cases/text-basic.ftd"),
        Path::new("test_cases/text-basic.ftd-rendered")
    ).unwrap();
}
```

### Integration Points

**Dependency on existing crates:**
- `fastn-compiler` - For CompiledDocument input
- `fastn-resolved` - For component definitions
- `fastn-section` - For basic data structures

**New dependency for v0.5 fastn binary:**
```toml
[dependencies]
fastn-ascii-renderer = { path = "../fastn-ascii-renderer" }
```

**CLI integration:**
```bash
cargo run --bin fastn -- render --format=ascii input.ftd > output.txt
cargo run --bin fastn -- test-ascii  # Run ASCII rendering tests
```

This creates a clean, modular ASCII rendering engine that integrates seamlessly with v0.5 architecture while enabling test-driven component development.