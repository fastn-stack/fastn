# Shared Layout Engine Specification

## Overview

Design for the **shared layout engine** that will be used by both the ASCII renderer and future GPU renderer. This ensures consistent layout behavior across all output formats.

## Core Design Principle

**"Write layout logic once, render everywhere"**

The layout engine provides **renderer-agnostic** layout calculations that work identically for ASCII, GPU, PDF, SVG, and any future output format.

## Architecture

### **Shared Layout Core** (`fastn-layout-core`)

```rust
pub struct LayoutEngine {
    // Core Taffy integration
    taffy_tree: taffy::TaffyTree,
    
    // FTD → CSS mapping
    css_mapper: FtdToCssMapper,
    
    // Component tree management  
    component_tree: ComponentTree,
    root_node: Option<taffy::NodeId>,
}

pub struct ComputedLayout {
    // Final calculated dimensions and positions
    nodes: HashMap<ComponentId, taffy::Layout>,
    total_size: Size,
    render_tree: RenderTree,
}
```

### **Component Tree Representation**

```rust
pub struct ComponentTree {
    components: Arena<FtdComponent>,
    hierarchy: HashMap<ComponentId, Vec<ComponentId>>, // parent → children
    root: ComponentId,
}

pub struct FtdComponent {
    id: ComponentId,
    component_type: ComponentType,
    properties: FtdProperties,
    text_content: Option<String>,
    children: Vec<ComponentId>,
}

pub enum ComponentType {
    Text,
    Column, 
    Row,
    Container,
    Image,
    // etc.
}
```

### **FTD Property System**

```rust
pub struct FtdProperties {
    // Layout properties  
    pub width: Option<FtdSize>,
    pub height: Option<FtdSize>,
    pub padding: Option<FtdSpacing>,
    pub margin: Option<FtdSpacing>,
    
    // Flexbox properties
    pub spacing: Option<FtdSpacing>,
    pub align_content: Option<FtdAlign>,
    pub justify_content: Option<FtdJustify>,
    
    // Visual properties
    pub color: Option<FtdColor>,
    pub background_color: Option<FtdColor>,
    pub border_width: Option<FtdBorderWidth>,
    pub border_color: Option<FtdColor>,
    
    // Typography
    pub role: Option<FtdTypographyRole>,
    pub text_align: Option<FtdTextAlign>,
}
```

### **CSS Property Mapping**

```rust
pub struct FtdToCssMapper;

impl FtdToCssMapper {
    // Size mappings
    pub fn map_size(&self, ftd_size: &FtdSize) -> taffy::Dimension {
        match ftd_size {
            FtdSize::Fixed { px } => taffy::Dimension::Length((*px as f32).into()),
            FtdSize::FillContainer => taffy::Dimension::Percent(1.0),
            FtdSize::HugContent => taffy::Dimension::Auto,
            FtdSize::Percent { value } => taffy::Dimension::Percent(*value as f32 / 100.0),
        }
    }
    
    // Spacing mappings
    pub fn map_spacing(&self, ftd_spacing: &FtdSpacing) -> taffy::LengthPercentage {
        match ftd_spacing {
            FtdSpacing::Fixed { px } => taffy::LengthPercentage::Length((*px as f32).into()),
            FtdSpacing::SpaceBetween => /* Handle via flex properties */,
            FtdSpacing::SpaceAround => /* Handle via flex properties */,
            FtdSpacing::SpaceEvenly => /* Handle via flex properties */,
        }
    }
    
    // Layout style generation
    pub fn component_to_taffy_style(&self, comp: &FtdComponent) -> taffy::Style {
        taffy::Style {
            size: taffy::Size {
                width: comp.properties.width.map(|w| self.map_size(&w)).unwrap_or(taffy::Dimension::Auto),
                height: comp.properties.height.map(|h| self.map_size(&h)).unwrap_or(taffy::Dimension::Auto),
            },
            padding: self.map_padding(&comp.properties.padding),
            margin: self.map_margin(&comp.properties.margin),
            // ... etc
        }
    }
}
```

## Layout Process Flow

### **1. Parse Phase**
```
FTD Document → ComponentTree → Validate Properties
```

### **2. CSS Mapping Phase**
```
FtdComponent → CSS Properties → taffy::Style
```

### **3. Layout Computation Phase** 
```
Taffy Tree → compute_layout() → taffy::Layout per node
```

### **4. Render Tree Generation**
```
taffy::Layout + FtdComponent → RenderTree (renderer-agnostic)
```

### **5. Format-Specific Rendering**
```
RenderTree → ASCII Canvas → ANSI string
RenderTree → GPU Canvas → Image/Vector (future)
```

## Render Tree Specification

```rust
// Renderer-agnostic representation of final layout
pub struct RenderTree {
    nodes: Vec<RenderNode>,
    bounds: Rect,
}

pub struct RenderNode {
    id: ComponentId,
    bounds: Rect,           // Final position and size
    style: ComputedStyle,   // All visual properties resolved  
    content: NodeContent,   // Text, children, etc.
    z_index: i32,          // Layering order
}

pub struct ComputedStyle {
    // Colors (converted to renderer format)
    text_color: Color,
    background_color: Option<Color>,
    border_color: Option<Color>,
    
    // Layout (final pixel/character values)
    border_width: BorderWidth,
    padding: Spacing,
    margin: Spacing,
    
    // Typography  
    font_size: f32,
    text_align: TextAlign,
}
```

## Testing Integration

### **Layout Engine Tests** (Renderer-agnostic)
```rust
#[test]
fn test_flexbox_spacing() {
    let mut layout_engine = LayoutEngine::new();
    let doc = parse_ftd(r#"
        -- ftd.column:
        spacing: space-between
        height.fixed.px: 100
        
            -- ftd.text: Top
            -- ftd.text: Bottom
    "#);
    
    let layout = layout_engine.compute_layout(&doc, Size::new(80, 25));
    
    // Verify layout calculations
    assert_eq!(layout.get_node_bounds("text-1").y, 0);
    assert_eq!(layout.get_node_bounds("text-2").y, 24);
}
```

### **ASCII Renderer Tests** (Format-specific)  
```rust
#[test]
fn test_ascii_spacing_output() {
    let layout = /* ... same layout from above ... */;
    let ascii_renderer = AsciiRenderer::new();
    let output = ascii_renderer.render(&layout);
    
    assert_eq!(output, r#"
Top


                   (space distributed)


Bottom
    "#.trim());
}
```

## Code Reuse Strategy

### **Immediate Reuse (ASCII → GPU)**
- **Layout calculations**: 100% identical
- **CSS property mapping**: 100% identical  
- **Component parsing**: 90% identical
- **Test cases**: 80% applicable (layout tests)

### **Render-Specific Code**
- **ASCII**: Character grid, ANSI colors, box drawing
- **GPU**: WGPU context, Vello scenes, vector graphics

### **Long-term Evolution**
```rust
// Evolution path
ASCII Renderer (Week 6) → Shared Layout Core (Week 8) → GPU Renderer (Week 16)
                       ↓                              ↓
               Working terminal apps          High-fidelity desktop apps
```

This specification ensures both renderers have **identical layout behavior** while allowing format-specific optimization and features.

The shared layout engine becomes the **definitive FTD layout specification** that any future renderer must implement.