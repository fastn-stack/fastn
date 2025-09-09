# ASCII Renderer Architecture - Simplified Design

## Overview

Design for a **production-quality ASCII renderer** as Phase 1 of fastn v0.5 rendering architecture. Focus on **static rendering** with proper CSS layout, ANSI colors, and comprehensive testing.

## Strategic Goals

### **Immediate Value**
- Production terminal applications
- UI specification and testing tool
- Learning platform for layout concepts
- Foundation for future GPU renderer

### **Scope Limitations (By Design)**
- ❌ **No runtime** - Static rendering only
- ❌ **No event handling** - No QuickJS integration  
- ❌ **No interactive features** - Pure output generation
- ✅ **Perfect for**: Documentation, testing, terminal display

## Technical Architecture

### **Dependency Strategy: Minimal + Proven**

```toml
[dependencies]
taffy = "0.5"           # CSS layout engine (flexbox, grid)
ansi_term = "0.12"      # ANSI color support
unicode-width = "0.2"   # Proper character width handling

# That's it! No GPU, no windowing, no complex graphics stack
```

### **Core Components**

#### 1. **Layout Engine** (`layout/`)
```rust
pub struct LayoutEngine {
    taffy_tree: taffy::TaffyTree,
    node_styles: HashMap<taffy::NodeId, FtdStyle>,
    root_node: Option<taffy::NodeId>,
}

impl LayoutEngine {
    fn parse_ftd_document(&mut self, doc: &FtdDocument) -> LayoutResult;
    fn compute_layout(&mut self, available_space: Size) -> ComputedLayout;
    fn get_final_layout(&self, node: taffy::NodeId) -> taffy::Layout;
}
```

**Responsibilities:**
- Parse FTD components into Taffy node tree
- Map FTD properties to CSS properties
- Handle flexbox, spacing, sizing calculations
- Provide final positions/dimensions for rendering

#### 2. **CSS Property Mapping** (`css/`)
```rust  
pub struct FtdToCssMapper;

impl FtdToCssMapper {
    fn map_spacing(&self, spacing: &FtdSpacing) -> taffy::Style;
    fn map_padding(&self, padding: &FtdPadding) -> taffy::Rect<taffy::LengthPercentage>;
    fn map_size(&self, size: &FtdSize) -> taffy::Size<taffy::Dimension>;
    fn map_flex_properties(&self, comp: &FtdComponent) -> FlexStyle;
}
```

**Key Mappings:**
- `spacing.fixed.px: 20` → `taffy::gap: Length::from_px(20.0)`
- `padding.px: 8` → `taffy::padding: Rect::from_length(Length::from_px(8.0))`  
- `width.fill-container` → `taffy::size.width: Dimension::Percent(1.0)`

#### 3. **ANSI Canvas** (`canvas/`)
```rust
pub struct AnsiCanvas {
    char_grid: Vec<Vec<char>>,
    color_grid: Vec<Vec<AnsiColor>>,
    bg_color_grid: Vec<Vec<AnsiColor>>,
    width: usize,  // characters
    height: usize, // lines
}

impl AnsiCanvas {
    fn draw_border(&mut self, rect: CharRect, style: BorderStyle);
    fn draw_filled_rect(&mut self, rect: CharRect, bg_color: AnsiColor);
    fn draw_text(&mut self, pos: CharPos, text: &str, style: TextStyle);
    fn to_ansi_string(&self) -> String;
}
```

**ANSI Color Support:**
```rust
pub enum AnsiColor {
    Black, Red, Green, Yellow, Blue, Magenta, Cyan, White,
    BrightBlack, BrightRed, BrightGreen, /* ... */,
    Rgb(u8, u8, u8),    // 24-bit color support
}
```

#### 4. **Component Renderers** (`components/`)
```rust
pub trait ComponentRenderer {
    fn render(&self, canvas: &mut AnsiCanvas, layout: &taffy::Layout, style: &FtdStyle);
}

pub struct TextRenderer;
impl ComponentRenderer for TextRenderer {
    fn render(&self, canvas: &mut AnsiCanvas, layout: &taffy::Layout, style: &FtdStyle) {
        // Handle text wrapping, alignment, colors, borders
    }
}

pub struct ColumnRenderer;  
impl ComponentRenderer for ColumnRenderer {
    fn render(&self, canvas: &mut AnsiCanvas, layout: &taffy::Layout, style: &FtdStyle) {
        // Render container borders, background, then children
    }
}
```

## CSS Property Support Matrix

### **Phase 1 Properties (Essential)**
- ✅ **Layout**: `width`, `height`, `padding`, `margin`
- ✅ **Flexbox**: `spacing`, `align-content`, `justify-content`  
- ✅ **Border**: `border-width`, `border-style` (ASCII box drawing)
- ✅ **Colors**: `color`, `background-color` (ANSI color space)
- ✅ **Typography**: `role` (font size via character scaling)

### **Phase 2 Properties (Advanced)**  
- ✅ **Grid**: CSS Grid layout (via Taffy)
- ✅ **Positioning**: `position`, `top`, `left`, `z-index`
- ✅ **Effects**: `opacity` (via ANSI transparency)
- ✅ **Responsive**: Media queries via canvas size detection

### **Limitations (By Design)**
- ❌ **Gradients**: ANSI doesn't support gradients
- ❌ **Shadows**: Limited ASCII shadow representation  
- ❌ **Complex shapes**: Only rectangles and text
- ❌ **Images**: Placeholder representations only

## Character Coordinate System

### **Coordinate Mapping**
```rust
// Pixel to character conversion
const CHAR_WIDTH_PX: f32 = 8.0;    // Typical monospace character width
const LINE_HEIGHT_PX: f32 = 16.0;   // Typical line height

fn px_to_chars(px: f32) -> usize {
    (px / CHAR_WIDTH_PX).round() as usize
}

fn px_to_lines(px: f32) -> usize {
    (px / LINE_HEIGHT_PX).round() as usize
}
```

### **Box Drawing Strategy**
```rust
// Unicode box drawing characters
const BOX_SINGLE: BoxChars = BoxChars {
    top_left: '┌', top_right: '┐',
    bottom_left: '└', bottom_right: '┘', 
    horizontal: '─', vertical: '│',
};

const BOX_DOUBLE: BoxChars = BoxChars {
    top_left: '╔', top_right: '╗',
    bottom_left: '╚', bottom_right: '╝',
    horizontal: '═', vertical: '║', 
};
```

## Testing Strategy

### **Test-Driven Specification**
```
input.ftd → ASCII Renderer → output.ansi
                           ↓
                    Compare with expected.ansi
```

**Test Case Structure:**
```
t/ascii-rendering/
├── basic/
│   ├── text-simple.ftd           # Input FTD
│   ├── text-simple.ansi          # Expected ASCII output
│   ├── text-with-border.ftd
│   └── text-with-border.ansi
├── layout/  
│   ├── column-spacing.ftd
│   ├── column-spacing.ansi
│   ├── flexbox-center.ftd
│   └── flexbox-center.ansi
└── colors/
    ├── ansi-colors.ftd
    └── ansi-colors.ansi          # With ANSI color codes
```

### **Automated Verification**
```rust
#[test]
fn verify_all_ascii_specs() {
    for test_case in discover_test_cases("t/ascii-rendering/") {
        let actual = render_ftd_file(&test_case.ftd_file)?;
        let expected = std::fs::read_to_string(&test_case.ansi_file)?;
        assert_eq!(actual.trim(), expected.trim());
    }
}
```

## Implementation Phases (Detailed)

### **Week 1: Taffy Layout Integration**
**Goal**: FTD → CSS → Taffy layout working

**Deliverables:**
- [ ] `FtdToCssMapper` converting FTD properties to Taffy styles
- [ ] Basic component tree → Taffy node tree
- [ ] Layout computation for simple ftd.text
- [ ] Debug ASCII boxes showing computed positions

**Test Cases:**
```ftd
-- ftd.text: Hello
width.fixed.px: 100
padding.px: 8
```
**Expected Debug Output:**
```
[Text: 100x24 at (0,0) padding:8]
```

### **Week 2: ANSI Canvas Implementation**
**Goal**: Character grid with color support

**Deliverables:**
- [ ] `AnsiCanvas` with character and color grids
- [ ] Unicode box drawing for borders
- [ ] ANSI escape code generation
- [ ] Text positioning with proper spacing

**Test Cases:**
```ftd  
-- ftd.text: Hello World
border-width.px: 1
color: red
background-color: blue
```
**Expected ANSI Output:**
```
┌─────────────────┐
│\033[31;44m Hello World \033[0m│  
└─────────────────┘
```

### **Week 3: Component Integration**
**Goal**: Full ftd.text, ftd.column, ftd.row rendering

**Deliverables:**
- [ ] `TextRenderer` with wrapping, alignment, typography
- [ ] `ColumnRenderer` with vertical layout and spacing
- [ ] `RowRenderer` with horizontal layout
- [ ] Nested component support

### **Week 4: Advanced Layout Features**
**Goal**: CSS-accurate layout behavior

**Deliverables:**  
- [ ] Flexbox spacing: `space-between`, `space-around`, `space-evenly`
- [ ] CSS Grid support via Taffy
- [ ] Responsive sizing with constraints
- [ ] Complex nested layouts

### **Week 5: Polish & Performance**
**Goal**: Production-ready ASCII renderer  

**Deliverables:**
- [ ] Performance optimization for large layouts
- [ ] Error handling and graceful degradation  
- [ ] CLI integration: `fastn render --format=ascii`
- [ ] Comprehensive test suite (100+ test cases)

### **Week 6: Specification Documentation**
**Goal**: Complete ASCII rendering specification

**Deliverables:**
- [ ] Visual specification showing all component behaviors
- [ ] ASCII art examples for every FTD component
- [ ] Layout behavior documentation
- [ ] Integration guide for future GPU renderer

## Success Metrics

### **Technical Metrics**
- [ ] **Layout Accuracy**: 100% match with CSS specification  
- [ ] **Performance**: Render complex layouts in <100ms
- [ ] **Test Coverage**: All FTD components have ASCII specs
- [ ] **Color Support**: All CSS colors map to ANSI equivalents

### **Strategic Metrics**  
- [ ] **Immediate Value**: Working terminal applications
- [ ] **Specification Quality**: Clear visual behavior documentation
- [ ] **GPU Foundation**: 70%+ layout code reusable
- [ ] **Developer Experience**: Easy to add new components

## Long-term Integration

### **Shared Layout Engine**
The ASCII renderer becomes the **testing ground** for layout logic that will be reused in the GPU renderer:

```rust
// Shared between ASCII and GPU renderers
fastn-layout-core/
├── ftd_parser.rs      // FTD component parsing
├── css_mapping.rs     // FTD → CSS property conversion  
├── taffy_wrapper.rs   // Taffy integration layer
└── layout_types.rs    // Common layout data structures
```

### **Future GPU Integration**
When ready for GPU renderer:
1. **Reuse layout engine** - Same CSS calculations
2. **Compare outputs** - ASCII vs GPU rendering for consistency
3. **Shared test suite** - Layout tests apply to both renderers
4. **Incremental migration** - Components move from ASCII to GPU gradually

This approach ensures we build **two production-quality renderers** instead of one complex renderer that takes forever to complete.

## Dependencies Justification

**Why minimal dependencies work:**
- **Taffy**: Battle-tested layout engine (used by Dioxus, Bevy)
- **ansi_term**: Simple, reliable ANSI color support
- **unicode-width**: Proper character width handling for layout

**Total complexity**: ~3 dependencies vs 10+ for GPU stack

This creates a **robust, maintainable foundation** that delivers immediate value while building toward comprehensive rendering capability.