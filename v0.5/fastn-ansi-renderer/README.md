# fastn-ascii-renderer

ASCII rendering engine for fastn UI components with CSS-accurate layout calculations.

## Features

- **CSS Layout Engine** - Taffy integration for flexbox, grid, and block layout
- **Unicode Box Drawing** - Professional ASCII art with borders and styling  
- **ANSI Color Support** - Terminal colors for text and backgrounds
- **Component Architecture** - Extensible renderer system for all fastn components
- **Test-Driven Development** - Comprehensive test suite with .ftd/.rendered validation

## Usage

```rust
use fastn_ascii_renderer::{TaffyLayoutEngine, AnsiCanvas, FtdToCssMapper};

// Render fastn components to ASCII
let output = render_fastn_component(&component);
println!("{}", output);
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