# fastn-ansi-renderer

ANSI terminal rendering engine for fastn documents with CSS-accurate layout calculations.

## Features

- **CSS Layout Engine** - Taffy integration for flexbox, grid, and block layout
- **ANSI Terminal Graphics** - Unicode box drawing with terminal colors and escape codes
- **Structured Output** - Rendered type with multiple format options (.to_ansi(), .to_plain())
- **Document Architecture** - Clean API for rendering complete fastn documents
- **Test-Driven Development** - Comprehensive test suite with CSS layout validation

## Clean API

```rust
use fastn_ansi_renderer::DocumentRenderer;

// Render fastn document to structured output
let rendered = DocumentRenderer::render_from_source(
    "-- ftd.text: Hello World\nborder-width.px: 1\npadding.px: 8",
    80,   // width in characters
    128   // height in lines
)?;

// Choose output format
println!("{}", rendered.to_ansi());        // Terminal with ANSI colors
println!("{}", rendered.to_plain());       // Plain ASCII for editors  
save_file(rendered.to_side_by_side());     // Spec file format
```

## Architecture

### Core Components

- **Taffy Integration** - CSS layout calculations
- **ANSI Canvas** - Character grid with color support  
- **Component Renderers** - Text, Column, Row, etc.
- **Coordinate Conversion** - Pixel to character mapping

### Dependencies

- `taffy` - CSS layout engine (used by Dioxus, Bevy UI)
- `ansi_term` - ANSI color support
- `unicode-width` - Proper character width handling

## Testing

```bash
cargo test -p fastn-ascii-renderer
```

The test suite includes:
- Layout engine validation  
- CSS property mapping verification
- End-to-end rendering pipeline tests
- ANSI color output verification

## Integration

Used by:
- `fastn-spec-viewer` - Component specification browser
- Future `fastn render` - Terminal application browser  

The renderer provides the **shared foundation** for all fastn terminal UI applications.