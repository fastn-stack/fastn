# fastn Language Specification v0.5

**Complete specification for the fastn language including syntax, semantics, rendering, CLI, and tools**

## Table of Contents

1. [Language Overview](#language-overview)
2. [Syntax Specification](#syntax-specification)  
3. [Type System](#type-system)
4. [Components](#components)
5. [Layout System](#layout-system)
6. [Rendering Pipeline](#rendering-pipeline)
7. [CLI Tools](#cli-tools)
8. [Development Workflow](#development-workflow)

## Language Overview

### **What is fastn?**
fastn is a full-stack web development language with its own syntax (.ftd files), CSS-like layout system, and multiple rendering backends (web, terminal, PDF, etc.).

### **Core Concepts:**
- **Documents** - `.ftd` files containing component definitions and layouts
- **Components** - UI elements like `ftd.text`, `ftd.column`, `ftd.button`  
- **CSS Properties** - Layout and styling using CSS-like syntax
- **Responsive Design** - Components adapt to available space
- **Terminal-first** - Native support for terminal/ANSI rendering

### **Example fastn Document:**
```ftd
-- ftd.text: Hello World
border-width.px: 1
padding.px: 8
color: red
background.solid: yellow

-- ftd.column:
spacing.fixed.px: 16
width: fill-container

    -- ftd.text: First Item
    -- ftd.text: Second Item
    
-- end: ftd.column
```

## Syntax Specification

### **Document Structure**
Every fastn document is composed of **sections** that define components, variables, or functions.

#### **Section Syntax:**
```
-- section-type argument:
header-property: value
nested-header.property: nested-value

    -- child-section: Child content
    
-- end: section-type
```

#### **Component Invocation:**
```ftd
-- ftd.text: Text content goes here
property: value
nested.property: nested-value

-- ftd.column:
spacing.fixed.px: 20

    -- ftd.text: Child component
    color: blue
    
-- end: ftd.column
```

### **Property Syntax**

#### **Basic Properties:**
```ftd
width: 100                    # Integer value  
height: fill-container        # Keyword value
color: red                    # Named color
background.solid: #FF0000     # Hex color
```

#### **Nested Properties:**
```ftd
border.width.px: 1            # border-width: 1px
border.color: black           # border-color: black  
margin.top.px: 10             # margin-top: 10px
padding.horizontal.px: 20     # padding-left: 20px, padding-right: 20px
```

#### **CSS-like Values:**
```ftd
width.fixed.px: 200          # width: 200px
width.percent: 50            # width: 50%  
width: fill-container        # width: 100%
width: hug-content          # width: auto
```

### **Comments:**
```ftd
;; Single line comment
-- ftd.text: Hello World  ;; Inline comment
;; Multi-line comments use multiple single-line comments
```

## Type System

### **Primitive Types:**
- `string` - Text values
- `integer` - Whole numbers  
- `decimal` - Floating point numbers
- `boolean` - true/false values

### **Built-in Types:**
- `ftd.color` - Color values (red, #FF0000, rgba(255,0,0,1))
- `ftd.length` - Size values with units (px, %, em, rem)
- `ftd.spacing` - Spacing behavior (fixed, space-between, space-around)
- `ftd.resizing` - Sizing behavior (fill-container, hug-content, auto)

### **User-Defined Types:**

#### **Records:**
```ftd
-- record person:
string name:
integer age:
optional string email:

-- person user:
name: John Doe
age: 30
email: john@example.com
```

#### **Or-Types (Unions):**
```ftd
-- or-type status:

-- status.loading:

-- status.success:
string message:

-- status.error:
string error-message:

-- status current-status: $status.success
message: Operation completed
```

## Components

### **Built-in Components**

#### **Text Component:**
```ftd
-- ftd.text: Hello World
role: $inherited.types.heading-large
color: red
text-align: center
width.fixed.px: 200
```

**Properties:**
- `text: caption or body` (required) - Text content
- `color: ftd.color` - Text color
- `role: ftd.type` - Typography role
- `text-align: ftd.text-align` - Text alignment
- All common properties (width, height, padding, margin, border)

#### **Layout Components:**
```ftd
-- ftd.column:
spacing.fixed.px: 20
align-content: center
width: fill-container

    -- ftd.text: Item 1
    -- ftd.text: Item 2
    
-- end: ftd.column

-- ftd.row:
spacing: space-between
width: fill-container

    -- ftd.text: Left
    -- ftd.text: Center  
    -- ftd.text: Right
    
-- end: ftd.row
```

**Column Properties:**
- `spacing: ftd.spacing` - Space between children
- `align-content: ftd.align` - Child alignment
- All container properties

**Row Properties:**
- Same as column but arranges children horizontally

### **Form Components:**
```ftd
-- ftd.text-input:
placeholder: Enter your name
value: $user-input
$on-input$: $ftd.set-string($a = $user-input, v = $VALUE)

-- ftd.checkbox:
checked: $is-enabled
$on-click$: $ftd.toggle($a = $is-enabled)
```

### **Custom Components:**
```ftd
-- component card:
caption title:
optional body description:
optional string image-url:

-- ftd.column:
border-width.px: 1
border-radius.px: 8
padding.px: 16
background.solid: white

    -- ftd.text: $card.title
    role: $inherited.types.heading-medium
    
    -- ftd.text: $card.description
    if: { card.description != NULL }
    
-- end: ftd.column

-- end: card
```

## Layout System

### **CSS Box Model**
All components follow CSS box model:
```
┌─ margin ──────────────────────────┐
│ ┌─ border ────────────────────────┐ │
│ │ ┌─ padding ─────────────────────┐ │ │
│ │ │                              │ │ │
│ │ │         content              │ │ │  
│ │ │                              │ │ │
│ │ └──────────────────────────────┘ │ │
│ └────────────────────────────────────┘ │
└──────────────────────────────────────────┘
```

### **Flexbox Layout**
Containers use CSS flexbox for child arrangement:

#### **Column Layout (flex-direction: column):**
```ftd
-- ftd.column:
spacing.fixed.px: 16        # gap: 16px
align-content: center       # align-items: center
```

#### **Row Layout (flex-direction: row):**  
```ftd
-- ftd.row:
spacing: space-between      # justify-content: space-between
align-content: start        # align-items: flex-start
```

### **Sizing Behavior:**
- `width: fill-container` → CSS `width: 100%`
- `width.fixed.px: 200` → CSS `width: 200px` 
- `width: hug-content` → CSS `width: auto`
- `width.percent: 75` → CSS `width: 75%`

### **Spacing Behavior:**
- `spacing.fixed.px: 20` → CSS `gap: 20px`
- `spacing: space-between` → CSS `justify-content: space-between`
- `spacing: space-around` → CSS `justify-content: space-around`
- `spacing: space-evenly` → CSS `justify-content: space-evenly`

## Rendering Pipeline

### **Architecture: ftd → CSS → Layout → ANSI**

#### **1. Document Parsing**
```rust
// Input: fastn source code
let source = "-- ftd.text: Hello World\nborder-width.px: 1";

// Output: Structured document
let document = FastnDocument {
    root_component: SimpleFtdComponent {
        text: Some("Hello World"),
        border_width: Some(1),
        // ... parsed properties
    }
};
```

#### **2. CSS Property Mapping**
```rust
// Convert fastn properties to CSS
let css_mapper = FtdToCssMapper::new();
let css_style = css_mapper.component_to_style(&document.root_component);

// fastn: border-width.px: 1  →  CSS: border: Rect::from_length(1px)
// fastn: padding.px: 8       →  CSS: padding: Rect::from_length(8px)
// fastn: width: fill         →  CSS: size.width: Dimension::Percent(1.0)
```

#### **3. Layout Calculation (Taffy CSS Engine)**
```rust
// Professional CSS layout calculation
let mut layout_engine = TaffyLayoutEngine::new();
let node = layout_engine.create_text_node(&text_content, css_style)?;

let available_space = Size {
    width: AvailableSpace::Definite((width * 8) as f32),    // chars → px  
    height: AvailableSpace::Definite((height * 16) as f32), // lines → px
};

layout_engine.compute_layout(available_space)?;
let computed_layout = layout_engine.get_layout(node)?;
```

#### **4. Coordinate Conversion**
```rust
// Convert pixel-based layout to character coordinates
let converter = CoordinateConverter::new();
let char_rect = converter.taffy_layout_to_char_rect(computed_layout);

// 96px width  → 12 characters (96 ÷ 8px/char)
// 32px height → 2 lines (32 ÷ 16px/line)
```

#### **5. ANSI Canvas Rendering**
```rust
let mut canvas = AnsiCanvas::new(width, height);

// Render using computed layout coordinates
canvas.draw_border(char_rect, BorderStyle::Single, AnsiColor::Default);
canvas.draw_text(text_pos, &text_content, AnsiColor::Red, None);

let ansi_output = canvas.to_ansi_string();
```

### **fastn-ansi-renderer API**

#### **Core API:**
```rust
use fastn_ansi_renderer::DocumentRenderer;

// Render fastn document to structured output
let rendered = DocumentRenderer::render_from_source(
    "-- ftd.text: Hello World\nborder-width.px: 1",
    80,   // width in characters
    128   // height in lines  
)?;

// Choose output format:
rendered.to_ansi()        // Terminal with ANSI colors
rendered.to_plain()       // Plain ASCII for editors
rendered.to_side_by_side() // Spec file format
```

#### **Output Formats:**

**ANSI Terminal (.to_ansi()):**
```
┌──────────────────┐
│ \x1b[31mHello World\x1b[0m │  ← With ANSI color codes
└──────────────────┘
```

**Plain ASCII (.to_plain()):**
```  
┌──────────────────┐
│ Hello World      │  ← Clean ASCII, no escape codes
└──────────────────┘
```

**Side-by-Side (.to_side_by_side()):**
```
┌──────────────────┐          ┌──────────────────┐
│ Hello World      │          │ Hello World      │  ← Plain + ANSI
└──────────────────┘          └──────────────────┘
```

## CLI Tools

### **fastn spec-viewer**
Interactive specification browser and testing tool.

#### **Usage:**
```bash
# Interactive TUI browser
fastn spec-viewer

# Render specific document
fastn spec-viewer text/with-border.ftd --stdout --width=80 --height=128

# Validate specifications
fastn spec-viewer --check

# Update specifications  
fastn spec-viewer --autofix
```

#### **File Format:**
Specifications stored as single `.rendered` files with all dimensions:

```
# 40x64

┌──────────┐          ┌──────────┐
│Hello World│          │Hello World│  ← Plain + ANSI side-by-side
└──────────┘          └──────────┘



# 80x128

┌────────────────────┐          ┌────────────────────┐ 
│     Hello World    │          │     Hello World    │
└────────────────────┘          └────────────────────┘



# 120x192

┌────────────────────────────────┐          ┌────────────────────────────────┐
│         Hello World            │          │         Hello World            │  
└────────────────────────────────┘          └────────────────────────────────┘
```

#### **Strict Format Requirements:**
- **Headers:** Exactly `# {width}x{height}` (no variations)
- **Spacing:** 1 blank line after header, 4 blank lines between sections
- **Alignment:** Exactly 10 spaces between plain and ANSI versions
- **Side-by-side:** Plain ASCII on left, ANSI on right

#### **Liberal Autofix:**
- **Accepts:** Broken headers, missing sections, inconsistent spacing
- **Regenerates:** Fresh content with perfect strict formatting  
- **Always complete:** All dimensions included regardless of input

### **fastn render** (Future)
Terminal browser for complete fastn applications.

#### **Usage:**
```bash
# Terminal browser for fastn applications
fastn render --package=./myapp --url=/dashboard --id52=myapp.local

# Interactive terminal application
fastn render --package=./myapp --interactive
```

## Development Workflow

### **Specification Development:**
```bash
# 1. Create component document
echo "-- ftd.text: New Component\nborder-width.px: 1" > specs/new-component.ftd

# 2. Preview in terminal
fastn spec-viewer new-component.ftd --stdout --width=80

# 3. Generate test snapshots
fastn spec-viewer --autofix new-component.ftd

# 4. Validate everything works
fastn spec-viewer --check
```

### **Responsive Testing:**
```bash
# Test at different widths
fastn spec-viewer component.ftd --stdout --width=40   # Mobile
fastn spec-viewer component.ftd --stdout --width=80   # Tablet  
fastn spec-viewer component.ftd --stdout --width=120  # Desktop

# Test with custom height
fastn spec-viewer component.ftd --stdout --width=80 --height=200
```

### **Quality Assurance:**
```bash
# Validate all specifications
fastn spec-viewer --check

# Auto-fix formatting issues
fastn spec-viewer --autofix

# CI/CD integration
fastn spec-viewer --check || exit 1  # Fail build on spec mismatch
```

## Implementation Details

### **fastn-ansi-renderer Architecture**
```rust
// Core rendering pipeline
pub struct DocumentRenderer;

impl DocumentRenderer {
    pub fn render_from_source(source: &str, width: usize, height: usize) -> Result<Rendered, Error>;
}

// Structured output
pub struct Rendered {
    pub fn to_ansi(&self) -> &str;           // Terminal display
    pub fn to_plain(&self) -> String;        // Editor viewing  
    pub fn to_side_by_side(&self) -> String; // Spec format
}
```

### **CSS Layout Integration**
- **Taffy engine** - Professional CSS flexbox/grid implementation
- **Property mapping** - fastn properties → CSS properties
- **Responsive calculation** - Width/height constraints → layout
- **Coordinate conversion** - Pixels → character coordinates

### **Terminal Graphics**
- **Unicode box drawing** - Professional borders (┌─┐│└┘)
- **ANSI colors** - Full terminal color support  
- **Escape code management** - Proper ANSI sequence generation
- **Format stripping** - Clean ASCII version generation

## Language Semantics

### **Component Composition**
Components can contain other components in a tree structure:

```ftd
-- ftd.column:              # Root container
    
    -- ftd.text: Header     # Child component
    role: $inherited.types.heading-large
    
    -- ftd.row:             # Nested container
        
        -- ftd.text: Left   # Nested child
        -- ftd.text: Right  # Nested child
        
    -- end: ftd.row
    
-- end: ftd.column
```

### **Property Inheritance**
Child components inherit certain properties from parents:

```ftd
-- ftd.column:
color: blue                 # Inherited by children

    -- ftd.text: Blue Text  # Inherits blue color
    
    -- ftd.text: Red Text   # Overrides with explicit color  
    color: red
    
-- end: ftd.column
```

### **Responsive Behavior**
Components automatically adapt to available space:

```ftd
-- ftd.text: Responsive Text
width: fill-container       # Fills available width
border-width.px: 1         # Border adapts to content size
padding.px: 8              # Padding maintains proportional spacing
```

**Rendered at different widths:**
- **40ch**: Narrow border with compact layout
- **80ch**: Medium border with comfortable spacing  
- **120ch**: Wide border with generous spacing

## Future Extensions

### **Runtime Integration**
- **Event handling** - Click, input, keyboard events
- **State management** - Component state and updates
- **JavaScript integration** - Custom behavior scripting

### **Multiple Backends**
- **Terminal (ANSI)** - Current implementation  
- **Web (HTML/CSS)** - Browser rendering
- **PDF** - Document generation
- **SVG** - Vector graphics export

### **Advanced Layout**
- **CSS Grid** - 2D layout capabilities
- **Animations** - Smooth transitions and effects
- **Media queries** - Responsive breakpoints

This specification provides the **complete reference** for fastn language implementation, ensuring consistency across all tools and rendering backends.