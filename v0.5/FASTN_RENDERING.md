# fastn Rendering Architecture

**Complete design for how fastn documents are rendered to ANSI terminal output**

## Overview

This document describes the complete pipeline from `.ftd` source files to rendered ANSI terminal output, including CSS layout calculations, terminal graphics, and multiple output formats.

## Pipeline: ftd → CSS → Layout → ANSI

### **1. Document Parsing**
```
-- ftd.text: Hello World     →  FastnDocument {
border-width.px: 1               root_component: SimpleFtdComponent {
padding.px: 8                        text: "Hello World",
color: red                           border_width: Some(1),
                                     padding: Some(8),
                                     // ... CSS properties
                                 }
                             }
```

### **2. CSS Property Mapping**  
```
FastnDocument  →  CSS Mapper  →  Taffy Style
                                 {
                                   size: { width: Auto, height: Auto },
                                   padding: Rect::from_length(8px),
                                   border: Rect::from_length(1px),
                                   // ... complete CSS properties
                                 }
```

### **3. Layout Calculation (Taffy CSS Engine)**
```
Taffy Style + Available Space  →  Computed Layout
{                                 {
  width: auto,                      location: { x: 0, y: 0 },
  height: auto,                     size: { width: 96px, height: 32px },
  padding: 8px,                     padding: { left: 8, right: 8, top: 8, bottom: 8 },
  border: 1px,                      border: { left: 1, right: 1, top: 1, bottom: 1 },
}                                 }
```

### **4. Coordinate Conversion**
```
Taffy Layout (pixels)  →  Character Coordinates
{                         {
  size: { 96px, 32px },     x: 0, y: 0,
  location: { 0, 0 },       width: 12 chars,  // 96px ÷ 8px/char
}                           height: 2 lines,  // 32px ÷ 16px/line
                          }
```

### **5. ANSI Canvas Rendering**
```
Character Layout  →  ANSI Canvas  →  Terminal Output
{                    {                ┌──────────┐
  x: 0, y: 0,          chars: [...],   │          │
  width: 12,           colors: [...],  │Hello World│
  height: 2,         }                │          │
}                                     └──────────┘
```

## Core Architecture

### **fastn-ansi-renderer (Pure Rendering Engine)**

#### **Single Responsibility:**
Render fastn documents to ANSI terminal output using CSS-accurate layout calculations.

#### **Clean API:**
```rust
pub struct DocumentRenderer;

impl DocumentRenderer {
    /// Main API - render fastn document source
    pub fn render_from_source(source: &str, width: usize, height: usize) -> Result<Rendered, Error>;
    
    /// Advanced API - render parsed document
    pub fn render_document(doc: &FastnDocument, width: usize, height: usize) -> Result<Rendered, Error>;
}

/// Structured output with multiple format options
pub struct Rendered {
    ansi_output: String,
}

impl Rendered {
    pub fn to_ansi(&self) -> &str;          // Terminal display with colors
    pub fn to_plain(&self) -> String;       // Editor viewing without escape codes
    pub fn to_side_by_side(&self) -> String; // Spec file format
}
```

#### **Core Components:**
- **DocumentRenderer** - High-level rendering API
- **TaffyLayoutEngine** - CSS flexbox/grid layout calculations
- **AnsiCanvas** - Character grid with ANSI color support
- **CoordinateConverter** - Pixel to character coordinate mapping
- **CSS Property Mapper** - fastn properties to CSS properties

#### **Dependencies:**
- `taffy` - CSS layout engine (used by Dioxus, Bevy UI)
- `ansi_term` - ANSI terminal color support  
- `unicode-width` - Proper character width calculations

### **CSS Property Support**

#### **Layout Properties:**
- `width: fill-container` → CSS `width: 100%`
- `width.fixed.px: 200` → CSS `width: 200px`
- `padding.px: 8` → CSS `padding: 8px`
- `margin.px: 16` → CSS `margin: 16px`

#### **Visual Properties:**
- `border-width.px: 1` → CSS `border-width: 1px`
- `color: red` → ANSI color code `\x1b[31m`
- `background.solid: blue` → ANSI background `\x1b[44m`

#### **Layout Behavior:**
- `spacing.fixed.px: 20` → CSS `gap: 20px`
- `spacing: space-between` → CSS `justify-content: space-between`
- `align-content: center` → CSS `align-items: center`

## Rendering Process

### **Step 1: Parse fastn Source**
```rust
fn parse_fastn_source(source: &str) -> Result<FastnDocument, ParseError> {
    // Simple parser for prototype - will integrate with real fastn parser
    // Extracts components and CSS properties from fastn source
}
```

### **Step 2: CSS Property Mapping**
```rust
let css_mapper = FtdToCssMapper::new();
let taffy_style = css_mapper.component_to_style(&document.root_component);

// Maps fastn properties to CSS:
// border-width.px: 1  →  border: Rect::from_length(1px)
// padding.px: 8       →  padding: Rect::from_length(8px)  
// width: fill         →  size.width: Dimension::Percent(1.0)
```

### **Step 3: Layout Calculation**
```rust
let mut layout_engine = TaffyLayoutEngine::new();
let node = layout_engine.create_text_node(&text_content, taffy_style)?;

let available_space = Size {
    width: AvailableSpace::Definite((width * 8) as f32),    // chars → px
    height: AvailableSpace::Definite((height * 16) as f32), // lines → px  
};

layout_engine.compute_layout(available_space)?;
let computed_layout = layout_engine.get_layout(node)?; // Final pixel coordinates
```

### **Step 4: Coordinate Conversion**
```rust
let converter = CoordinateConverter::new();
let char_rect = converter.taffy_layout_to_char_rect(computed_layout);

// Converts:
// 96px width  → 12 characters (96 ÷ 8)
// 32px height → 2 lines (32 ÷ 16) 
// 0px x,y     → 0,0 character position
```

### **Step 5: ANSI Canvas Rendering**
```rust
let mut canvas = AnsiCanvas::new(width, height);

// Render using computed layout
canvas.draw_border(char_rect, BorderStyle::Single, AnsiColor::Default);
canvas.draw_text(text_pos, &text_content, AnsiColor::Red, None);

let ansi_output = canvas.to_ansi_string(); // With ANSI escape codes
```

### **Step 6: Format Output**
```rust
let rendered = Rendered::new(ansi_output);

// Multiple format options:
rendered.to_ansi()        // "\x1b[31mHello World\x1b[0m" (with colors)
rendered.to_plain()       // "Hello World" (clean ASCII)
rendered.to_side_by_side() // "Hello World          \x1b[31mHello World\x1b[0m"
```

## Output Formats

### **ANSI Terminal Format (.to_ansi())**
```
┌──────────┐
│Hello World│  ← Red text with ANSI escape codes
└──────────┘
```
**Use cases:** Terminal preview, interactive applications, real-time display

### **Plain ASCII Format (.to_plain())**  
```
┌──────────┐
│Hello World│  ← Clean ASCII without escape codes
└──────────┘
```
**Use cases:** Editor viewing, documentation, text processing

### **Side-by-Side Format (.to_side_by_side())**
```
┌──────────┐          ┌──────────┐
│Hello World│          │Hello World│  ← Plain + ANSI side by side  
└──────────┘          └──────────┘
```
**Use cases:** Specification files, comparison, validation

## Responsive Behavior

### **Width Responsiveness**
Components adapt to available terminal width:

```
40 characters:          80 characters:                    120 characters:
┌──────────┐            ┌────────────────────────────┐     ┌────────────────────────────────────────┐
│Hello World│            │        Hello World         │     │              Hello World               │
└──────────┘            └────────────────────────────┘     └────────────────────────────────────────┘
```

### **Height Responsiveness**  
Components adapt to available terminal height:

```
64 lines (compact):     128 lines (standard):        192 lines (tall):
┌──────────┐            ┌──────────┐                 ┌──────────┐
│Hello World│            │          │                 │          │
└──────────┘            │Hello World│                 │          │
                        │          │                 │Hello World│
                        └──────────┘                 │          │
                                                     │          │
                                                     └──────────┘
```

## Integration Points

### **Current Integration:**
- **fastn-spec-viewer** - Specification browser using DocumentRenderer API
- **Test suite** - Validates CSS layout accuracy and ANSI output

### **Future Integration:**
- **fastn render** - Terminal application browser  
- **fastn build** - Static site generation with ANSI previews
- **Extensions** - Third-party tools rendering fastn documents

## Benefits of Architecture

### **Clean Separation:**
- **Rendering engine** - Pure function, no domain knowledge
- **Applications** - Use rendering API, manage their own concerns
- **Testable** - Rendering isolated and independently verifiable

### **CSS Accuracy:**
- **Taffy integration** - Same layout engine as major Rust UI frameworks
- **Proper box model** - CSS padding, margin, border calculations  
- **Responsive layout** - True CSS-like responsive behavior

### **Terminal Optimization:**
- **ANSI colors** - Full terminal color support
- **Unicode graphics** - Professional box drawing characters
- **Format flexibility** - Choose appropriate output format per use case

This architecture provides the foundation for **production-quality terminal applications** with **CSS-accurate layout** and **professional visual output**.